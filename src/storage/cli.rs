use super::{
    json_depth, ops, RsStore, RsStoreError, MAX_JSON_BYTES, MAX_JSON_DEPTH, SCHEMA_VERSION,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const HELP: &str = "Usage: xdesign store init [--root PATH]
       xdesign store info [--root PATH]
       xdesign store save --request FILE [--project-root PATH] [--root PATH]
       xdesign store outcome OPERATION_ID [--root PATH]
       xdesign store request OPERATION_ID --output FILE [--root PATH]
       xdesign store list [--kind KIND] [--status STATUS] [--root PATH]
       xdesign store get ITEM_ID [--revision N] [--root PATH]
       xdesign store history ITEM_ID [--root PATH]
       xdesign store read ITEM_ID --uri URI [--revision N] --output FILE [--root PATH]
       xdesign store checkout ITEM_ID [--revision N] --destination DIR [--root PATH]
       xdesign store doctor [--verify-bytes] [--root PATH]
       xdesign store location inspect [--project-root DIR] [--root PATH]
       xdesign store location enroll --operation-id ID [--project-root DIR]
           [--new-incarnation --previous-binding KEY]
       xdesign store migrate prepare MIGRATION_ID --output FILE --older-clients-stopped [--project-root DIR]
       xdesign store migrate apply --request FILE
       xdesign store migrate status MIGRATION_ID --root SOURCE_OR_DESTINATION
       xdesign store migrate request MIGRATION_ID --root SOURCE_OR_DESTINATION --output FILE
       xdesign store migrate abort MIGRATION_ID --root SOURCE_OR_DESTINATION
       xdesign store migrate discard-prepared MIGRATION_ID --root SOURCE_OR_DESTINATION

Linked worktree migration: inspect; enroll; stop older clients; prepare with
--older-clients-stopped; apply the saved request. Recover with status/request,
then replay the exact request bytes. Keep older clients stopped after cutover.
prepare writes a new external request file and does not migrate history.";

struct Arguments {
    command: String,
    positional: Vec<String>,
    options: BTreeMap<String, String>,
    flags: BTreeSet<String>,
}

struct Failure {
    code: String,
    message: String,
    exit: i32,
    result: Option<Value>,
}

impl Failure {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_input".into(),
            message: message.into(),
            exit: 2,
            result: None,
        }
    }

    fn from_error(error: RsStoreError, request_schema: bool) -> Self {
        if let Some(reason) = super::access::reason(&error) {
            let code = if reason == "binding_missing" {
                "not_found"
            } else {
                reason
            };
            let exit = match reason {
                "invalid_input" => 2,
                "not_found" | "binding_missing" => 3,
                "migration_required"
                | "history_conflict"
                | "backend_conflict"
                | "backend_ambiguous"
                | "migration_busy"
                | "source_retired"
                | "source_changed"
                | "source_moved"
                | "migration_committed"
                | "migration_aborted"
                | "operation_mismatch" => 4,
                "coordination_unavailable" | "io" | "database" => 6,
                "migration_outcome_unknown" => 7,
                _ => 5,
            };
            return Self {
                code: code.into(),
                message: error.to_string(),
                exit,
                result: None,
            };
        }
        let (code, exit, message, result) = match error {
            RsStoreError::InvalidInput(message) => {
                ("invalid_input", 2, message, None)
            }
            RsStoreError::UnsupportedFormat(message) => (
                "unsupported_format",
                if request_schema { 2 } else { 5 },
                message,
                None,
            ),
            RsStoreError::NotFound(message) => ("not_found", 3, message, None),
            RsStoreError::OperationMismatch { operation_id } => (
                "operation_mismatch",
                4,
                "The operation ID is reserved for different request bytes.".into(),
                Some(json!({"operation_id": operation_id})),
            ),
            RsStoreError::Integrity(message) => ("integrity", 5, message, None),
            RsStoreError::UnsafePath(message) => ("unsafe_path", 5, message, None),
            RsStoreError::Io {
                operation_id,
                message,
            } => (
                "io",
                6,
                message,
                operation_id.map(|operation_id| json!({"operation_id": operation_id})),
            ),
            RsStoreError::Database {
                operation_id,
                message,
            } => (
                "database",
                6,
                message,
                operation_id.map(|operation_id| json!({"operation_id": operation_id})),
            ),
            RsStoreError::UnknownOutcome { operation_id } => (
                "unknown_outcome",
                7,
                format!("The outcome of operation {operation_id} is unknown; query outcome or retry the exact request."),
                Some(json!({"operation_id": operation_id})),
            ),
        };
        Self {
            code: code.into(),
            message,
            exit,
            result,
        }
    }
}

