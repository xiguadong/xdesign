use super::store::{self, Backend, Registry, Stored, StoredMaterial};
use super::work::{self, WorkDefinition};
use super::{judge, parse_result, read_result_bytes, response, restore, unjudged, validate_result};
use super::{RetainedCheck, StageResult};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};
use xdesign::checkpoint::RsCheckpointLibrary;
use xdesign::core::{
    create_run, sha256_bytes, valid_digest, RsActivityEntry, RsAdvanceOutcome, RsFlowRun,
    RsFlowStatus,
};
use xdesign::simple_flow::{
    apply_check_bindings, apply_check_bindings_with_library, RsCheckBindingConfig,
    RsCheckBindingMode,
};
use xdesign::storage::cli;
use xdesign::storage::{
    RsItemRevision, RsMaterial, RsOperationOutcome, RsOperationReceipt, RsSaveMaterial,
    RsSaveRequest, RsStore, RsStoreError,
};

pub fn store_root() -> PathBuf {
    env::var_os("XDESIGN_STORE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".xdesign"))
}

fn project_root() -> Result<PathBuf, String> {
    env::current_dir()
        .and_then(fs::canonicalize)
        .map_err(|error| format!("cannot resolve project root: {error}"))
}

fn error(error: RsStoreError) -> String {
    match error {
        RsStoreError::InvalidInput(message)
        | RsStoreError::UnsupportedFormat(message)
        | RsStoreError::NotFound(message)
        | RsStoreError::Integrity(message)
        | RsStoreError::UnsafePath(message) => message,
        RsStoreError::OperationMismatch { operation_id } => {
            format!("operation_mismatch: operation {operation_id} has different request bytes")
        }
        RsStoreError::Io {
            operation_id,
            message,
        }
        | RsStoreError::Database {
            operation_id,
            message,
        } => match operation_id {
            Some(operation_id) => format!("{message} (operation_id={operation_id})"),
            None => message,
        },
        RsStoreError::UnknownOutcome { operation_id } => {
            format!("unknown_outcome: reconcile operation_id={operation_id} before new work")
        }
    }
}

fn open() -> Result<RsStore, String> {
    RsStore::open(&store_root()).map_err(error)
}

pub fn item_exists(id: &str) -> Result<bool, String> {
    match fs::symlink_metadata(store_root()) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("cannot inspect store root: {error}")),
    }
    let store = open()?;
    match store.get(id, None) {
        Ok(_) => Ok(true),
        Err(RsStoreError::NotFound(_)) => Ok(false),
        Err(store_error) => Err(error(store_error)),
    }
}

pub struct Lock {
    path: PathBuf,
    contents: Vec<u8>,
}

impl Lock {
    pub fn acquire(id: &str) -> Result<Self, String> {
        store::wish_id(id)?;
        let path = store_root().join("locks/items").join(format!("{id}.lock"));
        let contents = format!(
            "pid={} token={}\n",
            std::process::id(),
            uuid::Uuid::new_v4()
        )
        .into_bytes();
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&path).map_err(|error| {
            format!(
                "wish writer is busy or store lock cannot be acquired; stale locks require manual operator action: {error}"
            )
        })?;
        file.write_all(&contents)
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("cannot write store lock {}: {error}", path.display()))?;
        Ok(Self { path, contents })
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        if fs::read(&self.path).ok().as_deref() == Some(self.contents.as_slice()) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[derive(Clone, Copy)]
pub enum LoadMode {
    Reader,
    Writer,
}

#[derive(Clone, Copy)]
enum Existing {
    Verify,
    Preserve,
}

fn publish_parent(uri: &str) -> Result<(), String> {
    store::state_path(uri)?;
    if let Some(parent) = Path::new(uri)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create parent for {uri}: {error}"))?;
    }
    store::state_path(uri)
}

fn existing(uri: &str, bytes: &[u8], behavior: Existing) -> Result<(), String> {
    store::state_path(uri)?;
    match behavior {
        Existing::Verify => {
            let actual = fs::read(uri).map_err(|error| format!("cannot read {uri}: {error}"))?;
            if actual != bytes {
                return Err(format!(
                    "frozen or immutable operational material drifted: {uri}"
                ));
            }
        }
        Existing::Preserve => {}
    }
    Ok(())
}

fn materialize(uri: &str, bytes: &[u8], behavior: Existing) -> Result<(), String> {
    for _ in 0..2 {
        match fs::symlink_metadata(uri) {
            Ok(_) => return existing(uri, bytes, behavior),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("cannot inspect {uri}: {error}")),
        }
        publish_parent(uri)?;
        match store::write_new(uri, bytes) {
            Ok(()) => return Ok(()),
            Err(error) => {
                if fs::symlink_metadata(uri).is_ok() {
                    continue;
                }
                return Err(error);
            }
        }
    }
    existing(uri, bytes, behavior)
}

/// The on-disk run record is a projection of the committed head. Stage
/// attempts stage engine output in the attempt area and publish it after the
/// store commits, so drift here means an interrupted publication or an
/// external change; the store is authoritative and the projection is repaired
/// from its bytes instead of failing the read or write.
fn materialize_run_record(uri: &str, bytes: &[u8]) -> Result<(), String> {
    match fs::read(uri) {
        Ok(actual) if actual == bytes => Ok(()),
        Ok(_) => {
            store::state_path(uri)?;
            store::replace_state(uri, bytes)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            materialize(uri, bytes, Existing::Preserve)
        }
        Err(error) => Err(format!("cannot read {uri}: {error}")),
    }
}

fn metadata_text<'a>(head: &'a RsItemRevision, key: &str) -> Result<&'a str, String> {
    head.metadata
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("stored wish metadata is missing {key}"))
}

fn material<'a>(head: &'a RsItemRevision, uri: &str) -> Result<&'a RsMaterial, String> {
    head.materials
        .iter()
        .find(|material| material.uri == uri)
        .ok_or_else(|| format!("stored head is missing material {uri}"))
}

fn read_material(
    store: &RsStore,
    id: &str,
    revision: u64,
    expected: &RsMaterial,
) -> Result<Vec<u8>, String> {
    let bytes = store
        .read_material(id, Some(revision), &expected.uri)
        .map_err(error)?;
    if bytes.len() as u64 != expected.byte_length || sha256_bytes(&bytes) != expected.sha256 {
        return Err(format!(
            "stored material failed length or digest verification: {}",
            expected.uri
        ));
    }
    Ok(bytes)
}

fn read_named(store: &RsStore, head: &RsItemRevision, uri: &str) -> Result<Vec<u8>, String> {
    read_material(store, &head.item_id, head.revision, material(head, uri)?)
}

