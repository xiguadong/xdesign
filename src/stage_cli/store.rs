use super::work::WorkDefinition;
use super::StageError;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};
use xdesign::core::{sha256_bytes, valid_digest, RsFlowPlan};
use xdesign::simple_flow::{designed_simple_flow_plan, validate_designed_plan};
use xdesign::wish_verify::{validate_wish_verify_plan, wish_verify_flow_plan};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema_version: u32,
    pub wish_id: String,
    pub flow_id: String,
    pub run_id: String,
    pub wish_model_uri: String,
    pub plan_uri: String,
    pub plan_sha256: String,
    pub work_uri: String,
    pub work_sha256: String,
    pub run_record_uri: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Legacy,
    Persistent,
}

#[derive(Clone)]
pub struct StoredMaterial {
    pub uri: String,
    pub role: String,
    pub sha256: String,
}

#[derive(Clone)]
pub struct Stored {
    pub registry: Registry,
    pub plan: RsFlowPlan,
    pub work: WorkDefinition,
    pub token: String,
    pub restore_record_uri: String,
    pub backend: Backend,
    pub store_revision: Option<u64>,
    pub materials: Vec<StoredMaterial>,
}

pub fn wish_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(
            "wish ID must contain 1–128 ASCII letters, digits, hyphens or underscores".into(),
        );
    }
    Ok(())
}

pub fn directory(id: &str) -> String {
    format!("cache/xdesign/{id}")
}

fn normalized(uri: &str) -> Result<(), String> {
    if uri.is_empty()
        || uri.contains('\\')
        || uri.contains('\0')
        || uri
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || !Path::new(uri)
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(format!(
            "path must be normalized and project-relative: {uri}"
        ));
    }
    Ok(())
}

/// Validate the existing prefix too, so a missing output beyond a symlink cannot
/// evade containment checks. Missing files are left for the collection diagnostic.
pub fn contained_path(uri: &str) -> Result<PathBuf, String> {
    normalized(uri)?;
    let root = fs::canonicalize(".").map_err(|e| e.to_string())?;
    let mut prefix = PathBuf::new();
    for part in Path::new(uri).components() {
        prefix.push(part);
        match fs::symlink_metadata(&prefix) {
            Ok(_) => {
                let target = fs::canonicalize(&prefix)
                    .map_err(|e| format!("cannot resolve {}: {e}", prefix.display()))?;
                if !target.starts_with(&root) {
                    return Err(format!("path escapes project through symlink: {uri}"));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(format!("cannot inspect {uri}: {error}")),
        }
    }
    Ok(root.join(uri))
}

pub fn artifact_path(uri: &str) -> Result<PathBuf, String> {
    let path = contained_path(uri)?;
    let state = fs::canonicalize(".")
        .map_err(|e| e.to_string())?
        .join("cache/xdesign");
    let mut existing = path.as_path();
    while !existing.exists() {
        existing = existing.parent().ok_or("invalid artifact path")?;
    }
    let resolved = fs::canonicalize(existing).map_err(|e| e.to_string())?;
    if path.starts_with(&state) || resolved.starts_with(&state) {
        return Err(format!(
            "CLI state and checker exchange files cannot be stage artifacts: {uri}"
        ));
    }
    Ok(path)
}

pub fn read_file(uri: &str) -> Result<Vec<u8>, String> {
    let path = contained_path(uri)?;
    if !path.is_file() {
        return Err(format!("file does not exist: {uri}"));
    }
    fs::read(path).map_err(|e| format!("cannot read {uri}: {e}"))
}

pub fn state_path(uri: &str) -> Result<(), String> {
    contained_path(uri)?;
    let mut prefix = PathBuf::new();
    for part in Path::new(uri).components() {
        prefix.push(part);
        match fs::symlink_metadata(&prefix) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("state paths cannot contain symlinks: {uri}"))
            }
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(())
}

pub fn write_new(uri: &str, bytes: &[u8]) -> Result<(), String> {
    state_path(uri)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(uri)
        .map_err(|e| format!("cannot create {uri}: {e}"))?;
    file.write_all(bytes)
        .map_err(|e| format!("cannot write {uri}: {e}"))
}

/// Atomically replace a state file with new bytes: write a sibling temporary,
/// sync it, rename over the target, then sync the parent directory.
pub fn replace_state(uri: &str, bytes: &[u8]) -> Result<(), String> {
    state_path(uri)?;
    let temporary = format!("{uri}.{}.tmp", uuid::Uuid::new_v4());
    if let Some(parent) = Path::new(uri)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create parent for {uri}: {e}"))?;
    }
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| format!("cannot create {temporary}: {e}"))?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| format!("cannot write {temporary}: {e}"))?;
    }
    fs::rename(&temporary, uri).map_err(|e| format!("cannot replace {uri}: {e}"))?;
    if let Some(parent) = Path::new(uri)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        if let Ok(directory) = fs::File::open(parent) {
            let _ = directory.sync_all();
        }
    }
    Ok(())
}

/// Remove a state file if present; absent targets are not an error.
pub fn remove_state(uri: &str) -> Result<(), String> {
    state_path(uri)?;
    match fs::remove_file(uri) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("cannot remove {uri}: {error}")),
    }
}