fn option_name(token: &str) -> Option<(&'static str, bool)> {
    match token {
        "--root" => Some(("root", true)),
        "--project-root" => Some(("project-root", true)),
        "--request" => Some(("request", true)),
        "--kind" => Some(("kind", true)),
        "--status" => Some(("status", true)),
        "--revision" => Some(("revision", true)),
        "--uri" => Some(("uri", true)),
        "--out" | "--output" => Some(("out", true)),
        "--to" | "--destination" => Some(("to", true)),
        "--verify" | "--verify-bytes" => Some(("verify", false)),
        _ => None,
    }
}

fn parse(arguments: &[String]) -> Result<Arguments, Failure> {
    let arguments = if arguments.first().map(String::as_str) == Some("store") {
        &arguments[1..]
    } else {
        arguments
    };
    let Some(command) = arguments.first().cloned() else {
        return Err(Failure::invalid("missing store command"));
    };
    let positional_count = match command.as_str() {
        "init" | "info" | "save" | "list" | "doctor" => 0,
        "outcome" | "request" | "get" | "history" | "read" | "checkout" => 1,
        _ => {
            return Err(Failure::invalid(format!(
                "unknown store command: {command}"
            )))
        }
    };
    let allowed: BTreeSet<&'static str> = match command.as_str() {
        "init" | "info" => ["root"].into(),
        "save" => ["root", "project-root", "request"].into(),
        "outcome" | "history" => ["root"].into(),
        "request" => ["root", "out"].into(),
        "list" => ["root", "kind", "status"].into(),
        "get" => ["root", "revision"].into(),
        "read" => ["root", "revision", "uri", "out"].into(),
        "checkout" => ["root", "revision", "to"].into(),
        "doctor" => ["root", "verify"].into(),
        _ => unreachable!(),
    };
    let mut positional = Vec::new();
    let mut options = BTreeMap::new();
    let mut flags = BTreeSet::new();
    let mut index = 1;
    while index < arguments.len() {
        let token = &arguments[index];
        if token == "--help" {
            return Err(Failure::invalid("--help cannot be mixed with a command"));
        }
        if token.starts_with("--") {
            let Some((name, takes_value)) = option_name(token) else {
                return Err(Failure::invalid(format!("unknown option: {token}")));
            };
            if !allowed.contains(name) {
                return Err(Failure::invalid(format!(
                    "option {token} is not valid for {command}"
                )));
            }
            if takes_value {
                if options.contains_key(name) || flags.contains(name) {
                    return Err(Failure::invalid(format!("duplicate option: {token}")));
                }
                let value = arguments
                    .get(index + 1)
                    .ok_or_else(|| Failure::invalid(format!("missing value for {token}")))?;
                if value.starts_with("--") {
                    return Err(Failure::invalid(format!("missing value for {token}")));
                }
                options.insert(name.to_owned(), value.clone());
                index += 2;
            } else {
                if flags.contains(name) || options.contains_key(name) {
                    return Err(Failure::invalid(format!("duplicate option: {token}")));
                }
                flags.insert(name.to_owned());
                index += 1;
            }
        } else {
            positional.push(token.clone());
            index += 1;
        }
    }
    if positional.len() != positional_count {
        return Err(Failure::invalid(format!(
            "{command} expects {positional_count} positional argument(s), got {}",
            positional.len()
        )));
    }
    for required in match command.as_str() {
        "save" => &["request"][..],
        "read" => &["uri", "out"][..],
        "request" => &["out"][..],
        "checkout" => &["to"][..],
        _ => &[][..],
    } {
        if !options.contains_key(*required) {
            return Err(Failure::invalid(format!("{command} requires --{required}")));
        }
    }
    Ok(Arguments {
        command,
        positional,
        options,
        flags,
    })
}

fn root(arguments: &Arguments) -> Result<PathBuf, Failure> {
    let project =
        std::env::current_dir().map_err(|e| Failure::from_error(super::io_error(e), false))?;
    resolve_store_root(
        &project,
        arguments.options.get("root").map(Path::new),
        arguments.command == "init",
    )
    .map_err(|e| Failure::from_error(e, false))
}

fn revision(arguments: &Arguments) -> Result<Option<u64>, Failure> {
    arguments
        .options
        .get("revision")
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| Failure::invalid(format!("invalid revision: {value}")))
        })
        .transpose()
}

