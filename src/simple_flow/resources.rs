//! Release resources and run-owned checker bundles (design D111, cases ISO-01..09).
use crate::core::{
    sha256_bytes, valid_relative_path, RsArtifactDigest, RsCheckerBundle, RsCheckerSpec, RsFlowPlan,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

const RESOURCES: &[(&str, &[u8])] = &[
    (
        "tools/checks/common.py",
        include_bytes!("../../tools/checks/common.py"),
    ),
    (
        "tools/checks/simple_requirements.py",
        include_bytes!("../../tools/checks/simple_requirements.py"),
    ),
    (
        "tools/checks/simple_design.py",
        include_bytes!("../../tools/checks/simple_design.py"),
    ),
    (
        "tools/checks/simple_implementation.py",
        include_bytes!("../../tools/checks/simple_implementation.py"),
    ),
    (
        "tools/checks/simple_record.py",
        include_bytes!("../../tools/checks/simple_record.py"),
    ),
    (
        "tools/sysml_web/bin/sysml",
        include_bytes!("../../tools/sysml_web/bin/sysml"),
    ),
    (
        "tools/sysml_web/toolchain.json",
        include_bytes!("../../tools/sysml_web/toolchain.json"),
    ),
    (
        "tools/sysml_web/licenses/LICENSE-MIT",
        include_bytes!("../../tools/sysml_web/licenses/LICENSE-MIT"),
    ),
    (
        "tools/sysml_web/licenses/LICENSE-APACHE",
        include_bytes!("../../tools/sysml_web/licenses/LICENSE-APACHE"),
    ),
    (
        "tools/sysml_web/licenses/crates__sysml-model__NOTICE",
        include_bytes!("../../tools/sysml_web/licenses/crates__sysml-model__NOTICE"),
    ),
    (
        "tools/sysml_web/licenses/crates__sysml-stdlib__LICENSE",
        include_bytes!("../../tools/sysml_web/licenses/crates__sysml-stdlib__LICENSE"),
    ),
    (
        "tools/sysml_web/licenses/crates__sysml-stdlib__NOTICE",
        include_bytes!("../../tools/sysml_web/licenses/crates__sysml-stdlib__NOTICE"),
    ),
];

pub(crate) fn is_builtin_command(uri: &str) -> bool {
    RESOURCES
        .iter()
        .any(|(name, _)| *name == uri && name.starts_with("tools/checks/simple_"))
}

/// Read a normalized project file without following any symlink component.
pub(crate) fn read_project_file(uri: &str) -> Result<Vec<u8>, String> {
    if !valid_relative_path(uri) {
        return Err(format!("invalid checker file: {uri}"));
    }
    let mut path = PathBuf::new();
    for component in Path::new(uri).components() {
        path.push(component);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|e| format!("cannot inspect checker file {uri}: {e}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("checker files cannot contain symlinks: {uri}"));
        }
    }
    if !path.is_file() {
        return Err(format!("checker file is not regular: {uri}"));
    }
    fs::read(&path).map_err(|e| format!("cannot read checker file {uri}: {e}"))
}

pub(crate) fn project_bundle(
    entry: &str,
    files: &[String],
    python: Option<&str>,
) -> Result<RsCheckerBundle, String> {
    if !entry.ends_with(".py") {
        return Err("checker entry must end in .py".into());
    }
    if python.is_some_and(|p| !valid_relative_path(p) || !p.starts_with("cache/")) {
        return Err(
            "checker python must be a normalized project-relative executable under cache/".into(),
        );
    }
    let mut seen = BTreeSet::from([entry.to_owned()]);
    for file in files {
        if !seen.insert(file.clone()) {
            return Err(format!("duplicate checker file: {file}"));
        }
    }
    if seen.len() > 256 {
        return Err("checker bundle exceeds 256 files".into());
    }
    let files = seen
        .into_iter()
        .map(|uri| {
            let sha256 = sha256_bytes(&read_project_file(&uri)?);
            Ok(RsArtifactDigest { uri, sha256 })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(RsCheckerBundle {
        source: "project".into(),
        entry_uri: entry.into(),
        files,
        python: python.map(str::to_owned),
    })
}

fn write_copy(uri: &str, bytes: &[u8]) -> Result<(), String> {
    // Validate all existing ancestors, including the destination, before creating anything.
    if !valid_relative_path(uri) {
        return Err("invalid executor destination".into());
    }
    let mut prefix = PathBuf::new();
    for component in Path::new(uri).components() {
        prefix.push(component);
        match fs::symlink_metadata(&prefix) {
            Ok(m) if m.file_type().is_symlink() => {
                return Err(format!(
                    "executor destination is a symlink: {}",
                    prefix.display()
                ))
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    if Path::new(uri).exists() {
        if read_project_file(uri)? != bytes {
            return Err(format!("executor material drifted: {uri}"));
        }
        return Ok(());
    }
    fs::create_dir_all(Path::new(uri).parent().ok_or("missing executor parent")?)
        .map_err(|e| e.to_string())?;
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(uri)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())
}

/// Freeze every executor before input discovery; the stage adapter persists all returned files.
pub fn freeze_executors(
    plan: &mut RsFlowPlan,
    run_directory: &str,
) -> Result<Vec<RsArtifactDigest>, String> {
    let mut all = BTreeMap::new();
    for gate in &mut plan.checkpoints {
        let Some(checker) = gate.checker.as_mut() else {
            continue;
        };
        let builtin = checker
            .bundle
            .as_ref()
            .is_some_and(|b| b.source == "builtin-v1")
            || checker.bundle.is_none()
                && (checker.custom.is_none()
                    || checker
                        .custom
                        .as_ref()
                        .is_some_and(|c| c.runner == crate::core::RsCustomRunner::Cli)
                    || gate
                        .library
                        .as_deref()
                        .is_some_and(|id| crate::checkpoint::BUILTIN_IDS.contains(&id)));
        let prefix = if builtin {
            format!("{run_directory}/executors/builtin-v1")
        } else {
            format!("{run_directory}/executors/{}", gate.checkpoint_id)
        };
        let mut bundle = if builtin {
            if !is_builtin_command(&checker.command_uri) {
                return Err("unknown built-in checker resource".into());
            }
            RsCheckerBundle {
                source: "builtin-v1".into(),
                entry_uri: checker.command_uri.clone(),
                files: vec![],
                python: None,
            }
        } else {
            checker
                .bundle
                .clone()
                .map(Ok)
                .unwrap_or_else(|| project_bundle(&checker.command_uri, &[], None))?
        };
        let mut files = vec![];
        if builtin {
            for (uri, bytes) in RESOURCES {
                let destination = format!("{prefix}/{uri}");
                write_copy(&destination, bytes)?;
                files.push(RsArtifactDigest {
                    uri: destination,
                    sha256: sha256_bytes(bytes),
                });
            }
        } else {
            for material in &bundle.files {
                let bytes = read_project_file(&material.uri)?;
                if sha256_bytes(&bytes) != material.sha256 {
                    return Err(format!(
                        "checker source changed while freezing: {}",
                        material.uri
                    ));
                }
                let destination = format!("{prefix}/{}", material.uri);
                write_copy(&destination, &bytes)?;
                files.push(RsArtifactDigest {
                    uri: destination,
                    sha256: material.sha256.clone(),
                });
            }
        }
        if let Some(python) = &bundle.python {
            if !Path::new(python).is_file() {
                return Err(format!("checker interpreter is missing: {python}"));
            }
        }
        bundle.entry_uri = format!("{prefix}/{}", bundle.entry_uri);
        bundle.files = files;
        for file in &bundle.files {
            all.insert(file.uri.clone(), file.clone());
        }
        checker.bundle = Some(bundle);
    }
    Ok(all.into_values().collect())
}

pub(crate) fn verify_bundle(bundle: &RsCheckerBundle) -> Result<(), String> {
    if !bundle.valid() {
        return Err("invalid checker bundle".into());
    }
    for file in &bundle.files {
        if sha256_bytes(&read_project_file(&file.uri)?) != file.sha256 {
            return Err(format!("executor material drifted: {}", file.uri));
        }
    }
    Ok(())
}

pub fn command_map(plan: &RsFlowPlan) -> BTreeMap<String, String> {
    plan.checkpoints
        .iter()
        .filter_map(|gate| gate.checker.as_ref())
        .map(|checker| {
            (
                checker.command_uri.clone(),
                checker
                    .bundle
                    .as_ref()
                    .map_or_else(|| checker.command_uri.clone(), |b| b.entry_uri.clone()),
            )
        })
        .collect()
}

/// Construct the same bounded Python context for discovery and evaluation.
pub fn python_command(
    checker: &RsCheckerSpec,
    attempt: &str,
    commands: &BTreeMap<String, String>,
) -> Result<Command, String> {
    let root = std::env::current_dir()
        .and_then(fs::canonicalize)
        .map_err(|e| e.to_string())?;
    let (python, entry) = match &checker.bundle {
        Some(bundle) => {
            verify_bundle(bundle)?;
            (
                bundle.python.as_deref().unwrap_or("python3"),
                bundle.entry_uri.as_str(),
            )
        }
        None => ("python3", checker.command_uri.as_str()),
    };
    let python_path = if python.contains('/') {
        root.join(python)
    } else {
        PathBuf::from(python)
    };
    // Create private context without allowing a linked cache ancestor.
    write_copy(&format!("{attempt}/context"), b"xdesign checker attempt\n")?;
    for directory in ["tmp", "cache"] {
        fs::create_dir_all(format!("{attempt}/{directory}")).map_err(|e| e.to_string())?;
    }
    let mut command = Command::new(python_path);
    command
        .current_dir(&root)
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .env("PYTHONNOUSERSITE", "1")
        .env("XDESIGN_PROJECT_ROOT", &root)
        .env("XDESIGN_ATTEMPT_DIRECTORY", root.join(attempt))
        .env("TMPDIR", root.join(attempt).join("tmp"))
        .env("TMP", root.join(attempt).join("tmp"))
        .env("TEMP", root.join(attempt).join("tmp"))
        .env("XDG_CACHE_HOME", root.join(attempt).join("cache"))
        .env(
            "XDESIGN_CHECKER_COMMANDS_JSON",
            serde_json::to_string(commands).map_err(|e| e.to_string())?,
        );
    if let Some(bundle) = &checker.bundle {
        // The original tree is preserved below the bundle's root.
        let prefix = bundle
            .entry_uri
            .strip_suffix(&checker.command_uri)
            .ok_or("entry does not preserve command path")?;
        command.env("XDESIGN_CHECKER_BUNDLE_ROOT", root.join(prefix));
        if bundle.source == "builtin-v1" {
            let compiler = format!("{prefix}tools/sysml_web/bin/sysml");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&compiler, fs::Permissions::from_mode(0o700))
                    .map_err(|e| e.to_string())?;
            }
            command.env("XDESIGN_SYSML_COMPILER", root.join(compiler));
        } else {
            command.env_remove("XDESIGN_SYSML_COMPILER");
        }
    } else {
        command
            .env_remove("XDESIGN_CHECKER_BUNDLE_ROOT")
            .env_remove("XDESIGN_SYSML_COMPILER");
    }
    command.arg(root.join(entry));
    Ok(command)
}

pub(crate) fn interpreter_identity(command: &Command) -> Result<serde_json::Value, String> {
    let mut probe = Command::new(command.get_program());
    if let Some(cwd) = command.get_current_dir() {
        probe.current_dir(cwd);
    }
    for (name, value) in command.get_envs() {
        if let Some(value) = value {
            probe.env(name, value);
        } else {
            probe.env_remove(name);
        }
    }
    probe.args([
        "-c",
        "import json,sys; print(json.dumps({'executable':sys.executable,'version':sys.version}))",
    ]);
    let output = super::run_command_with_timeout(
        &mut probe,
        super::checker_timeout().min(std::time::Duration::from_secs(10)),
    )
    .map_err(|e| e.to_string())?;
    if output.timed_out || !output.success {
        return Err(format!(
            "Python interpreter identity unavailable: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let mut identity: serde_json::Value =
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let path = identity["executable"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("interpreter did not identify itself")?;
    let bytes = fs::read(path).map_err(|e| format!("cannot identify Python executable: {e}"))?;
    identity["sha256"] = serde_json::json!(sha256_bytes(&bytes));
    Ok(identity)
}