pub fn load(id: &str, mode: LoadMode) -> Result<Stored, String> {
    let store = open()?;
    let head = store.get(id, None).map_err(error)?;
    if head.item_id != id || head.kind != "wish" || head.revision == 0 {
        return Err("stored item identity is not a committed wish".into());
    }
    let registry_uri = metadata_text(&head, "registry_uri")?.to_owned();
    let run_uri = metadata_text(&head, "run_uri")?.to_owned();
    let flow_id = metadata_text(&head, "flow_id")?.to_owned();
    let registry_bytes = read_named(&store, &head, &registry_uri)?;
    let registry: Registry = serde_json::from_slice(&registry_bytes)
        .map_err(|error| format!("invalid stored registration: {error}"))?;
    let dir = store::directory(id);
    if registry.schema_version != 1
        || registry.wish_id != id
        || registry.flow_id != flow_id
        || registry.wish_model_uri != format!("cache/wishes/{id}.sysml")
        || registry.plan_uri != format!("{dir}/plan.json")
        || registry.work_uri != format!("{dir}/work.json")
        || registry.run_record_uri != format!("{dir}/run.json")
        || registry.run_record_uri != run_uri
        || registry_uri != format!("{dir}/registry.json")
        || !valid_digest(&registry.plan_sha256)
        || !valid_digest(&registry.work_sha256)
    {
        return Err("stored registration identity or paths do not match this wish".into());
    }
    let plan_bytes = read_named(&store, &head, &registry.plan_uri)?;
    let work_bytes = read_named(&store, &head, &registry.work_uri)?;
    let run_bytes = read_named(&store, &head, &registry.run_record_uri)?;
    let wish_bytes = read_named(&store, &head, &registry.wish_model_uri)?;
    if sha256_bytes(&work_bytes) != registry.work_sha256 {
        return Err("stored frozen work definition changed".into());
    }
    materialize(&registry_uri, &registry_bytes, Existing::Verify)?;
    materialize(&registry.plan_uri, &plan_bytes, Existing::Verify)?;
    materialize(&registry.work_uri, &work_bytes, Existing::Verify)?;
    materialize(&registry.wish_model_uri, &wish_bytes, Existing::Preserve)?;
    let plan = store::parse_plan(&registry, &plan_bytes)?;
    let work: WorkDefinition = serde_json::from_slice(&work_bytes)
        .map_err(|error| format!("invalid stored work definition: {error}"))?;
    work.validate(&registry.flow_id, &plan, &registry.wish_model_uri)?;
    match mode {
        LoadMode::Reader | LoadMode::Writer => {
            materialize_run_record(&registry.run_record_uri, &run_bytes)?
        }
    }

    let reports = format!("{dir}/reports/");
    for entry in &head.materials {
        let operational = matches!(
            entry.role.as_str(),
            "checker-attempt"
                | "checker-executor"
                | "checker-record"
                | "checker-report"
                | "checker-stdout"
                | "checker-stderr"
                | "checker-input"
                | "artifact-snapshot"
        );
        if entry.role == "checker-executor" && !entry.uri.starts_with(&format!("{dir}/executors/"))
        {
            return Err(format!("executor material escapes its run: {}", entry.uri));
        }
        if entry.role == "artifact-snapshot"
            && !entry.uri.starts_with(&format!("{dir}/artifact-snapshots/"))
        {
            return Err(format!("artifact snapshot escapes its run: {}", entry.uri));
        }
        if operational {
            let report_confined = matches!(
                entry.role.as_str(),
                "checker-attempt"
                    | "checker-record"
                    | "checker-report"
                    | "checker-stdout"
                    | "checker-stderr"
            );
            if report_confined && !entry.uri.starts_with(&reports) {
                return Err(format!(
                    "checker material escapes the report directory: {}",
                    entry.uri
                ));
            }
            let bytes = read_material(&store, id, head.revision, entry)?;
            let behavior = if entry.role == "checker-input" {
                Existing::Preserve
            } else {
                Existing::Verify
            };
            materialize(&entry.uri, &bytes, behavior)?;
        }
    }

    for checker in plan.checkpoints.iter().filter_map(|g| g.checker.as_ref()) {
        if let Some(bundle) = &checker.bundle {
            for file in &bundle.files {
                let stored_file = material(&head, &file.uri)?;
                if !file.uri.starts_with(&format!("{dir}/executors/"))
                    || stored_file.role != "checker-executor"
                    || stored_file.sha256 != file.sha256
                {
                    return Err(
                        "frozen executor identity differs from its retained material".into(),
                    );
                }
            }
        }
    }

    let run_material = material(&head, &registry.run_record_uri)?;
    let snapshot = format!(
        "{dir}/heads/run-{}-{}.json",
        head.revision,
        &run_material.sha256[..16]
    );
    let restore_record_uri = match mode {
        LoadMode::Reader => {
            materialize(&snapshot, &run_bytes, Existing::Verify)?;
            snapshot
        }
        LoadMode::Writer => registry.run_record_uri.clone(),
    };
    let token = store::state_token(&[&registry_bytes, &plan_bytes, &work_bytes, &run_bytes]);
    let materials = head
        .materials
        .iter()
        .map(|entry| StoredMaterial {
            uri: entry.uri.clone(),
            role: entry.role.clone(),
            sha256: entry.sha256.clone(),
        })
        .collect();
    Ok(Stored {
        restore_record_uri,
        registry,
        plan,
        work,
        token,
        backend: Backend::Persistent,
        store_revision: Some(head.revision),
        materials,
    })
}

fn absolute(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path))
    }
}

fn validate_source(uri: &str) -> Result<(), String> {
    store::state_path(uri)?;
    let source = absolute(Path::new(uri))?;
    let root = absolute(&store_root())?;
    if source.starts_with(root) {
        return Err(format!(
            "store source cannot be inside the store root: {uri}"
        ));
    }
    Ok(())
}

fn read_source(uri: &str) -> Result<Vec<u8>, String> {
    validate_source(uri)?;
    let metadata = fs::metadata(uri).map_err(|error| format!("cannot read {uri}: {error}"))?;
    if !metadata.file_type().is_file() {
        return Err(format!("store source is not a regular file: {uri}"));
    }
    fs::read(uri).map_err(|error| format!("cannot read {uri}: {error}"))
}

fn base_metadata(registry: &Registry) -> Map<String, Value> {
    let mut metadata = Map::new();
    metadata.insert("flow_id".into(), json!(registry.flow_id));
    metadata.insert(
        "registry_uri".into(),
        json!(format!(
            "{}/registry.json",
            store::directory(&registry.wish_id)
        )),
    );
    metadata.insert("run_uri".into(), json!(registry.run_record_uri));
    metadata
}

struct Manifest {
    previous: BTreeMap<String, StoredMaterial>,
    entries: BTreeMap<String, RsSaveMaterial>,
}

impl Manifest {
    fn new(stored: &Stored) -> Self {
        Self {
            previous: stored
                .materials
                .iter()
                .cloned()
                .map(|material| (material.uri.clone(), material))
                .collect(),
            entries: BTreeMap::new(),
        }
    }

