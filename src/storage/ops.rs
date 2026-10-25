use super::{
    blobs::{
        self, blob_path, canonical_existing, canonical_for_creation, cleanup_operation_staging,
        published_by_attempt, read_published, read_published_any, read_source, verify_published,
    },
    compact_json, database_error, from_i64, io_error, json_depth, manifest_entry_collision, sha256,
    strict_json, to_i64, valid_digest, valid_id, valid_revision, valid_token, valid_uri,
    RsCheckoutReport, RsDoctorFinding, RsDoctorFindingKind, RsDoctorReport, RsItemRevision,
    RsItemSummary, RsMaterial, RsOperationOutcome, RsOperationReceipt, RsSaveRequest, RsStore,
    RsStoreError, RsWishProjection, RsWishStatus, MAX_JSON_BYTES, MAX_JSON_DEPTH,
    MAX_MATERIAL_BYTES, RESPONSE_BYTES_MARGIN, RESPONSE_DEPTH_MARGIN,
};
use crate::core::{RsCheckKind, RsConformanceKind, RsFlowStatus, RsStageVisit};
use rusqlite::{Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use super::db::{self, Transaction};

#[derive(Clone, Debug)]
struct OperationRow {
    operation_id: String,
    request_bytes: Vec<u8>,
    request_sha256: String,
    outcome: RsOperationOutcome,
    item_id: String,
    expected_revision: u64,
    revision: Option<u64>,
    result_json: String,
}

fn load_operation(
    conn: &Connection,
    operation_id: &str,
) -> Result<Option<OperationRow>, RsStoreError> {
    conn.query_row(
        "SELECT operation_id, request_bytes, request_sha256, outcome, item_id,
                expected_revision, revision, result_json
         FROM operations WHERE operation_id = ?1",
        [operation_id],
        |row| {
            let outcome: String = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Vec<u8>>(1)?,
                row.get::<_, String>(2)?,
                outcome,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, Option<i64>>(6)?,
                row.get::<_, String>(7)?,
            ))
        },
    )
    .optional()
    .map_err(database_error)?
    .map(|row| {
        let outcome = match row.3.as_str() {
            "pending" => Ok(RsOperationOutcome::Pending),
            "committed" => Ok(RsOperationOutcome::Committed),
            "conflict" => Ok(RsOperationOutcome::Conflict),
            _ => Err(RsStoreError::Integrity(format!(
                "operation {} has an invalid outcome",
                row.0
            ))),
        }?;
        Ok(OperationRow {
            operation_id: row.0,
            request_bytes: row.1,
            request_sha256: row.2,
            outcome,
            item_id: row.4,
            expected_revision: from_i64(row.5, "expected revision")?,
            revision: row.6.map(|value| from_i64(value, "revision")).transpose()?,
            result_json: row.7,
        })
    })
    .transpose()
}

fn operation_receipt(row: &OperationRow) -> Result<RsOperationReceipt, RsStoreError> {
    if !valid_id(&row.operation_id)
        || !valid_id(&row.item_id)
        || !valid_digest(&row.request_sha256)
        || sha256(&row.request_bytes) != row.request_sha256
    {
        return Err(RsStoreError::Integrity(format!(
            "operation {} has invalid exact-request identity",
            row.operation_id
        )));
    }
    let request = parse_request(&row.request_bytes).map_err(|error| {
        RsStoreError::Integrity(format!(
            "operation {} contains an invalid reserved request: {error}",
            row.operation_id
        ))
    })?;
    validate_request(&request).map_err(|error| {
        RsStoreError::Integrity(format!(
            "operation {} contains an invalid reserved request: {error}",
            row.operation_id
        ))
    })?;
    let result = strict_json(row.result_json.as_bytes()).map_err(|error| {
        RsStoreError::Integrity(format!(
            "operation {} has an invalid result: {error}",
            row.operation_id
        ))
    })?;
    if request.operation_id != row.operation_id
        || request.item_id != row.item_id
        || request.expected_revision != row.expected_revision
        || request.result != result
    {
        return Err(RsStoreError::Integrity(format!(
            "operation {} does not match its reserved request",
            row.operation_id
        )));
    }
    match row.outcome {
        RsOperationOutcome::Committed if row.revision.is_none() => {
            return Err(RsStoreError::Integrity(format!(
                "committed operation {} has no revision",
                row.operation_id
            )))
        }
        RsOperationOutcome::Pending | RsOperationOutcome::Conflict if row.revision.is_some() => {
            return Err(RsStoreError::Integrity(format!(
                "noncommitted operation {} has a revision",
                row.operation_id
            )))
        }
        _ => {}
    }
    if let Some(revision) = row.revision {
        if revision == 0 || !valid_revision(revision) {
            return Err(RsStoreError::Integrity(format!(
                "operation {} has an invalid revision",
                row.operation_id
            )));
        }
    }
    Ok(RsOperationReceipt {
        operation_id: row.operation_id.clone(),
        request_sha256: row.request_sha256.clone(),
        outcome: row.outcome,
        revision: row.revision,
        result,
    })
}

fn parse_request(raw: &[u8]) -> Result<RsSaveRequest, RsStoreError> {
    let value = strict_json(raw).map_err(|error| {
        if raw.len() > MAX_JSON_BYTES {
            RsStoreError::InvalidInput(error)
        } else {
            RsStoreError::InvalidInput(format!("invalid save request JSON: {error}"))
        }
    })?;
    let object = value
        .as_object()
        .ok_or_else(|| RsStoreError::InvalidInput("save request must be a JSON object".into()))?;
    const REQUIRED: [&str; 8] = [
        "schema_version",
        "operation_id",
        "item_id",
        "kind",
        "expected_revision",
        "metadata",
        "materials",
        "result",
    ];
    for key in REQUIRED {
        if !object.contains_key(key) {
            return Err(RsStoreError::InvalidInput(format!(
                "save request is missing required member: {key}"
            )));
        }
    }
    let materials = object
        .get("materials")
        .and_then(Value::as_array)
        .ok_or_else(|| RsStoreError::InvalidInput("materials must be an array".into()))?;
    for material in materials {
        let material = material
            .as_object()
            .ok_or_else(|| RsStoreError::InvalidInput("each material must be an object".into()))?;
        for key in ["uri", "role", "sha256"] {
            if !material.contains_key(key) {
                return Err(RsStoreError::InvalidInput(format!(
                    "material is missing required member: {key}"
                )));
            }
        }
        if material.get("source") == Some(&Value::Null) {
            return Err(RsStoreError::InvalidInput(
                "material source must be omitted rather than null".into(),
            ));
        }
    }
    let request: RsSaveRequest = serde_json::from_value(value)
        .map_err(|error| RsStoreError::InvalidInput(format!("invalid save request: {error}")))?;
    if request.schema_version != 1 {
        return Err(RsStoreError::UnsupportedFormat(format!(
            "unsupported save request schema_version: {}",
            request.schema_version
        )));
    }
    Ok(request)
}

fn validate_request(request: &RsSaveRequest) -> Result<(), RsStoreError> {
    if request.schema_version != 1 {
        return Err(RsStoreError::UnsupportedFormat(
            "unsupported save request schema".into(),
        ));
    }
    if !valid_id(&request.operation_id) {
        return Err(RsStoreError::InvalidInput("invalid operation_id".into()));
    }
    if !valid_id(&request.item_id) {
        return Err(RsStoreError::InvalidInput("invalid item_id".into()));
    }
    if !valid_token(&request.kind) {
        return Err(RsStoreError::InvalidInput("invalid item kind".into()));
    }
    if !valid_revision(request.expected_revision)
        || request.expected_revision != 0 && request.expected_revision == i64::MAX as u64
    {
        return Err(RsStoreError::InvalidInput(
            "expected_revision or its successor exceeds SQLite's integer range".into(),
        ));
    }
    let mut uris = BTreeSet::new();
    for material in &request.materials {
        if !valid_uri(&material.uri) {
            return Err(RsStoreError::InvalidInput(format!(
                "invalid material URI: {}",
                material.uri
            )));
        }
        if !valid_token(&material.role) {
            return Err(RsStoreError::InvalidInput(format!(
                "invalid material role: {}",
                material.role
            )));
        }
        if !valid_digest(&material.sha256) {
            return Err(RsStoreError::InvalidInput(format!(
                "invalid SHA-256 for material: {}",
                material.uri
            )));
        }
        if !uris.insert(material.uri.clone()) {
            return Err(RsStoreError::InvalidInput(format!(
                "duplicate material URI: {}",
                material.uri
            )));
        }
        if let Some(source) = &material.source {
            let source = source.to_str().ok_or_else(|| {
                RsStoreError::InvalidInput("source path is not strict UTF-8".into())
            })?;
            if !valid_uri(source) {
                return Err(RsStoreError::UnsafePath(format!(
                    "invalid project-relative source path: {source}"
                )));
            }
        }
    }
    if let Some((existing, entry)) = manifest_entry_collision(&uris) {
        return Err(RsStoreError::InvalidInput(format!(
            "manifest names '{existing}' and '{entry}' collide: material names must stay distinct under Unicode canonical equivalence and case folding and cannot use one name as both file and directory"
        )));
    }
    reserve_response_capacity(request)?;
    if request.kind == "wish" {
        let flow_id = request.metadata.get("flow_id").and_then(Value::as_str);
        let registry_uri = request.metadata.get("registry_uri").and_then(Value::as_str);
        let run_uri = request.metadata.get("run_uri").and_then(Value::as_str);
        let (Some(flow_id), Some(registry_uri), Some(run_uri)) = (flow_id, registry_uri, run_uri)
        else {
            return Err(RsStoreError::InvalidInput(
                "wish metadata requires string flow_id, registry_uri and run_uri".into(),
            ));
        };
        if !matches!(flow_id, "simple-flow" | "wish-verify") {
            return Err(RsStoreError::InvalidInput(format!(
                "unsupported wish flow_id: {flow_id}"
            )));
        }
        if !valid_uri(registry_uri) || !valid_uri(run_uri) {
            return Err(RsStoreError::InvalidInput(
                "wish registry_uri or run_uri is invalid".into(),
            ));
        }
        for uri in [registry_uri, run_uri] {
            if !uris.contains(uri) {
                return Err(RsStoreError::InvalidInput(format!(
                    "wish metadata URI is absent from the complete manifest: {uri}"
                )));
            }
        }
    }
    Ok(())
}