fn read_request(path: &str) -> Result<Vec<u8>, RsStoreError> {
    let mut bytes = Vec::new();
    if path == "-" {
        std::io::stdin()
            .take(MAX_JSON_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| RsStoreError::Io {
                operation_id: None,
                message: error.to_string(),
            })?;
    } else {
        let file = fs::File::open(path).map_err(|error| match error.kind() {
            std::io::ErrorKind::NotFound => {
                RsStoreError::NotFound(format!("request file not found: {path}"))
            }
            _ => RsStoreError::Io {
                operation_id: None,
                message: error.to_string(),
            },
        })?;
        if !file
            .metadata()
            .map_err(|error| RsStoreError::Io {
                operation_id: None,
                message: error.to_string(),
            })?
            .is_file()
        {
            return Err(RsStoreError::InvalidInput(
                "request input is not a regular file".into(),
            ));
        }
        file.take(MAX_JSON_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| RsStoreError::Io {
                operation_id: None,
                message: error.to_string(),
            })?;
    }
    if bytes.len() > MAX_JSON_BYTES {
        return Err(RsStoreError::InvalidInput(
            "save request exceeds 16 MiB".into(),
        ));
    }
    Ok(bytes)
}

fn serialize<T: serde::Serialize>(value: &T) -> Result<Value, Failure> {
    serde_json::to_value(value).map_err(|error| Failure {
        code: "integrity".into(),
        message: format!("cannot encode command response: {error}"),
        exit: 5,
        result: None,
    })
}

fn execute(arguments: Arguments) -> Result<Value, Failure> {
    let root = root(&arguments)?;
    if arguments.command == "doctor" {
        let root =
            super::access::absolute_root(&root).map_err(|e| Failure::from_error(e, false))?;
        let _lease =
            super::access::acquire(&root, None).map_err(|e| Failure::from_error(e, false))?;
        if let Some(location) = super::migration::preparation_diagnostic(&root)
            .map_err(|e| Failure::from_error(e, false))?
        {
            return Ok(json!({"root":root,"location":location,"diagnostic_only":true}));
        }
    }
    if arguments.command == "doctor" {
        let report = ops::doctor_root(&root, arguments.flags.contains("verify"))
            .map_err(|e| Failure::from_error(e, false))?;
        return if report.findings.is_empty() {
            Ok(json!({"report":report}))
        } else {
            Err(Failure {
                code: "integrity_findings".into(),
                message: "Doctor found storage integrity defects.".into(),
                exit: 5,
                result: Some(json!({"report":report})),
            })
        };
    }
    let project =
        std::env::current_dir().map_err(|e| Failure::from_error(super::io_error(e), false))?;
    with_project_store(&project, &root, arguments.command == "init", |store| {
        execute_guarded(arguments, store)
    })
    .map_err(|e| match e {
        WithProjectStoreError::Store(e) => Failure::from_error(e, false),
        WithProjectStoreError::Action(e) => e,
    })
}

