//! Embedded operator web UI for the local xdesign store and stage CLI.
//!
//! The UI ships inside the `xdesign` executable: page assets are embedded by
//! `web/server/build.rs` and served by a small standard-library HTTP server.
//! The server is an operator-facing read/observe surface with a small set of
//! lifecycle actions. It is not the documentation site, it does not read the
//! `docs/` tree, and it has no Node.js, Python web server, or static file
//! dependency at runtime.
//!
//! This file is the module root; `web/server/` also contains the HTTP layer,
//! API assembly and tests. The page itself is authored under
//! `web/assets/` and is served from memory.

#![allow(dead_code)]

mod api;
mod http;

mod embedded_assets {
    include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));
}

pub use embedded_assets::ASSET_FILES;

use std::env;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use api::AppState;

/// Default bind host: loopback only, never the network.
const DEFAULT_HOST: &str = "127.0.0.1";
/// Default operator port, matching the documented command line.
const DEFAULT_PORT: u16 = 7788;
/// Socket write deadline for one response.
const WRITE_TIMEOUT: Duration = Duration::from_secs(30);

const USAGE: &str = "Usage: xdesign web [OPTIONS]

Options:
  --host HOST            Bind host (default 127.0.0.1)
  --port PORT            Bind port, 0 selects an ephemeral port (default 7788)
  --open                 Open the page in the platform browser (best effort)
  --project-root PATH    Project root to inspect (default: current directory)
  --store-root PATH      Operational store root (default: XDESIGN_STORE_ROOT or
                         <project-root>/.xdesign)";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Options {
    host: String,
    port: u16,
    open: bool,
    project_root: Option<String>,
    store_root: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.into(),
            port: DEFAULT_PORT,
            open: false,
            project_root: None,
            store_root: None,
        }
    }
}

/// Parse `xdesign web` arguments. Values may be given as `--name value` or
/// `--name=value`.
fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        let (name, inline_value) = match argument.split_once('=') {
            Some((name, value)) if name.starts_with("--") => (name, Some(value.to_owned())),
            _ => (argument.as_str(), None),
        };
        let mut value = |name: &str| -> Result<String, String> {
            if let Some(value) = inline_value.clone() {
                if value.is_empty() {
                    return Err(format!("missing value for {name}"));
                }
                return Ok(value);
            }
            index += 1;
            args.get(index)
                .filter(|value| !value.starts_with("--"))
                .cloned()
                .ok_or_else(|| format!("missing value for {name}"))
        };
        match name {
            "--host" => options.host = value("--host")?,
            "--port" => {
                let raw = value("--port")?;
                options.port = raw
                    .parse::<u16>()
                    .map_err(|_| format!("invalid --port value: {raw}"))?;
            }
            "--open" => {
                if inline_value.is_some() {
                    return Err("--open does not take a value".into());
                }
                options.open = true;
            }
            "--project-root" => options.project_root = Some(value("--project-root")?),
            "--store-root" => options.store_root = Some(value("--store-root")?),
            "--help" | "-h" => return Err(USAGE.into()),
            other => return Err(format!("unknown option: {other}\n{USAGE}")),
        }
        index += 1;
    }
    if options.host.trim().is_empty() {
        return Err("--host must not be empty".into());
    }
    Ok(options)
}

