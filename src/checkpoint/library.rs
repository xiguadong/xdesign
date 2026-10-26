//! Project-level checkpoint library: reusable syntactic rule sets with their
//! executor scripts and semantic criteria, authored as Git-tracked data under
//! `checkpoints/`. Built-in entries are provided by the binary; user entries
//! live in `checkpoints/library.json` and generated scripts under
//! `checkpoints/scripts/`.

use crate::core::{valid_checker_id, valid_library_id, valid_relative_path, RsCheckerRuleSpec};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// The project-relative URI of the user-maintained library document.
pub const LIBRARY_URI: &str = "checkpoints/library.json";
/// Generated checker scripts live under this project-relative directory.
pub const SCRIPTS_DIRECTORY: &str = "checkpoints/scripts";
pub const LIBRARY_SCHEMA_VERSION: u32 = 1;
/// The library document keeps at most this many previous snapshots per entry.
pub const MAX_HISTORY: usize = 10;
/// Stable timestamp for binary-provided built-in entries.
const BUILTIN_TIMESTAMP: &str = "2026-10-26T00:00:00Z";

/// The five built-in checkpoint ids. They are reserved: a user entry can
/// neither reuse one of these ids nor overwrite/delete the built-in.
pub const BUILTIN_IDS: [&str; 5] = [
    "builtin-simple-requirements",
    "builtin-simple-design",
    "builtin-simple-implementation",
    "builtin-simple-record",
    "builtin-simple-verification",
];

/// Whether a checkpoint is a deterministic syntactic rule set or a semantic
/// judgment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsCheckpointKind {
    Syntactic,
    Semantic,
}

/// How an entry came to exist: binary built-in, scaffold-generated script, or
/// hand-authored.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsCheckpointSource {
    Builtin,
    Generated,
    Manual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsCheckpointExecutorMode {
    Script,
    Cli,
}

/// How a syntactic checkpoint is executed: an existing project `.py` script
/// that keeps the checker contract, or an agent CLI answering the rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCheckpointExecutor {
    pub mode: RsCheckpointExecutorMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

impl RsCheckpointExecutor {
    fn script(command_uri: &str) -> Self {
        Self {
            mode: RsCheckpointExecutorMode::Script,
            command_uri: Some(command_uri.to_owned()),
            files: vec![],
            python: None,
            cli: None,
            model: None,
        }
    }

    /// The project-relative script URI for a script executor.
    pub fn script_uri(&self) -> Option<&str> {
        match self.mode {
            RsCheckpointExecutorMode::Script => self.command_uri.as_deref(),
            RsCheckpointExecutorMode::Cli => None,
        }
    }
}

/// The permitted judge authorities of a semantic checkpoint.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCheckpointJudges {
    #[serde(default)]
    pub agent: bool,
    #[serde(default)]
    pub human: bool,
}

impl RsCheckpointJudges {
    fn any(&self) -> bool {
        self.agent || self.human
    }
}

/// One previous snapshot kept in the library document (newest first).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCheckpointHistory {
    pub version: u32,
    pub updated_at: String,
    pub description: String,
    pub kind: RsCheckpointKind,
    #[serde(default)]
    pub rules: Vec<RsCheckerRuleSpec>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor: Option<RsCheckpointExecutor>,
    #[serde(default)]
    pub criteria: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judges: Option<RsCheckpointJudges>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// One fully validated checkpoint library entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCheckpointEntry {
    pub checkpoint_id: String,
    pub title: String,
    pub description: String,
    pub kind: RsCheckpointKind,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub rules: Vec<RsCheckerRuleSpec>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor: Option<RsCheckpointExecutor>,
    /// Optional script identity for a reused checker. The frozen plan's
    /// `checker.checker_id` is this value when present; otherwise the
    /// checkpoint id is the identity. The library id still names provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checker_id: Option<String>,
    #[serde(default)]
    pub criteria: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judges: Option<RsCheckpointJudges>,
    pub version: u32,
    pub created_at: String,
    pub updated_at: String,
    pub source: RsCheckpointSource,
    #[serde(default)]
    pub history: Vec<RsCheckpointHistory>,
}

impl RsCheckpointEntry {
    fn snapshot(&self) -> RsCheckpointHistory {
        RsCheckpointHistory {
            version: self.version,
            updated_at: self.updated_at.clone(),
            description: self.description.clone(),
            kind: self.kind,
            rules: self.rules.clone(),
            inputs: self.inputs.clone(),
            executor: self.executor.clone(),
            criteria: self.criteria.clone(),
            judges: self.judges.clone(),
            tags: self.tags.clone(),
        }
    }

    /// The project-relative script URI, for script-mode syntactic entries.
    pub fn script_uri(&self) -> Option<&str> {
        self.executor
            .as_ref()
            .and_then(|executor| executor.script_uri())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RsCheckpointLibraryDocument {
    schema_version: u32,
    checkpoints: Vec<RsCheckpointEntry>,
}

/// The save request accepted by `xdesign checkpoint save --request FILE`.
///
/// The entry fields are authoritative. Server-managed members are tolerated
/// and ignored so a full `show` payload can be posted back unchanged.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCheckpointSaveRequest {
    pub checkpoint_id: String,
    pub title: String,
    pub description: String,
    pub kind: RsCheckpointKind,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub rules: Vec<RsCheckerRuleSpec>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub executor: Option<RsCheckpointExecutor>,
    #[serde(default)]
    pub checker_id: Option<String>,
    #[serde(default)]
    pub criteria: Vec<String>,
    #[serde(default)]
    pub judges: Option<RsCheckpointJudges>,
    #[serde(default)]
    pub create_script: Option<String>,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub source: Option<RsCheckpointSource>,
    #[serde(default)]
    pub history: Vec<serde_json::Value>,
    #[serde(default)]
    pub builtin: Option<bool>,
    #[serde(default)]
    pub script_exists: Option<bool>,
}

/// The successful outcome of a library save.
#[derive(Clone, Debug)]
pub struct RsCheckpointSaveOutcome {
    pub checkpoint: RsCheckpointEntry,
    pub script_uri: Option<String>,
    pub created: bool,
    pub version: u32,
}

/// A checkpoint-library command failure. The variant selects the stable
/// machine code and exit status exposed by the CLI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RsCheckpointError {
    /// Invalid CLI arguments or an unreadable request file.
    Request(String),
    /// Invalid entry, malformed library JSON, or an invalid library state.
    Invalid(String),
    /// A built-in entry cannot be changed through the library API.
    Builtin(String),
    /// A requested checkpoint id does not exist.
    Unknown(String),
    /// Operational failure (I/O) while reading or writing library material.
    Internal(String),
}