fn execute_guarded(arguments: Arguments, store: &RsStore) -> Result<Value, Failure> {
    match arguments.command.as_str() {
        "init" => {
            let mut result = serialize(
                &store
                    .info()
                    .map_err(|error| Failure::from_error(error, false))?,
            )?;
            result["root"] = json!(store.root);
            Ok(result)
        }
        "info" => serialize(
            &store
                .info()
                .map_err(|error| Failure::from_error(error, false))?,
        ),
        "save" => {
            let raw = read_request(&arguments.options["request"])
                .map_err(|error| Failure::from_error(error, true))?;
            let project_root = arguments
                .options
                .get("project-root")
                .map(PathBuf::from)
                .unwrap_or(std::env::current_dir().map_err(|error| {
                    Failure::from_error(
                        RsStoreError::Io {
                            operation_id: None,
                            message: error.to_string(),
                        },
                        false,
                    )
                })?);
            let receipt = store.save_bytes(&project_root, &raw).map_err(|error| {
                let mut failure = Failure::from_error(error, true);
                if let Ok(request) = serde_json::from_slice::<Value>(&raw) {
                    if let Some(operation_id) = request["operation_id"].as_str() {
                        if let Ok(receipt) = store.outcome(operation_id) {
                            let mut result = failure.result.take().unwrap_or_else(|| json!({}));
                            result["operation_id"] = json!(operation_id);
                            result["receipt_state"] = json!(receipt.outcome);
                            failure.result = Some(result);
                        }
                    }
                }
                failure
            })?;
            if receipt.outcome == super::RsOperationOutcome::Conflict {
                return Err(Failure {
                    code: "revision_conflict".into(),
                    message: "The requested revision conflicts with the current item head or kind."
                        .into(),
                    exit: 4,
                    result: Some(json!({"receipt": receipt})),
                });
            }
            Ok(json!({"receipt": receipt}))
        }
        "outcome" => {
            let receipt = store
                .outcome(&arguments.positional[0])
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({"receipt": receipt}))
        }
        // The exact retained request bytes are the recovery input an unknown
        // outcome needs: `save` requires byte-identical retry input, and the
        // original request file may be gone after a crash. Output follows the
        // same no-replace, all-ancestor, outside-the-store rules as `read`.
        "request" => {
            let operation_id = arguments.positional[0].clone();
            let raw = ops::request_bytes(&store, &operation_id)
                .map_err(|error| Failure::from_error(error, false))?
                .ok_or_else(|| {
                    Failure::from_error(
                        RsStoreError::NotFound(format!("operation not found: {operation_id}")),
                        false,
                    )
                })?;
            let receipt = store
                .outcome(&operation_id)
                .map_err(|error| Failure::from_error(error, false))?;
            let output = Path::new(&arguments.options["out"]);
            let canonical = super::blobs::canonical_for_creation(output)
                .map_err(|error| Failure::from_error(error, false))?;
            if canonical.starts_with(&store.root) {
                return Err(Failure::from_error(
                    RsStoreError::UnsafePath(
                        "request output cannot be created inside the store".into(),
                    ),
                    false,
                ));
            }
            let output = super::blobs::write_new_external(output, &raw)
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({
                "operation_id": operation_id,
                "request_sha256": receipt.request_sha256,
                "receipt_state": receipt.outcome,
                "byte_length": raw.len(),
                "output": output,
            }))
        }
        "list" => {
            let items = store
                .list(
                    arguments.options.get("kind").map(String::as_str),
                    arguments.options.get("status").map(String::as_str),
                )
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({"items": items, "complete": true}))
        }
        "get" => {
            let revision = store
                .get(&arguments.positional[0], revision(&arguments)?)
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({"revision": revision}))
        }
        "history" => {
            let revisions = store
                .history(&arguments.positional[0])
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({"revisions": revisions, "complete": true}))
        }
        "read" => {
            let selected_revision = revision(&arguments)?;
            let selected = store
                .get(&arguments.positional[0], selected_revision)
                .map_err(|error| Failure::from_error(error, false))?;
            let uri = &arguments.options["uri"];
            let material = selected
                .materials
                .iter()
                .find(|material| material.uri == *uri)
                .cloned()
                .ok_or_else(|| {
                    Failure::from_error(
                        RsStoreError::NotFound(format!("material URI not found: {uri}")),
                        false,
                    )
                })?;
            let bytes = store
                .read_material(&arguments.positional[0], Some(selected.revision), uri)
                .map_err(|error| Failure::from_error(error, false))?;
            let mut result = json!({
                "item_id": selected.item_id,
                "revision": selected.revision,
                "uri": material.uri,
                "sha256": material.sha256,
                "byte_length": material.byte_length,
            });
            let output = Path::new(&arguments.options["out"]);
            let canonical = super::blobs::canonical_for_creation(output)
                .map_err(|error| Failure::from_error(error, false))?;
            if canonical.starts_with(&store.root) {
                return Err(Failure::from_error(
                    RsStoreError::UnsafePath(
                        "read output cannot be created inside the store".into(),
                    ),
                    false,
                ));
            }
            let output = super::blobs::write_new_external(output, &bytes)
                .map_err(|error| Failure::from_error(error, false))?;
            result["output"] = json!(output);
            Ok(result)
        }
        "checkout" => {
            let report = store
                .checkout(
                    &arguments.positional[0],
                    revision(&arguments)?,
                    Path::new(&arguments.options["to"]),
                )
                .map_err(|error| Failure::from_error(error, false))?;
            serialize(&report)
        }
        _ => Err(Failure::invalid("unknown store command")),
    }
}