pub fn blueprint(flow: &str) -> Result<RsFlowPlan, String> {
    match flow {
        "simple-flow" => Ok(designed_simple_flow_plan()),
        "wish-verify" => Ok(wish_verify_flow_plan()),
        _ => Err(format!("unsupported flow: {flow}")),
    }
}

pub fn state_token(parts: &[&[u8]]) -> String {
    let mut identity = Vec::new();
    for bytes in parts {
        identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        identity.extend_from_slice(bytes);
    }
    sha256_bytes(&identity)
}

/// Reconstruct a stored plan from its frozen bytes. The frozen serialization is
/// authoritative: rebuilding the builtin blueprint would silently drop a frozen
/// custom check binding. Legacy cache-backed plans load through the same path.
pub fn parse_plan(registry: &Registry, plan_bytes: &[u8]) -> Result<RsFlowPlan, String> {
    if sha256_bytes(plan_bytes) != registry.plan_sha256 {
        return Err("frozen plan definition changed".into());
    }
    let plan = RsFlowPlan::load_frozen(&registry.plan_uri, &registry.plan_sha256)
        .map_err(|error| format!("invalid frozen plan: {error:?}"))?;
    match registry.flow_id.as_str() {
        "simple-flow" => validate_designed_plan(&plan),
        "wish-verify" => validate_wish_verify_plan(&plan),
        _ => return Err("unsupported stored flow".into()),
    }
    .map_err(|error| format!("invalid flow plan: {error:?}"))?;
    Ok(plan)
}

pub fn is_persistent(
    id: &str,
    handle: Option<&xdesign::storage::RsStore>,
) -> Result<bool, StageError> {
    wish_id(id)?;
    match handle {
        Some(handle) => super::persistent::item_exists(id, handle),
        None => Ok(false),
    }
}

pub fn load(id: &str, handle: Option<&xdesign::storage::RsStore>) -> Result<Stored, StageError> {
    load_with_mode(id, super::persistent::LoadMode::Reader, handle)
}

pub fn load_for_writer(
    id: &str,
    handle: Option<&xdesign::storage::RsStore>,
) -> Result<Stored, StageError> {
    load_with_mode(id, super::persistent::LoadMode::Writer, handle)
}

fn load_with_mode(
    id: &str,
    mode: super::persistent::LoadMode,
    handle: Option<&xdesign::storage::RsStore>,
) -> Result<Stored, StageError> {
    wish_id(id)?;
    if is_persistent(id, handle)? {
        super::persistent::load(id, mode, handle.unwrap())
    } else {
        Ok(load_legacy(id)?)
    }
}

pub(super) fn load_legacy(id: &str) -> Result<Stored, String> {
    let dir = directory(id);
    let registry_uri = format!("{dir}/registry.json");
    state_path(&registry_uri)?;
    let registry_bytes = read_file(&registry_uri)
        .map_err(|e| format!("wish {id} is not initialized or registration is unreadable: {e}"))?;
    let registry: Registry = serde_json::from_slice(&registry_bytes)
        .map_err(|e| format!("invalid registration: {e}"))?;
    if registry.schema_version != 1
        || registry.wish_id != id
        || registry.wish_model_uri != format!("mbse/wishes/{id}.sysml")
        || registry.plan_uri != format!("{dir}/plan.json")
        || registry.work_uri != format!("{dir}/work.json")
        || registry.run_record_uri != format!("{dir}/run.json")
        || !valid_digest(&registry.plan_sha256)
        || !valid_digest(&registry.work_sha256)
    {
        return Err("registration identity or paths do not match this wish".into());
    }
    artifact_path(&registry.wish_model_uri)?;
    for uri in [
        &registry.plan_uri,
        &registry.work_uri,
        &registry.run_record_uri,
    ] {
        state_path(uri)?;
    }
    let plan_bytes = read_file(&registry.plan_uri)?;
    let work_bytes = read_file(&registry.work_uri)?;
    let run_bytes = read_file(&registry.run_record_uri)?;
    if sha256_bytes(&plan_bytes) != registry.plan_sha256
        || sha256_bytes(&work_bytes) != registry.work_sha256
    {
        return Err("frozen plan or work definition changed".into());
    }
    let plan = parse_plan(&registry, &plan_bytes)?;
    let work: WorkDefinition =
        serde_json::from_slice(&work_bytes).map_err(|e| format!("invalid work definition: {e}"))?;
    work.validate(&registry.flow_id, &plan, &registry.wish_model_uri)?;
    let token = state_token(&[&registry_bytes, &plan_bytes, &work_bytes, &run_bytes]);
    Ok(Stored {
        restore_record_uri: registry.run_record_uri.clone(),
        registry,
        plan,
        work,
        token,
        backend: Backend::Legacy,
        store_revision: None,
        materials: vec![],
    })
}

pub struct Lock {
    uri: String,
}

impl Lock {
    pub fn acquire(id: &str) -> Result<Self, String> {
        let uri = format!("{}/write.lock", directory(id));
        state_path(&uri)?;
        write_new(&uri, format!("pid={}\n", std::process::id()).as_bytes())
            .map_err(|error| format!("wish writer is busy or lock cannot be acquired: {error}"))?;
        Ok(Self { uri })
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.uri);
    }
}
