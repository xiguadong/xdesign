use super::work::WorkDefinition;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};
use xdesign::core::{sha256_bytes, valid_digest, RsFlowPlan};
use xdesign::simple_flow::{designed_simple_flow_plan, validate_designed_plan};
use xdesign::wish_verify::{validate_wish_verify_plan, wish_verify_flow_plan};

#[derive(Serialize, Deserialize)]
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

pub struct Stored {
    pub registry: Registry,
    pub plan: RsFlowPlan,
    pub work: WorkDefinition,
    pub token: String,
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
    file.write_all(bytes).map_err(|e| e.to_string())
}

pub fn blueprint(flow: &str) -> Result<RsFlowPlan, String> {
    match flow {
        "simple-flow" => Ok(designed_simple_flow_plan()),
        "wish-verify" => Ok(wish_verify_flow_plan()),
        _ => Err(format!("unsupported flow: {flow}")),
    }
}

pub fn load(id: &str) -> Result<Stored, String> {
    wish_id(id)?;
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
    let mut plan = blueprint(&registry.flow_id)?;
    plan.plan_uri = registry.plan_uri.clone();
    plan.plan_sha256 = registry.plan_sha256.clone();
    plan.validate()
        .map_err(|error| format!("invalid frozen plan: {error:?}"))?;
    match registry.flow_id.as_str() {
        "simple-flow" => validate_designed_plan(&plan),
        _ => validate_wish_verify_plan(&plan),
    }
    .map_err(|error| format!("invalid builtin plan: {error:?}"))?;
    let work: WorkDefinition =
        serde_json::from_slice(&work_bytes).map_err(|e| format!("invalid work definition: {e}"))?;
    work.validate(&registry.flow_id, &plan, &registry.wish_model_uri)?;
    let mut identity = Vec::new();
    for bytes in [&registry_bytes, &plan_bytes, &work_bytes, &run_bytes] {
        identity.extend_from_slice(&(bytes.len() as u64).to_be_bytes());
        identity.extend_from_slice(bytes);
    }
    Ok(Stored {
        registry,
        plan,
        work,
        token: sha256_bytes(&identity),
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
