use crate::core::{
    sha256_bytes, valid_relative_path, RsCheckCommand, RsCheckerSpec, RsGateError, RsGateReport,
    RsRuleReport, RsStageVisit,
};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Read,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

/// A hung checker must never hang the CLI with it: subprocesses are killed at
/// this deadline (overridable for slow machines) and reported as timed out.
pub fn checker_timeout() -> Duration {
    std::env::var("XDESIGN_CHECKER_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map_or(Duration::from_secs(600), Duration::from_secs)
}

pub struct CommandOutcome {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub success: bool,
    pub timed_out: bool,
}

/// Run a command to completion with a hard deadline. On timeout the child and
/// any of its process group are killed and `timed_out` is reported, so a
/// checker stuck on I/O or a compiler deadlock surfaces as an explicit error
/// instead of an indefinite hang.
pub fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> std::io::Result<CommandOutcome> {
    let deadline = Instant::now() + timeout;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");
    let stdout = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stdout_pipe.read_to_end(&mut buffer);
        buffer
    });
    let stderr = thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stderr_pipe.read_to_end(&mut buffer);
        buffer
    });
    loop {
        match child.try_wait()? {
            Some(status) => {
                return Ok(CommandOutcome {
                    stdout: stdout.join().unwrap_or_default(),
                    stderr: stderr.join().unwrap_or_default(),
                    success: status.success(),
                    timed_out: false,
                })
            }
            None if Instant::now() < deadline => thread::sleep(Duration::from_millis(50)),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(CommandOutcome {
                    stdout: stdout.join().unwrap_or_default(),
                    stderr: stderr.join().unwrap_or_default(),
                    success: false,
                    timed_out: true,
                })
            }
        }
    }
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
        let timeout = checker_timeout();
        let output = run_command_with_timeout(
            Command::new("python3")
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
                .arg(&report_uri),
            timeout,
        )
        .map_err(|_| RsGateError::CommandFailed)?;
        let _ = fs::write(format!("{report_uri}.stdout.log"), &output.stdout);
        let _ = fs::write(
            format!("{report_uri}.stderr.log"),
            if output.timed_out {
                format!(
                    "checker {} exceeded the {:?} deadline and was killed",
                    checker.checker_id, timeout
                )
                .into_bytes()
            } else {
                output.stderr.clone()
            },
        );
        if output.timed_out {
            return Err(RsGateError::CheckerTimeout);
        }
        if !output.success {
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
                        "retained plan serialization" => inputs.contains(plan_uri),
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