impl RsCheckpointError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Request(_) => "invalid_request",
            Self::Invalid(_) => "invalid_checkpoint",
            Self::Builtin(_) => "builtin_checkpoint",
            Self::Unknown(_) => "unknown_checkpoint",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Request(message)
            | Self::Invalid(message)
            | Self::Builtin(message)
            | Self::Unknown(message)
            | Self::Internal(message) => message,
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Internal(_) => 1,
            _ => 2,
        }
    }
}

impl std::fmt::Display for RsCheckpointError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

/// The effective project library: built-in entries followed by the validated
/// user entries of `checkpoints/library.json`.
pub struct RsCheckpointLibrary {
    root: PathBuf,
    builtins: Vec<RsCheckpointEntry>,
    users: Vec<RsCheckpointEntry>,
}

impl RsCheckpointLibrary {
    /// Load the project library rooted at `root`. A missing library file is
    /// the built-ins plus zero user entries; malformed JSON or an invalid
    /// entry is an explicit error.
    pub fn load(root: &Path) -> Result<Self, RsCheckpointError> {
        let builtins = builtin_entries();
        let path = root.join(LIBRARY_URI);
        let users = match fs::read(&path) {
            Ok(bytes) => {
                let document: RsCheckpointLibraryDocument = serde_json::from_slice(&bytes)
                    .map_err(|error| {
                        RsCheckpointError::Invalid(format!(
                            "malformed checkpoint library JSON: {error}"
                        ))
                    })?;
                if document.schema_version != LIBRARY_SCHEMA_VERSION {
                    return Err(RsCheckpointError::Invalid(format!(
                        "unsupported checkpoint library schema_version: {} (expected {LIBRARY_SCHEMA_VERSION})",
                        document.schema_version
                    )));
                }
                let mut seen = HashSet::new();
                for entry in &document.checkpoints {
                    if !seen.insert(entry.checkpoint_id.as_str()) {
                        return Err(RsCheckpointError::Invalid(format!(
                            "duplicate checkpoint id in library: {}",
                            entry.checkpoint_id
                        )));
                    }
                    if BUILTIN_IDS.contains(&entry.checkpoint_id.as_str()) {
                        return Err(RsCheckpointError::Invalid(format!(
                            "library entry reuses reserved built-in checkpoint id: {}",
                            entry.checkpoint_id
                        )));
                    }
                    validate_entry_shape(entry, root, true)?;
                }
                document.checkpoints
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => vec![],
            Err(error) => {
                return Err(RsCheckpointError::Internal(format!(
                    "cannot read {LIBRARY_URI}: {error}"
                )))
            }
        };
        Ok(Self {
            root: root.to_owned(),
            builtins,
            users,
        })
    }

    pub fn entries(&self) -> impl Iterator<Item = (&RsCheckpointEntry, bool)> {
        self.builtins
            .iter()
            .map(|entry| (entry, true))
            .chain(self.users.iter().map(|entry| (entry, false)))
    }

    pub fn entry(&self, checkpoint_id: &str) -> Option<(&RsCheckpointEntry, bool)> {
        if let Some(entry) = self
            .builtins
            .iter()
            .find(|entry| entry.checkpoint_id == checkpoint_id)
        {
            return Some((entry, true));
        }
        self.users
            .iter()
            .find(|entry| entry.checkpoint_id == checkpoint_id)
            .map(|entry| (entry, false))
    }

    /// The validated user entry with this id, if any.
    pub fn user_entry(&self, checkpoint_id: &str) -> Option<&RsCheckpointEntry> {
        self.users
            .iter()
            .find(|entry| entry.checkpoint_id == checkpoint_id)
    }

    pub fn is_builtin(&self, checkpoint_id: &str) -> bool {
        BUILTIN_IDS.contains(&checkpoint_id)
    }

    pub fn user_entries(&self) -> &[RsCheckpointEntry] {
        &self.users
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Whether the entry's script executor path exists in the project.
    pub fn script_exists(&self, entry: &RsCheckpointEntry) -> bool {
        entry.script_uri().is_some_and(|uri| {
            BUILTIN_IDS.contains(&entry.checkpoint_id.as_str())
                && crate::simple_flow::resources::is_builtin_command(uri)
                || self.root.join(uri).is_file()
        })
    }

    /// Non-fatal library concerns: currently entries whose script executor
    /// path no longer exists. Malformed library data stays a hard error.
    pub fn issues(&self) -> Vec<String> {
        self.entries()
            .filter(|(entry, _)| entry.script_uri().is_some())
            .filter(|(entry, _)| !self.script_exists(entry))
            .filter_map(|(entry, _)| {
                entry
                    .script_uri()
                    .map(|uri| format!("missing script: {uri}"))
            })
            .collect()
    }
}

/// The five built-in entries: the four SimpleFlow syntactic checkers with
/// rules and command URIs copied from the default plan, plus the SimpleFlow
/// semantic verification gate.
pub fn builtin_entries() -> Vec<RsCheckpointEntry> {
    let plan = crate::simple_flow::blueprint::designed_simple_flow_plan();
    let syntactic =
        |index: usize, id: &str, title: &str, description: &str, tags: &[&str], inputs: &[&str]| {
            let gate = &plan.checkpoints[index];
            let checker = gate
                .checker
                .as_ref()
                .expect("built-in syntactic gate carries a checker");
            RsCheckpointEntry {
                checkpoint_id: id.into(),
                title: title.into(),
                description: description.into(),
                kind: RsCheckpointKind::Syntactic,
                tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
                rules: checker.rules.clone(),
                inputs: inputs.iter().map(|input| (*input).to_owned()).collect(),
                executor: Some(RsCheckpointExecutor::script(&checker.command_uri)),
                checker_id: Some(checker.checker_id.clone()),
                criteria: vec![],
                judges: None,
                version: 1,
                created_at: BUILTIN_TIMESTAMP.into(),
                updated_at: BUILTIN_TIMESTAMP.into(),
                source: RsCheckpointSource::Builtin,
                history: vec![],
            }
        };
    let semantic_gate = &plan.checkpoints[4];
    vec![
        syntactic(
            0,
            "builtin-simple-requirements",
            "SimpleFlow requirements rationale and satisfy links",
            "Deterministic SimpleFlow requirements gate: every declared requirement carries its rationale and every satisfy link resolves to a declared requirement.",
            &["simple-flow", "requirements"],
            &["mbse/model/requirements.sysml", "mbse/model/traceability.sysml"],
        ),
        syntactic(
            1,
            "builtin-simple-design",
            "SimpleFlow design declarations and views",
            "Deterministic SimpleFlow design gate: declarations, gate bindings, documentation sections, checker rule sets and the nine carried views.",
            &["simple-flow", "design"],
            &[
                "mbse/model/detailedDesign.sysml",
                "mbse/model/composition.sysml",
            ],
        ),
        syntactic(
            2,
            "builtin-simple-implementation",
            "SimpleFlow implementation artifacts exist",
            "Deterministic SimpleFlow implementation gate: the current visit records at least one artifact and every recorded artifact exists.",
            &["simple-flow", "implementation"],
            &["Cargo.toml"],
        ),
        syntactic(
            3,
            "builtin-simple-record",
            "SimpleFlow verification record is authorized",
            "Deterministic SimpleFlow record gate: every stage is visited, completed visits are authorized, and passed assessments retain evidence.",
            &["simple-flow", "record"],
            &["Cargo.toml"],
        ),
        RsCheckpointEntry {
            checkpoint_id: "builtin-simple-verification".into(),
            title: "SimpleFlow verification evidence".into(),
            description: "Semantic SimpleFlow verification gate: the retained evidence demonstrates that the implementation satisfies the requirements and stands on its own.".into(),
            kind: RsCheckpointKind::Semantic,
            tags: vec!["simple-flow".into(), "verification".into()],
            rules: vec![],
            inputs: vec![],
            executor: None,
            checker_id: None,
            criteria: semantic_gate.criteria.clone(),
            judges: Some(RsCheckpointJudges {
                agent: true,
                human: true,
            }),
            version: 1,
            created_at: BUILTIN_TIMESTAMP.into(),
            updated_at: BUILTIN_TIMESTAMP.into(),
            source: RsCheckpointSource::Builtin,
            history: vec![],
        },
    ]
}

/// The project-relative scaffold script path for a checkpoint id:
/// `checkpoints/scripts/<id with '-' replaced by '_'>.py`.
pub fn expected_scaffold_uri(checkpoint_id: &str) -> String {
    format!("{SCRIPTS_DIRECTORY}/{}.py", checkpoint_id.replace('-', "_"))
}

/// Whether `uri` is a normalized project-relative path.
pub fn normalized_project_uri(uri: &str) -> bool {
    valid_relative_path(uri) && !uri.contains('\\') && !uri.contains('\0')
}

fn valid_tag(tag: &str) -> bool {
    valid_library_id(tag)
}

fn validate_entry_shape(
    entry: &RsCheckpointEntry,
    root: &Path,
    script_must_exist: bool,
) -> Result<(), RsCheckpointError> {
    if !valid_library_id(&entry.checkpoint_id) {
        return Err(RsCheckpointError::Invalid(format!(
            "invalid checkpoint id: {} (expected [a-z0-9][a-z0-9_-]{{0,63}})",
            entry.checkpoint_id
        )));
    }
    let title = entry.title.trim();
    if title.is_empty() || title.chars().count() > 120 {
        return Err(RsCheckpointError::Invalid(format!(
            "checkpoint {}: title must be 1–120 characters",
            entry.checkpoint_id
        )));
    }
    let description = entry.description.trim();
    if description.is_empty() || description.chars().count() > 1000 {
        return Err(RsCheckpointError::Invalid(format!(
            "checkpoint {}: description must be 1–1000 characters",
            entry.checkpoint_id
        )));
    }
    if entry.tags.len() > 8 {
        return Err(RsCheckpointError::Invalid(format!(
            "checkpoint {}: at most 8 tags are allowed",
            entry.checkpoint_id
        )));
    }
    if let Some(tag) = entry.tags.iter().find(|tag| !valid_tag(tag)) {
        return Err(RsCheckpointError::Invalid(format!(
            "checkpoint {}: invalid tag: {tag}",
            entry.checkpoint_id
        )));
    }
    if entry.version == 0 {
        return Err(RsCheckpointError::Invalid(format!(
            "checkpoint {}: version must be at least 1",
            entry.checkpoint_id
        )));
    }
    if entry.created_at.trim().is_empty() || entry.updated_at.trim().is_empty() {
        return Err(RsCheckpointError::Invalid(format!(
            "checkpoint {}: created_at and updated_at are required",
            entry.checkpoint_id
        )));
    }
    if entry.history.len() > MAX_HISTORY {
        return Err(RsCheckpointError::Invalid(format!(
            "checkpoint {}: history is capped at {MAX_HISTORY} entries",
            entry.checkpoint_id
        )));
    }
    for previous in &entry.history {
        if previous.version == 0
            || previous.updated_at.trim().is_empty()
            || previous.description.trim().is_empty()
        {
            return Err(RsCheckpointError::Invalid(format!(
                "checkpoint {}: malformed history snapshot",
                entry.checkpoint_id
            )));
        }
    }
    match entry.kind {
        RsCheckpointKind::Syntactic => {
            if let Some(checker_id) = entry.checker_id.as_deref() {
                if checker_id.trim().is_empty() || !valid_checker_id(checker_id) {
                    return Err(RsCheckpointError::Invalid(format!(
                        "checkpoint {}: invalid checker_id (expected [A-Za-z0-9][A-Za-z0-9_-]{{0,127}})",
                        entry.checkpoint_id
                    )));
                }
            }
            if entry
                .criteria
                .iter()
                .any(|criterion| !criterion.trim().is_empty())
                || entry.judges.as_ref().is_some_and(RsCheckpointJudges::any)
            {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a syntactic checkpoint cannot carry criteria or judges",
                    entry.checkpoint_id
                )));
            }
            if entry.rules.is_empty() {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a syntactic checkpoint needs at least one rule",
                    entry.checkpoint_id
                )));
            }
            let mut rule_ids = HashSet::new();
            for rule in &entry.rules {
                if rule.rule_id.trim().is_empty()
                    || rule.condition.trim().is_empty()
                    || rule.reads.is_empty()
                    || rule.reads.iter().any(|read| read.trim().is_empty())
                {
                    return Err(RsCheckpointError::Invalid(format!(
                        "checkpoint {}: rule {} is incomplete",
                        entry.checkpoint_id, rule.rule_id
                    )));
                }
                if !rule_ids.insert(rule.rule_id.as_str()) {
                    return Err(RsCheckpointError::Invalid(format!(
                        "checkpoint {}: duplicate rule id: {}",
                        entry.checkpoint_id, rule.rule_id
                    )));
                }
            }
            if entry.inputs.is_empty() {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a syntactic checkpoint needs at least one input",
                    entry.checkpoint_id
                )));
            }
            let mut seen = HashSet::new();
            for input in &entry.inputs {
                if !normalized_project_uri(input) || !seen.insert(input.as_str()) {
                    return Err(RsCheckpointError::Invalid(format!(
                        "checkpoint {}: invalid or duplicate input: {input}",
                        entry.checkpoint_id
                    )));
                }
                if !root.join(input).is_file() {
                    return Err(RsCheckpointError::Invalid(format!(
                        "checkpoint {}: input does not exist: {input}",
                        entry.checkpoint_id
                    )));
                }
            }
            match entry.executor.as_ref() {
                Some(executor) => {
                    validate_executor_shape(entry, executor, root, script_must_exist)?
                }
                None => {
                    return Err(RsCheckpointError::Invalid(format!(
                        "checkpoint {}: a syntactic checkpoint needs an executor",
                        entry.checkpoint_id
                    )))
                }
            }
        }
        RsCheckpointKind::Semantic => {
            if !entry.rules.is_empty() || !entry.inputs.is_empty() || entry.executor.is_some() {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a semantic checkpoint cannot carry rules, inputs or an executor",
                    entry.checkpoint_id
                )));
            }
            if entry.checker_id.is_some() {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a semantic checkpoint cannot carry checker_id",
                    entry.checkpoint_id
                )));
            }
            if entry.criteria.is_empty()
                || entry
                    .criteria
                    .iter()
                    .any(|criterion| criterion.trim().is_empty())
            {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a semantic checkpoint needs at least one criterion",
                    entry.checkpoint_id
                )));
            }
            if !entry.judges.as_ref().is_some_and(RsCheckpointJudges::any) {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a semantic checkpoint needs a judges object with at least one permitted judge",
                    entry.checkpoint_id
                )));
            }
        }
    }
    Ok(())
}