fn finalize(envelope: Value, exit: i32) -> (Value, i32) {
    let too_large = json_depth(&envelope) > MAX_JSON_DEPTH
        || serde_json::to_vec(&envelope)
            .map(|bytes| bytes.len() > MAX_JSON_BYTES)
            .unwrap_or(true);
    if too_large {
        return (
            json!({
                "schema_version": SCHEMA_VERSION,
                "ok": false,
                "command": "store",
                "error": {
                    "code": "response_too_large",
                    "message": "The command response exceeds the versioned JSON envelope limits."
                },
                "data": null
            }),
            5,
        );
    }
    (envelope, exit)
}

pub fn dispatch(args: &[String]) -> (Value, i32) {
    if args.len() == 1 && args[0] == "--help"
        || args.len() == 2 && args[0] == "store" && args[1] == "--help"
    {
        return finalize(
            json!({"schema_version": SCHEMA_VERSION, "ok": true, "command": "help", "data": {"help": HELP}}),
            0,
        );
    }
    let attempted = if args.first().map(String::as_str) == Some("store") {
        args.get(1)
    } else {
        args.first()
    }
    .cloned()
    .unwrap_or_else(|| "unknown".into());
    let bare = if args.first().is_some_and(|s| s == "store") {
        &args[1..]
    } else {
        args
    };
    if matches!(
        bare.first().map(String::as_str),
        Some("location" | "migrate")
    ) {
        let command = format!(
            "{}.{}",
            bare[0],
            bare.get(1).map(String::as_str).unwrap_or("unknown")
        );
        return match location_command(bare) {
            Ok(data)
                if command == "location.inspect"
                    && data["failed_predicates"]
                        .as_array()
                        .is_some_and(|a| !a.is_empty()) =>
            {
                let failed = &data["failed_predicates"][0];
                let f = Failure::from_error(
                    super::access::diagnostic(
                        failed["code"].as_str().unwrap(),
                        "inspection refused",
                    ),
                    false,
                );
                finalize(
                    json!({"schema_version":1,"ok":false,"command":command,"error":{"code":f.code,"message":f.message},"data":data}),
                    f.exit,
                )
            }
            Ok(data) => finalize(
                json!({"schema_version":1,"ok":true,"command":command,"data":data}),
                0,
            ),
            Err(e) => {
                let f = Failure::from_error(e, false);
                finalize(
                    json!({"schema_version":1,"ok":false,"command":command,"error":{"code":f.code,"message":f.message},"data":f.result}),
                    f.exit,
                )
            }
        };
    }
    let (command, outcome) = match parse(args) {
        Ok(arguments) => {
            let command = arguments.command.clone();
            (command, execute(arguments))
        }
        Err(failure) => (attempted, Err(failure)),
    };
    match outcome {
        Ok(data) => finalize(
            json!({"schema_version": SCHEMA_VERSION, "ok": true, "command": command, "data": data}),
            0,
        ),
        Err(failure) => {
            let mut data = Value::Null;
            let mut error = json!({"code": failure.code, "message": failure.message});
            if let Some(result) = failure.result {
                if let Some(operation_id) = result.get("operation_id").and_then(Value::as_str) {
                    error["operation_id"] = json!(operation_id);
                }
                if let Some(state) = result.get("receipt_state").and_then(Value::as_str) {
                    error["receipt_state"] = json!(state);
                }
                if result.get("receipt").is_some() || result.get("report").is_some() {
                    data = result;
                }
            }
            finalize(
                json!({"schema_version": SCHEMA_VERSION, "ok": false, "command": command, "error": error, "data": data}),
                failure.exit,
            )
        }
    }
}

pub fn request_bytes(store: &RsStore, operation_id: &str) -> Result<Option<Vec<u8>>, RsStoreError> {
    ops::request_bytes(store, operation_id)
}

/// Preserve a callback's error separately from access/setup errors.
#[derive(Debug)]
pub enum WithProjectStoreError<E> {
    Store(RsStoreError),
    Action(E),
}

/// Select a root without granting permission to access it.
pub fn resolve_store_root(
    project_root: &Path,
    requested: Option<&Path>,
    enroll: bool,
) -> Result<PathBuf, RsStoreError> {
    super::location::resolve(project_root, requested, enroll)
}

