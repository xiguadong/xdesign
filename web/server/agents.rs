//! Static agent CLI catalog and best-effort runtime detection.
//!
//! The catalog is display metadata for the operator page: the two supported
//! agent CLIs, their known models, and whether the local installation can be
//! detected. Detection runs `<command> --version` with a 3 s deadline and reads
//! the user's CLI configuration for a default model. Nothing here ever invokes
//! an agent for real work, and detection is injectable so tests never spawn a
//! process.

use serde_json::{json, Value};
use std::process::Command;
use std::time::Duration;

use crate::simple_flow::run_command_with_timeout;

/// Deadline for the `--version` probe.
pub const VERSION_TIMEOUT: Duration = Duration::from_secs(3);

pub struct ModelDef {
    pub id: &'static str,
    pub label: &'static str,
}

pub struct AgentDef {
    pub cli: &'static str,
    pub label: &'static str,
    pub command: &'static str,
    /// Path of the CLI's configuration file relative to the user's home.
    pub config_relative: &'static str,
    /// Default model shown when the user's configuration does not name one.
    pub default_model: &'static str,
    pub models: &'static [ModelDef],
}

const CODEX_MODELS: &[ModelDef] = &[
    ModelDef {
        id: "gpt-6-astra",
        label: "GPT-6 Astra",
    },
    ModelDef {
        id: "gpt-5.6-terra",
        label: "GPT-5.6 Terra",
    },
    ModelDef {
        id: "gpt-5.1-codex",
        label: "GPT-5.1 Codex",
    },
    ModelDef {
        id: "gpt-5-codex",
        label: "GPT-5 Codex",
    },
    ModelDef {
        id: "o4-mini",
        label: "o4-mini",
    },
];

const KIMI_MODELS: &[ModelDef] = &[
    ModelDef {
        id: "kimi-code/k3.1-pro",
        label: "K3.1 Pro",
    },
    ModelDef {
        id: "kimi-code/k3.1-pro-highspeed",
        label: "K3.1 Pro Highspeed",
    },
    ModelDef {
        id: "kimi-code/k3",
        label: "K3",
    },
    ModelDef {
        id: "kimi-code/k3-256k",
        label: "K3 256k",
    },
    ModelDef {
        id: "kimi-code/kimi-for-coding",
        label: "K2.8 Preview",
    },
    ModelDef {
        id: "kimi-code/kimi-for-coding-highspeed",
        label: "K2.7 Code Highspeed",
    },
];

pub const AGENTS: &[AgentDef] = &[
    AgentDef {
        cli: "codex",
        label: "Codex CLI",
        command: "codex",
        config_relative: ".codex/config.toml",
        default_model: "gpt-6-astra",
        models: CODEX_MODELS,
    },
    AgentDef {
        cli: "kimi",
        label: "Kimi Code CLI",
        command: "kimi",
        config_relative: ".kimi-code/config.toml",
        default_model: "kimi-code/k3.1-pro",
        models: KIMI_MODELS,
    },
];

/// One catalog entry as exposed by `GET /api/flows`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentInfo {
    pub cli: String,
    pub label: String,
    pub command: String,
    pub available: bool,
    pub version: Option<String>,
    pub default_model: String,
    pub models: Vec<(String, String)>,
}

impl AgentInfo {
    /// Full catalog form used by `GET /api/flows`.
    pub fn to_json(&self) -> Value {
        json!({
            "cli": self.cli,
            "label": self.label,
            "command": self.command,
            "available": self.available,
            "version": self.version,
            "default_model": self.default_model,
            "models": self
                .models
                .iter()
                .map(|(id, label)| json!({"id": id, "label": label}))
                .collect::<Vec<_>>(),
        })
    }

    /// Compact form used by `GET /api/health`.
    pub fn health_json(&self) -> Value {
        json!({
            "cli": self.cli,
            "available": self.available,
            "version": self.version,
        })
    }
}