/// Receipts and revision echoes re-emit request JSON inside bounded command
/// envelopes. A request that leaves no room for those wrappers is rejected
/// before reservation, so a committed operation always has a reportable
/// receipt.
fn reserve_response_capacity(request: &RsSaveRequest) -> Result<(), RsStoreError> {
    let metadata_value = Value::Object(request.metadata.clone());
    if json_depth(&request.result) + RESPONSE_DEPTH_MARGIN > MAX_JSON_DEPTH
        || json_depth(&metadata_value) + RESPONSE_DEPTH_MARGIN > MAX_JSON_DEPTH
    {
        return Err(RsStoreError::InvalidInput(format!(
            "result or metadata nesting exceeds the response envelope reservation of {} levels",
            MAX_JSON_DEPTH - RESPONSE_DEPTH_MARGIN
        )));
    }
    let result_bytes = compact_json(&request.result)?.len();
    let metadata_bytes = compact_json(&metadata_value)?.len();
    let manifest_bytes: usize = request
        .materials
        .iter()
        .map(|material| material.uri.len() + material.role.len() + material.sha256.len() + 96)
        .sum();
    let receipt_bytes = 256 + result_bytes;
    let revision_bytes = 384 + metadata_bytes + manifest_bytes;
    if receipt_bytes.max(revision_bytes) + RESPONSE_BYTES_MARGIN > MAX_JSON_BYTES {
        return Err(RsStoreError::InvalidInput(
            "result, metadata or manifest size leaves no room for the response envelope".into(),
        ));
    }
    Ok(())
}

/// Resolve a project-relative source onto its real filesystem path: the
/// longest existing prefix is canonicalized (resolving symlinks and
/// case-insensitive aliases) and the not-yet-existing tail is appended
/// lexically. Containment decisions must use this resolution, not the raw
/// spelling the caller happened to type.
fn resolve_within_project(project_root: &Path, source: &str) -> Result<PathBuf, RsStoreError> {
    let joined = project_root.join(source);
    let mut existing = joined.clone();
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    loop {
        match fs::symlink_metadata(&existing) {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(name) = existing.file_name().map(ToOwned::to_owned) else {
                    return Err(RsStoreError::UnsafePath(format!(
                        "invalid project-relative source path: {source}"
                    )));
                };
                tail.push(name);
                let Some(parent) = existing.parent().map(Path::to_path_buf) else {
                    return Err(RsStoreError::UnsafePath(format!(
                        "invalid project-relative source path: {source}"
                    )));
                };
                existing = parent;
            }
            Err(error) => return Err(io_error(error)),
        }
    }
    let mut resolved = fs::canonicalize(&existing).map_err(io_error)?;
    for name in tail.iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}

fn with_operation(error: RsStoreError, operation_id: &str) -> RsStoreError {
    match error {
        RsStoreError::Io { message, .. } => RsStoreError::Io {
            operation_id: Some(operation_id.to_owned()),
            message,
        },
        RsStoreError::Database { message, .. } => RsStoreError::Database {
            operation_id: Some(operation_id.to_owned()),
            message,
        },
        other => other,
    }
}

fn unknown(operation_id: &str) -> RsStoreError {
    RsStoreError::UnknownOutcome {
        operation_id: operation_id.to_owned(),
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug)]
enum FaultPoint {
    AfterReservation,
    AfterPublish,
    BeforeCommit,
    AfterCommit,
}