/// Keep the complete directory lease through a lifecycle command.
pub fn with_project_store<T, E>(
    project_root: &Path,
    root: &Path,
    initialize: bool,
    action: impl for<'a> FnOnce(&'a RsStore) -> Result<T, E>,
) -> Result<T, WithProjectStoreError<E>> {
    if initialize && !root.exists() {
        super::blobs::canonical_for_creation(root).map_err(WithProjectStoreError::Store)?;
    }
    let lease =
        super::access::acquire(root, Some(project_root)).map_err(WithProjectStoreError::Store)?;
    let root = super::access::absolute_root(root).map_err(WithProjectStoreError::Store)?;
    if !initialize {
        super::access::validate(&root, false).map_err(WithProjectStoreError::Store)?;
    }
    let mut store = if initialize {
        super::migration::initialize(&root)
    } else {
        super::db::open(&root)
    }
    .map_err(WithProjectStoreError::Store)?;
    store.lease = Some(lease);
    let _scope = super::access::CommandScope::enter();
    action(&store).map_err(WithProjectStoreError::Action)
}

/// Check authority before a command writes derived workspace state. A store
/// save will independently repeat the check before its durable commit.
pub fn validate_project_write(store: &RsStore) -> Result<(), RsStoreError> {
    let _lease = store.access(true)?;
    Ok(())
}
/// Build a terminal lifecycle response from retained bytes, without restoring cache.
pub fn read_completed_project(
    store: &RsStore,
    item_id: &str,
) -> Result<Option<Value>, RsStoreError> {
    ops::completed_project(store, item_id)
}

/// Inspect all discoverable backend candidates using the borrowed command lease.
pub fn project_candidate_contains(
    store: &RsStore,
    project: &Path,
    id: &str,
) -> Result<bool, RsStoreError> {
    use super::access::diagnostic;
    let lease = store.lease.as_ref().ok_or_else(|| {
        diagnostic(
            "migration_busy",
            "candidate inspection requires a command lease",
        )
    })?;
    let roots = crate::project_backend::candidates(project, Some(&store.root))
        .map_err(|e| diagnostic("backend_ambiguous", e))?;
    let mut found = false;
    for root in roots {
        let root = super::access::absolute_root(&root)?;
        if !lease.contains(root.parent().unwrap())
            || !super::migration::peers(&root)?
                .iter()
                .all(|p| lease.contains(p))
        {
            return Err(diagnostic(
                "migration_busy",
                "candidate graph changed; retry the outer command",
            ));
        }
        match fs::symlink_metadata(&root) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(diagnostic("backend_ambiguous", e)),
            Ok(_) => {}
        }
        if root == store.root {
            found |= match store.get(id, None) {
                Ok(_) => true,
                Err(RsStoreError::NotFound(_)) => false,
                Err(error) => return Err(error),
            };
            continue;
        }
        super::access::validate(&root, false).map_err(|e| diagnostic("backend_ambiguous", e))?;
        let mut candidate =
            super::db::open(&root).map_err(|e| diagnostic("backend_ambiguous", e))?;
        candidate.lease = Some(lease.clone());
        found |= match candidate.get(id, None) {
            Ok(_) => true,
            Err(RsStoreError::NotFound(_)) => false,
            Err(e) => return Err(diagnostic("backend_ambiguous", e)),
        };
    }
    Ok(found)
}

/// Internal read bridge for Web views. Absence and backend classification are
/// protected by the same lease as a present store, without initializing it.
pub(crate) fn with_optional_project_store<T, E>(
    project: &Path,
    root: &Path,
    action: impl for<'a> FnOnce(Option<&'a RsStore>) -> Result<T, E>,
) -> Result<T, WithProjectStoreError<E>> {
    let lease =
        super::access::acquire(root, Some(project)).map_err(WithProjectStoreError::Store)?;
    let root = super::access::absolute_root(root).map_err(WithProjectStoreError::Store)?;
    super::access::validate(&root, false).map_err(WithProjectStoreError::Store)?;
    let store = match super::db::open(&root) {
        Ok(mut store) => {
            store.lease = Some(lease.clone());
            Some(store)
        }
        Err(RsStoreError::NotFound(_))
            if fs::symlink_metadata(&root)
                .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
        {
            None
        }
        Err(error) => return Err(WithProjectStoreError::Store(error)),
    };
    let _scope = super::access::CommandScope::enter();
    action(store.as_ref()).map_err(WithProjectStoreError::Action)
}