/// Runtime probing used by [`AgentCatalog::detect`]. Kept separate so tests can
/// provide recorded answers without executing anything.
pub trait Detector: Send + Sync {
    /// First non-empty line of `<command> --version`, or `None` when the CLI is
    /// absent, fails, or times out.
    fn version(&self, command: &str) -> Option<String>;
    /// Contents of a configuration file relative to the user's home directory.
    fn config(&self, relative: &str) -> Option<String>;
}

pub struct SystemDetector;

impl SystemDetector {
    fn home() -> Option<std::path::PathBuf> {
        std::env::var_os("HOME").map(std::path::PathBuf::from)
    }
}

impl Detector for SystemDetector {
    fn version(&self, command: &str) -> Option<String> {
        let mut probe = Command::new(command);
        probe.arg("--version");
        let outcome = run_command_with_timeout(&mut probe, VERSION_TIMEOUT).ok()?;
        if outcome.timed_out || !outcome.success {
            return None;
        }
        first_nonempty_line(&outcome.stdout)
    }

    fn config(&self, relative: &str) -> Option<String> {
        let path = Self::home()?.join(relative);
        let bytes = std::fs::read(&path).ok()?;
        // A configuration file is only scanned for one line; do not hold an
        // arbitrarily large or non-UTF-8 document in memory.
        let bytes = &bytes[..bytes.len().min(256 * 1024)];
        Some(String::from_utf8_lossy(bytes).into_owned())
    }
}

fn first_nonempty_line(stdout: &[u8]) -> Option<String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
}

/// Default model from a TOML line scan (`model = "..."` for codex,
/// `default_model = "..."` for kimi). Deliberately not a TOML parser: the
/// value is display metadata only and a malformed file simply means "not
/// detected".
pub fn config_default_model(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let value = rest.trim();
        let value = match value.strip_prefix('"') {
            Some(inner) => inner.split('"').next().unwrap_or_default(),
            None => match value.strip_prefix('\'') {
                Some(inner) => inner.split('\'').next().unwrap_or_default(),
                None => value
                    .split(|character: char| character.is_whitespace() || character == '#')
                    .next()
                    .unwrap_or_default(),
            },
        };
        let value = value.trim();
        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }
    None
}

#[derive(Clone, Debug, Default)]
pub struct AgentCatalog {
    agents: Vec<AgentInfo>,
}

impl AgentCatalog {
    /// Detect every catalog entry against the live system.
    pub fn detect() -> Self {
        Self::detect_with(&SystemDetector)
    }

    /// Detect every catalog entry through an injected detector.
    pub fn detect_with(detector: &dyn Detector) -> Self {
        let agents = AGENTS
            .iter()
            .map(|definition| {
                let version = detector.version(definition.command);
                let available = version.is_some();
                let config = detector.config(definition.config_relative);
                let detected = config
                    .as_deref()
                    .and_then(|contents| config_default_model(contents, model_key(definition.cli)));
                AgentInfo {
                    cli: definition.cli.to_owned(),
                    label: definition.label.to_owned(),
                    command: definition.command.to_owned(),
                    available,
                    version,
                    default_model: detected.unwrap_or_else(|| definition.default_model.to_owned()),
                    models: definition
                        .models
                        .iter()
                        .map(|model| (model.id.to_owned(), model.label.to_owned()))
                        .collect(),
                }
            })
            .collect();
        Self { agents }
    }

    /// An empty catalog, used where agent detection is not meaningful.
    pub fn empty() -> Self {
        Self { agents: Vec::new() }
    }

    pub fn agents(&self) -> &[AgentInfo] {
        &self.agents
    }

    pub fn get(&self, cli: &str) -> Option<&AgentInfo> {
        self.agents.iter().find(|agent| agent.cli == cli)
    }

    pub fn to_json(&self) -> Value {
        Value::Array(self.agents.iter().map(AgentInfo::to_json).collect())
    }

    pub fn health_json(&self) -> Value {
        Value::Array(self.agents.iter().map(AgentInfo::health_json).collect())
    }
}