/// Run the operator web server until the process is interrupted.
pub fn serve(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    let project_root = resolve_project_root(options.project_root.as_deref())?;
    env::set_current_dir(&project_root).map_err(|error| {
        format!(
            "cannot change directory to project root {}: {error}",
            project_root.display()
        )
    })?;
    let legacy_only = options.store_root.is_none()
        && env::var_os("XDESIGN_STORE_ROOT").is_none()
        && api::legacy_only_project(&project_root)?;
    let selection = if legacy_only {
        Ok(None)
    } else {
        (crate::storage::cli::resolve_store_root(
            &project_root,
            options.store_root.as_deref().map(Path::new),
            true,
        )
        .map(Some)
        .map_err(|e| e.to_string()))
    };
    let store_root = selection.as_ref().ok().cloned().flatten();
    let listener = TcpListener::bind((options.host.as_str(), options.port))
        .map_err(|error| format!("cannot bind {}:{}: {error}", options.host, options.port))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("cannot read the bound address: {error}"))?;
    let state = Arc::new(match selection {
        Ok(Some(root)) => AppState::new(project_root.clone(), root),
        Ok(None) => AppState::legacy_only(project_root.clone()),
        Err(error) => AppState::diagnostic(project_root.clone(), error),
    });
    let url = http_url(address);
    println!("{}", listening_line(address));
    println!("  project root: {}", project_root.display());
    println!(
        "  store root:   {}",
        store_root
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "legacy only".into())
    );
    println!("  open {url} in a browser; press Ctrl-C to stop");
    if options.open {
        open_browser(&url);
    }
    serve_listener(listener, state, None)
}

/// The single machine-readable startup line the test harness polls.
fn listening_line(address: SocketAddr) -> String {
    format!("xdesign web listening on {}", http_url(address))
}

/// Absolute loopback URL for the bound address, bracketing IPv6 literals.
fn http_url(address: SocketAddr) -> String {
    match address.ip() {
        std::net::IpAddr::V6(ip) => format!("http://[{ip}]:{}", address.port()),
        ip => format!("http://{ip}:{}", address.port()),
    }
}

fn resolve_project_root(requested: Option<&str>) -> Result<PathBuf, String> {
    let path = match requested {
        Some(path) => PathBuf::from(path),
        None => {
            env::current_dir().map_err(|error| format!("cannot read current directory: {error}"))?
        }
    };
    canonical_directory(&path)
}

fn canonical_directory(path: &Path) -> Result<PathBuf, String> {
    let canonical = std::fs::canonicalize(path)
        .map_err(|error| format!("cannot resolve {}: {error}", path.display()))?;
    if !canonical.is_dir() {
        return Err(format!("not a directory: {}", canonical.display()));
    }
    Ok(canonical)
}

/// Resolve the store root: explicit option, then `XDESIGN_STORE_ROOT`, then
/// `<project>/.xdesign`. Missing stores are allowed; the page starts empty.
fn resolve_store_root(project_root: &Path, requested: Option<&str>) -> PathBuf {
    let selected = requested
        .map(PathBuf::from)
        .or_else(|| env::var_os("XDESIGN_STORE_ROOT").map(PathBuf::from))
        .unwrap_or_else(|| project_root.join(".xdesign"));
    if selected.is_absolute() {
        normalize(&selected)
    } else {
        normalize(&project_root.join(selected))
    }
}