fn location_command(args: &[String]) -> Result<Value, RsStoreError> {
    use super::{access::diagnostic, location, migration};
    let verb = args
        .get(1)
        .ok_or_else(|| diagnostic("invalid_input", "missing location/migrate subcommand"))?;
    let mut options = BTreeMap::new();
    let mut positional = vec![];
    let mut new_incarnation = false;
    let mut older_clients_stopped = false;
    let mut i = 2;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--new-incarnation" {
            if new_incarnation {
                return Err(diagnostic("invalid_input", "duplicate --new-incarnation"));
            }
            new_incarnation = true;
        } else if arg == "--older-clients-stopped" {
            if older_clients_stopped {
                return Err(diagnostic(
                    "invalid_input",
                    "duplicate --older-clients-stopped",
                ));
            }
            older_clients_stopped = true;
        } else if arg.starts_with("--") {
            let name = &arg[2..];
            if ![
                "root",
                "project-root",
                "operation-id",
                "previous-binding",
                "request",
                "output",
            ]
            .contains(&name)
            {
                return Err(diagnostic("invalid_input", "unknown option"));
            }
            i += 1;
            let value = args
                .get(i)
                .filter(|s| !s.is_empty() && !s.starts_with("--"))
                .ok_or_else(|| diagnostic("invalid_input", "missing option value"))?;
            if options.insert(name, value.as_str()).is_some() {
                return Err(diagnostic("invalid_input", "duplicate option"));
            }
        } else {
            positional.push(arg.as_str());
        }
        i += 1;
    }
    let required = |name: &str| -> Result<&str, RsStoreError> {
        options
            .get(name)
            .copied()
            .ok_or_else(|| diagnostic("invalid_input", format!("missing --{name}")))
    };
    let allowed = match (args[0].as_str(), verb.as_str()) {
        ("location", "inspect") => vec!["project-root", "root"],
        ("location", "enroll") => vec!["project-root", "operation-id", "previous-binding"],
        ("migrate", "prepare") => vec!["project-root", "output"],
        ("migrate", "apply") => vec!["request"],
        ("migrate", "request") => vec!["root", "output"],
        ("migrate", "status" | "abort" | "discard-prepared") => vec!["root"],
        _ => {
            return Err(diagnostic(
                "invalid_input",
                "unknown location/migrate subcommand",
            ))
        }
    };
    if options.keys().any(|k| !allowed.contains(k))
        || (new_incarnation && !(args[0] == "location" && verb == "enroll"))
        || (older_clients_stopped && !(args[0] == "migrate" && verb == "prepare"))
    {
        return Err(diagnostic(
            "invalid_input",
            "option not allowed for subcommand",
        ));
    }
    if args[0] == "location" {
        if !positional.is_empty() {
            return Err(diagnostic("invalid_input", "unexpected argument"));
        }
        let cwd = std::env::current_dir()?;
        let project = fs::canonicalize(options.get("project-root").map(Path::new).unwrap_or(&cwd))?;
        if verb == "inspect" {
            return Ok(inspect_location(
                &project,
                options.get("root").map(Path::new),
            ));
        }
        if new_incarnation != options.contains_key("previous-binding") {
            return Err(diagnostic(
                "invalid_input",
                "new-incarnation requires previous-binding and vice versa",
            ));
        }
        let ctx = location::context(&project)?
            .ok_or_else(|| diagnostic("invalid_input", "enrollment requires a linked worktree"))?;
        let b = location::enroll(
            &ctx,
            &location::account_home()?,
            required("operation-id")?,
            options.get("previous-binding").copied(),
        )?;
        return Ok(
            json!({"binding_key":b.key(),"binding_sha256":b.digest(),"store_root":b.root(),"binding":b.payload}),
        );
    }
    if verb == "apply" {
        if !positional.is_empty() {
            return Err(diagnostic("invalid_input", "unexpected argument"));
        }
        return migration::apply(&read_request(required("request")?)?);
    }
    if positional.len() != 1 {
        return Err(diagnostic(
            "invalid_input",
            "exactly one migration ID is required",
        ));
    }
    let id = positional[0];
    if verb == "prepare" {
        let cwd = std::env::current_dir()?;
        let project = fs::canonicalize(options.get("project-root").map(Path::new).unwrap_or(&cwd))?;
        return migration::prepare(
            &project,
            id,
            Path::new(required("output")?),
            older_clients_stopped,
        );
    }
    let root = Path::new(required("root")?);
    match verb.as_str() {
        "status" => migration::status(root, id),
        "abort" => migration::abort(root, id),
        "discard-prepared" => migration::discard(root, id),
        "request" => migration::request_output(root, id, Path::new(required("output")?)),
        _ => Err(diagnostic("invalid_input", "unknown subcommand")),
    }
}