#[cfg(test)]
thread_local! {
    static FAULT_POINT: std::cell::Cell<Option<FaultPoint>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn fault(point: FaultPoint, operation_id: &str) -> Result<(), RsStoreError> {
    let selected = FAULT_POINT.with(|current| current.get());
    if selected
        .is_some_and(|selected| std::mem::discriminant(&selected) == std::mem::discriminant(&point))
    {
        FAULT_POINT.with(|current| current.set(None));
        return match point {
            FaultPoint::AfterReservation => Err(RsStoreError::Io {
                operation_id: Some(operation_id.to_owned()),
                message: "injected fault after reservation".into(),
            }),
            _ => Err(unknown(operation_id)),
        };
    }
    Ok(())
}

#[cfg(not(test))]
fn fault_after_reservation(_operation_id: &str) -> Result<(), RsStoreError> {
    Ok(())
}
#[cfg(test)]
fn fault_after_reservation(operation_id: &str) -> Result<(), RsStoreError> {
    fault(FaultPoint::AfterReservation, operation_id)
}
#[cfg(not(test))]
fn fault_after_publish(_operation_id: &str) -> Result<(), RsStoreError> {
    Ok(())
}
#[cfg(test)]
fn fault_after_publish(operation_id: &str) -> Result<(), RsStoreError> {
    fault(FaultPoint::AfterPublish, operation_id)
}
#[cfg(not(test))]
fn fault_before_commit(_operation_id: &str) -> Result<(), RsStoreError> {
    Ok(())
}
#[cfg(test)]
fn fault_before_commit(operation_id: &str) -> Result<(), RsStoreError> {
    fault(FaultPoint::BeforeCommit, operation_id)
}
#[cfg(not(test))]
fn fault_after_commit(_operation_id: &str) -> Result<(), RsStoreError> {
    Ok(())
}
#[cfg(test)]
fn fault_after_commit(operation_id: &str) -> Result<(), RsStoreError> {
    fault(FaultPoint::AfterCommit, operation_id)
}

enum Reservation {
    Pending,
    Terminal(RsOperationReceipt),
}

fn reserve(
    store: &RsStore,
    request: &RsSaveRequest,
    raw: &[u8],
) -> Result<Reservation, RsStoreError> {
    let transaction = Transaction::begin(&store.conn, true)?;
    store.verify_live()?;
    if let Some(row) = load_operation(&store.conn, &request.operation_id)? {
        let receipt = operation_receipt(&row)?;
        if row.request_bytes != raw || row.request_sha256 != sha256(raw) {
            transaction.rollback()?;
            return Err(RsStoreError::OperationMismatch {
                operation_id: request.operation_id.clone(),
            });
        }
        if row.outcome != RsOperationOutcome::Pending {
            transaction.rollback()?;
            return Ok(Reservation::Terminal(receipt));
        }
        transaction.rollback()?;
        return Ok(Reservation::Pending);
    }
    let result_json = String::from_utf8(compact_json(&request.result)?)
        .map_err(|_| RsStoreError::InvalidInput("result JSON is not UTF-8".into()))?;
    store.conn.execute(
        "INSERT INTO operations
         (operation_id, request_bytes, request_sha256, outcome, item_id,
          expected_revision, revision, result_json)
         VALUES (?1, ?2, ?3, 'pending', ?4, ?5, NULL, ?6)",
        rusqlite::params![
            request.operation_id,
            raw,
            sha256(raw),
            request.item_id,
            to_i64(request.expected_revision, "expected_revision")?,
            result_json,
        ],
    )?;
    let mut statement = store
        .conn
        .prepare("INSERT INTO pending_blobs (operation_id, sha256) VALUES (?1, ?2)")?;
    for digest in request
        .materials
        .iter()
        .map(|material| material.sha256.as_str())
        .collect::<BTreeSet<_>>()
    {
        statement.execute(rusqlite::params![request.operation_id, digest])?;
    }
    drop(statement);
    store.verify_live()?;
    transaction
        .commit()
        .map_err(|_| unknown(&request.operation_id))?;
    Ok(Reservation::Pending)
}

/// Read-only exact-request reconciliation before any project-tree access.
///
/// An idempotent retry must not depend on the transient source paths that the
/// first attempt happened to use. Once a request has been validated, an
/// existing operation row decides the call by its reserved bytes alone: a
/// different request is a mismatch, and a terminal outcome is returned as its
/// stored receipt. Only a new or still-pending operation proceeds to
/// project-root canonicalization, source containment, and `reserve`, which
/// rechecks the same conditions under its write transaction.
fn inspect_operation(
    store: &RsStore,
    request: &RsSaveRequest,
    raw: &[u8],
) -> Result<Option<Reservation>, RsStoreError> {
    let Some(row) = load_operation(&store.conn, &request.operation_id)? else {
        return Ok(None);
    };
    let receipt = operation_receipt(&row)?;
    if row.request_bytes != raw || row.request_sha256 != sha256(raw) {
        return Err(RsStoreError::OperationMismatch {
            operation_id: request.operation_id.clone(),
        });
    }
    if row.outcome == RsOperationOutcome::Pending {
        Ok(Some(Reservation::Pending))
    } else {
        Ok(Some(Reservation::Terminal(receipt)))
    }
}

fn blob_length(conn: &Connection, digest: &str) -> Result<Option<u64>, RsStoreError> {
    conn.query_row(
        "SELECT byte_length FROM blobs WHERE sha256 = ?1",
        [digest],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(database_error)?
    .map(|value| from_i64(value, "blob length"))
    .transpose()
}

struct Prepared {
    uri: String,
    role: String,
    sha256: String,
    byte_length: u64,
}

fn prepare_materials(
    store: &RsStore,
    project_root: &Path,
    request: &RsSaveRequest,
) -> Result<(Vec<Prepared>, Vec<(String, Vec<u8>)>), RsStoreError> {
    let mut prepared = Vec::new();
    let mut to_publish = Vec::new();
    let mut new_digests = BTreeSet::new();
    for material in &request.materials {
        // Published content is reused without consulting a source only when it
        // is attributable: either a committed blob row already records the
        // digest, or an earlier attempt of this same operation published it.
        // An unrelated operation's failed publication leaves an orphan blob
        // with no committed row and no attribution here, so source omission is
        // refused rather than adopting another operation's leftover.
        let committed_length = blob_length(&store.conn, &material.sha256)?;
        let byte_length = if let Some(length) = committed_length {
            let bytes = read_published_any(&store.root, &material.sha256)
                .map_err(|error| with_operation(error, &request.operation_id))?;
            if bytes.len() as u64 != length {
                return Err(RsStoreError::Integrity(format!(
                    "committed blob length differs for {}: expected {length}, read {}",
                    material.sha256,
                    bytes.len()
                )));
            }
            bytes.len() as u64
        } else if published_by_attempt(&store.root, &request.operation_id, &material.sha256)? {
            let bytes = read_published_any(&store.root, &material.sha256)
                .map_err(|error| with_operation(error, &request.operation_id))?;
            bytes.len() as u64
        } else if let Some(source) = &material.source {
            let source = source.to_str().ok_or_else(|| {
                RsStoreError::InvalidInput("source path is not strict UTF-8".into())
            })?;
            let bytes = read_source(project_root, source)
                .map_err(|error| with_operation(error, &request.operation_id))?;
            if sha256(&bytes) != material.sha256 {
                return Err(RsStoreError::InvalidInput(format!(
                    "source bytes do not match declared SHA-256: {}",
                    material.uri
                )));
            }
            if new_digests.insert(material.sha256.clone()) {
                let length = bytes.len() as u64;
                to_publish.push((material.sha256.clone(), bytes));
                length
            } else {
                bytes.len() as u64
            }
        } else {
            return Err(RsStoreError::NotFound(format!(
                "carried digest has never been committed: {}",
                material.sha256
            )));
        };
        if byte_length > MAX_MATERIAL_BYTES {
            return Err(RsStoreError::InvalidInput(
                "material exceeds 256 MiB".into(),
            ));
        }
        prepared.push(Prepared {
            uri: material.uri.clone(),
            role: material.role.clone(),
            sha256: material.sha256.clone(),
            byte_length,
        });
    }
    prepared.sort_by(|left, right| left.uri.cmp(&right.uri));
    to_publish.sort_by(|left, right| left.0.cmp(&right.0));
    Ok((prepared, to_publish))
}

fn item_row(conn: &Connection, item_id: &str) -> Result<Option<(String, u64)>, RsStoreError> {
    conn.query_row(
        "SELECT kind, head_revision FROM items WHERE item_id = ?1",
        [item_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
    )
    .optional()
    .map_err(database_error)?
    .map(|(kind, head)| Ok((kind, from_i64(head, "head revision")?)))
    .transpose()
}

fn finalize_operation(
    store: &RsStore,
    request: &RsSaveRequest,
    raw: &[u8],
    prepared: &[Prepared],
) -> Result<RsOperationReceipt, RsStoreError> {
    let transaction =
        Transaction::begin(&store.conn, true).map_err(|_| unknown(&request.operation_id))?;
    store
        .verify_live()
        .map_err(|_| unknown(&request.operation_id))?;
    let row = load_operation(&store.conn, &request.operation_id)
        .map_err(|_| unknown(&request.operation_id))?
        .ok_or_else(|| unknown(&request.operation_id))?;
    if row.request_bytes != raw || row.request_sha256 != sha256(raw) {
        return Err(RsStoreError::Integrity(
            "reserved operation bytes changed before commit".into(),
        ));
    }
    if row.outcome != RsOperationOutcome::Pending {
        let receipt = operation_receipt(&row).map_err(|_| unknown(&request.operation_id))?;
        transaction
            .rollback()
            .map_err(|_| unknown(&request.operation_id))?;
        return Ok(receipt);
    }
    let current =
        item_row(&store.conn, &request.item_id).map_err(|_| unknown(&request.operation_id))?;
    let conflict = match (&current, request.expected_revision) {
        (Some(_), 0) => true,
        (None, expected) => expected != 0,
        (Some((kind, head)), expected) => *kind != request.kind || *head != expected,
    };
    let next_revision = request.expected_revision + 1;
    if conflict {
        store.conn.execute(
            "UPDATE operations SET outcome = 'conflict', revision = NULL
             WHERE operation_id = ?1",
            [&request.operation_id],
        )?;
    } else {
        for material in prepared {
            read_published(&store.root, &material.sha256, material.byte_length)
                .map_err(|_| unknown(&request.operation_id))?;
        }
        if current.is_none() {
            store.conn.execute(
                "INSERT INTO items (item_id, kind, head_revision) VALUES (?1, ?2, 0)",
                rusqlite::params![request.item_id, request.kind],
            )?;
        }
        for material in prepared {
            if blob_length(&store.conn, &material.sha256)?.is_none() {
                store.conn.execute(
                    "INSERT INTO blobs (sha256, byte_length) VALUES (?1, ?2)",
                    rusqlite::params![
                        material.sha256,
                        to_i64(material.byte_length, "blob length")?
                    ],
                )?;
            }
        }
        let metadata =
            String::from_utf8(compact_json(&Value::Object(request.metadata.clone()))?)
                .map_err(|_| RsStoreError::InvalidInput("metadata JSON is not UTF-8".into()))?;
        store.conn.execute(
            "INSERT INTO revisions (item_id, revision, metadata_json, operation_id)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                request.item_id,
                to_i64(next_revision, "revision")?,
                metadata,
                request.operation_id,
            ],
        )?;
        for material in prepared {
            store.conn.execute(
                "INSERT INTO materials (item_id, revision, uri, role, sha256, byte_length)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    request.item_id,
                    to_i64(next_revision, "revision")?,
                    material.uri,
                    material.role,
                    material.sha256,
                    to_i64(material.byte_length, "material length")?,
                ],
            )?;
        }
        store.conn.execute(
            "UPDATE operations SET outcome = 'committed', revision = ?2
             WHERE operation_id = ?1",
            rusqlite::params![request.operation_id, to_i64(next_revision, "revision")?],
        )?;
        store.conn.execute(
            "UPDATE items SET head_revision = ?2 WHERE item_id = ?1",
            rusqlite::params![request.item_id, to_i64(next_revision, "revision")?],
        )?;
    }
    store.conn.execute(
        "DELETE FROM pending_blobs WHERE operation_id = ?1",
        [&request.operation_id],
    )?;
    store
        .verify_live()
        .map_err(|_| unknown(&request.operation_id))?;
    fault_before_commit(&request.operation_id)?;
    transaction
        .commit()
        .map_err(|_| unknown(&request.operation_id))?;
    fault_after_commit(&request.operation_id)?;
    blobs::fsync_dir(&store.root).map_err(|_| unknown(&request.operation_id))?;
    store
        .verify_live()
        .map_err(|_| unknown(&request.operation_id))?;
    let stored = load_operation(&store.conn, &request.operation_id)
        .map_err(|_| unknown(&request.operation_id))?
        .ok_or_else(|| unknown(&request.operation_id))?;
    operation_receipt(&stored).map_err(|_| unknown(&request.operation_id))
}

impl RsStore {
    pub fn save(
        &self,
        project_root: &Path,
        request: &RsSaveRequest,
    ) -> Result<RsOperationReceipt, RsStoreError> {
        let value = serde_json::to_value(request).map_err(|error| {
            RsStoreError::InvalidInput(format!("cannot encode save request: {error}"))
        })?;
        if json_depth(&value) > super::MAX_JSON_DEPTH {
            return Err(RsStoreError::InvalidInput(
                "save request nesting exceeds 64 levels".into(),
            ));
        }
        let raw = compact_json(&value)?;
        self.save_bytes(project_root, &raw)
    }

    pub fn save_bytes(
        &self,
        project_root: &Path,
        raw: &[u8],
    ) -> Result<RsOperationReceipt, RsStoreError> {
        let request = parse_request(raw)?;
        validate_request(&request)?;
        if sha256(raw).len() != 64 {
            return Err(RsStoreError::Integrity("cannot hash request".into()));
        }
        self.verify_live()?;
        // Decide exact-request identity and terminal receipts from the store
        // alone, before the transient project tree is touched: an identical
        // retry of a committed operation must replay even when the original
        // project root or source paths no longer exist. A new or pending
        // operation still re-validates its project tree and reserves below.
        if let Some(reservation) = inspect_operation(self, &request, raw)? {
            if let Reservation::Terminal(receipt) = reservation {
                // The receipt is durable; recovery staging for this operation
                // is no longer authoritative. Cleanup stays best effort so a
                // transient filesystem problem cannot hide a committed receipt.
                let _ = cleanup_operation_staging(&self.root, &request.operation_id);
                return Ok(receipt);
            }
        }
        let project_root = canonical_existing(project_root)?;
        if !project_root.is_dir() {
            return Err(RsStoreError::UnsafePath(
                "project_root is not a directory".into(),
            ));
        }
        for material in &request.materials {
            if let Some(source) = &material.source {
                let source = source.to_str().ok_or_else(|| {
                    RsStoreError::InvalidInput("source path is not strict UTF-8".into())
                })?;
                // Containment is decided on resolved filesystem identity:
                // case-insensitive volumes and symlinks resolve aliases of
                // the store root onto the root itself.
                let resolved = resolve_within_project(&project_root, source)?;
                if resolved.starts_with(&self.root) {
                    return Err(RsStoreError::UnsafePath(
                        "a store cannot ingest its own root".into(),
                    ));
                }
            }
        }
        match reserve(self, &request, raw)? {
            Reservation::Terminal(receipt) => {
                let _ = cleanup_operation_staging(&self.root, &request.operation_id);
                return Ok(receipt);
            }
            Reservation::Pending => {}
        }
        fault_after_reservation(&request.operation_id)?;
        let (prepared, to_publish) = prepare_materials(self, &project_root, &request)
            .map_err(|error| with_operation(error, &request.operation_id))?;
        let publication = blobs::publish_blobs(&self.root, &request.operation_id, &to_publish)
            .map_err(|_| unknown(&request.operation_id))?;
        fault_after_publish(&request.operation_id)?;
        // Publication staging is retained until a terminal receipt is durable:
        // a failed or unknown finalize must leave the attribution evidence that
        // lets a retry reuse this operation's own published bytes.
        let receipt =
            finalize_operation(self, &request, raw, &prepared).map_err(|error| match error {
                RsStoreError::UnknownOutcome { .. } => error,
                _ => unknown(&request.operation_id),
            })?;
        // A committed or conflict receipt no longer needs recovery evidence.
        // Removal cannot alter the durable outcome, so it is best effort and is
        // never turned into an unknown-outcome error.
        let _ = blobs::cleanup_publication(publication);
        let _ = cleanup_operation_staging(&self.root, &request.operation_id);
        Ok(receipt)
    }

    pub fn outcome(&self, operation_id: &str) -> Result<RsOperationReceipt, RsStoreError> {
        if !valid_id(operation_id) {
            return Err(RsStoreError::InvalidInput("invalid operation_id".into()));
        }
        self.verify_live()?;
        let transaction = Transaction::begin(&self.conn, false)?;
        let row = load_operation(&self.conn, operation_id)?.ok_or_else(|| {
            RsStoreError::NotFound(format!("operation not found: {operation_id}"))
        })?;
        let receipt = operation_receipt(&row)?;
        transaction.commit()?;
        Ok(receipt)
    }

    /// Pending operation IDs for an item, so callers can tell a leftover
    /// speculative attempt from work whose outcome still needs reconciling.
    pub fn pending_item_operations(&self, item_id: &str) -> Result<Vec<String>, RsStoreError> {
        if !valid_id(item_id) {
            return Err(RsStoreError::InvalidInput("invalid item_id".into()));
        }
        self.verify_live()?;
        let transaction = Transaction::begin(&self.conn, false)?;
        let mut statement = self.conn.prepare(
            "SELECT operation_id FROM operations
             WHERE item_id = ?1 AND outcome = 'pending' ORDER BY operation_id",
        )?;
        let ids = statement
            .query_map([item_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?;
        drop(statement);
        transaction.commit()?;
        Ok(ids)
    }

    pub fn list(
        &self,
        kind: Option<&str>,
        status: Option<&str>,
    ) -> Result<Vec<RsItemSummary>, RsStoreError> {
        if kind.is_some_and(|kind| !valid_token(kind)) {
            return Err(RsStoreError::InvalidInput("invalid kind filter".into()));
        }
        if status.is_some_and(|status| {
            !matches!(status, "received" | "active" | "blocked" | "delivered")
        }) {
            return Err(RsStoreError::InvalidInput(
                "invalid wish status filter".into(),
            ));
        }
        self.verify_live()?;
        let transaction = Transaction::begin(&self.conn, false)?;
        let mut statement = self.conn.prepare(
            "SELECT item_id FROM items
             WHERE (?1 IS NULL OR kind = ?1) ORDER BY item_id",
        )?;
        let ids = statement
            .query_map([kind], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?;
        drop(statement);
        let mut summaries = Vec::new();
        for item_id in ids {
            let revision = read_revision_tx(&self.conn, &self.root, &item_id, None)?;
            let Some(status) = status else {
                summaries.push(RsItemSummary {
                    item_id: revision.item_id,
                    head_revision: revision.revision,
                    kind: revision.kind,
                    projection: revision.projection,
                });
                continue;
            };
            let projected_status = revision
                .projection
                .as_ref()
                .map(|projection| projection.status);
            let matches = match (status, projected_status) {
                ("received", Some(RsWishStatus::Received))
                | ("active", Some(RsWishStatus::Active))
                | ("blocked", Some(RsWishStatus::Blocked))
                | ("delivered", Some(RsWishStatus::Delivered)) => true,
                _ => false,
            };
            if matches {
                summaries.push(RsItemSummary {
                    item_id: revision.item_id,
                    head_revision: revision.revision,
                    kind: revision.kind,
                    projection: revision.projection,
                });
            }
        }
        transaction.commit()?;
        Ok(summaries)
    }

    pub fn get(
        &self,
        item_id: &str,
        revision: Option<u64>,
    ) -> Result<RsItemRevision, RsStoreError> {
        validate_item_revision(item_id, revision)?;
        self.verify_live()?;
        let transaction = Transaction::begin(&self.conn, false)?;
        let result = read_revision_tx(&self.conn, &self.root, item_id, revision)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn history(&self, item_id: &str) -> Result<Vec<RsItemRevision>, RsStoreError> {
        validate_item_revision(item_id, None)?;
        self.verify_live()?;
        let transaction = Transaction::begin(&self.conn, false)?;
        let (_, head) = item_row(&self.conn, item_id)?
            .ok_or_else(|| RsStoreError::NotFound(format!("item not found: {item_id}")))?;
        let mut statement = self
            .conn
            .prepare("SELECT revision FROM revisions WHERE item_id = ?1 ORDER BY revision")?;
        let numbers = statement
            .query_map([item_id], |row| row.get::<_, i64>(0))?
            .map(|row| {
                row.map_err(database_error)
                    .and_then(|value| from_i64(value, "revision"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);
        if numbers.len() as u64 != head
            || numbers
                .iter()
                .enumerate()
                .any(|(index, revision)| *revision != index as u64 + 1)
        {
            return Err(RsStoreError::Integrity(format!(
                "item {item_id} has noncontiguous history"
            )));
        }
        let mut revisions = Vec::new();
        for revision in numbers {
            revisions.push(read_revision_tx(
                &self.conn,
                &self.root,
                item_id,
                Some(revision),
            )?);
        }
        transaction.commit()?;
        Ok(revisions)
    }

    pub fn read_material(
        &self,
        item_id: &str,
        revision: Option<u64>,
        uri: &str,
    ) -> Result<Vec<u8>, RsStoreError> {
        validate_item_revision(item_id, revision)?;
        if !valid_uri(uri) {
            return Err(RsStoreError::InvalidInput("invalid material URI".into()));
        }
        self.verify_live()?;
        let transaction = Transaction::begin(&self.conn, false)?;
        let selected = read_revision_tx(&self.conn, &self.root, item_id, revision)?;
        let material = selected
            .materials
            .iter()
            .find(|material| material.uri == uri)
            .cloned()
            .ok_or_else(|| RsStoreError::NotFound(format!("material URI not found: {uri}")))?;
        transaction.commit()?;
        read_published(&self.root, &material.sha256, material.byte_length)
    }

    pub fn checkout(
        &self,
        item_id: &str,
        revision: Option<u64>,
        destination: &Path,
    ) -> Result<RsCheckoutReport, RsStoreError> {
        let selected = self.get(item_id, revision)?;
        let canonical_destination = canonical_for_creation(destination)?;
        if canonical_destination.starts_with(&self.root) {
            return Err(RsStoreError::UnsafePath(
                "checkout destination cannot be inside the store".into(),
            ));
        }
        // First pass: verify every distinct published digest with bounded
        // buffers. Aliases share a digest, so this work is proportional to the
        // distinct blob set, not the manifest size.
        let mut verified = BTreeSet::new();
        for material in &selected.materials {
            if verified.insert(material.sha256.as_str()) {
                verify_published(&self.root, &material.sha256, material.byte_length)?;
            }
        }
        // Second pass: stream each blob into its manifest entry. Peak memory is
        // bounded by the copy buffer rather than growing with the URI count.
        let destination = blobs::materialize(&self.root, destination, &selected.materials)?;
        Ok(RsCheckoutReport {
            item_id: selected.item_id,
            revision: selected.revision,
            destination,
            material_count: selected.materials.len(),
        })
    }
}

fn validate_item_revision(item_id: &str, revision: Option<u64>) -> Result<(), RsStoreError> {
    if !valid_id(item_id) {
        return Err(RsStoreError::InvalidInput("invalid item_id".into()));
    }
    if revision.is_some_and(|revision| revision == 0 || !valid_revision(revision)) {
        return Err(RsStoreError::InvalidInput(
            "revision must be a positive SQLite integer".into(),
        ));
    }
    Ok(())
}

fn read_revision_tx(
    conn: &Connection,
    root: &Path,
    item_id: &str,
    revision: Option<u64>,
) -> Result<RsItemRevision, RsStoreError> {
    let (kind, head) = item_row(conn, item_id)?
        .ok_or_else(|| RsStoreError::NotFound(format!("item not found: {item_id}")))?;
    if head == 0 {
        return Err(RsStoreError::Integrity(format!(
            "item {item_id} has no committed revision"
        )));
    }
    let selected_revision = revision.unwrap_or(head);
    if selected_revision == 0 || !valid_revision(selected_revision) {
        return Err(RsStoreError::InvalidInput(
            "invalid selected revision".into(),
        ));
    }
    let (count, maximum): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COALESCE(MAX(revision), 0) FROM revisions WHERE item_id = ?1",
            [item_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(database_error)?;
    if count < 1 || maximum != head as i64 || count != maximum {
        return Err(RsStoreError::Integrity(format!(
            "item {item_id} has an invalid head or noncontiguous revisions"
        )));
    }
    let row = conn
        .query_row(
            "SELECT metadata_json, operation_id FROM revisions
             WHERE item_id = ?1 AND revision = ?2",
            rusqlite::params![item_id, to_i64(selected_revision, "revision")?],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| {
            RsStoreError::NotFound(format!(
                "revision {selected_revision} not found for item {item_id}"
            ))
        })?;
    let metadata = strict_json(row.0.as_bytes())
        .map_err(|error| RsStoreError::Integrity(format!("invalid revision metadata: {error}")))?
        .as_object()
        .cloned()
        .ok_or_else(|| RsStoreError::Integrity("revision metadata is not an object".into()))?;
    let operation = load_operation(conn, &row.1)?.ok_or_else(|| {
        RsStoreError::Integrity(format!(
            "revision {selected_revision} has no producing operation"
        ))
    })?;
    let receipt = operation_receipt(&operation)?;
    if receipt.outcome != RsOperationOutcome::Committed
        || receipt.revision != Some(selected_revision)
        || operation.item_id != item_id
    {
        return Err(RsStoreError::Integrity(format!(
            "revision {selected_revision} has an invalid producing receipt"
        )));
    }
    let request = parse_request(&operation.request_bytes).map_err(|error| {
        RsStoreError::Integrity(format!(
            "revision {selected_revision} has an invalid producing request: {error}"
        ))
    })?;
    if request.kind != kind
        || request.expected_revision + 1 != selected_revision
        || request.metadata != metadata
    {
        return Err(RsStoreError::Integrity(format!(
            "revision {selected_revision} disagrees with its producing request"
        )));
    }
    let mut statement = conn.prepare(
        "SELECT uri, role, sha256, byte_length FROM materials
         WHERE item_id = ?1 AND revision = ?2 ORDER BY uri",
    )?;
    let rows = statement
        .query_map(
            rusqlite::params![item_id, to_i64(selected_revision, "revision")?],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            },
        )?
        .collect::<Result<Vec<_>, _>>()
        .map_err(database_error)?;
    drop(statement);
    let mut materials = Vec::new();
    let mut uris = BTreeSet::new();
    for row in rows {
        let byte_length = from_i64(row.3, "material length")?;
        if !valid_uri(&row.0)
            || !valid_token(&row.1)
            || !valid_digest(&row.2)
            || byte_length > MAX_MATERIAL_BYTES
            || !uris.insert(row.0.clone())
            || blob_length(conn, &row.2)? != Some(byte_length)
        {
            return Err(RsStoreError::Integrity(format!(
                "revision {selected_revision} has an invalid material manifest"
            )));
        }
        materials.push(RsMaterial {
            uri: row.0,
            role: row.1,
            sha256: row.2,
            byte_length,
        });
    }
    let expected: Vec<_> = {
        let mut expected: Vec<_> = request
            .materials
            .iter()
            .map(|material| {
                (
                    material.uri.clone(),
                    material.role.clone(),
                    material.sha256.clone(),
                )
            })
            .collect();
        expected.sort_by(|left, right| left.0.cmp(&right.0));
        expected
    };
    let actual: Vec<_> = materials
        .iter()
        .map(|material| {
            (
                material.uri.clone(),
                material.role.clone(),
                material.sha256.clone(),
            )
        })
        .collect();
    if expected != actual {
        return Err(RsStoreError::Integrity(format!(
            "revision {selected_revision} manifest differs from its producing request"
        )));
    }
    let projection = if kind == "wish" {
        Some(wish_projection(
            root,
            item_id,
            selected_revision,
            &metadata,
            &materials,
        )?)
    } else {
        None
    };
    Ok(RsItemRevision {
        item_id: item_id.to_owned(),
        kind,
        revision: selected_revision,
        metadata,
        materials,
        operation_id: row.1,
        projection,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RegistryDocument {
    schema_version: u32,
    wish_id: String,
    flow_id: String,
    run_id: String,
    wish_model_uri: String,
    plan_uri: String,
    plan_sha256: String,
    work_uri: String,
    work_sha256: String,
    run_record_uri: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanDocument {
    plan_id: String,
    initial_stage_id: String,
    stages: Vec<PlanStage>,
    checkpoints: Vec<PlanCheckpoint>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanStage {
    stage_id: String,
    next_stage_ids: Vec<String>,
    exit_checkpoint_ids: Vec<String>,
    completes_flow: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanCheckpoint {
    checkpoint_id: String,
    stage_id: String,
    kind: RsCheckKind,
    criteria: Vec<String>,
    checker: Option<PlanChecker>,
    agent_may_judge: bool,
    human_may_judge: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanChecker {
    checker_id: String,
    command_uri: String,
    establishes: Vec<RsConformanceKind>,
    rules: Vec<PlanRule>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanRule {
    rule_id: String,
    condition: String,
    reads: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RunDocument {
    run_id: String,
    execution_type: String,
    plan_id: String,
    plan_uri: String,
    plan_sha256: String,
    wish_model_uri: String,
    status: RsFlowStatus,
    current_visit: usize,
    visits: Vec<RsStageVisit>,
}

fn protocol_material<'a>(
    root: &Path,
    materials: &'a [RsMaterial],
    uri: &str,
) -> Result<Vec<u8>, RsStoreError> {
    let material = materials
        .iter()
        .find(|material| material.uri == uri)
        .ok_or_else(|| {
            RsStoreError::Integrity(format!("manifest is missing required URI: {uri}"))
        })?;
    if material.byte_length > MAX_JSON_BYTES as u64 {
        return Err(RsStoreError::Integrity(format!(
            "protocol JSON exceeds 16 MiB: {uri}"
        )));
    }
    read_published(root, &material.sha256, material.byte_length)
}

fn strict_protocol<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    name: &str,
) -> Result<T, RsStoreError> {
    let value = strict_json(bytes)
        .map_err(|error| RsStoreError::Integrity(format!("invalid {name} JSON: {error}")))?;
    serde_json::from_value(value)
        .map_err(|error| RsStoreError::Integrity(format!("invalid {name} schema: {error}")))
}

fn wish_projection(
    root: &Path,
    item_id: &str,
    revision: u64,
    metadata: &Map<String, Value>,
    materials: &[RsMaterial],
) -> Result<RsWishProjection, RsStoreError> {
    let flow_id = metadata
        .get("flow_id")
        .and_then(Value::as_str)
        .ok_or_else(|| RsStoreError::Integrity("wish metadata has no flow_id".into()))?;
    let registry_uri = metadata
        .get("registry_uri")
        .and_then(Value::as_str)
        .ok_or_else(|| RsStoreError::Integrity("wish metadata has no registry_uri".into()))?;
    let run_uri = metadata
        .get("run_uri")
        .and_then(Value::as_str)
        .ok_or_else(|| RsStoreError::Integrity("wish metadata has no run_uri".into()))?;
    if !matches!(flow_id, "simple-flow" | "wish-verify")
        || !valid_uri(registry_uri)
        || !valid_uri(run_uri)
    {
        return Err(RsStoreError::Integrity(
            "wish projection metadata is invalid".into(),
        ));
    }
    let registry_bytes = protocol_material(root, materials, registry_uri)?;
    let registry: RegistryDocument = strict_protocol(&registry_bytes, "registry")?;
    let expected_plan_id = match flow_id {
        "simple-flow" => "designedSimpleFlow",
        "wish-verify" => "designedWishVerifyFlow",
        _ => return Err(RsStoreError::Integrity("unsupported wish flow".into())),
    };
    let run_uuid = uuid::Uuid::parse_str(&registry.run_id)
        .ok()
        .filter(|uuid| !uuid.is_nil() && uuid.to_string() == registry.run_id);
    if registry.schema_version != 1
        || registry.wish_id != item_id
        || registry.flow_id != flow_id
        || registry.run_record_uri != run_uri
        || run_uuid.is_none()
        || !valid_uri(&registry.wish_model_uri)
        || !valid_uri(&registry.plan_uri)
        || !valid_uri(&registry.work_uri)
        || !valid_digest(&registry.plan_sha256)
        || !valid_digest(&registry.work_sha256)
    {
        return Err(RsStoreError::Integrity(
            "registry does not match wish metadata or its strict identity rules".into(),
        ));
    }
    let plan_bytes = protocol_material(root, materials, &registry.plan_uri)?;
    if sha256(&plan_bytes) != registry.plan_sha256 {
        return Err(RsStoreError::Integrity(
            "retained plan digest differs from registry".into(),
        ));
    }
    let plan_value = strict_json(&plan_bytes)
        .map_err(|error| RsStoreError::Integrity(format!("invalid plan JSON: {error}")))?;
    let checkpoints = plan_value
        .get("checkpoints")
        .and_then(Value::as_array)
        .ok_or_else(|| RsStoreError::Integrity("plan checkpoints must be an array".into()))?;
    if checkpoints.iter().any(|checkpoint| {
        checkpoint
            .as_object()
            .is_none_or(|checkpoint| !checkpoint.contains_key("checker"))
    }) {
        return Err(RsStoreError::Integrity(
            "plan checkpoint is missing its checker member".into(),
        ));
    }
    let plan: PlanDocument = serde_json::from_value(plan_value)
        .map_err(|error| RsStoreError::Integrity(format!("invalid plan schema: {error}")))?;
    if plan.plan_id != expected_plan_id {
        return Err(RsStoreError::Integrity(
            "retained plan identity differs from selected flow".into(),
        ));
    }
    let work_bytes = protocol_material(root, materials, &registry.work_uri)?;
    if sha256(&work_bytes) != registry.work_sha256 {
        return Err(RsStoreError::Integrity(
            "retained work definition digest differs from registry".into(),
        ));
    }
    if !materials
        .iter()
        .any(|material| material.uri == registry.wish_model_uri)
    {
        return Err(RsStoreError::Integrity(
            "wish source is absent from the complete manifest".into(),
        ));
    }
    let run_bytes = protocol_material(root, materials, run_uri)?;
    let run: RunDocument = strict_protocol(&run_bytes, "run")?;
    if run.run_id != registry.run_id
        || run.execution_type != "FlowRun"
        || run.plan_id != plan.plan_id
        || run.plan_uri != registry.plan_uri
        || run.plan_sha256 != registry.plan_sha256
        || run.wish_model_uri != registry.wish_model_uri
        || run.current_visit >= run.visits.len()
    {
        return Err(RsStoreError::Integrity(
            "run does not match registry, retained plan, or current visit".into(),
        ));
    }
    let (status, current_stage) = match run.status {
        RsFlowStatus::Pending => (
            RsWishStatus::Received,
            Some(run.visits[run.current_visit].stage_id().to_owned()),
        ),
        RsFlowStatus::Active => (
            RsWishStatus::Active,
            Some(run.visits[run.current_visit].stage_id().to_owned()),
        ),
        RsFlowStatus::Blocked => (
            RsWishStatus::Blocked,
            Some(run.visits[run.current_visit].stage_id().to_owned()),
        ),
        RsFlowStatus::Completed => (RsWishStatus::Delivered, None),
    };
    if current_stage.as_ref().is_some_and(|stage| stage.is_empty()) {
        return Err(RsStoreError::Integrity(
            "current visit has an empty stage ID".into(),
        ));
    }
    Ok(RsWishProjection {
        revision,
        flow_id: flow_id.to_owned(),
        registry_uri: registry_uri.to_owned(),
        run_uri: run_uri.to_owned(),
        status,
        current_stage,
    })
}

pub(crate) fn request_bytes(
    store: &RsStore,
    operation_id: &str,
) -> Result<Option<Vec<u8>>, RsStoreError> {
    if !valid_id(operation_id) {
        return Err(RsStoreError::InvalidInput("invalid operation_id".into()));
    }
    store.verify_live()?;
    let transaction = Transaction::begin(&store.conn, false)?;
    let row = load_operation(&store.conn, operation_id)?;
    if let Some(row) = &row {
        operation_receipt(row)?;
    }
    transaction.commit()?;
    Ok(row.map(|row| row.request_bytes))
}

fn absolute_path(path: &Path) -> Result<PathBuf, RsStoreError> {
    if path.as_os_str().is_empty() {
        return Err(RsStoreError::InvalidInput("path must not be empty".into()));
    }
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir().map_err(io_error)?.join(path))
    }
}

fn push_finding(
    findings: &mut Vec<RsDoctorFinding>,
    kind: RsDoctorFindingKind,
    subject: impl Into<String>,
    detail: impl Into<String>,
) {
    findings.push(RsDoctorFinding {
        kind,
        subject_uri: subject.into(),
        detail: detail.into(),
    });
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn inspect_directory(
    root: &Path,
    path: &Path,
    findings: &mut Vec<RsDoctorFinding>,
) -> Option<fs::Metadata> {
    let subject = relative_display(root, path);
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            push_finding(
                findings,
                RsDoctorFindingKind::FormatDefect,
                subject,
                "required store directory is missing",
            );
            return None;
        }
        Err(error) => {
            push_finding(
                findings,
                RsDoctorFindingKind::PermissionDefect,
                subject,
                error.to_string(),
            );
            return None;
        }
    };
    if metadata.file_type().is_symlink() {
        push_finding(
            findings,
            RsDoctorFindingKind::IdentityMismatch,
            subject,
            "store directory is a symbolic link",
        );
        return None;
    }
    if !metadata.is_dir() {
        push_finding(
            findings,
            RsDoctorFindingKind::FormatDefect,
            subject,
            "store directory is not a directory",
        );
        return None;
    }
    if let Err(error) = blobs::store_dir(path) {
        push_finding(
            findings,
            RsDoctorFindingKind::PermissionDefect,
            subject,
            error.to_string(),
        );
    }
    Some(metadata)
}

fn inspect_records(
    conn: &Connection,
    root: &Path,
    findings: &mut Vec<RsDoctorFinding>,
    verify_bytes: bool,
) -> Result<(), RsStoreError> {
    macro_rules! query {
        ($sql:expr, $params:expr, $mapper:expr) => {{
            let mut statement = conn.prepare($sql)?;
            let rows = statement
                .query_map($params, $mapper)?
                .collect::<Result<Vec<_>, _>>()
                .map_err(database_error)?;
            rows
        }};
    }

    let transaction = Transaction::begin(conn, false)?;
    let operation_ids = query!(
        "SELECT operation_id FROM operations ORDER BY operation_id",
        [],
        |row| row.get::<_, String>(0)
    );
    for operation_id in operation_ids {
        match load_operation(conn, &operation_id) {
            Ok(Some(row)) => {
                if row.outcome == RsOperationOutcome::Pending {
                    push_finding(
                        findings,
                        RsDoctorFindingKind::PendingOperation,
                        &operation_id,
                        "operation is reserved but has no terminal receipt",
                    );
                }
                if let Err(error) = operation_receipt(&row) {
                    push_finding(
                        findings,
                        RsDoctorFindingKind::InvalidRecord,
                        &operation_id,
                        error.to_string(),
                    );
                }
                let reserved = query!(
                    "SELECT sha256 FROM pending_blobs WHERE operation_id = ?1 ORDER BY sha256",
                    [&operation_id],
                    |row| row.get::<_, String>(0)
                );
                if row.outcome != RsOperationOutcome::Pending && !reserved.is_empty() {
                    push_finding(
                        findings,
                        RsDoctorFindingKind::InvalidRecord,
                        &operation_id,
                        "terminal operation retains pending blob reservations",
                    );
                }
                if row.outcome == RsOperationOutcome::Pending {
                    if let Ok(request) = parse_request(&row.request_bytes) {
                        let expected: BTreeSet<_> = request
                            .materials
                            .iter()
                            .map(|material| material.sha256.clone())
                            .collect();
                        let actual: BTreeSet<_> = reserved.iter().cloned().collect();
                        if expected != actual {
                            push_finding(
                                findings,
                                RsDoctorFindingKind::InvalidRecord,
                                &operation_id,
                                "pending blob reservations differ from the request",
                            );
                        }
                    }
                }
            }
            Ok(None) => push_finding(
                findings,
                RsDoctorFindingKind::InvalidRecord,
                &operation_id,
                "operation disappeared during its snapshot",
            ),
            Err(error) => push_finding(
                findings,
                RsDoctorFindingKind::InvalidRecord,
                &operation_id,
                error.to_string(),
            ),
        }
    }

    let items = query!(
        "SELECT item_id, kind, head_revision FROM items ORDER BY item_id",
        [],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
        ))
    );
    for (item_id, kind, head) in items {
        if !valid_id(&item_id) || !valid_token(&kind) || head <= 0 {
            push_finding(
                findings,
                RsDoctorFindingKind::InvalidRecord,
                &item_id,
                "item identity, kind, or head is invalid",
            );
        }
        let (count, minimum, maximum): (i64, i64, i64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(MIN(revision), 0), COALESCE(MAX(revision), 0)
             FROM revisions WHERE item_id = ?1",
            [&item_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        if count < 1 || minimum != 1 || maximum != head || count != maximum {
            push_finding(
                findings,
                RsDoctorFindingKind::InvalidRecord,
                &item_id,
                "item revisions are missing, noncontiguous, or disagree with its head",
            );
        }
        let revisions = query!(
            "SELECT revision FROM revisions WHERE item_id = ?1 ORDER BY revision",
            [&item_id],
            |row| row.get::<_, i64>(0)
        );
        for revision in revisions {
            if revision <= 0 {
                push_finding(
                    findings,
                    RsDoctorFindingKind::InvalidRecord,
                    format!("{item_id}/{revision}"),
                    "revision number is not positive",
                );
            } else if let Err(error) = read_revision_tx(conn, root, &item_id, Some(revision as u64))
            {
                push_finding(
                    findings,
                    RsDoctorFindingKind::InvalidRecord,
                    format!("{item_id}/{revision}"),
                    error.to_string(),
                );
            }
        }
    }

    let blob_rows = query!(
        "SELECT sha256, byte_length FROM blobs ORDER BY sha256",
        [],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    );
    let material_rows = query!(
        "SELECT item_id, revision, uri, role, sha256, byte_length FROM materials
         ORDER BY item_id, revision, uri",
        [],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
        ))
    );
    let mut expected_blobs = BTreeMap::<String, u64>::new();
    for (digest, length) in blob_rows {
        if !valid_digest(&digest) || length < 0 || length as u64 > MAX_MATERIAL_BYTES {
            push_finding(
                findings,
                RsDoctorFindingKind::InvalidRecord,
                format!("blobs/{digest}"),
                "blob row has an invalid digest or length",
            );
        } else {
            expected_blobs.insert(digest, length as u64);
        }
    }
    let mut referenced = BTreeSet::new();
    for (item_id, revision, uri, role, digest, length) in material_rows {
        if !valid_uri(&uri)
            || !valid_token(&role)
            || !valid_digest(&digest)
            || revision <= 0
            || length < 0
            || length as u64 > MAX_MATERIAL_BYTES
        {
            push_finding(
                findings,
                RsDoctorFindingKind::InvalidRecord,
                format!("{item_id}/{revision}/{uri}"),
                "material row has an invalid identity, URI, role, or length",
            );
            continue;
        }
        referenced.insert(digest.clone());
        match expected_blobs.get(&digest) {
            Some(expected) if *expected == length as u64 => {}
            Some(_) => push_finding(
                findings,
                RsDoctorFindingKind::InvalidRecord,
                format!("{item_id}/{revision}/{uri}"),
                "material and blob lengths differ",
            ),
            None => {
                push_finding(
                    findings,
                    RsDoctorFindingKind::InvalidRecord,
                    format!("{item_id}/{revision}/{uri}"),
                    "material references no committed blob row",
                );
                expected_blobs.insert(digest, length as u64);
            }
        }
    }
    transaction.commit()?;
    inspect_blobs(root, expected_blobs, referenced, findings, verify_bytes)
}

fn inspect_blobs(
    root: &Path,
    expected: BTreeMap<String, u64>,
    referenced: BTreeSet<String>,
    findings: &mut Vec<RsDoctorFinding>,
    verify_bytes: bool,
) -> Result<(), RsStoreError> {
    for (digest, expected_length) in &expected {
        let path = blob_path(root, digest)?;
        let subject = relative_display(root, &path);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                push_finding(
                    findings,
                    RsDoctorFindingKind::MissingBlob,
                    subject,
                    "published blob is missing",
                );
                continue;
            }
            Err(error) => {
                push_finding(
                    findings,
                    RsDoctorFindingKind::MissingBlob,
                    subject,
                    error.to_string(),
                );
                continue;
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            push_finding(
                findings,
                RsDoctorFindingKind::MissingBlob,
                subject,
                "published blob is not a regular non-symlink file",
            );
            continue;
        }
        if let Err(error) = blobs::store_file(&path, 0o400) {
            push_finding(
                findings,
                RsDoctorFindingKind::PermissionDefect,
                &subject,
                error.to_string(),
            );
        }
        if metadata.len() != *expected_length {
            let kind = if metadata.len() < *expected_length {
                RsDoctorFindingKind::MissingBlob
            } else {
                RsDoctorFindingKind::AlteredBlob
            };
            push_finding(
                findings,
                kind,
                &subject,
                format!(
                    "blob length differs: recorded {expected_length}, observed {}",
                    metadata.len()
                ),
            );
        }
        if verify_bytes {
            match hash_file(&path) {
                Ok((length, actual)) => {
                    if actual != *digest || length != *expected_length {
                        push_finding(
                            findings,
                            RsDoctorFindingKind::AlteredBlob,
                            &subject,
                            "full byte verification differs from the blob identity",
                        );
                    }
                }
                Err(error) => push_finding(
                    findings,
                    RsDoctorFindingKind::AlteredBlob,
                    &subject,
                    format!("blob cannot be read through EOF: {error}"),
                ),
            }
        }
    }

    let sha256_root = root.join("blobs/sha256");
    let entries = match fs::read_dir(&sha256_root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error(error)),
    };
    for entry in entries {
        let entry = entry.map_err(io_error)?;
        let prefix_path = entry.path();
        let prefix = entry.file_name().to_string_lossy().into_owned();
        let metadata = fs::symlink_metadata(&prefix_path).map_err(io_error)?;
        if metadata.file_type().is_symlink() {
            push_finding(
                findings,
                RsDoctorFindingKind::IdentityMismatch,
                relative_display(root, &prefix_path),
                "blob prefix is a symbolic link",
            );
            continue;
        }
        if !metadata.is_dir() {
            push_finding(
                findings,
                RsDoctorFindingKind::FormatDefect,
                relative_display(root, &prefix_path),
                "unexpected non-directory in blob prefix root",
            );
            continue;
        }
        if prefix.len() != 2 || !prefix.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            push_finding(
                findings,
                RsDoctorFindingKind::FormatDefect,
                relative_display(root, &prefix_path),
                "invalid SHA-256 prefix directory",
            );
        }
        if let Err(error) = blobs::store_dir(&prefix_path) {
            push_finding(
                findings,
                RsDoctorFindingKind::PermissionDefect,
                relative_display(root, &prefix_path),
                error.to_string(),
            );
        }
        for entry in fs::read_dir(&prefix_path).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let digest = format!("{prefix}{name}");
            let subject = relative_display(root, &path);
            let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                push_finding(
                    findings,
                    RsDoctorFindingKind::FormatDefect,
                    subject,
                    "unexpected object in SHA-256 prefix directory",
                );
                continue;
            }
            if !valid_digest(&digest) {
                push_finding(
                    findings,
                    RsDoctorFindingKind::FormatDefect,
                    &subject,
                    "published filename is not a SHA-256 digest split",
                );
            }
            if !referenced.contains(&digest) {
                push_finding(
                    findings,
                    RsDoctorFindingKind::OrphanBlob,
                    &subject,
                    "published blob is not referenced by a committed manifest",
                );
            }
            if !expected.contains_key(&digest) {
                if let Err(error) = blobs::store_file(&path, 0o400) {
                    push_finding(
                        findings,
                        RsDoctorFindingKind::PermissionDefect,
                        &subject,
                        error.to_string(),
                    );
                }
                if verify_bytes && valid_digest(&digest) {
                    match hash_file(&path) {
                        Ok((_, actual)) if actual == digest => {}
                        Ok(_) => push_finding(
                            findings,
                            RsDoctorFindingKind::AlteredBlob,
                            &subject,
                            "published bytes do not hash to their filename",
                        ),
                        Err(error) => push_finding(
                            findings,
                            RsDoctorFindingKind::AlteredBlob,
                            &subject,
                            format!("blob cannot be read through EOF: {error}"),
                        ),
                    }
                }
            }
        }
    }
    Ok(())
}

fn hash_file(path: &Path) -> Result<(u64, String), std::io::Error> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut length = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        length += count as u64;
        hasher.update(&buffer[..count]);
    }
    Ok((length, format!("{:x}", hasher.finalize())))
}

fn inspect_staging(
    root: &Path,
    directory: &Path,
    findings: &mut Vec<RsDoctorFinding>,
) -> Result<(), RsStoreError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error(error)),
    };
    for entry in entries {
        let entry = entry.map_err(io_error)?;
        let path = entry.path();
        let subject = relative_display(root, &path);
        push_finding(
            findings,
            RsDoctorFindingKind::StagingArtifact,
            &subject,
            "transient staging residue is not authoritative material",
        );
        let metadata = fs::symlink_metadata(&path).map_err(io_error)?;
        if metadata.file_type().is_symlink() {
            push_finding(
                findings,
                RsDoctorFindingKind::IdentityMismatch,
                subject,
                "staging entry is a symbolic link",
            );
            continue;
        }
        if metadata.is_dir() {
            if let Err(error) = blobs::store_dir(&path) {
                push_finding(
                    findings,
                    RsDoctorFindingKind::PermissionDefect,
                    &subject,
                    error.to_string(),
                );
            }
            inspect_staging(root, &path, findings)?;
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode() & 0o7777;
                if mode != 0o600 && mode != 0o400 {
                    push_finding(
                        findings,
                        RsDoctorFindingKind::PermissionDefect,
                        &subject,
                        format!("staging file has unsafe mode {mode:04o}"),
                    );
                }
            }
        }
    }
    Ok(())
}