/// Lexically normalize `.`/`..` components without requiring the path to
/// exist, so a missing store root is still reported as an absolute path.
fn normalize(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Best-effort browser launch. Never fatal: the server is already listening.
fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(not(target_os = "macos"))]
    let opener = "xdg-open";
    if let Ok(mut child) = Command::new(opener).arg(url).spawn() {
        // Reap the opener asynchronously; the server must not stop to wait for
        // a desktop process.
        thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

/// Accept connections until `shutdown` flips (tests) or forever (production,
/// where Ctrl-C terminates the process with its default signal behavior).
///
/// Each connection is handled on its own thread, so a long-running lifecycle
/// command never blocks another request.
fn serve_listener(
    listener: TcpListener,
    state: Arc<AppState>,
    shutdown: Option<Arc<AtomicBool>>,
) -> Result<(), String> {
    loop {
        if shutdown
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::Relaxed))
        {
            return Ok(());
        }
        match listener.accept() {
            Ok((stream, _peer)) => {
                let state = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(error) = handle_connection(stream, state) {
                        // A client that disappears mid-response is normal for a
                        // local page; the operator console should not spam.
                        eprintln!("xdesign web: connection error: {error}");
                    }
                });
            }
            Err(error) => {
                // Transient accept failures (for example EINTR) must not kill
                // the operator server.
                eprintln!("xdesign web: accept failed: {error}");
                thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, state: Arc<AppState>) -> io::Result<()> {
    let _ = stream.set_read_timeout(Some(http::READ_TIMEOUT));
    let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
    let response = match http::read_request(&mut stream) {
        Ok(request) => match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            api::handle(&state, &request)
        })) {
            Ok(response) => response,
            Err(_) => http::json_error(
                500,
                "internal_error",
                "The request handler failed unexpectedly.",
            ),
        },
        Err(error) => error.into_response(),
    };
    let _ = http::write_response(&mut stream, &response);
    let _ = stream.shutdown(std::net::Shutdown::Both);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn defaults_match_the_spec() {
        let options = parse_options(&[]).unwrap();
        assert_eq!(options.host, "127.0.0.1");
        assert_eq!(options.port, 7788);
        assert!(!options.open);
        assert_eq!(options.project_root, None);
        assert_eq!(options.store_root, None);
    }

    #[test]
    fn parses_every_option_in_both_forms() {
        let options = parse_options(&args(&[
            "--host",
            "0.0.0.0",
            "--port",
            "0",
            "--open",
            "--project-root",
            "/tmp/project",
            "--store-root=/tmp/store",
        ]))
        .unwrap();
        assert_eq!(options.host, "0.0.0.0");
        assert_eq!(options.port, 0);
        assert!(options.open);
        assert_eq!(options.project_root.as_deref(), Some("/tmp/project"));
        assert_eq!(options.store_root.as_deref(), Some("/tmp/store"));
    }

    #[test]
    fn rejects_unknown_and_incomplete_options() {
        assert!(parse_options(&args(&["--bogus"])).is_err());
        assert!(parse_options(&args(&["--port"])).is_err());
        assert!(parse_options(&args(&["--port", "not-a-port"])).is_err());
        assert!(parse_options(&args(&["--port", "70000"])).is_err());
        assert!(parse_options(&args(&["--open=yes"])).is_err());
        assert!(parse_options(&args(&["--host="])).is_err());
    }

    #[test]
    fn startup_line_is_exactly_documented() {
        let address: SocketAddr = "127.0.0.1:7788".parse().unwrap();
        assert_eq!(
            listening_line(address),
            "xdesign web listening on http://127.0.0.1:7788"
        );
        let ephemeral: SocketAddr = "127.0.0.1:49321".parse().unwrap();
        assert_eq!(
            listening_line(ephemeral),
            "xdesign web listening on http://127.0.0.1:49321"
        );
        let loopback_v6: SocketAddr = "[::1]:7788".parse().unwrap();
        assert_eq!(
            listening_line(loopback_v6),
            "xdesign web listening on http://[::1]:7788"
        );
    }

    #[test]
    fn store_root_precedence_is_explicit_then_env_then_project() {
        let project = Path::new("/tmp/example-project");
        assert_eq!(
            resolve_store_root(project, Some("/tmp/explicit-store")),
            PathBuf::from("/tmp/explicit-store")
        );
        assert_eq!(
            resolve_store_root(project, Some("cache/relative-store")),
            PathBuf::from("/tmp/example-project/cache/relative-store")
        );
        assert_eq!(
            resolve_store_root(project, None),
            PathBuf::from("/tmp/example-project/.xdesign")
        );
    }

    #[test]
    fn normalize_removes_dot_segments() {
        assert_eq!(
            normalize(Path::new("/tmp/a/./b/../c")),
            PathBuf::from("/tmp/a/c")
        );
        assert_eq!(
            normalize(Path::new("/tmp/project/.xdesign")),
            PathBuf::from("/tmp/project/.xdesign")
        );
    }
}
