use crate::{
    plan::{sha256_bytes, valid_relative_path, RsCheckerSpec},
    record::RsStageVisit,
    status::{RsCheckpointOutcome, RsGateError, RsJudgeAuthority},
};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
    process::Command,
};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsRuleReport {
    pub rule_id: String,
    pub holds: bool,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsGateReport {
    pub holds: bool,
    pub artifact_revision: u32,
    pub artifact_snapshot_digest: String,
    pub report_uri: String,
    pub rules: Vec<RsRuleReport>,
}

pub trait RsCheckCommand: Send {
    fn run_check(
        &self,
        checker: &RsCheckerSpec,
        plan_id: &str,
        plan_uri: &str,
        plan_sha256: &str,
        wish_model: &str,
        run_record: &str,
        stage: &str,
        artifact_revision: u32,
        snapshot_digest: &str,
    ) -> Result<RsGateReport, RsGateError>;
}

pub trait RsJudge: Send {
    fn review(
        &self,
        criteria: &[String],
        evidence_uris: &[String],
    ) -> (RsCheckpointOutcome, String);
    fn authority(&self) -> RsJudgeAuthority;
    fn identity(&self) -> &str;
}

#[derive(Clone, Debug)]
pub struct RsProcessCheck {
    pub report_directory: String,
}

#[derive(Deserialize)]
struct RawRule {
    rule_id: String,
    verdict: String,
    inputs: Vec<String>,
    observed: serde_json::Value,
    failures: Vec<String>,
}

#[derive(Deserialize)]
struct RawReport {
    checker_id: String,
    stage: String,
    artifact_revision: u32,
    snapshot_digest: String,
    report_uri: String,
    overall_verdict: String,
    rules: Vec<RawRule>,
    input_sha256: HashMap<String, String>,
}

impl RsCheckCommand for RsProcessCheck {
    fn run_check(
        &self,
        checker: &RsCheckerSpec,
        plan_id: &str,
        plan_uri: &str,
        plan_sha256: &str,
        wish_model: &str,
        run_record: &str,
        stage: &str,
        artifact_revision: u32,
        snapshot_digest: &str,
    ) -> Result<RsGateReport, RsGateError> {
        let plan_bytes = fs::read(plan_uri).map_err(|_| RsGateError::CommandFailed)?;
        if sha256_bytes(&plan_bytes) != plan_sha256 {
            return Err(RsGateError::CommandFailed);
        }
        let plan_json: serde_json::Value =
            serde_json::from_slice(&plan_bytes).map_err(|_| RsGateError::CommandFailed)?;
        if plan_json["plan_id"] != plan_id {
            return Err(RsGateError::CommandFailed);
        }
        let record_bytes = fs::read(run_record).map_err(|_| RsGateError::CommandFailed)?;
        let record: serde_json::Value =
            serde_json::from_slice(&record_bytes).map_err(|_| RsGateError::CommandFailed)?;
        if record["plan_id"] != plan_id
            || record["plan_uri"] != plan_uri
            || record["plan_sha256"] != plan_sha256
            || record["wish_model_uri"] != wish_model
        {
            return Err(RsGateError::CommandFailed);
        }
        let idx = record["current_visit"]
            .as_u64()
            .ok_or(RsGateError::CommandFailed)? as usize;
        let visit_json = record["visits"]
            .get(idx)
            .ok_or(RsGateError::CommandFailed)?;
        if visit_json["stage_id"] != stage
            || visit_json["artifact_revision"] != artifact_revision
            || visit_json["artifact_snapshot_digest"] != snapshot_digest
        {
            return Err(RsGateError::CommandFailed);
        }
        let visit: RsStageVisit =
            serde_json::from_value(visit_json.clone()).map_err(|_| RsGateError::CommandFailed)?;
        if !visit.verify_artifacts() {
            return Err(RsGateError::CommandFailed);
        }
        if !Path::new(&checker.command_uri).is_file() {
            return Err(RsGateError::CommandFailed);
        }
        fs::create_dir_all(&self.report_directory).map_err(|_| RsGateError::CommandFailed)?;
        let report_uri = format!(
            "{}/{}_{}_{}.json",
            self.report_directory,
            stage,
            artifact_revision,
            Uuid::new_v4()
        );
        let output = Command::new("python3")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .arg(&checker.command_uri)
            .arg("--wish-model")
            .arg(wish_model)
            .arg("--record")
            .arg(run_record)
            .arg("--checker-id")
            .arg(&checker.checker_id)
            .arg("--plan-id")
            .arg(plan_id)
            .arg("--plan-uri")
            .arg(plan_uri)
            .arg("--plan-sha256")
            .arg(plan_sha256)
            .arg("--stage")
            .arg(stage)
            .arg("--artifact-revision")
            .arg(artifact_revision.to_string())
            .arg("--snapshot-digest")
            .arg(snapshot_digest)
            .arg("--report")
            .arg(&report_uri)
            .output()
            .map_err(|_| RsGateError::CommandFailed)?;
        let _ = fs::write(format!("{report_uri}.stdout.log"), &output.stdout);
        let _ = fs::write(format!("{report_uri}.stderr.log"), &output.stderr);
        if !output.status.success() {
            return Err(RsGateError::CommandFailed);
        }
        let raw_bytes = fs::read(&report_uri).map_err(|_| RsGateError::CommandFailed)?;
        let raw: RawReport =
            serde_json::from_slice(&raw_bytes).map_err(|_| RsGateError::CommandFailed)?;
        if raw.checker_id != checker.checker_id
            || raw.stage != stage
            || raw.artifact_revision != artifact_revision
            || raw.snapshot_digest != snapshot_digest
            || raw.report_uri != report_uri
            || !["passed", "failed"].contains(&raw.overall_verdict.as_str())
        {
            return Err(RsGateError::CommandFailed);
        }
        if raw.input_sha256.get(wish_model).map(String::as_str)
            != Some(
                sha256_bytes(&fs::read(wish_model).map_err(|_| RsGateError::CommandFailed)?)
                    .as_str(),
            )
            || raw.input_sha256.get(run_record).map(String::as_str)
                != Some(sha256_bytes(&record_bytes).as_str())
            || raw.input_sha256.get(plan_uri).map(String::as_str) != Some(plan_sha256)
        {
            return Err(RsGateError::CommandFailed);
        }
        let bound: HashMap<_, _> = visit
            .artifact_digests()
            .iter()
            .map(|d| (d.uri.as_str(), d.sha256.as_str()))
            .collect();
        for (uri, digest) in &raw.input_sha256 {
            if !valid_relative_path(uri) {
                return Err(RsGateError::CommandFailed);
            }
            let bytes = fs::read(uri).map_err(|_| RsGateError::CommandFailed)?;
            if sha256_bytes(&bytes) != *digest {
                return Err(RsGateError::CommandFailed);
            }
            if (stage == "requirements" || stage == "design")
                && uri != wish_model
                && uri != run_record
                && uri != plan_uri
                && bound.get(uri.as_str()).copied() != Some(digest.as_str())
            {
                return Err(RsGateError::CommandFailed);
            }
        }
        let mut expected = HashSet::new();
        for rule in &checker.rules {
            expected.insert(rule.rule_id.as_str());
        }
        let mut seen = HashSet::new();
        let mut rules = Vec::new();
        for rule in raw.rules {
            let Some(declared) = checker.rules.iter().find(|r| r.rule_id == rule.rule_id) else {
                return Err(RsGateError::CommandFailed);
            };
            let inputs: HashSet<_> = rule.inputs.iter().map(String::as_str).collect();
            if !expected.contains(rule.rule_id.as_str())
                || !seen.insert(rule.rule_id.clone())
                || !["passed", "failed"].contains(&rule.verdict.as_str())
                || rule.inputs.is_empty()
                || rule.inputs.iter().any(|input| input.trim().is_empty())
                || rule.observed.is_null()
                || rule
                    .observed
                    .as_object()
                    .is_some_and(|value| value.is_empty())
                || rule
                    .observed
                    .as_array()
                    .is_some_and(|value| value.is_empty())
                || rule.observed.as_str().is_some_and(str::is_empty)
                || (rule.verdict == "passed" && !rule.failures.is_empty())
                || (rule.verdict == "failed" && rule.failures.is_empty())
            {
                return Err(RsGateError::CommandFailed);
            }
            if rule.verdict == "passed" {
                if rule
                    .inputs
                    .iter()
                    .any(|input| !raw.input_sha256.contains_key(input))
                {
                    return Err(RsGateError::CommandFailed);
                }
                for read in &declared.reads {
                    let covered = match read.as_str() {
                        "runtime run record JSON" => inputs.contains(run_record),
                        "recorded artifact URIs" => {
                            let source_visits: Vec<_> = if stage == "implement" {
                                vec![visit_json]
                            } else {
                                record["visits"]
                                    .as_array()
                                    .ok_or(RsGateError::CommandFailed)?
                                    .iter()
                                    .filter(|v| v["status"] == "completed")
                                    .collect()
                            };
                            source_visits.iter().all(|source| {
                                source["artifact_digests"]
                                    .as_array()
                                    .is_some_and(|artifacts| {
                                        artifacts.iter().all(|item| {
                                            item["uri"]
                                                .as_str()
                                                .is_some_and(|uri| inputs.contains(uri))
                                        })
                                    })
                            })
                        }
                        "recorded evidence URIs" => record["visits"]
                            .as_array()
                            .ok_or(RsGateError::CommandFailed)?
                            .iter()
                            .enumerate()
                            .filter(|(index, v)| *index == idx || v["status"] == "completed")
                            .all(|(_, v)| {
                                v["assessments"].as_array().is_some_and(|assessments| {
                                    assessments.iter().filter(|a| a["outcome"] == "passed").all(
                                        |a| {
                                            a["evidence_uris"].as_array().is_some_and(|uris| {
                                                uris.iter().all(|uri| {
                                                    uri.as_str()
                                                        .is_some_and(|uri| inputs.contains(uri))
                                                })
                                            })
                                        },
                                    )
                                })
                            }),
                        "tools/checks/" => {
                            plan_json["checkpoints"].as_array().is_some_and(|gates| {
                                let commands: Vec<_> = gates
                                    .iter()
                                    .filter_map(|gate| gate["checker"]["command_uri"].as_str())
                                    .collect();
                                commands.len() == 4
                                    && commands.iter().all(|command| inputs.contains(command))
                            })
                        }
                        prefix if prefix.ends_with('/') => {
                            inputs.iter().any(|input| input.starts_with(prefix))
                        }
                        path => inputs.contains(path),
                    };
                    if !covered {
                        return Err(RsGateError::CommandFailed);
                    }
                }
            }
            let diagnostic = if rule.failures.is_empty() {
                None
            } else {
                Some(rule.failures.join("; "))
            };
            rules.push(RsRuleReport {
                rule_id: rule.rule_id,
                holds: rule.verdict == "passed",
                diagnostic,
            });
        }
        if seen.len() != expected.len() {
            return Err(RsGateError::CommandFailed);
        }
        let holds = rules.iter().all(|rule| rule.holds);
        if holds != (raw.overall_verdict == "passed") {
            return Err(RsGateError::CommandFailed);
        }
        Ok(RsGateReport {
            holds,
            artifact_revision,
            artifact_snapshot_digest: snapshot_digest.to_owned(),
            report_uri,
            rules,
        })
    }
}

/// A caller-supplied judgment. It is never created by the check adapter.
#[derive(Clone, Debug)]
pub struct RsSuppliedJudge {
    pub authority: RsJudgeAuthority,
    pub identity: String,
    pub outcome: RsCheckpointOutcome,
    pub rationale: String,
}

impl RsJudge for RsSuppliedJudge {
    fn review(
        &self,
        _criteria: &[String],
        _evidence_uris: &[String],
    ) -> (RsCheckpointOutcome, String) {
        (self.outcome, self.rationale.clone())
    }
    fn authority(&self) -> RsJudgeAuthority {
        self.authority
    }
    fn identity(&self) -> &str {
        &self.identity
    }
}