fn validate_executor_shape(
    entry: &RsCheckpointEntry,
    executor: &RsCheckpointExecutor,
    root: &Path,
    script_must_exist: bool,
) -> Result<(), RsCheckpointError> {
    match executor.mode {
        RsCheckpointExecutorMode::Script => {
            if executor.cli.is_some() || executor.model.is_some() {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a script executor cannot carry cli or model",
                    entry.checkpoint_id
                )));
            }
            let Some(uri) = executor.command_uri.as_deref() else {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a script executor needs a command_uri",
                    entry.checkpoint_id
                )));
            };
            if !normalized_project_uri(uri) || !uri.ends_with(".py") {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: command_uri must be a normalized project-relative .py path: {uri}",
                    entry.checkpoint_id
                )));
            }
            let mut seen = HashSet::from([uri]);
            for file in &executor.files {
                if !normalized_project_uri(file)
                    || !seen.insert(file.as_str())
                    || script_must_exist && !root.join(file).is_file()
                {
                    return Err(RsCheckpointError::Invalid(format!(
                        "invalid, duplicate or missing checker dependency: {file}"
                    )));
                }
            }
            if executor
                .python
                .as_deref()
                .is_some_and(|p| !normalized_project_uri(p) || !p.starts_with("cache/"))
            {
                return Err(RsCheckpointError::Invalid(
                    "checker python must be project-relative under cache/".into(),
                ));
            }
            if script_must_exist && !root.join(uri).is_file() {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: script does not exist: {uri}",
                    entry.checkpoint_id
                )));
            }
        }
        RsCheckpointExecutorMode::Cli => {
            if executor.command_uri.is_some()
                || !executor.files.is_empty()
                || executor.python.is_some()
            {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a cli executor cannot carry a command_uri",
                    entry.checkpoint_id
                )));
            }
            let cli = executor
                .cli
                .as_deref()
                .map(str::trim)
                .filter(|cli| !cli.is_empty());
            if !matches!(cli, Some("codex" | "kimi")) {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a cli executor requires cli codex or kimi",
                    entry.checkpoint_id
                )));
            }
            if executor
                .model
                .as_deref()
                .is_none_or(|model| model.trim().is_empty())
            {
                return Err(RsCheckpointError::Invalid(format!(
                    "checkpoint {}: a cli executor requires a nonempty model",
                    entry.checkpoint_id
                )));
            }
        }
    }
    Ok(())
}