    fn role_for(&self, uri: &str, proposed: &str) -> String {
        if self.entries.get(uri).is_some_and(|entry| {
            matches!(
                entry.role.as_str(),
                "checker-executor" | "artifact-snapshot"
            )
        }) {
            return self.entries[uri].role.clone();
        }
        self.entries
            .get(uri)
            .filter(|entry| {
                matches!(
                    entry.role.as_str(),
                    "registration" | "frozen-plan" | "frozen-work" | "run-record" | "wish-source"
                ) && proposed == "checker-input"
            })
            .map(|entry| entry.role.clone())
            .unwrap_or_else(|| proposed.to_owned())
    }

    fn put_bytes(&mut self, uri: &str, role: &str, bytes: &[u8]) {
        let sha256 = sha256_bytes(bytes);
        let source = if self.previous.get(uri).map(|entry| &entry.sha256) == Some(&sha256) {
            None
        } else {
            Some(PathBuf::from(uri))
        };
        let material = RsSaveMaterial {
            uri: uri.to_owned(),
            role: self.role_for(uri, role),
            sha256,
            source,
        };
        self.entries.insert(uri.to_owned(), material);
    }

    fn put_file(&mut self, uri: &str, role: &str) -> Result<(), String> {
        self.put_file_from(uri, uri, role)
    }

    /// Register a material under its canonical URI while reading the bytes
    /// from an isolated attempt-area path, so speculative engine output never
    /// has to occupy its canonical location before the store commits.
    fn put_file_from(&mut self, source_uri: &str, uri: &str, role: &str) -> Result<(), String> {
        let bytes = read_source(source_uri)?;
        let sha256 = sha256_bytes(&bytes);
        let source = if self.previous.get(uri).map(|entry| &entry.sha256) == Some(&sha256) {
            None
        } else {
            Some(PathBuf::from(source_uri))
        };
        let material = RsSaveMaterial {
            uri: uri.to_owned(),
            role: self.role_for(uri, role),
            sha256,
            source,
        };
        self.entries.insert(uri.to_owned(), material);
        Ok(())
    }

    fn put_expected(&mut self, uri: &str, role: &str, digest: &str) -> Result<(), String> {
        if self
            .entries
            .get(uri)
            .is_some_and(|entry| entry.sha256 == digest)
        {
            let role = self.role_for(uri, role);
            if let Some(entry) = self.entries.get_mut(uri) {
                entry.role = role;
            }
            return Ok(());
        }
        if self.entries.contains_key(uri) {
            return match self.previous.get(uri) {
                Some(previous) if previous.sha256 == digest => Ok(()),
                _ => Err(format!("immutable checker input drifted: {uri}")),
            };
        }
        match read_source(uri) {
            Ok(bytes) if sha256_bytes(&bytes) == digest => {
                self.put_bytes(uri, role, &bytes);
                Ok(())
            }
            Ok(_) => self.carried_expected(uri, role, digest),
            Err(_error)
                if matches!(
                    fs::symlink_metadata(uri),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound
                ) =>
            {
                self.carried_expected(uri, role, digest)
            }
            Err(error) => Err(error),
        }
    }

    fn carried_expected(&mut self, uri: &str, role: &str, digest: &str) -> Result<(), String> {
        match self.previous.get(uri) {
            Some(previous) if previous.sha256 == digest => {
                let role = self.role_for(uri, role);
                self.entries.insert(
                    uri.to_owned(),
                    RsSaveMaterial {
                        uri: uri.to_owned(),
                        role,
                        sha256: digest.to_owned(),
                        source: None,
                    },
                );
                Ok(())
            }
            _ => Err(format!("immutable checker input drifted: {uri}")),
        }
    }

    fn retain_report_input(
        &mut self,
        uri: &str,
        digest: &str,
        source_uri: &str,
    ) -> Result<(), String> {
        if self.entries.get(uri).is_some_and(|m| m.sha256 == digest) {
            return Ok(());
        }
        // Earlier attempts can have assessed a different version of the same
        // source. Reuse that verified committed blob under its digest address,
        // rather than aliasing it to the visit's latest source version.
        if self.previous.values().any(|m| m.sha256 == digest) {
            self.entries.insert(
                uri.to_owned(),
                RsSaveMaterial {
                    uri: uri.to_owned(),
                    role: "artifact-snapshot".into(),
                    sha256: digest.to_owned(),
                    source: None,
                },
            );
            return Ok(());
        }
        let bytes = read_source(source_uri)?;
        if sha256_bytes(&bytes) != digest {
            return Err(format!(
                "checker input drifted before retention: {source_uri}"
            ));
        }
        self.entries.insert(
            uri.to_owned(),
            RsSaveMaterial {
                uri: uri.to_owned(),
                role: "artifact-snapshot".into(),
                sha256: digest.to_owned(),
                source: Some(PathBuf::from(source_uri)),
            },
        );
        Ok(())
    }

