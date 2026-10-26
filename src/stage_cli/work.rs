use super::store::{artifact_path, read_file};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::Path, process::Command};
use xdesign::core::{hash_artifact, snapshot_digest, RsArtifactDigest, RsFlowPlan, RsStageVisit};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkDefinition {
    pub schema_version: u32,
    pub flow_id: String,
    pub stages: Vec<StageWork>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageWork {
    pub stage_id: String,
    pub goal: String,
    pub instructions: String,
    pub inputs: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub frozen_paths: Vec<String>,
}

pub struct Collection {
    pub artifacts: Vec<RsArtifactDigest>,
    pub snapshot: String,
    pub diagnostics: Vec<String>,
}

fn stage(id: &str, goal: &str, instructions: &str, output: &str) -> StageWork {
    StageWork {
        stage_id: id.into(),
        goal: goal.into(),
        instructions: instructions.into(),
        inputs: vec!["wish:model".into()],
        expected_outputs: vec![output.into()],
        frozen_paths: vec![],
    }
}

fn templates(flow: &str) -> Result<Vec<StageWork>, String> {
    let verify = stage("verify", "Demonstrate that the implementation satisfies the requirements.",
        "Run the planned tests, retain verification evidence and supply a permitted semantic judgment.", "result:artifacts");
    match flow {
        "simple-flow" => Ok(vec![
            stage("requirements", "Define requirements and their rationale.",
                "State the requirements, rationale and satisfaction relationships in the project model.", "checker:simpleRequirementsCheck"),
            stage("design", "Design the solution and its verification.",
                "Update the SysML design, API contracts and documentation; keep model views and checker declarations consistent.", "checker:simpleDesignCheck"),
            stage("implement", "Implement the reviewed design.",
                "Implement the source and tests required by the design; retain source artifacts for checking.", "source:src"),
            verify,
        ]),
        "wish-verify" => Ok(vec![stage("wish", "Record the wish and the artifacts that answer it.",
            "Record the requested outcome and its answering artifacts, then supply a permitted semantic judgment.", "wish:model"), verify]),
        _ => Err(format!("unsupported flow: {flow}")),
    }
}

impl WorkDefinition {
    /// Freeze work from the built-in per-stage input selection.
    #[allow(dead_code)]
    pub fn freeze(flow: &str, plan: &RsFlowPlan, wish: &str) -> Result<Self, String> {
        Self::freeze_with_inputs(flow, plan, wish, &std::collections::BTreeMap::new())
    }

    /// Freeze work with caller-supplied inputs for library-bound syntactic
    /// gates. Those `entry.inputs` replace the checker's `--list-inputs`
    /// invocation; every other stage keeps the built-in selection.
    pub fn freeze_with_inputs(
        flow: &str,
        plan: &RsFlowPlan,
        wish: &str,
        library_inputs: &std::collections::BTreeMap<String, Vec<String>>,
    ) -> Result<Self, String> {
        let mut stages = templates(flow)?;
        for work in &mut stages {
            let selector = work.expected_outputs[0].as_str();
            if selector.starts_with("checker:") {
                let checker = plan
                    .checkpoints_of(&work.stage_id)
                    .into_iter()
                    .find_map(|gate| gate.checker.as_ref())
                    .ok_or("work has no fixed checker")?;
                if let Some(inputs) = library_inputs.get(&work.stage_id) {
                    if inputs.is_empty() {
                        return Err(format!(
                            "library checkpoint for {} declares no inputs",
                            work.stage_id
                        ));
                    }
                    for input in inputs {
                        artifact_path(input)?;
                    }
                    work.frozen_paths = inputs.clone();
                    work.frozen_paths.sort();
                } else {
                    artifact_path(&checker.command_uri)?;
                    let output = Command::new("python3")
                        .env("PYTHONDONTWRITEBYTECODE", "1")
                        .arg(&checker.command_uri)
                        .arg("--list-inputs")
                        .output()
                        .map_err(|e| format!("cannot list {} inputs: {e}", checker.checker_id))?;
                    if !output.status.success() {
                        return Err(format!(
                            "{} --list-inputs failed: {}",
                            checker.command_uri,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    work.frozen_paths = serde_json::from_slice(&output.stdout)
                        .map_err(|e| format!("invalid checker input list: {e}"))?;
                    work.frozen_paths.sort();
                }
            } else if selector == "wish:model" {
                work.frozen_paths = vec![wish.into()];
            } else if selector == "source:src" {
                work.frozen_paths = vec!["Cargo.toml".into()];
            }
        }
        let definition = Self {
            schema_version: 1,
            flow_id: flow.into(),
            stages,
        };
        definition.validate(flow, plan, wish)?;
        Ok(definition)
    }

    pub fn validate(&self, flow: &str, plan: &RsFlowPlan, wish: &str) -> Result<(), String> {
        let expected = templates(flow)?;
        if self.schema_version != 1
            || self.flow_id != flow
            || self.stages.len() != expected.len()
            || self.stages.len() != plan.stages.len()
        {
            return Err("unsupported work definition".into());
        }
        for ((work, template), planned) in self.stages.iter().zip(expected).zip(&plan.stages) {
            let mut compare = work.clone();
            compare.frozen_paths.clear();
            if compare != template || work.stage_id != planned.stage_id {
                return Err(format!(
                    "incomplete or incompatible work definition: {}",
                    work.stage_id
                ));
            }
            let mut seen = BTreeSet::new();
            for path in &work.frozen_paths {
                artifact_path(path)?;
                if !seen.insert(path) {
                    return Err("duplicate frozen output path".into());
                }
            }
            match work.expected_outputs[0].as_str() {
                "wish:model" if work.frozen_paths != [wish] => {
                    return Err("work names a different wish".into())
                }
                "source:src" if work.frozen_paths != ["Cargo.toml"] => {
                    return Err("invalid source work definition".into())
                }
                "result:artifacts" if !work.frozen_paths.is_empty() => {
                    return Err("invalid verification work definition".into())
                }
                selector if selector.starts_with("checker:") && work.frozen_paths.is_empty() => {
                    return Err("empty checker work definition".into())
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn stage(&self, id: &str) -> Result<&StageWork, String> {
        self.stages
            .iter()
            .find(|stage| stage.stage_id == id)
            .ok_or_else(|| format!("work definition missing stage: {id}"))
    }
}

fn source_paths(
    directory: &Path,
    paths: &mut BTreeSet<String>,
    diagnostics: &mut Vec<String>,
) -> Result<(), String> {
    let uri = directory.to_str().ok_or("non-UTF-8 source path")?;
    artifact_path(uri)?;
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(format!("Cannot collect {uri}: {error}"));
            return Ok(());
        }
    };
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        // Do not follow directory links: they can cycle or silently hide source trees.
        if file_type.is_symlink() {
            let uri = path.to_str().ok_or("non-UTF-8 source path")?;
            artifact_path(uri)?;
            if path.is_dir() {
                return Err(format!("source directory symlinks are unsupported: {uri}"));
            }
        }
        if file_type.is_dir() {
            source_paths(&path, paths, diagnostics)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.insert(path.to_str().ok_or("non-UTF-8 source path")?.into());
        }
    }
    Ok(())
}

fn declared(work: &StageWork, uri: &str) -> bool {
    work.frozen_paths.iter().any(|path| path == uri)
        || work.expected_outputs[0] == "source:src"
            && uri.starts_with("src/")
            && uri.ends_with(".rs")
}

pub fn collect(
    work: &StageWork,
    visit: &RsStageVisit,
    supplied: &[RsArtifactDigest],
) -> Result<Collection, String> {
    let mut paths: BTreeSet<String> = work.frozen_paths.iter().cloned().collect();
    let mut diagnostics = vec![];
    if work.expected_outputs[0] == "source:src" {
        source_paths(Path::new("src"), &mut paths, &mut diagnostics)?;
        if !paths
            .iter()
            .any(|uri| uri.starts_with("src/") && uri.ends_with(".rs"))
        {
            diagnostics.push("No Rust source files exist under src/.".into());
        }
    }
    // Supplemental paths are retained for the current visit. Dynamic source removals
    // are intentionally omitted so record_activity can record their deletion.
    for artifact in visit.artifact_digests() {
        if !declared(work, &artifact.uri) {
            paths.insert(artifact.uri.clone());
        }
    }
    for activity in visit.activities() {
        for uri in &activity.artifact_uris {
            if !declared(work, uri) {
                paths.insert(uri.clone());
            }
        }
    }
    paths.extend(supplied.iter().map(|artifact| artifact.uri.clone()));
    let mut artifacts = vec![];
    for uri in paths {
        artifact_path(&uri)?;
        match read_file(&uri).and_then(|_| hash_artifact(&uri)) {
            Ok(artifact) => artifacts.push(artifact),
            Err(error) => diagnostics.push(format!("Missing or unreadable output {uri}: {error}")),
        }
    }
    if work.expected_outputs[0] == "result:artifacts" && artifacts.is_empty() {
        diagnostics.push(
            "Supply verification evidence in result.artifacts before this stage can pass.".into(),
        );
    }
    let snapshot = snapshot_digest(&artifacts);
    Ok(Collection {
        artifacts,
        snapshot,
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_work_matches_the_six_modeled_stage_overrides() {
        let model = include_str!("../../mbse/model/detailedDesign.sysml");
        for (flow, definition) in [
            ("simple-flow", "DesignedSimpleFlow"),
            ("wish-verify", "DesignedWishVerifyFlow"),
        ] {
            let start = model.find(&format!("part def {definition} ")).unwrap();
            let body = &model[start..];
            for work in templates(flow).unwrap() {
                let name = if work.stage_id == "verify" {
                    "'verify'"
                } else {
                    &work.stage_id
                };
                let stage_start = body.find(&format!("part redefines {name} {{")).unwrap();
                let stage = body[stage_start..].split_once('}').unwrap().0;
                for (attribute, value) in [
                    ("goal", work.goal),
                    ("workInstructions", work.instructions),
                    ("inputArtifacts", work.inputs[0].clone()),
                    ("expectedOutputs", work.expected_outputs[0].clone()),
                ] {
                    assert!(
                        stage.contains(&format!("attribute redefines {attribute} = \"{value}\";")),
                        "{definition}.{name}.{attribute} diverges from the runtime work definition"
                    );
                }
            }
        }
    }
}