fn inspect_root(
    root: &Path,
    verify_bytes: bool,
    opened: Option<&RsStore>,
) -> Result<RsDoctorReport, RsStoreError> {
    let mut findings = Vec::new();
    let root_metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && opened.is_some() => {
            push_finding(
                &mut findings,
                RsDoctorFindingKind::IdentityMismatch,
                ".",
                "opened store root has been removed",
            );
            return Ok(RsDoctorReport {
                verify_bytes,
                findings,
            });
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(RsStoreError::NotFound(format!(
                "store does not exist: {}",
                root.display()
            )))
        }
        Err(error) => return Err(io_error(error)),
    };
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        push_finding(
            &mut findings,
            RsDoctorFindingKind::IdentityMismatch,
            ".",
            "store root is not a regular non-symlink directory",
        );
        return Ok(RsDoctorReport {
            verify_bytes,
            findings,
        });
    }
    if let Err(error) = blobs::store_dir(root) {
        push_finding(
            &mut findings,
            RsDoctorFindingKind::PermissionDefect,
            ".",
            error.to_string(),
        );
    }
    let (live_root_device, live_root_inode) = blobs::identity(&root_metadata);
    if let Some(store) = opened {
        if store.identity.root_device != live_root_device
            || store.identity.root_inode != live_root_inode
        {
            push_finding(
                &mut findings,
                RsDoctorFindingKind::IdentityMismatch,
                ".",
                "live root device/inode differs from the opened identity",
            );
        }
    }

    let allowed = [
        "format.json",
        "store.db",
        "store.db-wal",
        "store.db-shm",
        "blobs",
        "staging",
        "locks",
    ];
    match fs::read_dir(root) {
        Ok(entries) => {
            for entry in entries {
                let entry = entry.map_err(io_error)?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if !allowed.contains(&name.as_str()) {
                    push_finding(
                        &mut findings,
                        RsDoctorFindingKind::FormatDefect,
                        name,
                        "foreign content in store root",
                    );
                }
            }
        }
        Err(error) => push_finding(
            &mut findings,
            RsDoctorFindingKind::PermissionDefect,
            ".",
            error.to_string(),
        ),
    }
    let format = match db::load_format(root) {
        Ok(format) => {
            if let Some(store) = opened {
                if format.store_id != store.store_id {
                    push_finding(
                        &mut findings,
                        RsDoctorFindingKind::IdentityMismatch,
                        "format.json",
                        "format store_id differs from the opened store",
                    );
                }
            }
            Some(format)
        }
        Err(error) => {
            let kind = match error {
                RsStoreError::UnsafePath(_) => RsDoctorFindingKind::PermissionDefect,
                _ => RsDoctorFindingKind::FormatDefect,
            };
            push_finding(&mut findings, kind, "format.json", error.to_string());
            None
        }
    };
    for directory in ["blobs", "blobs/sha256", "staging", "locks", "locks/items"] {
        inspect_directory(root, &root.join(directory), &mut findings);
    }
    for sidecar in ["store.db-wal", "store.db-shm"] {
        let path = root.join(sidecar);
        if path.exists() {
            if let Err(error) = blobs::store_file(&path, 0o600) {
                push_finding(
                    &mut findings,
                    RsDoctorFindingKind::PermissionDefect,
                    sidecar,
                    error.to_string(),
                );
            }
        }
    }

    let database_path = root.join("store.db");
    let mut database_metadata = None;
    match fs::symlink_metadata(&database_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                push_finding(
                    &mut findings,
                    RsDoctorFindingKind::IdentityMismatch,
                    "store.db",
                    "database is not a regular non-symlink file",
                );
            } else {
                if let Err(error) = blobs::store_file(&database_path, 0o600) {
                    push_finding(
                        &mut findings,
                        RsDoctorFindingKind::PermissionDefect,
                        "store.db",
                        error.to_string(),
                    );
                }
                database_metadata = Some(metadata);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => push_finding(
            &mut findings,
            RsDoctorFindingKind::FormatDefect,
            "store.db",
            "store database is missing",
        ),
        Err(error) => push_finding(
            &mut findings,
            RsDoctorFindingKind::PermissionDefect,
            "store.db",
            error.to_string(),
        ),
    }
    if let (Some(store), Some(metadata)) = (opened, &database_metadata) {
        let (device, inode) = blobs::identity(metadata);
        if store.identity.database_device != device || store.identity.database_inode != inode {
            push_finding(
                &mut findings,
                RsDoctorFindingKind::IdentityMismatch,
                "store.db",
                "live database device/inode differs from the opened identity",
            );
        }
    }

    #[allow(unused_assignments)]
    let mut degraded = None;
    let connection = if let Some(store) = opened {
        Some(&store.conn)
    } else if database_metadata.is_some() {
        match db::read_only_connection(&database_path) {
            Ok(connection) => {
                degraded = Some(connection);
                degraded.as_ref()
            }
            Err(error) => {
                push_finding(
                    &mut findings,
                    RsDoctorFindingKind::FormatDefect,
                    "store.db",
                    error.to_string(),
                );
                None
            }
        }
    } else {
        None
    };

    if let Some(conn) = connection {
        let application_id =
            conn.query_row("PRAGMA application_id", [], |row| row.get::<_, i64>(0));
        let user_version = conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0));
        match (application_id, user_version) {
            (Ok(application), Ok(version)) => {
                if application != super::APPLICATION_ID || version != 1 {
                    push_finding(
                        &mut findings,
                        RsDoctorFindingKind::FormatDefect,
                        "store.db",
                        format!("unsupported SQLite application_id={application}, user_version={version}"),
                    );
                }
            }
            _ => push_finding(
                &mut findings,
                RsDoctorFindingKind::FormatDefect,
                "store.db",
                "SQLite identity cannot be read",
            ),
        }
        if let Err(error) = db::require_tables(conn) {
            push_finding(
                &mut findings,
                RsDoctorFindingKind::FormatDefect,
                "store.db",
                error.to_string(),
            );
        } else {
            match db::read_identity(conn) {
                Ok((store_id, identity)) => {
                    if format
                        .as_ref()
                        .is_some_and(|format| format.store_id != store_id)
                        || opened.is_some_and(|store| store.store_id != store_id)
                    {
                        push_finding(
                            &mut findings,
                            RsDoctorFindingKind::IdentityMismatch,
                            "store_identity",
                            "SQLite and format/opened store IDs differ",
                        );
                    }
                    if identity.root_device != live_root_device
                        || identity.root_inode != live_root_inode
                    {
                        push_finding(
                            &mut findings,
                            RsDoctorFindingKind::IdentityMismatch,
                            "store_identity",
                            "recorded root device/inode differs from the live root",
                        );
                    }
                    if let Some(metadata) = &database_metadata {
                        let (device, inode) = blobs::identity(metadata);
                        if identity.database_device != device || identity.database_inode != inode {
                            push_finding(
                                &mut findings,
                                RsDoctorFindingKind::IdentityMismatch,
                                "store_identity",
                                "recorded database device/inode differs from the live database",
                            );
                        }
                    }
                }
                Err(error) => push_finding(
                    &mut findings,
                    RsDoctorFindingKind::InvalidRecord,
                    "store_identity",
                    error.to_string(),
                ),
            }
            inspect_records(conn, root, &mut findings, verify_bytes)?;
        }
    }
    if connection.is_none() {
        inspect_blobs(
            root,
            BTreeMap::new(),
            BTreeSet::new(),
            &mut findings,
            verify_bytes,
        )?;
    }
    inspect_staging(root, &root.join("staging"), &mut findings)?;
    Ok(RsDoctorReport {
        verify_bytes,
        findings,
    })
}