    fn put_current(&mut self, uri: &str, role: &str) -> Result<(), String> {
        match self.put_file(uri, role) {
            Ok(()) => Ok(()),
            Err(error)
                if matches!(
                    fs::symlink_metadata(uri),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound
                ) =>
            {
                match self.previous.get(uri).cloned() {
                    Some(previous) => {
                        self.entries.insert(
                            uri.to_owned(),
                            RsSaveMaterial {
                                uri: previous.uri,
                                role: previous.role,
                                sha256: previous.sha256,
                                source: None,
                            },
                        );
                        Ok(())
                    }
                    None => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }

    fn response(uri: &str, bytes: &[u8]) -> Result<(), String> {
        publish_parent(uri)?;
        store::write_new(uri, bytes)
    }

    fn into_vec(self) -> Vec<RsSaveMaterial> {
        self.entries.into_values().collect()
    }
}

/// The attempt area holds speculative stage output (engine run record and
/// stage response) under `<wish>/attempts/` until the store commits. Nothing
/// in it is authoritative, and a fresh attempt of the same operation replaces
/// its files wholesale.
fn attempt_directory(id: &str) -> String {
    format!("{}/attempts", store::directory(id))
}

fn stage_attempt_file(attempts: &str, name: &str, bytes: &[u8]) -> Result<String, String> {
    let staged = format!("{attempts}/{name}");
    store::state_path(&staged)?;
    match fs::symlink_metadata(&staged) {
        Ok(_) => store::replace_state(&staged, bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            store::write_new(&staged, bytes)
        }
        Err(error) => return Err(format!("cannot inspect {staged}: {error}")),
    }
    .map_err(|error| format!("cannot stage {staged}: {error}"))?;
    Ok(staged)
}

/// Stage the engine's working copy of the run record: seeded from the
/// canonical record (which the load step just repaired to the committed
/// head) and never published back until the store accepts the save.
fn stage_run_attempt(
    id: &str,
    operation_id: &str,
    canonical_run_uri: &str,
) -> Result<String, String> {
    let attempts = attempt_directory(id);
    store::state_path(&attempts)?;
    fs::create_dir_all(&attempts).map_err(|error| format!("cannot create {attempts}: {error}"))?;
    let base = fs::read(canonical_run_uri)
        .map_err(|error| format!("cannot read {canonical_run_uri}: {error}"))?;
    stage_attempt_file(&attempts, &format!("run-{operation_id}.json"), &base)
}

/// Attempt scratch of one operation; kept while its outcome is unknown (the
/// exact bytes are recovery sources for a pending retry), discarded otherwise.
fn discard_attempt(id: &str, operation_id: &str) {
    let attempts = attempt_directory(id);
    let _ = fs::remove_file(format!("{attempts}/run-{operation_id}.json"));
    let _ = fs::remove_file(format!("{attempts}/response-{operation_id}.json"));
}

/// Publish one committed material to its canonical URI. The run record
/// replaces the previous head's projection; a stage response is written new
/// or verified against an already-published copy.
fn publish_staged(uri: &str, bytes: &[u8], replace: bool) -> Result<(), String> {
    if replace {
        return store::replace_state(uri, bytes);
    }
    match fs::symlink_metadata(uri) {
        Ok(_) => {
            let existing = fs::read(uri).map_err(|error| format!("cannot read {uri}: {error}"))?;
            if existing == bytes {
                Ok(())
            } else {
                Err(format!(
                    "operational file conflicts with committed content: {uri}"
                ))
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            publish_parent(uri)?;
            store::write_new(uri, bytes)
        }
        Err(error) => Err(format!("cannot inspect {uri}: {error}")),
    }
}

/// Publish every attempt-staged material of a committed request from the
/// committed revision's verified material bytes. The attempt scratch and the
/// canonical projections are disposable, so a committed replay never depends
/// on either: the store's committed revision is the only source. Idempotent:
/// a retry after a crash between commit and publication replays this safely.
fn publish_attempt(store: &RsStore, revision: u64, request: &RsSaveRequest) -> Result<(), String> {
    let run_uri = request
        .metadata
        .get("run_uri")
        .and_then(Value::as_str)
        .ok_or_else(|| "stage request has no run_uri".to_owned())?
        .to_owned();
    let attempts_prefix = format!("{}/", attempt_directory(&request.item_id));
    for material in &request.materials {
        let Some(source) = &material.source else {
            continue;
        };
        if !source.to_string_lossy().starts_with(&attempts_prefix) {
            continue;
        }
        // Read the committed bytes through EOF (digest and length are verified
        // against the committed manifest) and require them to match the
        // committed request before projecting them into the canonical cache.
        let bytes = store
            .read_material(&request.item_id, Some(revision), &material.uri)
            .map_err(error)?;
        if sha256_bytes(&bytes) != material.sha256 {
            return Err(format!(
                "committed revision {revision} material {} does not match the committed request digest",
                material.uri
            ));
        }
        publish_staged(&material.uri, &bytes, material.uri == run_uri)?;
    }
    Ok(())
}

fn replay(receipt: &RsOperationReceipt, command: &str) -> Result<(Value, i32), String> {
    let result = receipt
        .result
        .as_object()
        .ok_or_else(|| "stored operation result is not an object".to_owned())?;
    if result.get("command").and_then(Value::as_str) != Some(command) {
        return Err("stored operation result names a different command".into());
    }
    let response = result
        .get("response")
        .filter(|response| response.is_object())
        .cloned()
        .ok_or_else(|| "stored operation result has no response".to_owned())?;
    let exit_code = result
        .get("exit_code")
        .and_then(Value::as_i64)
        .and_then(|code| i32::try_from(code).ok())
        .ok_or_else(|| "stored operation result has an invalid exit code".to_owned())?;
    Ok((response, exit_code))
}

fn finish_save(receipt: &RsOperationReceipt, command: &str) -> Result<(Value, i32), String> {
    match &receipt.outcome {
        RsOperationOutcome::Committed => replay(receipt, command),
        RsOperationOutcome::Conflict => Err(format!(
            "revision_conflict: stored head was not advanced by operation {}",
            receipt.operation_id
        )),
        RsOperationOutcome::Pending => Err(format!(
            "operation {} unexpectedly remained pending",
            receipt.operation_id
        )),
    }
}

pub fn init(
    id: &str,
    flow: &str,
    wish: Option<&str>,
    checks: Option<&str>,
) -> Result<Value, String> {
    store::wish_id(id)?;
    let mut plan = store::blueprint(flow)?;
    if item_exists(id)? {
        return Err(format!("wish {id} is already initialized"));
    }
    let mut library_inputs = std::collections::BTreeMap::new();
    if let Some(checks_uri) = checks {
        let bytes = store::read_file(checks_uri)?;
        let config: RsCheckBindingConfig = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid check binding JSON: {error}"))?;
        if config
            .checks
            .iter()
            .any(|binding| binding.mode == RsCheckBindingMode::Library)
        {
            let root = project_root()?;
            let library = RsCheckpointLibrary::load(&root)
                .map_err(|error| format!("invalid checkpoint library: {error}"))?;
            library_inputs = apply_check_bindings_with_library(&mut plan, &config, &library)
                .map_err(|error| format!("invalid check bindings: {error}"))?;
        } else {
            apply_check_bindings(&mut plan, &config)
                .map_err(|error| format!("invalid check bindings: {error}"))?;
        }
    }
    let source = wish
        .map(str::to_owned)
        .unwrap_or_else(|| format!("mbse/wishes/{id}.sysml"));
    store::artifact_path(&source)?;
    let imported = store::read_file(&source)?;
    let working = format!("cache/wishes/{id}.sysml");
    store::artifact_path(&working)?;
    if let Ok(existing) = fs::read(&working) {
        if existing != imported {
            return Err(format!("working wish already exists: {working}"));
        }
    }
    let dir = store::directory(id);
    // Operational files whose registry binds the mbse/ wish model, or that
    // exist without any store, belong to a legacy cache-backed run and are
    // never adopted or discarded by a persistent initialization.
    let legacy_directory = fs::read(format!("{dir}/registry.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|registry| {
            registry
                .get("wish_model_uri")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|uri| uri.starts_with("mbse/wishes/"));
    if fs::symlink_metadata(&dir).is_ok()
        && (legacy_directory || !fs::symlink_metadata(store_root()).is_ok())
    {
        return Err(format!("wish {id} is already initialized"));
    }
    let store = RsStore::init(&store_root()).map_err(error)?;
    let root = fs::canonicalize(store_root()).map_err(|error| error.to_string())?;
    let imported_path = fs::canonicalize(&source).map_err(|error| error.to_string())?;
    if imported_path.starts_with(root) {
        return Err("wish import source cannot be inside the store root".into());
    }
    match store.get(id, None) {
        Ok(_) => return Err(format!("wish {id} is already initialized")),
        Err(RsStoreError::NotFound(_)) => {}
        Err(store_error) => return Err(error(store_error)),
    }
    let _lock = Lock::acquire(id)?;
    // A leftover operational directory of this store without a committed
    // item belongs to an attempt that never committed. Only operations still
    // pending in the store can claim its files; anything else is discarded so
    // a transient failure before reservation never blocks a later
    // initialization.
    if fs::symlink_metadata(&dir).is_ok() {
        let pending = store.pending_item_operations(id).map_err(error)?;
        if !pending.is_empty() {
            return Err(format!(
                "wish {id} has no committed item but pending operations {pending:?} still reference its files; reconcile them with `xdesign store outcome` before reinitializing"
            ));
        }
        fs::remove_dir_all(&dir).map_err(|error| {
            format!("cannot discard leftover operational directory {dir}: {error}")
        })?;
    }
    let operation_id = uuid::Uuid::new_v4().to_string();
    let initialized = (|| -> Result<RsSaveRequest, String> {
        match fs::symlink_metadata(&working) {
            Ok(_) => {}
            Err(store_error) if store_error.kind() == std::io::ErrorKind::NotFound => {
                publish_parent(&working)?;
                store::write_new(&working, &imported)?;
            }
            Err(store_error) => return Err(format!("cannot inspect {working}: {store_error}")),
        }
        fs::create_dir_all("cache/xdesign").map_err(|error| error.to_string())?;
        fs::create_dir(&dir)
            .map_err(|error| format!("cannot reserve wish operational directory: {error}"))?;
        let plan_uri = format!("{dir}/plan.json");
        let work_uri = format!("{dir}/work.json");
        let run_record_uri = format!("{dir}/run.json");
        let executor_files = xdesign::simple_flow::resources::freeze_executors(&mut plan, &dir)?;
        plan.freeze(plan_uri.clone())
            .map_err(|error| format!("cannot freeze plan: {error:?}"))?;
        let work = WorkDefinition::freeze_with_inputs(flow, &plan, &working, &library_inputs)?;
        let work_bytes = serde_json::to_vec_pretty(&work).map_err(|error| error.to_string())?;
        store::write_new(&work_uri, &work_bytes)?;
        let mut run = create_run(
            plan.clone(),
            working.clone(),
            run_record_uri.clone(),
            Box::new(RetainedCheck {
                directory: format!("{dir}/reports"),
                missing_outputs: false,
            }),
            Box::new(unjudged()),
        )
        .map_err(|error| format!("cannot initialize run: {error:?}"))?;
        run.retain_completed_artifacts(format!("{dir}/artifact-snapshots"))?;
        run.write_record()?;
        let registry = Registry {
            schema_version: 1,
            wish_id: id.into(),
            flow_id: flow.into(),
            run_id: run.run_id().into(),
            wish_model_uri: working.clone(),
            plan_uri: plan_uri.clone(),
            plan_sha256: plan.plan_sha256.clone(),
            work_uri: work_uri.clone(),
            work_sha256: sha256_bytes(&work_bytes),
            run_record_uri: run_record_uri.clone(),
        };
        let registry_uri = format!("{dir}/registry.json");
        let registry_bytes =
            serde_json::to_vec_pretty(&registry).map_err(|error| error.to_string())?;
        store::write_new(&registry_uri, &registry_bytes)?;
        let plan_bytes = store::read_file(&plan_uri)?;
        let run_bytes = store::read_file(&run_record_uri)?;
        let stored = Stored {
            registry: registry.clone(),
            plan: plan.clone(),
            work,
            token: store::state_token(&[&registry_bytes, &plan_bytes, &work_bytes, &run_bytes]),
            restore_record_uri: run_record_uri,
            backend: Backend::Persistent,
            store_revision: Some(1),
            materials: vec![],
        };
        let mut reply = response(&stored, &run)?;
        reply["operation_id"] = json!(operation_id);
        let mut manifest = Manifest::new(&stored);
        manifest.put_file(&registry_uri, "registration")?;
        manifest.put_file(&plan_uri, "frozen-plan")?;
        manifest.put_file(&work_uri, "frozen-work")?;
        manifest.put_file(&stored.registry.run_record_uri, "run-record")?;
        manifest.put_file(&working, "wish-source")?;
        for file in &executor_files {
            manifest.put_expected(&file.uri, "checker-executor", &file.sha256)?;
        }
        let response_uri = format!("{dir}/responses/init-{operation_id}.json");
        let response_bytes =
            serde_json::to_vec_pretty(&reply).map_err(|error| error.to_string())?;
        Manifest::response(&response_uri, &response_bytes)?;
        manifest.put_file(&response_uri, "stage-response")?;
        let request = RsSaveRequest {
            schema_version: 1,
            operation_id: operation_id.clone(),
            item_id: id.into(),
            kind: "wish".into(),
            expected_revision: 0,
            metadata: base_metadata(&registry),
            materials: manifest.into_vec(),
            result: json!({"command": "init", "response": reply, "exit_code": 0}),
        };
        Ok(request)
    })();
    let request = match initialized {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_dir_all(&dir);
            if fs::read(&working).ok().as_deref() == Some(imported.as_slice()) {
                let _ = fs::remove_file(&working);
            }
            return Err(error);
        }
    };
    let receipt = match store.save(&project_root()?, &request) {
        Ok(receipt) => receipt,
        Err(store_error) => {
            // Keep the speculative files when the operation may be reserved
            // (unknown outcome) or a pending reservation still claims them;
            // otherwise a definitive failure cleans them so a retry starts fresh.
            let reserved = store
                .pending_item_operations(id)
                .map(|pending| !pending.is_empty())
                .unwrap_or(true);
            if !reserved && !matches!(store_error, RsStoreError::UnknownOutcome { .. }) {
                let _ = fs::remove_dir_all(&dir);
                if fs::read(&working).ok().as_deref() == Some(imported.as_slice()) {
                    let _ = fs::remove_file(&working);
                }
            }
            return Err(error(store_error));
        }
    };
    match &receipt.outcome {
        RsOperationOutcome::Committed => replay(&receipt, "init").map(|(response, _)| response),
        RsOperationOutcome::Conflict => Err(format!(
            "revision_conflict: wish {id} was initialized by another writer (operation_id={operation_id})"
        )),
        RsOperationOutcome::Pending => Err(format!(
            "initialization unexpectedly remained pending (operation_id={operation_id})"
        )),
    }
}

fn after_engine(stored: &Stored, run_record_uri: &str) -> Result<Stored, String> {
    let registry_uri = format!(
        "{}/registry.json",
        store::directory(&stored.registry.wish_id)
    );
    let registry_bytes = read_source(&registry_uri)?;
    let plan_bytes = read_source(&stored.registry.plan_uri)?;
    let work_bytes = read_source(&stored.registry.work_uri)?;
    let run_bytes = read_source(run_record_uri)?;
    let previous: BTreeMap<_, _> = stored
        .materials
        .iter()
        .map(|entry| (entry.uri.as_str(), entry.sha256.as_str()))
        .collect();
    for (uri, bytes) in [
        (&registry_uri, &registry_bytes),
        (&stored.registry.plan_uri, &plan_bytes),
        (&stored.registry.work_uri, &work_bytes),
    ] {
        if previous.get(uri.as_str()).copied() != Some(sha256_bytes(bytes).as_str()) {
            return Err(format!(
                "frozen registry, plan or work drifted during check: {uri}"
            ));
        }
    }
    if sha256_bytes(&plan_bytes) != stored.registry.plan_sha256
        || sha256_bytes(&work_bytes) != stored.registry.work_sha256
    {
        return Err("frozen plan or work drifted during check".into());
    }
    let mut projected = stored.clone();
    projected.store_revision = Some(
        stored
            .store_revision
            .ok_or("persistent check has no expected head")?
            .checked_add(1)
            .ok_or("store revision exhausted")?,
    );
    projected.restore_record_uri = stored.registry.run_record_uri.clone();
    projected.token = store::state_token(&[&registry_bytes, &plan_bytes, &work_bytes, &run_bytes]);
    Ok(projected)
}

#[allow(clippy::too_many_arguments)]
fn add_run_materials(
    manifest: &mut Manifest,
    stored: &Stored,
    run: &RsFlowRun,
    run_record_uri: &str,
) -> Result<(), String> {
    let registry_uri = format!(
        "{}/registry.json",
        store::directory(&stored.registry.wish_id)
    );
    manifest.put_expected(
        &registry_uri,
        "registration",
        &sha256_bytes(&read_source(&registry_uri)?),
    )?;
    manifest.put_file(&stored.registry.plan_uri, "frozen-plan")?;
    manifest.put_file(&stored.registry.work_uri, "frozen-work")?;
    manifest.put_file_from(
        run_record_uri,
        &stored.registry.run_record_uri,
        "run-record",
    )?;
    manifest.put_file(&stored.registry.wish_model_uri, "wish-source")?;

    for checker in stored
        .plan
        .checkpoints
        .iter()
        .filter_map(|gate| gate.checker.as_ref())
    {
        if let Some(bundle) = &checker.bundle {
            for file in &bundle.files {
                manifest.put_expected(&file.uri, "checker-executor", &file.sha256)?;
            }
        }
    }

    let mut artifacts = BTreeSet::new();
    let mut evidence = BTreeSet::new();
    let mut reports = BTreeMap::new();
    let report_prefix = format!("{}/reports/", store::directory(&stored.registry.wish_id));
    for visit in run.visits() {
        artifacts.extend(
            visit
                .artifact_digests()
                .iter()
                .map(|artifact| visit.retained_artifact_uri(&artifact.uri).to_owned()),
        );
        for assessment in visit.assessments() {
            for uri in &assessment.evidence_uris {
                evidence.insert(uri.clone());
                if uri.starts_with(&report_prefix)
                    && uri.ends_with(".json")
                    && !uri.ends_with(".record.json")
                {
                    reports.insert(uri.clone(), visit);
                }
            }
        }
    }
    for uri in artifacts.iter().chain(evidence.iter()) {
        if uri == &stored.registry.wish_model_uri || uri == &stored.registry.run_record_uri {
            continue;
        }
        let role = if uri.starts_with(&format!(
            "{}/artifact-snapshots/",
            store::directory(&stored.registry.wish_id)
        )) {
            "artifact-snapshot"
        } else {
            "artifact"
        };
        manifest.put_current(uri, role)?;
    }
    for (report, visit) in reports {
        let bytes = read_source(&report)?;
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("invalid retained checker report {report}: {error}"))?;
        let inputs = value
            .get("input_sha256")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("retained checker report has no input_sha256: {report}"))?;
        for (uri, digest) in inputs {
            let digest = digest
                .as_str()
                .filter(|digest| valid_digest(digest))
                .ok_or_else(|| format!("invalid checker input digest in {report}"))?;
            let was_artifact = visit.activities().iter().any(|activity| {
                activity
                    .artifact_digests
                    .iter()
                    .any(|a| a.uri == *uri && a.sha256 == digest)
            });
            if let Some(directory) = run.artifact_snapshot_directory().filter(|_| was_artifact) {
                manifest.retain_report_input(&format!("{directory}/{digest}"), digest, uri)?;
            } else if uri.starts_with(&report_prefix) && uri.ends_with(".record.json") {
                manifest.put_expected(uri, "checker-record", digest)?;
            } else {
                manifest.put_expected(uri, "checker-input", digest)?;
            }
        }
        manifest.put_expected(&report, "checker-report", &sha256_bytes(&bytes))?;
        for (suffix, role) in [
            (".stdout.log", "checker-stdout"),
            (".stderr.log", "checker-stderr"),
        ] {
            let uri = format!("{report}{suffix}");
            manifest.put_file(&uri, role)?;
        }
    }
    // Failed launches, timeouts and malformed reports have no trusted gate report,
    // but their direct output bytes still belong to the attempted stage revision.
    if let Ok(entries) = fs::read_dir(report_prefix.trim_end_matches('/')) {
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
                continue;
            }
            let uri = entry
                .path()
                .to_str()
                .ok_or("non-UTF-8 attempt path")?
                .to_owned();
            if manifest.entries.contains_key(&uri) {
                continue;
            }
            if let Some(previous) = manifest.previous.get(&uri).cloned() {
                manifest.put_expected(&uri, &previous.role, &previous.sha256)?;
            } else {
                manifest.put_file(&uri, "checker-attempt")?;
            }
        }
    }
    Ok(())
}