fn model_key(cli: &str) -> &'static str {
    match cli {
        "kimi" => "default_model",
        _ => "model",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct FakeDetector {
        versions: HashMap<String, String>,
        configs: HashMap<String, String>,
    }

    impl Detector for FakeDetector {
        fn version(&self, command: &str) -> Option<String> {
            self.versions.get(command).cloned()
        }

        fn config(&self, relative: &str) -> Option<String> {
            self.configs.get(relative).cloned()
        }
    }

    fn fake_detector() -> FakeDetector {
        FakeDetector {
            versions: HashMap::from([
                ("codex".into(), "codex-cli 0.162.1".into()),
                ("kimi".into(), "2.2.0".into()),
            ]),
            configs: HashMap::new(),
        }
    }

    #[test]
    fn catalog_matches_the_spec() {
        let catalog = AgentCatalog::detect_with(&fake_detector());
        let value = catalog.to_json();
        let agents = value.as_array().unwrap();
        assert_eq!(agents.len(), 2);
        assert_eq!(agents[0]["cli"], "codex");
        assert_eq!(agents[0]["label"], "Codex CLI");
        assert_eq!(agents[0]["command"], "codex");
        assert_eq!(agents[0]["available"], true);
        assert_eq!(agents[0]["version"], "codex-cli 0.162.1");
        assert_eq!(agents[0]["default_model"], "gpt-6-astra");
        assert_eq!(agents[0]["models"].as_array().unwrap().len(), 5);
        assert_eq!(agents[0]["models"][0]["id"], "gpt-6-astra");
        assert_eq!(agents[0]["models"][0]["label"], "GPT-6 Astra");
        assert_eq!(agents[0]["models"][4]["label"], "o4-mini");

        assert_eq!(agents[1]["cli"], "kimi");
        assert_eq!(agents[1]["label"], "Kimi Code CLI");
        assert_eq!(agents[1]["command"], "kimi");
        assert_eq!(agents[1]["default_model"], "kimi-code/k3.1-pro");
        assert_eq!(agents[1]["models"].as_array().unwrap().len(), 6);
        assert_eq!(agents[1]["models"][5]["label"], "K2.7 Code Highspeed");
    }

    #[test]
    fn health_projection_is_the_compact_catalog() {
        let catalog = AgentCatalog::detect_with(&fake_detector());
        let value = catalog.health_json();
        assert_eq!(
            value,
            serde_json::json!([
                {"cli": "codex", "available": true, "version": "codex-cli 0.162.1"},
                {"cli": "kimi", "available": true, "version": "2.2.0"},
            ])
        );
    }

    #[test]
    fn missing_cli_is_unavailable() {
        struct Nothing;
        impl Detector for Nothing {
            fn version(&self, _command: &str) -> Option<String> {
                None
            }
            fn config(&self, _relative: &str) -> Option<String> {
                None
            }
        }
        let catalog = AgentCatalog::detect_with(&Nothing);
        assert!(catalog.agents().iter().all(|agent| !agent.available));
        assert!(catalog.agents().iter().all(|agent| agent.version.is_none()));
    }

    #[test]
    fn config_scan_reads_model_lines() {
        let codex = "approval_policy = \"never\"\nmodel = \"gpt-5.6-terra\" # default\n";
        assert_eq!(
            config_default_model(codex, "model").as_deref(),
            Some("gpt-5.6-terra")
        );
        let kimi = "[default]\ndefault_model = 'kimi-code/k3'\n";
        assert_eq!(
            config_default_model(kimi, "default_model").as_deref(),
            Some("kimi-code/k3")
        );
        // Keys that merely start with the same prefix must not match.
        assert_eq!(
            config_default_model("model_reasoning = \"high\"\n", "model"),
            None
        );
        assert_eq!(config_default_model("", "model"), None);
        assert_eq!(config_default_model("model = \"\"\n", "model"), None);
    }

    #[test]
    fn detected_default_model_replaces_the_static_one() {
        let mut detector = fake_detector();
        detector.configs.insert(
            ".kimi-code/config.toml".into(),
            "default_model = \"kimi-code/k3-256k\"\n".into(),
        );
        let catalog = AgentCatalog::detect_with(&detector);
        assert_eq!(
            catalog.get("kimi").unwrap().default_model,
            "kimi-code/k3-256k"
        );
        assert_eq!(catalog.get("codex").unwrap().default_model, "gpt-6-astra");
    }
}
