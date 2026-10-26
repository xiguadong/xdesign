//! `xdesign checkpoint list|show|save --request FILE` command envelope.
//!
//! Every invocation prints exactly one JSON object on stdout: a success reply
//! (exit 0) or a stable machine-readable error object (exit 2 for invalid
//! requests and definitions, exit 1 for internal failures). The web operator
//! program spawns these commands and maps the stable error codes.

use super::library::{
    normalized_project_uri, save, RsCheckpointError, RsCheckpointSaveRequest,
    LIBRARY_SCHEMA_VERSION, LIBRARY_URI,
};
use crate::checkpoint::RsCheckpointLibrary;
use serde_json::{json, Value};
use std::path::Path;

pub const HELP: &str = "Usage: xdesign checkpoint list
       xdesign checkpoint show CHECKPOINT_ID
       xdesign checkpoint save --request FILE

Manage the project checkpoint library at checkpoints/library.json. list prints the
effective built-in plus user library; show prints one full entry; save creates or
updates a user entry (version + 1, previous snapshot kept in history). Built-in
checkpoint ids are read-only. The save request is strict JSON with the entry fields
(checkpoint_id, title, description, kind, tags, rules, inputs, executor, criteria,
judges) and an optional create_script: \"scaffold\" that writes a deterministic
checker script under checkpoints/scripts/. Exit codes: 0 success, 2 invalid
request or definition, 1 internal failure. All output is one JSON object on stdout.";

/// Run the checkpoint command from the current working directory.
pub fn dispatch(args: &[String]) -> (Value, i32) {
    let outcome = std::env::current_dir()
        .map_err(|error| {
            RsCheckpointError::Internal(format!("cannot resolve project root: {error}"))
        })
        .and_then(|root| dispatch_at(&root, args));
    match outcome {
        Ok(reply) => (reply, 0),
        Err(error) => (error_reply(&error), error.exit_code()),
    }
}

/// Run the checkpoint command against an explicit project root. Used by the
/// CLI and by library-level tests that do not change the process directory.
pub fn dispatch_at(root: &Path, args: &[String]) -> Result<Value, RsCheckpointError> {
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["list"] => list_reply(root),
        ["show", checkpoint_id] => show_reply(root, checkpoint_id),
        ["save", "--request", request_uri] => {
            let request = read_request(root, request_uri)?;
            let outcome = save(root, request)?;
            Ok(save_reply(&outcome))
        }
        _ => Err(RsCheckpointError::Request(format!(
            "invalid checkpoint arguments\n{HELP}"
        ))),
    }
}

fn error_reply(error: &RsCheckpointError) -> Value {
    json!({
        "schema_version": LIBRARY_SCHEMA_VERSION,
        "ok": false,
        "error": {
            "code": error.code(),
            "message": error.message(),
        },
    })
}

fn list_reply(root: &Path) -> Result<Value, RsCheckpointError> {
    let library = RsCheckpointLibrary::load(root)?;
    let checkpoints: Vec<Value> = library
        .entries()
        .map(|(entry, builtin)| {
            json!({
                "checkpoint_id": entry.checkpoint_id,
                "title": entry.title,
                "kind": entry.kind,
                "builtin": builtin,
                "version": entry.version,
                "updated_at": entry.updated_at,
                "tags": entry.tags,
                "executor": entry.executor,
                "checker_id": entry.checker_id,
                "rule_count": entry.rules.len(),
                "criteria_count": entry.criteria.len(),
            })
        })
        .collect();
    Ok(json!({
        "schema_version": LIBRARY_SCHEMA_VERSION,
        "library_uri": LIBRARY_URI,
        "issues": library.issues(),
        "checkpoints": checkpoints,
    }))
}

fn show_reply(root: &Path, checkpoint_id: &str) -> Result<Value, RsCheckpointError> {
    let library = RsCheckpointLibrary::load(root)?;
    let Some((entry, builtin)) = library.entry(checkpoint_id) else {
        return Err(RsCheckpointError::Unknown(format!(
            "checkpoint does not exist: {checkpoint_id}"
        )));
    };
    let mut value = serde_json::to_value(entry).map_err(|error| {
        RsCheckpointError::Internal(format!("cannot serialize checkpoint entry: {error}"))
    })?;
    value["builtin"] = json!(builtin);
    value["script_exists"] = json!(library.script_exists(entry));
    Ok(json!({
        "schema_version": LIBRARY_SCHEMA_VERSION,
        "checkpoint": value,
    }))
}

fn save_reply(outcome: &super::library::RsCheckpointSaveOutcome) -> Value {
    let mut value = serde_json::to_value(&outcome.checkpoint).unwrap_or_else(|_| json!({}));
    value["builtin"] = json!(false);
    json!({
        "schema_version": LIBRARY_SCHEMA_VERSION,
        "checkpoint": value,
        "script_uri": outcome.script_uri,
        "created": outcome.created,
        "version": outcome.version,
    })
}

fn read_request(
    root: &Path,
    request_uri: &str,
) -> Result<RsCheckpointSaveRequest, RsCheckpointError> {
    if !normalized_project_uri(request_uri) {
        return Err(RsCheckpointError::Request(format!(
            "request file must be a normalized project-relative path: {request_uri}"
        )));
    }
    let path = root.join(request_uri);
    let bytes = std::fs::read(&path).map_err(|error| {
        RsCheckpointError::Request(format!("cannot read request file {request_uri}: {error}"))
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        RsCheckpointError::Invalid(format!("invalid checkpoint save request JSON: {error}"))
    })
}