fn operation_mismatch(operation_id: &str) -> String {
    format!("operation_mismatch: operation {operation_id} names a different stage request")
}

fn recover(
    store: &RsStore,
    id: &str,
    operation_id: &str,
    result_bytes: &[u8],
    receipt: RsOperationReceipt,
) -> Result<(Value, i32), String> {
    let raw = cli::request_bytes(store, operation_id)
        .map_err(error)?
        .ok_or_else(|| format!("stored operation {operation_id} has no exact request bytes"))?;
    if sha256_bytes(&raw) != receipt.request_sha256 {
        return Err("stored operation request digest does not match its exact bytes".into());
    }
    let request: Value = serde_json::from_slice(&raw)
        .map_err(|error| format!("invalid retained stage request: {error}"))?;
    let attempt = request
        .get("metadata")
        .and_then(|metadata| metadata.get("stage_attempt"))
        .and_then(Value::as_object)
        .ok_or_else(|| operation_mismatch(operation_id))?;
    if request.get("item_id").and_then(Value::as_str) != Some(id)
        || request.get("operation_id").and_then(Value::as_str) != Some(operation_id)
        || attempt.get("command").and_then(Value::as_str) != Some("check")
        || attempt.get("result_sha256").and_then(Value::as_str)
            != Some(sha256_bytes(result_bytes).as_str())
    {
        return Err(operation_mismatch(operation_id));
    }
    let publish_committed = |receipt: &RsOperationReceipt| -> Result<(), String> {
        if receipt.outcome == RsOperationOutcome::Committed {
            let revision = receipt.revision.ok_or_else(|| {
                format!(
                    "committed operation {operation_id} has no committed revision to restore from"
                )
            })?;
            let request: RsSaveRequest = serde_json::from_slice(&raw)
                .map_err(|error| format!("invalid retained stage request: {error}"))?;
            publish_attempt(store, revision, &request)?;
            discard_attempt(id, operation_id);
        }
        Ok(())
    };
    match &receipt.outcome {
        RsOperationOutcome::Committed | RsOperationOutcome::Conflict => {
            let reply = finish_save(&receipt, "check")?;
            publish_committed(&receipt)?;
            Ok(reply)
        }
        RsOperationOutcome::Pending => {
            let receipt = store.save_bytes(&project_root()?, &raw).map_err(error)?;
            let reply = finish_save(&receipt, "check")?;
            publish_committed(&receipt)?;
            Ok(reply)
        }
    }
}

