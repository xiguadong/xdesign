use crate::core::{
    sha256_bytes, valid_relative_path, RsCheckCommand, RsCheckerSpec, RsGateError, RsGateReport,
    RsRuleReport, RsStageVisit,
};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
#[cfg(unix)]
use std::{
    os::unix::{io::AsRawFd, process::CommandExt},
    process::{Child, ExitStatus},
};
use uuid::Uuid;

/// How long the Unix deadline loop waits between non-blocking output drains
/// when neither the checker nor its pipes have finished.
#[cfg(unix)]
const OUTPUT_POLL_INTERVAL: Duration = Duration::from_millis(5);

/// A hung checker must never hang the CLI with it: subprocesses are killed at
/// this deadline (overridable for slow machines) and reported as timed out.
/// The same deadline bounds the checker's whole process tree, not just its
/// direct child (see [`run_command_with_timeout`]).
pub fn checker_timeout() -> Duration {
    std::env::var("XDESIGN_CHECKER_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map_or(Duration::from_secs(600), Duration::from_secs)
}

pub struct CommandOutcome {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub success: bool,
    pub timed_out: bool,
}

/// Run a command to completion with a hard deadline that bounds its whole
/// process tree, not just the direct child.
///
/// On Unix the command is spawned as the leader of its own process group
/// (`pre_exec` + `setpgid`), its stdout/stderr are read through non-blocking
/// pipe ends while the deadline loop runs, and an expired deadline sends
/// `SIGKILL` to the entire group before reaping the direct child. This matters
/// because a checker can start a descendant that inherits its stdout/stderr:
/// killing only the direct child leaves those pipes open, so reader threads
/// would block until the descendant exits, silently extending the deadline
/// while a checker holds the per-wish writer lock.
///
/// Output is accumulated byte-for-byte by the deadline loop. If the direct
/// child exits before the deadline but descendants still hold the pipes, the
/// loop keeps draining and only returns success once both pipes reach EOF.
/// If the deadline expires first, the group is killed and the call returns
/// `timed_out: true, success: false` without waiting for descendant-held pipe
/// ends, so the total time is bounded by the deadline plus the process kill
/// and reap time. A normal command that closes its pipes within the deadline
/// returns its full captured output and exit status exactly as before.
///
/// The non-Unix fallback keeps the previous behavior (kill the direct child,
/// join the pipe-reader threads) because process groups are a Unix facility.
pub fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> std::io::Result<CommandOutcome> {
    #[cfg(unix)]
    return run_command_with_timeout_unix(command, timeout);
    #[cfg(not(unix))]
    return run_command_with_timeout_fallback(command, timeout);
}

/// Spawn the command as the leader of a new process group so the deadline can
/// terminate every descendant with a single `kill(-pid, SIGKILL)`.
#[cfg(unix)]
fn spawn_checker_process_group(command: &mut Command) -> std::io::Result<Child> {
    // SAFETY: the closure runs in the forked child between `fork` and `exec`,
    // where only async-signal-safe operations are permitted. `setpgid` is
    // async-signal-safe and touches no shared Rust state, so it is sound here.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

/// Switch a pipe read end to non-blocking mode so the deadline loop can drain
/// output without ever blocking on a descendant-held pipe.
#[cfg(unix)]
fn set_pipe_nonblocking<T: AsRawFd>(pipe: &T) -> std::io::Result<()> {
    let fd = pipe.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags == -1 {
        return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Drain every byte currently available from a non-blocking pipe.
///
/// Returns `true` when the pipe has reached EOF (or a read error leaves no
/// more bytes to collect), meaning the deadline loop can stop polling it;
/// `false` means the pipe is still open and should be polled again.
#[cfg(unix)]
fn drain_available(pipe: &mut dyn Read, buffer: &mut Vec<u8>) -> bool {
    let mut chunk = [0u8; 16 * 1024];
    loop {
        match pipe.read(&mut chunk) {
            Ok(0) => return true,
            Ok(read) => buffer.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return false,
            // A real read error makes no further bytes available, mirroring
            // the previous lenient `read_to_end` handling by keeping the
            // bytes already captured.
            Err(_) => return true,
        }
    }
}

/// Kill the checker's whole process group. `child.kill` is only the fallback
/// for the rare case where the group signal cannot be delivered.
#[cfg(unix)]
fn kill_process_group(child: &mut Child) {
    let pid = child.id();
    let signaled =
        pid <= i32::MAX as u32 && unsafe { libc::kill(-(pid as libc::pid_t), libc::SIGKILL) } == 0;
    if !signaled {
        let _ = child.kill();
    }
}

#[cfg(unix)]
fn run_command_with_timeout_unix(
    command: &mut Command,
    timeout: Duration,
) -> std::io::Result<CommandOutcome> {
    let deadline = Instant::now() + timeout;
    let mut child = spawn_checker_process_group(command)?;
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");
    set_pipe_nonblocking(&stdout_pipe)?;
    set_pipe_nonblocking(&stderr_pipe)?;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut stdout_closed = false;
    let mut stderr_closed = false;
    let mut status: Option<ExitStatus> = None;
    loop {
        // Drain output first: descendants may keep writing while the direct
        // child is already gone, and a large child output must not block on a
        // full pipe buffer.
        if !stdout_closed {
            stdout_closed = drain_available(&mut stdout_pipe, &mut stdout);
        }
        if !stderr_closed {
            stderr_closed = drain_available(&mut stderr_pipe, &mut stderr);
        }
        if status.is_none() {
            status = child.try_wait()?;
        }
        // The command is finished only when the direct child has exited and
        // no remaining writer (including descendants) holds its pipes open.
        if let Some(status) = status {
            if stdout_closed && stderr_closed {
                return Ok(CommandOutcome {
                    stdout,
                    stderr,
                    success: status.success(),
                    timed_out: false,
                });
            }
        }
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        thread::sleep((deadline - now).min(OUTPUT_POLL_INTERVAL));
    }

    // Deadline expired. Kill the group before returning so descendants holding
    // the inherited pipes cannot outlive the checker or hang it. The final
    // non-blocking drain only collects bytes already buffered; it never waits
    // for a descendant that survived the group signal.
    kill_process_group(&mut child);
    let _ = child.wait();
    if !stdout_closed {
        drain_available(&mut stdout_pipe, &mut stdout);
    }
    if !stderr_closed {
        drain_available(&mut stderr_pipe, &mut stderr);
    }
    Ok(CommandOutcome {
        stdout,
        stderr,
        success: false,
        timed_out: true,
    })
}

/// Fallback for platforms without Unix process groups: kill only the direct
/// child and join its output-reader threads. Descendants that inherit the
/// pipes can still extend the deadline on these platforms.
#[cfg(not(unix))]
fn run_command_with_timeout_fallback(
    command: &mut Command,
    timeout: Duration,
) -> std::io::Result<CommandOutcome> {
    let deadline = Instant::now() + timeout;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");
    let stdout = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buffer);
        buffer
    });
    let stderr = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buffer);
        buffer
    });
    loop {
        match child.try_wait()? {
            Some(status) => {
                return Ok(CommandOutcome {
                    stdout: stdout.join().unwrap_or_default(),
                    stderr: stderr.join().unwrap_or_default(),
                    success: status.success(),
                    timed_out: false,
                })
            }
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(CommandOutcome {
                    stdout: stdout.join().unwrap_or_default(),
                    stderr: stderr.join().unwrap_or_default(),
                    success: false,
                    timed_out: true,
                });
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RsProcessCheck {
    pub report_directory: String,
}

#[derive(Deserialize)]
struct RawRule {
    rule_id: String,
    verdict: String,
    inputs: Vec<String>,
    observed: serde_json::Value,
    failures: Vec<String>,
}

#[derive(Deserialize)]
struct RawReport {
    checker_id: String,
    stage: String,
    artifact_revision: u32,
    snapshot_digest: String,
    report_uri: String,
    overall_verdict: String,
    rules: Vec<RawRule>,
    input_sha256: HashMap<String, String>,
}

impl RsCheckCommand for RsProcessCheck {
    fn run_check(
        &self,
        checker: &RsCheckerSpec,
        plan_id: &str,
        plan_uri: &str,
        plan_sha256: &str,
        wish_model: &str,
        run_record: &str,
        stage: &str,
        artifact_revision: u32,
        snapshot_digest: &str,
    ) -> Result<RsGateReport, RsGateError> {
        let plan_bytes = fs::read(plan_uri).map_err(|_| RsGateError::CommandFailed)?;
        if sha256_bytes(&plan_bytes) != plan_sha256 {
            return Err(RsGateError::CommandFailed);
        }
        let plan_json: serde_json::Value =
            serde_json::from_slice(&plan_bytes).map_err(|_| RsGateError::CommandFailed)?;
        if plan_json["plan_id"] != plan_id {
            return Err(RsGateError::CommandFailed);
        }
        let record_bytes = fs::read(run_record).map_err(|_| RsGateError::CommandFailed)?;
        let record: serde_json::Value =
            serde_json::from_slice(&record_bytes).map_err(|_| RsGateError::CommandFailed)?;
        if record["plan_id"] != plan_id
            || record["plan_uri"] != plan_uri
            || record["plan_sha256"] != plan_sha256
            || record["wish_model_uri"] != wish_model
        {
            return Err(RsGateError::CommandFailed);
        }
        let idx = record["current_visit"]
            .as_u64()
            .ok_or(RsGateError::CommandFailed)? as usize;
        let visit_json = record["visits"]
            .get(idx)
            .ok_or(RsGateError::CommandFailed)?;
        if visit_json["stage_id"] != stage
            || visit_json["artifact_revision"] != artifact_revision
            || visit_json["artifact_snapshot_digest"] != snapshot_digest
        {
            return Err(RsGateError::CommandFailed);
        }
        let visit: RsStageVisit =
            serde_json::from_value(visit_json.clone()).map_err(|_| RsGateError::CommandFailed)?;
        if !visit.verify_artifacts() {
            return Err(RsGateError::CommandFailed);
        }
        if !Path::new(&checker.command_uri).is_file() {
            return Err(RsGateError::CommandFailed);
        }
        fs::create_dir_all(&self.report_directory).map_err(|_| RsGateError::CommandFailed)?;
        let report_uri = format!(
            "{}/{}_{}_{}.json",
            self.report_directory,
            stage,
            artifact_revision,
            Uuid::new_v4()
        );
        let timeout = checker_timeout();
        let output = run_command_with_timeout(
            Command::new("python3")
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .arg(&checker.command_uri)
                .arg("--wish-model")
                .arg(wish_model)
                .arg("--record")
                .arg(run_record)
                .arg("--checker-id")
                .arg(&checker.checker_id)
                .arg("--plan-id")
                .arg(plan_id)
                .arg("--plan-uri")
                .arg(plan_uri)
                .arg("--plan-sha256")
                .arg(plan_sha256)
                .arg("--stage")
                .arg(stage)
                .arg("--artifact-revision")
                .arg(artifact_revision.to_string())
                .arg("--snapshot-digest")
                .arg(snapshot_digest)
                .arg("--report")
                .arg(&report_uri),
            timeout,
        )
        .map_err(|_| RsGateError::CommandFailed)?;
        let _ = fs::write(format!("{report_uri}.stdout.log"), &output.stdout);
        let _ = fs::write(
            format!("{report_uri}.stderr.log"),
            if output.timed_out {
                format!(
                    "checker {} exceeded the {:?} deadline and was killed",
                    checker.checker_id, timeout
                )
                .into_bytes()
            } else {
                output.stderr.clone()
            },
        );
        if output.timed_out {
            return Err(RsGateError::CheckerTimeout);
        }
        if !output.success {
            return Err(RsGateError::CommandFailed);
        }
        let raw_bytes = fs::read(&report_uri).map_err(|_| RsGateError::CommandFailed)?;
        let raw: RawReport =
            serde_json::from_slice(&raw_bytes).map_err(|_| RsGateError::CommandFailed)?;
        if raw.checker_id != checker.checker_id
            || raw.stage != stage
            || raw.artifact_revision != artifact_revision
            || raw.snapshot_digest != snapshot_digest
            || raw.report_uri != report_uri
            || !["passed", "failed"].contains(&raw.overall_verdict.as_str())
        {
            return Err(RsGateError::CommandFailed);
        }
        if raw.input_sha256.get(wish_model).map(String::as_str)
            != Some(
                sha256_bytes(&fs::read(wish_model).map_err(|_| RsGateError::CommandFailed)?)
                    .as_str(),
            )
            || raw.input_sha256.get(run_record).map(String::as_str)
                != Some(sha256_bytes(&record_bytes).as_str())
            || raw.input_sha256.get(plan_uri).map(String::as_str) != Some(plan_sha256)
        {
            return Err(RsGateError::CommandFailed);
        }
        let bound: HashMap<_, _> = visit
            .artifact_digests()
            .iter()
            .map(|d| (d.uri.as_str(), d.sha256.as_str()))
            .collect();
        for (uri, digest) in &raw.input_sha256 {
            if !valid_relative_path(uri) {
                return Err(RsGateError::CommandFailed);
            }
            let bytes = fs::read(uri).map_err(|_| RsGateError::CommandFailed)?;
            if sha256_bytes(&bytes) != *digest {
                return Err(RsGateError::CommandFailed);
            }
            if (stage == "requirements" || stage == "design")
                && uri != wish_model
                && uri != run_record
                && uri != plan_uri
                && bound.get(uri.as_str()).copied() != Some(digest.as_str())
            {
                return Err(RsGateError::CommandFailed);
            }
        }
        let mut expected = HashSet::new();
        for rule in &checker.rules {
            expected.insert(rule.rule_id.as_str());
        }
        let mut seen = HashSet::new();
        let mut rules = Vec::new();
        for rule in raw.rules {
            let Some(declared) = checker.rules.iter().find(|r| r.rule_id == rule.rule_id) else {
                return Err(RsGateError::CommandFailed);
            };
            let inputs: HashSet<_> = rule.inputs.iter().map(String::as_str).collect();
            if !expected.contains(rule.rule_id.as_str())
                || !seen.insert(rule.rule_id.clone())
                || !["passed", "failed"].contains(&rule.verdict.as_str())
                || rule.inputs.is_empty()
                || rule.inputs.iter().any(|input| input.trim().is_empty())
                || rule.observed.is_null()
                || rule
                    .observed
                    .as_object()
                    .is_some_and(|value| value.is_empty())
                || rule
                    .observed
                    .as_array()
                    .is_some_and(|value| value.is_empty())
                || rule.observed.as_str().is_some_and(str::is_empty)
                || (rule.verdict == "passed" && !rule.failures.is_empty())
                || (rule.verdict == "failed" && rule.failures.is_empty())
            {
                return Err(RsGateError::CommandFailed);
            }
            if rule.verdict == "passed" {
                if rule
                    .inputs
                    .iter()
                    .any(|input| !raw.input_sha256.contains_key(input))
                {
                    return Err(RsGateError::CommandFailed);
                }
                for read in &declared.reads {
                    let covered = match read.as_str() {
                        "runtime run record JSON" => inputs.contains(run_record),
                        "retained plan serialization" => inputs.contains(plan_uri),
                        "recorded artifact URIs" => {
                            let source_visits: Vec<_> = if stage == "implement" {
                                vec![visit_json]
                            } else {
                                record["visits"]
                                    .as_array()
                                    .ok_or(RsGateError::CommandFailed)?
                                    .iter()
                                    .filter(|v| v["status"] == "completed")
                                    .collect()
                            };
                            source_visits.iter().all(|source| {
                                source["artifact_digests"]
                                    .as_array()
                                    .is_some_and(|artifacts| {
                                        artifacts.iter().all(|item| {
                                            item["uri"]
                                                .as_str()
                                                .is_some_and(|uri| inputs.contains(uri))
                                        })
                                    })
                            })
                        }
                        "recorded evidence URIs" => record["visits"]
                            .as_array()
                            .ok_or(RsGateError::CommandFailed)?
                            .iter()
                            .enumerate()
                            .filter(|(index, v)| *index == idx || v["status"] == "completed")
                            .all(|(_, v)| {
                                v["assessments"].as_array().is_some_and(|assessments| {
                                    assessments.iter().filter(|a| a["outcome"] == "passed").all(
                                        |a| {
                                            a["evidence_uris"].as_array().is_some_and(|uris| {
                                                uris.iter().all(|uri| {
                                                    uri.as_str()
                                                        .is_some_and(|uri| inputs.contains(uri))
                                                })
                                            })
                                        },
                                    )
                                })
                            }),
                        "tools/checks/" => {
                            plan_json["checkpoints"].as_array().is_some_and(|gates| {
                                let commands: Vec<_> = gates
                                    .iter()
                                    .filter_map(|gate| gate["checker"]["command_uri"].as_str())
                                    .collect();
                                commands.len() == 4
                                    && commands.iter().all(|command| inputs.contains(command))
                            })
                        }
                        prefix if prefix.ends_with('/') => {
                            inputs.iter().any(|input| input.starts_with(prefix))
                        }
                        path => inputs.contains(path),
                    };
                    if !covered {
                        return Err(RsGateError::CommandFailed);
                    }
                }
            }
            let diagnostic = if rule.failures.is_empty() {
                None
            } else {
                Some(rule.failures.join("; "))
            };
            rules.push(RsRuleReport {
                rule_id: rule.rule_id,
                holds: rule.verdict == "passed",
                diagnostic,
            });
        }
        if seen.len() != expected.len() {
            return Err(RsGateError::CommandFailed);
        }
        let holds = rules.iter().all(|rule| rule.holds);
        if holds != (raw.overall_verdict == "passed") {
            return Err(RsGateError::CommandFailed);
        }
        Ok(RsGateReport {
            holds,
            artifact_revision,
            artifact_snapshot_digest: snapshot_digest.to_owned(),
            report_uri,
            rules,
        })
    }
}