/// Save one checkpoint entry into the user library under `root`. Saving an
/// existing user id is an update that increments the version and keeps at most
/// [`MAX_HISTORY`] previous snapshots, newest first. A built-in id is refused.
pub fn save(
    root: &Path,
    request: RsCheckpointSaveRequest,
) -> Result<RsCheckpointSaveOutcome, RsCheckpointError> {
    if !valid_library_id(&request.checkpoint_id) {
        return Err(RsCheckpointError::Invalid(format!(
            "invalid checkpoint id: {} (expected [a-z0-9][a-z0-9_-]{{0,63}})",
            request.checkpoint_id
        )));
    }
    if BUILTIN_IDS.contains(&request.checkpoint_id.as_str()) {
        return Err(RsCheckpointError::Builtin(format!(
            "built-in checkpoint {} is read-only",
            request.checkpoint_id
        )));
    }
    let scaffold = match request.create_script.as_deref() {
        None => false,
        Some("scaffold") => true,
        Some(other) => {
            return Err(RsCheckpointError::Invalid(format!(
                "unknown create_script action: {other} (expected \"scaffold\")"
            )))
        }
    };
    let library = RsCheckpointLibrary::load(root)?;
    let existing = library.user_entry(&request.checkpoint_id).cloned();
    if scaffold {
        let executor = request.executor.as_ref().filter(|executor| {
            executor.mode == RsCheckpointExecutorMode::Script
                && executor.command_uri.as_deref()
                    == Some(expected_scaffold_uri(&request.checkpoint_id).as_str())
        });
        if request.kind != RsCheckpointKind::Syntactic || executor.is_none() {
            return Err(RsCheckpointError::Invalid(format!(
                "checkpoint {}: create_script \"scaffold\" requires a syntactic checkpoint with a script executor at {}",
                request.checkpoint_id,
                expected_scaffold_uri(&request.checkpoint_id)
            )));
        }
    }
    let now = utc_now();
    let (version, created_at, source, history) = match &existing {
        Some(previous) => {
            let mut history = Vec::with_capacity(previous.history.len() + 1);
            history.push(previous.snapshot());
            history.extend(previous.history.iter().cloned());
            history.truncate(MAX_HISTORY);
            let source = if scaffold {
                RsCheckpointSource::Generated
            } else {
                previous.source
            };
            (
                previous.version + 1,
                previous.created_at.clone(),
                source,
                history,
            )
        }
        None => (
            1,
            now.clone(),
            if scaffold {
                RsCheckpointSource::Generated
            } else {
                RsCheckpointSource::Manual
            },
            vec![],
        ),
    };
    let entry = RsCheckpointEntry {
        checkpoint_id: request.checkpoint_id.clone(),
        title: request.title.trim().to_owned(),
        description: request.description.trim().to_owned(),
        kind: request.kind,
        tags: request.tags.clone(),
        rules: request.rules.clone(),
        inputs: request.inputs.clone(),
        executor: request.executor.clone(),
        checker_id: request.checker_id.clone(),
        criteria: request.criteria.clone(),
        judges: request.judges.clone(),
        version,
        created_at,
        updated_at: now,
        source,
        history,
    };
    // The script is generated before the library commit so the saved entry
    // always names a protocol-compatible file. A failed library write leaves at
    // worst an unused deterministic scaffold script behind.
    validate_entry_shape(&entry, root, !scaffold)?;
    if scaffold {
        let script_uri = entry
            .script_uri()
            .expect("validated scaffold script executor")
            .to_owned();
        let content = scaffold_script(&entry);
        write_atomic(root, &script_uri, content.as_bytes())?;
    }
    let mut users = library.user_entries().to_vec();
    match users
        .iter_mut()
        .find(|user| user.checkpoint_id == entry.checkpoint_id)
    {
        Some(user) => *user = entry.clone(),
        None => users.push(entry.clone()),
    }
    let document = RsCheckpointLibraryDocument {
        schema_version: LIBRARY_SCHEMA_VERSION,
        checkpoints: users,
    };
    let bytes = serde_json::to_vec_pretty(&document).map_err(|error| {
        RsCheckpointError::Internal(format!("cannot serialize checkpoint library: {error}"))
    })?;
    write_atomic(root, LIBRARY_URI, &bytes)?;
    Ok(RsCheckpointSaveOutcome {
        script_uri: entry.script_uri().map(str::to_owned),
        created: existing.is_none(),
        version: entry.version,
        checkpoint: entry,
    })
}