/// Partial inspection remains available in the error envelope on a refusal.
fn inspect_location(project: &Path, requested: Option<&Path>) -> Value {
    use super::{access, location, migration};
    let environment = std::env::var_os("XDESIGN_STORE_ROOT").map(PathBuf::from);
    let local = project.join(".xdesign");
    let mut data = json!({"project_root":project,"local_candidate":local,"candidate_root":null,"context_kind":null,"selected_by":null,"authority_state":"unresolved","failed_predicates":[],"next_commands":[]});
    let result = (|| -> Result<(), RsStoreError> {
        if let Some(root) = requested.or(environment.as_deref()) {
            data["selected_by"] = json!(if requested.is_some() {
                "explicit"
            } else {
                "environment"
            });
            data["context_kind"] = json!("not_probed");
            let root = access::absolute_root(root)?;
            data["candidate_root"] = json!(root);
            let _lease = access::acquire(&root, None)?;
            let details = migration::inspect(&root)?;
            data.as_object_mut()
                .unwrap()
                .extend(details.as_object().unwrap().clone());
            return Ok(());
        }
        let ctx = location::context(project)?;
        data["selected_by"] = json!(if ctx.is_some() {
            "linked_default"
        } else {
            "local_default"
        });
        data["context_kind"] = json!(if ctx.is_some() {
            "linked_worktree"
        } else {
            "ordinary_project"
        });
        let Some(ctx) = ctx else {
            data["candidate_root"] = json!(local);
            let _lease = access::acquire(&local, Some(project))?;
            data.as_object_mut()
                .unwrap()
                .extend(migration::inspect(&local)?.as_object().unwrap().clone());
            return Ok(());
        };
        let home = location::account_home()?;
        data["namespace_path"] = json!(home.join(".xdesign/worktree-stores/v1"));
        let binding = location::binding_for_project(&ctx, &home)?;
        let _lease = access::acquire(&local, Some(project))?;
        if let Some(binding) = binding {
            binding.verify()?;
            data["candidate_root"] = json!(binding.root());
            data["binding_key"] = json!(binding.key());
            data["binding_sha256"] = json!(binding.digest());
            data["namespace_epoch"] = binding.payload["namespace_epoch"].clone();
            let details = migration::inspect(&binding.root())?;
            data.as_object_mut()
                .unwrap()
                .extend(details.as_object().unwrap().clone());
            if fs::symlink_metadata(&local).is_ok() {
                data["source"] = migration::inspect(&local)?;
                if data["authority_state"] == "absent" {
                    data["authority_state"] = json!("local_candidate_requires_migration");
                    data["next_commands"].as_array_mut().unwrap().push(json!([
                        "xdesign",
                        "store",
                        "migrate",
                        "prepare",
                        format!("migration-{}", uuid::Uuid::new_v4()),
                        "--project-root",
                        project,
                        "--output",
                        "migration-request.json",
                        "--older-clients-stopped"
                    ]));
                }
            }
            migration::select(&binding, fs::symlink_metadata(&local).is_ok())?;
        } else {
            data["next_commands"] = json!([[
                "xdesign",
                "store",
                "location",
                "enroll",
                "--project-root",
                project,
                "--operation-id",
                format!("enroll-{}", uuid::Uuid::new_v4())
            ]]);
            if fs::symlink_metadata(&local).is_ok() {
                data["source"] = migration::inspect(&local)?;
                data["authority_state"] = json!("local_candidate_requires_migration");
                return Err(access::diagnostic(
                    "migration_required",
                    "enroll, stop older clients, then prepare and apply a migration request",
                ));
            }
            return Err(access::diagnostic(
                "binding_missing",
                "no enrolled binding; enroll explicitly or use store init for a new empty default",
            ));
        }
        Ok(())
    })();
    if let Err(error) = result {
        let f = Failure::from_error(error, false);
        data["failed_predicates"]
            .as_array_mut()
            .unwrap()
            .push(json!({"code":f.code,"message":f.message}));
    }
    // Recovery uses IDs already authenticated by the route; no invented identity inputs.
    if let Some(migrations) = data["migrations"].as_array().cloned() {
        for m in migrations {
            let id = &m["migration_id"];
            let root = data["candidate_root"].clone();
            for argv in [
                json!(["xdesign", "store", "migrate", "status", id, "--root", root]),
                json!([
                    "xdesign",
                    "store",
                    "migrate",
                    "request",
                    id,
                    "--root",
                    root,
                    "--output",
                    format!("recovered-{}.json", id.as_str().unwrap())
                ]),
            ] {
                data["next_commands"].as_array_mut().unwrap().push(argv);
            }
        }
    }
    data
}