impl RsStore {
    pub fn doctor(&self, verify_bytes: bool) -> Result<RsDoctorReport, RsStoreError> {
        inspect_root(&self.root, verify_bytes, Some(self))
    }
}

pub(crate) fn doctor_root(root: &Path, verify_bytes: bool) -> Result<RsDoctorReport, RsStoreError> {
    let root = match canonical_existing(root) {
        Ok(root) => root,
        Err(RsStoreError::UnsafePath(_)) => absolute_path(root)?,
        Err(error) => return Err(error),
    };
    inspect_root(&root, verify_bytes, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{RsSaveMaterial, RsSaveRequest};

    struct Fixture {
        project: PathBuf,
        root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let base = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("cache/implementation/storage-tests")
                .join(uuid::Uuid::new_v4().to_string());
            let project = base.join("project");
            fs::create_dir_all(&project).unwrap();
            Self {
                project,
                root: base.join("store"),
            }
        }

        fn request(&self) -> RsSaveRequest {
            let bytes = b"fault boundary bytes";
            fs::write(self.project.join("material.txt"), bytes).unwrap();
            RsSaveRequest {
                schema_version: 1,
                operation_id: "fault-operation".into(),
                item_id: "fault-item".into(),
                kind: "artifact".into(),
                expected_revision: 0,
                metadata: Map::new(),
                materials: vec![RsSaveMaterial {
                    uri: "material.txt".into(),
                    role: "artifact".into(),
                    sha256: sha256(bytes),
                    source: Some(PathBuf::from("material.txt")),
                }],
                result: serde_json::json!({"fault": true}),
            }
        }
    }

    #[test]
    fn fault_boundaries_reconcile_with_exact_retry_after_reopen() {
        for point in [
            FaultPoint::AfterReservation,
            FaultPoint::AfterPublish,
            FaultPoint::BeforeCommit,
            FaultPoint::AfterCommit,
        ] {
            let fixture = Fixture::new();
            let store = RsStore::init(&fixture.root).unwrap();
            let request = fixture.request();
            let raw = serde_json::to_vec_pretty(&request).unwrap();
            FAULT_POINT.with(|current| current.set(Some(point)));
            let error = store.save_bytes(&fixture.project, &raw).unwrap_err();
            match point {
                FaultPoint::AfterReservation => assert!(matches!(
                    error,
                    RsStoreError::Io { operation_id: Some(id), .. } if id == "fault-operation"
                )),
                _ => assert!(matches!(
                    error,
                    RsStoreError::UnknownOutcome { operation_id } if operation_id == "fault-operation"
                )),
            }
            drop(store);
            let store = RsStore::open(&fixture.root).unwrap();
            let outcome = store.outcome("fault-operation").unwrap();
            match point {
                FaultPoint::AfterCommit => {
                    assert_eq!(outcome.outcome, RsOperationOutcome::Committed);
                    assert_eq!(outcome.revision, Some(1));
                }
                _ => {
                    assert_eq!(outcome.outcome, RsOperationOutcome::Pending);
                    assert!(matches!(
                        store.get("fault-item", None),
                        Err(RsStoreError::NotFound(_))
                    ));
                }
            }
            assert_eq!(
                crate::storage::cli::request_bytes(&store, "fault-operation").unwrap(),
                Some(raw.clone())
            );
            let retried = store.save_bytes(&fixture.project, &raw).unwrap();
            assert_eq!(retried.outcome, RsOperationOutcome::Committed);
            assert_eq!(retried.revision, Some(1));
            assert_eq!(retried, store.outcome("fault-operation").unwrap());
            assert_eq!(store.history("fault-item").unwrap().len(), 1);
            assert_eq!(store.save_bytes(&fixture.project, &raw).unwrap(), retried);
            assert_eq!(store.history("fault-item").unwrap().len(), 1);
        }
    }

    #[test]
    fn pending_completes_from_published_blobs_after_source_loss() {
        let fixture = Fixture::new();
        let store = RsStore::init(&fixture.root).unwrap();
        let request = fixture.request();
        let raw = serde_json::to_vec_pretty(&request).unwrap();
        FAULT_POINT.with(|current| current.set(Some(FaultPoint::AfterPublish)));
        let error = store.save_bytes(&fixture.project, &raw).unwrap_err();
        assert!(matches!(
            error,
            RsStoreError::UnknownOutcome { operation_id } if operation_id == "fault-operation"
        ));
        // The process "died" after publication; the transient source is gone,
        // yet the digest-verified blob is inside the store.
        fs::remove_file(fixture.project.join("material.txt")).unwrap();
        let outcome = store.outcome("fault-operation").unwrap();
        assert_eq!(outcome.outcome, RsOperationOutcome::Pending);
        let retried = store.save_bytes(&fixture.project, &raw).unwrap();
        assert_eq!(retried.outcome, RsOperationOutcome::Committed);
        assert_eq!(retried.revision, Some(1));
        assert_eq!(
            store
                .read_material("fault-item", Some(1), "material.txt")
                .unwrap(),
            b"fault boundary bytes"
        );
    }

    #[test]
    fn finalize_refuses_missing_published_blob() {
        let fixture = Fixture::new();
        let store = RsStore::init(&fixture.root).unwrap();
        let request = fixture.request();
        let raw = serde_json::to_vec_pretty(&request).unwrap();
        reserve(&store, &request, &raw).unwrap();
        let (prepared, to_publish) = prepare_materials(&store, &fixture.project, &request).unwrap();
        let publication =
            blobs::publish_blobs(&store.root, &request.operation_id, &to_publish).unwrap();
        blobs::cleanup_publication(publication).unwrap();
        let digest = &request.materials[0].sha256;
        fs::remove_file(blob_path(&store.root, digest).unwrap()).unwrap();
        assert!(matches!(
            finalize_operation(&store, &request, &raw, &prepared),
            Err(RsStoreError::UnknownOutcome { operation_id }) if operation_id == "fault-operation"
        ));
        assert_eq!(
            store.outcome("fault-operation").unwrap().outcome,
            RsOperationOutcome::Pending
        );
        assert!(matches!(
            store.get("fault-item", None),
            Err(RsStoreError::NotFound(_))
        ));
    }

    #[test]
    fn own_publication_is_attributed_only_to_its_operation() {
        let fixture = Fixture::new();
        let store = RsStore::init(&fixture.root).unwrap();
        let request = fixture.request();
        let raw = serde_json::to_vec_pretty(&request).unwrap();
        FAULT_POINT.with(|current| current.set(Some(FaultPoint::AfterPublish)));
        let error = store.save_bytes(&fixture.project, &raw).unwrap_err();
        assert!(matches!(
            error,
            RsStoreError::UnknownOutcome { operation_id } if operation_id == "fault-operation"
        ));
        let digest = request.materials[0].sha256.clone();
        // The crashed attempt left its own publication staging attributed to it.
        assert!(published_by_attempt(&store.root, "fault-operation", &digest).unwrap());
        fs::remove_file(fixture.project.join("material.txt")).unwrap();

        // A different operation cannot adopt an orphan it did not publish,
        // even when it declares the same carried digest without a source.
        let mut foreign = request.clone();
        foreign.operation_id = "foreign-operation".into();
        foreign.item_id = "foreign-item".into();
        foreign.materials[0].source = None;
        let foreign_raw = serde_json::to_vec_pretty(&foreign).unwrap();
        assert!(matches!(
            store.save_bytes(&fixture.project, &foreign_raw),
            Err(RsStoreError::NotFound(_))
        ));
        assert_eq!(
            store.outcome("foreign-operation").unwrap().outcome,
            RsOperationOutcome::Pending
        );
        assert!(matches!(
            store.get("foreign-item", None),
            Err(RsStoreError::NotFound(_))
        ));

        // The original operation still recovers from the bytes it published.
        let retried = store.save_bytes(&fixture.project, &raw).unwrap();
        assert_eq!(retried.outcome, RsOperationOutcome::Committed);
        assert_eq!(retried.revision, Some(1));
        assert_eq!(
            store
                .read_material("fault-item", Some(1), "material.txt")
                .unwrap(),
            b"fault boundary bytes"
        );
        // A terminal receipt releases the recovery staging.
        assert!(!published_by_attempt(&store.root, "fault-operation", &digest).unwrap());

        // The now-committed blob row authorizes a later carried digest.
        let mut carried = request.clone();
        carried.operation_id = "carried-operation".into();
        carried.item_id = "carried-item".into();
        carried.materials[0].source = None;
        let carried_raw = serde_json::to_vec_pretty(&carried).unwrap();
        assert_eq!(
            store
                .save_bytes(&fixture.project, &carried_raw)
                .unwrap()
                .outcome,
            RsOperationOutcome::Committed
        );
    }
}