fn write_atomic(root: &Path, uri: &str, bytes: &[u8]) -> Result<(), RsCheckpointError> {
    if !normalized_project_uri(uri) {
        return Err(RsCheckpointError::Internal(format!(
            "refusing to write unnormalized library path: {uri}"
        )));
    }
    let path = root.join(uri);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            RsCheckpointError::Internal(format!("cannot create directory for {uri}: {error}"))
        })?;
    }
    let temporary = path.with_extension(format!(
        "{}.{}.tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default(),
        uuid::Uuid::new_v4()
    ));
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| {
                RsCheckpointError::Internal(format!("cannot create {uri} staging file: {error}"))
            })?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| {
                RsCheckpointError::Internal(format!("cannot write {uri} staging file: {error}"))
            })?;
    }
    fs::rename(&temporary, &path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        RsCheckpointError::Internal(format!("cannot publish {uri}: {error}"))
    })?;
    if let Some(parent) = path.parent() {
        if let Ok(directory) = fs::File::open(parent) {
            let _ = directory.sync_all();
        }
    }
    Ok(())
}

/// A deterministic, self-contained Python checker scaffold for a syntactic
/// script-executor entry. It supports `--list-inputs` and the full checker
/// invocation contract, verifies the plan/record/visit binding, and reports one
/// honest failed verdict per declared rule until the author implements it.
pub fn scaffold_script(entry: &RsCheckpointEntry) -> String {
    let inputs = serde_json::to_string_pretty(&entry.inputs).unwrap_or_else(|_| "[]".into());
    let rules = serde_json::to_string_pretty(&entry.rules).unwrap_or_else(|_| "[]".into());
    let mut script = String::new();
    script.push_str("#!/usr/bin/env python3\n");
    script.push_str(&format!(
        "\"\"\"Generated checker scaffold for checkpoint library entry `{}`.\n\n\
         {}\n\n\
         TODO: implement each rule function below. Until then every rule honestly\n\
         reports a failed verdict with \"rule not implemented: <rule_id>\".\n\
         \"\"\"\n\n\
         from __future__ import annotations\n\n\
         import argparse\n\
         import hashlib\n\
         import json\n\
         import sys\n\
         from pathlib import Path\n\n",
        entry.checkpoint_id,
        entry.description.replace('\n', " "),
    ));
    script.push_str(&format!(
        "CHECKPOINT_ID = {}\n",
        python_string(&entry.checkpoint_id)
    ));
    script.push_str(&format!("TITLE = {}\n", python_string(&entry.title)));
    script.push_str(&format!("DECLARED_INPUTS = {inputs}\n"));
    // The pretty JSON of the declared rules is a valid Python literal (only
    // string, list and object values are used).
    script.push_str(&format!("RULES = {rules}\n"));
    let mut function_names = Vec::new();
    for (index, rule) in entry.rules.iter().enumerate() {
        let function_name = format!("rule_{index}_{}", sanitize_identifier(&rule.rule_id));
        function_names.push((rule.rule_id.clone(), function_name.clone()));
        script.push_str(&format!("\n\ndef {function_name}(context):\n"));
        script.push_str(&format!("    # condition: {}\n", one_line(&rule.condition)));
        script.push_str(&format!(
            "    # reads: {}\n",
            one_line(&rule.reads.join(", "))
        ));
        script.push_str(&format!(
            "    return False, [{}]\n",
            python_string(&format!("rule not implemented: {}", rule.rule_id))
        ));
    }
    script.push_str("\n\nRULE_FUNCTIONS = {\n");
    for (rule_id, function_name) in &function_names {
        script.push_str(&format!(
            "    {}: {},\n",
            python_string(rule_id),
            function_name
        ));
    }
    script.push_str("}\n");
    script.push_str(SCAFFOLD_RUNTIME);
    script
}

fn python_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".into())
}

fn sanitize_identifier(value: &str) -> String {
    let mut sanitized: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        sanitized.push_str("rule");
    }
    sanitized
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

const SCAFFOLD_RUNTIME: &str = r#"