fn existing_outcome(
    store: &RsStore,
    operation_id: &str,
) -> Result<Option<RsOperationReceipt>, String> {
    match store.outcome(operation_id) {
        Ok(receipt) => Ok(Some(receipt)),
        Err(RsStoreError::NotFound(_)) => Ok(None),
        Err(store_error) => Err(error(store_error)),
    }
}

pub fn check(
    id: &str,
    result_uri: Option<&str>,
    operation_id: &str,
) -> Result<(Value, i32), String> {
    let outcome = (|| -> Result<(Value, i32), String> {
        let store = open()?;
        let _lock = Lock::acquire(id)?;
        if let Some(receipt) = existing_outcome(&store, operation_id)? {
            let result_bytes = read_result_bytes(result_uri)?;
            return recover(&store, id, operation_id, &result_bytes, receipt);
        }
        // No receipt exists for this operation, so nothing it staged has
        // authority: drop its speculative files and start from the head. A
        // transient failure before reservation therefore never blocks a retry.
        discard_attempt(id, operation_id);
        store::remove_state(&format!(
            "{}/responses/check-{operation_id}.json",
            store::directory(id)
        ))?;
        let stored = store::load_for_writer(id)?;
        let loaded = restore(&stored, unjudged(), false)?;
        if loaded.status() == RsFlowStatus::Completed {
            let mut reply = response(&stored, &loaded)?;
            reply["assessed"] = Value::Null;
            reply["advance"] = json!(RsAdvanceOutcome::CompleteRun);
            reply["operation_id"] = json!(operation_id);
            return Ok((reply, 0));
        }
        let result_bytes = read_result_bytes(result_uri)?;
        let result = parse_result(result_uri, &result_bytes)?;
        run_check(&store, stored, loaded, result, &result_bytes, operation_id)
    })();
    outcome.map_err(|error| format!("{error} (operation_id={operation_id})"))
}

