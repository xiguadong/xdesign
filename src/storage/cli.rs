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
       xdesign store list [--kind KIND] [--status STATUS] [--root PATH]
       xdesign store get ITEM_ID [--revision N] [--root PATH]
       xdesign store history ITEM_ID [--root PATH]
       xdesign store read ITEM_ID --uri URI [--revision N] --output FILE [--root PATH]
       xdesign store checkout ITEM_ID [--revision N] --destination DIR [--root PATH]
       xdesign store doctor [--verify-bytes] [--root PATH]";

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
        "outcome" | "get" | "history" | "read" | "checkout" => 1,
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
    let root = if let Some(root) = arguments.options.get("root") {
        PathBuf::from(root)
    } else if let Some(root) = std::env::var_os("XDESIGN_STORE_ROOT") {
        PathBuf::from(root)
    } else {
        PathBuf::from(".xdesign")
    };
    if root.as_os_str().is_empty() {
        return Err(Failure::invalid("store root must not be empty"));
    }
    Ok(root)
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
    match arguments.command.as_str() {
        "init" => {
            let store = RsStore::init(&root).map_err(|error| Failure::from_error(error, false))?;
            let mut result = serialize(
                &store
                    .info()
                    .map_err(|error| Failure::from_error(error, false))?,
            )?;
            result["root"] = json!(store.root);
            Ok(result)
        }
        "info" => {
            let store = RsStore::open(&root).map_err(|error| Failure::from_error(error, false))?;
            serialize(
                &store
                    .info()
                    .map_err(|error| Failure::from_error(error, false))?,
            )
        }
        "save" => {
            let store = RsStore::open(&root).map_err(|error| Failure::from_error(error, false))?;
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
            let store = RsStore::open(&root).map_err(|error| Failure::from_error(error, false))?;
            let receipt = store
                .outcome(&arguments.positional[0])
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({"receipt": receipt}))
        }
        "list" => {
            let store = RsStore::open(&root).map_err(|error| Failure::from_error(error, false))?;
            let items = store
                .list(
                    arguments.options.get("kind").map(String::as_str),
                    arguments.options.get("status").map(String::as_str),
                )
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({"items": items, "complete": true}))
        }
        "get" => {
            let store = RsStore::open(&root).map_err(|error| Failure::from_error(error, false))?;
            let revision = store
                .get(&arguments.positional[0], revision(&arguments)?)
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({"revision": revision}))
        }
        "history" => {
            let store = RsStore::open(&root).map_err(|error| Failure::from_error(error, false))?;
            let revisions = store
                .history(&arguments.positional[0])
                .map_err(|error| Failure::from_error(error, false))?;
            Ok(json!({"revisions": revisions, "complete": true}))
        }
        "read" => {
            let store = RsStore::open(&root).map_err(|error| Failure::from_error(error, false))?;
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
            let store = RsStore::open(&root).map_err(|error| Failure::from_error(error, false))?;
            let report = store
                .checkout(
                    &arguments.positional[0],
                    revision(&arguments)?,
                    Path::new(&arguments.options["to"]),
                )
                .map_err(|error| Failure::from_error(error, false))?;
            serialize(&report)
        }
        "doctor" => {
            let report = ops::doctor_root(&root, arguments.flags.contains("verify"))
                .map_err(|error| Failure::from_error(error, false))?;
            if report.findings.is_empty() {
                Ok(json!({"report": report}))
            } else {
                Err(Failure {
                    code: "integrity_findings".into(),
                    message: "Doctor found storage integrity defects.".into(),
                    exit: 5,
                    result: Some(json!({"report": report})),
                })
            }
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