def sha256_file(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def relative_uri(path_text):
    path = Path(path_text)
    if path.is_absolute():
        try:
            path = path.relative_to(Path.cwd())
        except ValueError:
            unevaluated("path is outside the project: " + path_text)
    return path.as_posix()


def unevaluated(message):
    print(CHECKPOINT_ID + ": unevaluated: " + message, file=sys.stderr)
    raise SystemExit(2)


def parse_arguments():
    parser = argparse.ArgumentParser(description="Run checker scaffold " + CHECKPOINT_ID)
    parser.add_argument("--list-inputs", action="store_true")
    parser.add_argument("--wish-model")
    parser.add_argument("--record")
    parser.add_argument("--checker-id")
    parser.add_argument("--plan-id")
    parser.add_argument("--plan-uri")
    parser.add_argument("--plan-sha256")
    parser.add_argument("--stage")
    parser.add_argument("--artifact-revision", type=int)
    parser.add_argument("--snapshot-digest")
    parser.add_argument("--report")
    return parser.parse_args()


def load_context(args):
    for name in (
        "wish_model",
        "record",
        "checker_id",
        "plan_id",
        "plan_uri",
        "plan_sha256",
        "stage",
        "artifact_revision",
        "snapshot_digest",
        "report",
    ):
        if getattr(args, name) is None:
            unevaluated("--" + name.replace("_", "-") + " is required")
    if not args.checker_id.strip():
        unevaluated("--checker-id must be nonempty")
    plan_path = Path(args.plan_uri)
    if not plan_path.is_file():
        unevaluated("frozen plan is missing: " + args.plan_uri)
    plan_bytes = plan_path.read_bytes()
    if hashlib.sha256(plan_bytes).hexdigest() != args.plan_sha256:
        unevaluated("frozen plan digest differs")
    plan = json.loads(plan_bytes)
    if plan.get("plan_id") != args.plan_id:
        unevaluated("frozen plan id differs")
    record_path = Path(args.record)
    if not record_path.is_file():
        unevaluated("run record snapshot is missing: " + args.record)
    record = json.loads(record_path.read_bytes())
    if (
        record.get("plan_id") != args.plan_id
        or record.get("plan_uri") != args.plan_uri
        or record.get("plan_sha256") != args.plan_sha256
    ):
        unevaluated("run record does not match the frozen plan")
    visits = record.get("visits")
    index = record.get("current_visit")
    if not isinstance(visits, list) or not isinstance(index, int) or not (0 <= index < len(visits)):
        unevaluated("run record has no current visit")
    visit = visits[index]
    if visit.get("stage_id") != args.stage:
        unevaluated("current visit stage differs")
    if visit.get("artifact_revision") != args.artifact_revision:
        unevaluated("current visit artifact revision differs")
    if visit.get("artifact_snapshot_digest") != args.snapshot_digest:
        unevaluated("current visit snapshot digest differs")
    artifacts = visit.get("artifact_digests")
    if not isinstance(artifacts, list):
        unevaluated("current visit artifacts are malformed")
    canonical = []
    for item in artifacts:
        uri = item.get("uri")
        digest = item.get("sha256")
        path = Path(uri)
        if not path.is_file():
            unevaluated("recorded artifact is missing: " + str(uri))
        if sha256_file(path) != digest:
            unevaluated("recorded artifact digest differs: " + str(uri))
        canonical.append((uri, digest))
    canonical.sort()
    snapshot = hashlib.sha256(
        "".join(uri + "\0" + digest + "\n" for uri, digest in canonical).encode("utf-8")
    ).hexdigest()
    if snapshot != args.snapshot_digest:
        unevaluated("artifact snapshot digest differs")
    return {"args": args, "plan": plan, "record": record, "visit": visit}


def rule_inputs(reads, context):
    inputs = []

    def add(uri):
        if isinstance(uri, str) and uri.strip() and uri not in inputs:
            inputs.append(uri)

    for read in reads:
        if read == "runtime run record JSON":
            add(context["args"].record)
        elif read == "retained plan serialization":
            add(context["args"].plan_uri)
        elif read == "recorded artifact URIs":
            for artifact in context["visit"].get("artifact_digests", []):
                add(artifact.get("uri"))
        elif read == "recorded evidence URIs":
            for visit in context["record"].get("visits", []):
                for assessment in visit.get("assessments", []):
                    if assessment.get("outcome") == "passed":
                        for uri in assessment.get("evidence_uris", []):
                            add(uri)
        elif read.endswith("/"):
            for uri in DECLARED_INPUTS:
                if uri.startswith(read):
                    add(uri)
        else:
            add(read)
    if not inputs:
        add(context["args"].wish_model)
    return inputs


def main():
    args = parse_arguments()
    if args.list_inputs:
        print(json.dumps(sorted(set(DECLARED_INPUTS))))
        return 0
    context = load_context(args)
    results = []
    for declaration in RULES:
        passed, failures = RULE_FUNCTIONS[declaration["rule_id"]](context)
        results.append(
            {
                "rule_id": declaration["rule_id"],
                "verdict": "passed" if passed else "failed",
                "inputs": rule_inputs(declaration["reads"], context),
                "observed": {
                    "scaffold": True,
                    "stage": args.stage,
                    "declared_inputs": DECLARED_INPUTS,
                },
                "failures": [] if passed else failures,
            }
        )
    digest_uris = []
    for uri in [args.wish_model, args.record, args.plan_uri, sys.argv[0], *DECLARED_INPUTS]:
        candidate = relative_uri(uri)
        if candidate not in digest_uris:
            digest_uris.append(candidate)
    for result in results:
        for uri in result["inputs"]:
            if Path(uri).is_file():
                candidate = relative_uri(uri)
                if candidate not in digest_uris:
                    digest_uris.append(candidate)
    input_sha256 = {}
    for uri in digest_uris:
        path = Path(uri)
        if not path.is_file():
            unevaluated("required checker input is missing: " + uri)
        input_sha256[uri] = sha256_file(path)
    report = {
        "checker_id": args.checker_id,
        "stage": args.stage,
        "artifact_revision": args.artifact_revision,
        "snapshot_digest": args.snapshot_digest,
        "report_uri": relative_uri(args.report),
        "overall_verdict": "passed"
        if all(result["verdict"] == "passed" for result in results)
        else "failed",
        "rules": results,
        "input_sha256": input_sha256,
    }
    report_path = Path(args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#;

/// UTC timestamp in RFC 3339 form without pulling in a time crate.
fn utc_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_utc(seconds)
}

fn format_utc(seconds: i64) -> String {
    let days = seconds.div_euclid(86_400);
    let remainder = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        remainder / 3_600,
        (remainder % 3_600) / 60,
        remainder % 60
    )
}