fn run_check(
    store: &RsStore,
    stored: Stored,
    loaded: RsFlowRun,
    result: Option<StageResult>,
    result_bytes: &[u8],
    operation_id: &str,
) -> Result<(Value, i32), String> {
    let integrity: Vec<_> = loaded
        .visits()
        .iter()
        .filter(|visit| visit.status() == xdesign::core::RsStageStatus::Completed)
        .flat_map(|visit| visit.artifact_diagnostics())
        .collect();
    if !integrity.is_empty() {
        return Err(format!(
            "completed-stage integrity preflight failed before running checkers: {}",
            integrity.join("; ")
        ));
    }
    let supplied = validate_result(result.as_ref(), &stored, &loaded)?;
    let collection = work::collect(
        stored.work.stage(loaded.current().stage_id())?,
        loaded.current(),
        &supplied,
    )?;
    for artifact in &collection.artifacts {
        validate_source(&artifact.uri)?;
    }
    if loaded.artifact_snapshot_directory().is_some()
        && result
            .as_ref()
            .and_then(|r| r.judgment.as_ref())
            .is_some_and(|j| j.outcome == xdesign::core::RsCheckpointOutcome::Passed)
    {
        for change in super::workspace_changes(&loaded) {
            let uri = change["uri"].as_str().ok_or("invalid workspace change")?;
            let current = change["workspace_sha256"].as_str();
            if !collection
                .artifacts
                .iter()
                .any(|a| a.uri == uri && Some(a.sha256.as_str()) == current)
            {
                return Err(format!("unreviewed workspace change after completed stage: {uri}; include the current bytes in result.artifacts and bind the semantic judgment to the resulting snapshot"));
            }
        }
    }
    validate_source(&stored.registry.wish_model_uri)?;
    let judgment = judge(result.as_ref(), &collection)?;
    // The engine's writes are staged in the isolated attempt area; the
    // canonical run record stays at the committed head until publication.
    let attempt_run_uri = stage_run_attempt(
        &stored.registry.wish_id,
        operation_id,
        &stored.registry.run_record_uri,
    )?;
    let mut attempt_stored = stored.clone();
    attempt_stored.restore_record_uri = attempt_run_uri.clone();
    let mut run = restore(
        &attempt_stored,
        judgment,
        !collection.diagnostics.is_empty(),
    )?;
    let prior = run.current().artifact_digests();
    let updated: Vec<_> = collection
        .artifacts
        .iter()
        .filter(|artifact| !prior.contains(artifact))
        .cloned()
        .collect();
    let removed: Vec<_> = prior
        .iter()
        .filter(|artifact| {
            !collection
                .artifacts
                .iter()
                .any(|candidate| candidate.uri == artifact.uri)
        })
        .map(|artifact| artifact.uri.clone())
        .collect();
    let changed = !updated.is_empty() || !removed.is_empty();
    let named = if changed {
        updated
            .iter()
            .map(|artifact| artifact.uri.clone())
            .chain(removed.iter().cloned())
            .collect()
    } else {
        collection
            .artifacts
            .iter()
            .map(|artifact| artifact.uri.clone())
            .collect()
    };
    run.record_activity(RsActivityEntry {
        description: result
            .as_ref()
            .map(|result| result.description.clone())
            .unwrap_or_else(|| format!("Collect {} stage outputs", run.current().stage_id())),
        performed_by: result
            .as_ref()
            .map(|result| result.performed_by.clone())
            .unwrap_or_else(|| "xdesign check caller".into()),
        occurred_at: crate::timestamp(),
        artifact_uris: named,
        artifacts_changed: changed,
        artifact_digests: updated,
        removed_artifact_uris: removed,
    })?;
    let assessments = run.evaluate_gates();
    if let Some(error) = run.last_error() {
        return Err(format!("cannot retain assessment: {error}"));
    }
    let assessed = json!({ "stage_id": run.current().stage_id(), "visit_index": run.current().visit_index(),
        "artifact_revision": run.current().artifact_revision(), "artifact_snapshot_digest": run.current().artifact_snapshot_digest(),
        "assessments": assessments });
    let assessed_index = run.current_visit_index();
    let advance = run.advance();
    if let Some(error) = run.last_error() {
        return Err(format!("cannot retain advancement: {error}"));
    }
    if !run.visits()[assessed_index]
        .artifact_digests()
        .iter()
        .all(|a| xdesign::core::hash_artifact(&a.uri).is_ok_and(|actual| actual.sha256 == a.sha256))
    {
        return Err("current artifacts drifted during check; stored head was not advanced".into());
    }
    let final_state = after_engine(&stored, &attempt_run_uri)?;
    let mut reply = response(&final_state, &run)?;
    reply["assessed"] = assessed;
    reply["advance"] = json!(advance);
    reply["operation_id"] = json!(operation_id);
    let exit_code = if advance == RsAdvanceOutcome::RemainBlocked {
        1
    } else {
        0
    };
    commit(
        store,
        &stored,
        &final_state,
        &run,
        &attempt_run_uri,
        reply,
        exit_code,
        result_bytes,
        operation_id,
    )
}

#[allow(clippy::too_many_arguments)]
fn commit(
    store: &RsStore,
    stored: &Stored,
    final_state: &Stored,
    run: &RsFlowRun,
    attempt_run_uri: &str,
    reply: Value,
    exit_code: i32,
    result_bytes: &[u8],
    operation_id: &str,
) -> Result<(Value, i32), String> {
    if after_engine(stored, attempt_run_uri)?.token != final_state.token {
        return Err("run record changed while preparing the stage commit".into());
    }
    let id = stored.registry.wish_id.clone();
    let expected_revision = stored
        .store_revision
        .ok_or("persistent check has no expected head")?;
    let mut metadata = base_metadata(&stored.registry);
    metadata.insert(
        "stage_attempt".into(),
        json!({
            "base_state_token": stored.token,
            "result_sha256": sha256_bytes(result_bytes),
            "command": "check"
        }),
    );
    let mut manifest = Manifest::new(stored);
    add_run_materials(&mut manifest, stored, run, attempt_run_uri)?;
    let response_uri = format!(
        "{}/responses/check-{operation_id}.json",
        store::directory(&id)
    );
    let response_bytes = serde_json::to_vec_pretty(&reply).map_err(|error| error.to_string())?;
    let attempts = attempt_directory(&id);
    fs::create_dir_all(&attempts).map_err(|error| format!("cannot create {attempts}: {error}"))?;
    let attempt_response_uri = stage_attempt_file(
        &attempts,
        &format!("response-{operation_id}.json"),
        &response_bytes,
    )?;
    manifest.put_file_from(&attempt_response_uri, &response_uri, "stage-response")?;
    let request = RsSaveRequest {
        schema_version: 1,
        operation_id: operation_id.to_owned(),
        item_id: id.clone(),
        kind: "wish".into(),
        expected_revision,
        metadata,
        materials: manifest.into_vec(),
        result: json!({"command": "check", "response": reply, "exit_code": exit_code}),
    };
    let receipt = match store.save(&project_root()?, &request) {
        Ok(receipt) => receipt,
        Err(store_error) => {
            // Attempt files are the exact recovery sources of a reserved
            // operation; they are discarded only when nothing was reserved.
            let unknown = matches!(store_error, RsStoreError::UnknownOutcome { .. });
            let reserved = !matches!(existing_outcome(store, operation_id), Ok(None));
            if !unknown && !reserved {
                discard_attempt(&id, operation_id);
            }
            return Err(error(store_error));
        }
    };
    match &receipt.outcome {
        RsOperationOutcome::Committed => {
            let reply = finish_save(&receipt, "check")?;
            let revision = receipt.revision.ok_or_else(|| {
                format!(
                    "committed operation {operation_id} has no committed revision to restore from"
                )
            })?;
            publish_attempt(store, revision, &request)?;
            discard_attempt(&id, operation_id);
            Ok(reply)
        }
        RsOperationOutcome::Conflict => {
            discard_attempt(&id, operation_id);
            Err(format!(
                "revision_conflict: stored head was not advanced by operation {}",
                receipt.operation_id
            ))
        }
        RsOperationOutcome::Pending => Err(format!(
            "operation {} unexpectedly remained pending",
            receipt.operation_id
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_conflict_is_not_replayed_as_applied_work() {
        let base = Path::new("cache/implementation/persistent-tests")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&base).unwrap();
        let store = RsStore::init(&base.join("store")).unwrap();
        let result_bytes = b"stage result";
        let mut metadata = Map::new();
        metadata.insert(
            "stage_attempt".into(),
            json!({"base_state_token": "base", "result_sha256": sha256_bytes(result_bytes), "command": "check"}),
        );
        let request = |operation_id: &str| RsSaveRequest {
            schema_version: 1,
            operation_id: operation_id.into(),
            item_id: "item-1".into(),
            kind: "artifact".into(),
            expected_revision: 0,
            metadata: metadata.clone(),
            materials: vec![],
            result: json!({"command": "check", "response": {"not_applied": true}, "exit_code": 0}),
        };
        store.save(&base, &request("seed")).unwrap();
        let receipt = store.save(&base, &request("conflict")).unwrap();
        assert_eq!(receipt.outcome, RsOperationOutcome::Conflict);
        let error = recover(&store, "item-1", "conflict", result_bytes, receipt).unwrap_err();
        assert!(error.contains("revision_conflict"), "{error}");
    }

    #[test]
    fn checker_input_digest_mismatch_is_not_silently_omitted() {
        let directory = Path::new("cache/implementation/persistent-tests")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&directory).unwrap();
        let uri = directory.join("input.txt").to_string_lossy().into_owned();
        fs::write(&uri, b"actual").unwrap();
        let mut manifest = Manifest {
            previous: BTreeMap::new(),
            entries: BTreeMap::new(),
        };
        let error = manifest
            .put_expected(&uri, "checker-input", &sha256_bytes(b"expected"))
            .unwrap_err();
        assert!(error.contains("immutable checker input drifted"), "{error}");
        assert!(manifest.entries.is_empty());
    }

    #[test]
    fn superseded_checker_input_does_not_replace_current() {
        let directory = Path::new("cache/implementation/persistent-tests")
            .join(uuid::Uuid::new_v4().to_string());
        fs::create_dir_all(&directory).unwrap();
        let uri = directory.join("input.txt").to_string_lossy().into_owned();
        fs::write(&uri, b"current").unwrap();
        let old = sha256_bytes(b"old");
        let current = sha256_bytes(b"current");
        let mut manifest = Manifest {
            previous: BTreeMap::from([(
                uri.clone(),
                StoredMaterial {
                    uri: uri.clone(),
                    role: "artifact".into(),
                    sha256: old.clone(),
                },
            )]),
            entries: BTreeMap::from([(
                uri.clone(),
                RsSaveMaterial {
                    uri: uri.clone(),
                    role: "artifact".into(),
                    sha256: current.clone(),
                    source: Some(PathBuf::from(&uri)),
                },
            )]),
        };
        manifest.put_expected(&uri, "checker-input", &old).unwrap();
        assert_eq!(manifest.entries[&uri].sha256, current);
    }

    #[test]
    fn checker_report_role_is_not_left_as_artifact() {
        let uri = "cache/report.json".to_owned();
        let digest = sha256_bytes(b"report");
        let mut manifest = Manifest {
            previous: BTreeMap::new(),
            entries: BTreeMap::from([(
                uri.clone(),
                RsSaveMaterial {
                    uri: uri.clone(),
                    role: "artifact".into(),
                    sha256: digest.clone(),
                    source: None,
                },
            )]),
        };
        manifest
            .put_expected(&uri, "checker-report", &digest)
            .unwrap();
        assert_eq!(manifest.entries[&uri].role, "checker-report");
    }
}