/// Howard Hinnant's civil-from-days algorithm (proleptic Gregorian).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = (shifted - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * month_prime + 2) / 5 + 1) as u32;
    let month = if month_prime < 10 {
        month_prime + 3
    } else {
        month_prime - 9
    } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::RsCheckerRuleSpec;
    use std::path::PathBuf;

    fn temp_project(name: &str) -> PathBuf {
        let root = std::env::current_dir().unwrap().join(format!(
            "cache/implementation/checkpoint-tests/{name}-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_project_file(root: &Path, uri: &str, contents: &str) {
        let path = root.join(uri);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    fn syntactic_request(
        root: &Path,
        checkpoint_id: &str,
        script_uri: &str,
        input_uri: &str,
    ) -> RsCheckpointSaveRequest {
        write_project_file(root, input_uri, "package Test {}\n");
        write_project_file(root, script_uri, "print('checker')\n");
        RsCheckpointSaveRequest {
            checkpoint_id: checkpoint_id.into(),
            title: format!("Checkpoint {checkpoint_id}"),
            description: "Deterministic test checkpoint.".into(),
            kind: RsCheckpointKind::Syntactic,
            tags: vec!["test".into()],
            rules: vec![RsCheckerRuleSpec {
                rule_id: "rule-one".into(),
                condition: "The rule holds.".into(),
                reads: vec![input_uri.into()],
            }],
            inputs: vec![input_uri.into()],
            executor: Some(RsCheckpointExecutor::script(script_uri)),
            checker_id: None,
            criteria: vec![],
            judges: None,
            create_script: None,
            version: None,
            created_at: None,
            updated_at: None,
            source: None,
            history: vec![],
            builtin: None,
            script_exists: None,
        }
    }

    fn semantic_request(checkpoint_id: &str) -> RsCheckpointSaveRequest {
        RsCheckpointSaveRequest {
            checkpoint_id: checkpoint_id.into(),
            title: format!("Semantic {checkpoint_id}"),
            description: "Semantic test checkpoint.".into(),
            kind: RsCheckpointKind::Semantic,
            tags: vec![],
            rules: vec![],
            inputs: vec![],
            executor: None,
            checker_id: None,
            criteria: vec!["Every requirement is answered by retained evidence.".into()],
            judges: Some(RsCheckpointJudges {
                agent: true,
                human: false,
            }),
            create_script: None,
            version: None,
            created_at: None,
            updated_at: None,
            source: None,
            history: vec![],
            builtin: None,
            script_exists: None,
        }
    }

    #[test]
    fn missing_library_is_builtins_only_and_builtins_are_read_only() {
        let root = temp_project("missing-library");
        let library = RsCheckpointLibrary::load(&root).unwrap();
        let builtins: Vec<_> = library.entries().collect();
        assert_eq!(builtins.len(), BUILTIN_IDS.len());
        assert!(builtins.iter().all(|(_, builtin)| *builtin));
        assert_eq!(
            builtins
                .iter()
                .filter(|(entry, _)| entry.kind == RsCheckpointKind::Syntactic)
                .count(),
            4
        );
        let (requirements, _) = library.entry("builtin-simple-requirements").unwrap();
        assert_eq!(
            requirements.checker_id.as_deref(),
            Some("simpleRequirementsCheck"),
            "built-in syntactic entries carry the shipped script identity"
        );
        let (semantic, _) = library.entry("builtin-simple-verification").unwrap();
        assert_eq!(semantic.kind, RsCheckpointKind::Semantic);
        assert!(semantic.rules.is_empty());
        assert!(!semantic.criteria.is_empty());
        assert!(semantic.checker_id.is_none());
        let request = syntactic_request(
            &root,
            "reserved-id-attempt",
            "tools/checks/simple_requirements.py",
            "mbse/model/requirements.sysml",
        );
        let mut builtin_request = request.clone();
        builtin_request.checkpoint_id = "builtin-simple-requirements".into();
        assert!(matches!(
            save(&root, builtin_request),
            Err(RsCheckpointError::Builtin(_))
        ));
        assert!(library.is_builtin("builtin-simple-record"));
        assert!(!library.is_builtin("reserved-id-attempt"));
    }

    #[test]
    fn save_round_trip_versions_and_caps_history() {
        let root = temp_project("history");
        let request = syntactic_request(
            &root,
            "history-check",
            "tools/checks/simple_requirements.py",
            "mbse/model/requirements.sysml",
        );
        let first = save(&root, request.clone()).unwrap();
        assert!(first.created);
        assert_eq!(first.version, 1);
        assert_eq!(first.checkpoint.history.len(), 0);
        let mut latest = first.clone();
        for index in 2..=12 {
            let mut update = request.clone();
            update.title = format!("History update {index}");
            latest = save(&root, update).unwrap();
            assert!(!latest.created);
            assert_eq!(latest.version, index);
        }
        assert_eq!(latest.version, 12);
        assert_eq!(latest.checkpoint.history.len(), MAX_HISTORY);
        assert_eq!(latest.checkpoint.history[0].version, 11);
        assert_eq!(latest.checkpoint.history[9].version, 2);
        assert_eq!(latest.checkpoint.created_at, first.checkpoint.created_at);
        assert_ne!(latest.checkpoint.updated_at, String::new());
        let library = RsCheckpointLibrary::load(&root).unwrap();
        let (stored, builtin) = library.entry("history-check").unwrap();
        assert!(!builtin);
        assert_eq!(stored.version, 12);
        assert_eq!(stored.history.len(), MAX_HISTORY);
        let raw: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(LIBRARY_URI)).unwrap()).unwrap();
        assert_eq!(raw["schema_version"], 1);
        assert_eq!(raw["checkpoints"].as_array().unwrap().len(), 1);
        assert_eq!(
            raw["checkpoints"][0]["history"].as_array().unwrap().len(),
            10
        );
        assert_eq!(raw["checkpoints"][0]["source"], "manual");
    }

    #[test]
    fn malformed_library_and_invalid_entries_are_explicit_errors() {
        let root = temp_project("malformed");
        write_project_file(&root, LIBRARY_URI, "{ not json");
        assert!(matches!(
            RsCheckpointLibrary::load(&root),
            Err(RsCheckpointError::Invalid(_))
        ));
        write_project_file(
            &root,
            LIBRARY_URI,
            "{\"schema_version\":2,\"checkpoints\":[]}",
        );
        assert!(matches!(
            RsCheckpointLibrary::load(&root),
            Err(RsCheckpointError::Invalid(_))
        ));
        fs::remove_file(root.join(LIBRARY_URI)).unwrap();
        // A syntactically valid document with a semantically invalid entry.
        let request = syntactic_request(
            &root,
            "bad-entry",
            "tools/checks/simple_requirements.py",
            "mbse/model/requirements.sysml",
        );
        save(&root, request).unwrap();
        let mut document: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(LIBRARY_URI)).unwrap()).unwrap();
        document["checkpoints"][0]["rules"] = serde_json::json!([]);
        write_project_file(
            &root,
            LIBRARY_URI,
            &serde_json::to_string(&document).unwrap(),
        );
        assert!(matches!(
            RsCheckpointLibrary::load(&root),
            Err(RsCheckpointError::Invalid(_))
        ));
    }

    fn expect_invalid(result: Result<RsCheckpointSaveOutcome, RsCheckpointError>) {
        match result {
            Err(RsCheckpointError::Invalid(message)) => {
                assert!(!message.trim().is_empty());
            }
            other => panic!("expected an invalid-checkpoint error, got {other:?}"),
        }
    }

    #[test]
    fn entry_validation_rejects_bad_definitions() {
        let root = temp_project("validation");
        let base = syntactic_request(
            &root,
            "valid-entry",
            "tools/checks/simple_requirements.py",
            "mbse/model/requirements.sysml",
        );
        let mut invalid = base.clone();
        invalid.rules.clear();
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.inputs.clear();
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.executor = None;
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.criteria = vec!["not allowed".into()];
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.tags = vec!["Not A Slug".into()];
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.title = "x".repeat(121);
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.inputs = vec!["missing/file.sysml".into()];
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.rules[0].rule_id = "".into();
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.rules.push(base.rules[0].clone());
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.executor = Some(RsCheckpointExecutor {
            files: vec![],
            python: None,
            mode: RsCheckpointExecutorMode::Cli,
            command_uri: None,
            cli: Some("other".into()),
            model: Some("model".into()),
        });
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.executor = Some(RsCheckpointExecutor::script(
            "tools/checks/missing_checker.py",
        ));
        expect_invalid(save(&root, invalid));
        let mut invalid = base.clone();
        invalid.executor = Some(RsCheckpointExecutor {
            files: vec![],
            python: None,
            mode: RsCheckpointExecutorMode::Cli,
            command_uri: None,
            cli: Some("codex".into()),
            model: Some("model".into()),
        });
        invalid.checker_id = Some("shared agent identity".into());
        expect_invalid(save(&root, invalid));
        // A syntactic entry may reuse a checker script identity.
        let mut reused = base.clone();
        reused.checker_id = Some("simpleRequirementsCheck".into());
        let saved = save(&root, reused).unwrap();
        assert_eq!(
            saved.checkpoint.checker_id.as_deref(),
            Some("simpleRequirementsCheck")
        );
        // Semantic entries cannot carry a checker identity.
        let mut semantic = semantic_request("semantic-checker-id");
        semantic.checker_id = Some("someChecker".into());
        expect_invalid(save(&root, semantic));
        let mut semantic = semantic_request("bad-semantic");
        semantic.rules = base.rules.clone();
        expect_invalid(save(&root, semantic));
        let mut semantic = semantic_request("bad-judges");
        semantic.judges = Some(RsCheckpointJudges::default());
        expect_invalid(save(&root, semantic));
        let mut semantic = semantic_request("empty-criteria");
        semantic.criteria.clear();
        expect_invalid(save(&root, semantic));
        // A valid manual semantic entry saves and is listable.
        let saved = save(&root, semantic_request("semantic-ok")).unwrap();
        assert!(saved.created);
        let library = RsCheckpointLibrary::load(&root).unwrap();
        let (stored, builtin) = library.entry("semantic-ok").unwrap();
        assert!(!builtin);
        assert_eq!(stored.kind, RsCheckpointKind::Semantic);
        assert_eq!(stored.source, RsCheckpointSource::Manual);
    }

    #[test]
    fn scaffold_script_is_deterministic_and_protocol_compatible() {
        let root = temp_project("scaffold");
        write_project_file(&root, "mbse/model/design.sysml", "package Design {}\n");
        let id = "energy-budget-positive";
        let script_uri = expected_scaffold_uri(id);
        let mut request = RsCheckpointSaveRequest {
            checkpoint_id: id.into(),
            title: "Energy budget stays non-negative".into(),
            description:
                "Deterministic check that every actuator declares a non-negative energy budget."
                    .into(),
            kind: RsCheckpointKind::Syntactic,
            tags: vec!["energy".into()],
            rules: vec![
                RsCheckerRuleSpec {
                    rule_id: "budget-declared".into(),
                    condition: "Every actuator declares a numeric energyBudget.".into(),
                    reads: vec!["mbse/model/design.sysml".into()],
                },
                RsCheckerRuleSpec {
                    rule_id: "budget-non-negative".into(),
                    condition: "Every declared energyBudget is >= 0.".into(),
                    reads: vec!["mbse/model/design.sysml".into()],
                },
            ],
            inputs: vec!["mbse/model/design.sysml".into()],
            executor: Some(RsCheckpointExecutor::script(&script_uri)),
            checker_id: Some("energyBudgetCheck".into()),
            criteria: vec![],
            judges: None,
            create_script: Some("scaffold".into()),
            version: None,
            created_at: None,
            updated_at: None,
            source: None,
            history: vec![],
            builtin: None,
            script_exists: None,
        };
        let saved = save(&root, request.clone()).unwrap();
        assert_eq!(saved.script_uri.as_deref(), Some(script_uri.as_str()));
        assert_eq!(saved.checkpoint.source, RsCheckpointSource::Generated);
        let script_path = root.join(&script_uri);
        assert!(script_path.is_file());
        let content = fs::read_to_string(&script_path).unwrap();
        assert!(content.starts_with("#!/usr/bin/env python3"));
        assert!(content.contains("def rule_0_budget_declared(context):"));
        assert!(content.contains("rule not implemented: budget-declared"));
        assert!(content.contains("rule not implemented: budget-non-negative"));
        assert!(content.contains("--list-inputs"));
        assert!(content.contains("input_sha256"));
        assert!(
            content.contains("\"checker_id\": args.checker_id"),
            "the scaffold must echo the runtime --checker-id, not hardcode the entry id"
        );
        assert!(!content.contains("expected checker ID"));
        assert!(content.contains("snapshot_digest"));
        // Deterministic: a fresh scaffold request regenerates identical bytes.
        let entry = saved.checkpoint.clone();
        assert_eq!(scaffold_script(&entry), scaffold_script(&entry));
        let output = std::process::Command::new("python3")
            .current_dir(&root)
            .arg(&script_uri)
            .arg("--list-inputs")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let declared: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(declared, vec!["mbse/model/design.sysml".to_string()]);
        // Scaffold creation requires the reserved script path.
        request.executor = Some(RsCheckpointExecutor::script("checkpoints/scripts/other.py"));
        request.create_script = Some("scaffold".into());
        expect_invalid(save(&root, request));
    }

    #[test]
    fn builtin_scripts_are_available_without_project_source_files() {
        let root = temp_project("issues");
        let library = RsCheckpointLibrary::load(&root).unwrap();
        assert!(library.issues().is_empty());
        assert!(library
            .entries()
            .filter(|(_, builtin)| *builtin)
            .filter(|(entry, _)| entry.script_uri().is_some())
            .all(|(entry, _)| library.script_exists(entry)));
    }
}
