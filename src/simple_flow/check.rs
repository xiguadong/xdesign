use crate::core::{
    sha256_bytes, valid_relative_path, RsCheckCommand, RsCheckerSpec, RsCustomChecker,
    RsCustomRunner, RsGateError, RsGateReport, RsRuleReport, RsStageVisit,
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
#[cfg(unix)]
use std::{
    os::unix::{io::AsRawFd, process::CommandExt},
    process::{Child, ExitStatus},
};
use uuid::Uuid;

/// How long the Unix deadline loop waits between non-blocking output drains
/// when neither the checker nor its pipes have finished.
#[cfg(unix)]
const OUTPUT_POLL_INTERVAL: Duration = Duration::from_millis(5);

/// A hung checker must never hang the CLI with it: subprocesses are killed at
/// this deadline (overridable for slow machines) and reported as timed out.
/// The same deadline bounds the checker's whole process tree, not just its
/// direct child (see [`run_command_with_timeout`]).
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

/// Run a command to completion with a hard deadline that bounds its whole
/// process tree, not just the direct child.
///
/// On Unix the command is spawned as the leader of its own process group
/// (`pre_exec` + `setpgid`), its stdout/stderr are read through non-blocking
/// pipe ends while the deadline loop runs, and an expired deadline sends
/// `SIGKILL` to the entire group before reaping the direct child. This matters
/// because a checker can start a descendant that inherits its stdout/stderr:
/// killing only the direct child leaves those pipes open, so reader threads
/// would block until the descendant exits, silently extending the deadline
/// while a checker holds the per-wish writer lock.
///
/// Output is accumulated byte-for-byte by the deadline loop. If the direct
/// child exits before the deadline but descendants still hold the pipes, the
/// loop keeps draining and only returns success once both pipes reach EOF.
/// If the deadline expires first, the group is killed and the call returns
/// `timed_out: true, success: false` without waiting for descendant-held pipe
/// ends, so the total time is bounded by the deadline plus the process kill
/// and reap time. A normal command that closes its pipes within the deadline
/// returns its full captured output and exit status exactly as before.
///
/// The non-Unix fallback keeps the previous behavior (kill the direct child,
/// join the pipe-reader threads) because process groups are a Unix facility.
pub fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> std::io::Result<CommandOutcome> {
    #[cfg(unix)]
    return run_command_with_timeout_unix(command, timeout);
    #[cfg(not(unix))]
    return run_command_with_timeout_fallback(command, timeout);
}

/// Spawn the command as the leader of a new process group so the deadline can
/// terminate every descendant with a single `kill(-pid, SIGKILL)`.
#[cfg(unix)]
fn spawn_checker_process_group(command: &mut Command) -> std::io::Result<Child> {
    // SAFETY: the closure runs in the forked child between `fork` and `exec`,
    // where only async-signal-safe operations are permitted. `setpgid` is
    // async-signal-safe and touches no shared Rust state, so it is sound here.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

/// Switch a pipe read end to non-blocking mode so the deadline loop can drain
/// output without ever blocking on a descendant-held pipe.
#[cfg(unix)]
fn set_pipe_nonblocking<T: AsRawFd>(pipe: &T) -> std::io::Result<()> {
    let fd = pipe.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags == -1 {
        return Err(std::io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Drain every byte currently available from a non-blocking pipe.
///
/// Returns `true` when the pipe has reached EOF (or a read error leaves no
/// more bytes to collect), meaning the deadline loop can stop polling it;
/// `false` means the pipe is still open and should be polled again.
#[cfg(unix)]
fn drain_available(pipe: &mut dyn Read, buffer: &mut Vec<u8>) -> bool {
    let mut chunk = [0u8; 16 * 1024];
    loop {
        match pipe.read(&mut chunk) {
            Ok(0) => return true,
            Ok(read) => buffer.extend_from_slice(&chunk[..read]),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return false,
            // A real read error makes no further bytes available, mirroring
            // the previous lenient `read_to_end` handling by keeping the
            // bytes already captured.
            Err(_) => return true,
        }
    }
}

/// Kill the checker's whole process group. `child.kill` is only the fallback
/// for the rare case where the group signal cannot be delivered.
#[cfg(unix)]
fn kill_process_group(child: &mut Child) {
    let pid = child.id();
    let signaled =
        pid <= i32::MAX as u32 && unsafe { libc::kill(-(pid as libc::pid_t), libc::SIGKILL) } == 0;
    if !signaled {
        let _ = child.kill();
    }
}

#[cfg(unix)]
fn run_command_with_timeout_unix(
    command: &mut Command,
    timeout: Duration,
) -> std::io::Result<CommandOutcome> {
    let deadline = Instant::now() + timeout;
    let mut child = spawn_checker_process_group(command)?;
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");
    set_pipe_nonblocking(&stdout_pipe)?;
    set_pipe_nonblocking(&stderr_pipe)?;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut stdout_closed = false;
    let mut stderr_closed = false;
    let mut status: Option<ExitStatus> = None;
    loop {
        // Drain output first: descendants may keep writing while the direct
        // child is already gone, and a large child output must not block on a
        // full pipe buffer.
        if !stdout_closed {
            stdout_closed = drain_available(&mut stdout_pipe, &mut stdout);
        }
        if !stderr_closed {
            stderr_closed = drain_available(&mut stderr_pipe, &mut stderr);
        }
        if status.is_none() {
            status = child.try_wait()?;
        }
        // The command is finished only when the direct child has exited and
        // no remaining writer (including descendants) holds its pipes open.
        if let Some(status) = status {
            if stdout_closed && stderr_closed {
                return Ok(CommandOutcome {
                    stdout,
                    stderr,
                    success: status.success(),
                    timed_out: false,
                });
            }
        }
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        thread::sleep((deadline - now).min(OUTPUT_POLL_INTERVAL));
    }

    // Deadline expired. Kill the group before returning so descendants holding
    // the inherited pipes cannot outlive the checker or hang it. The final
    // non-blocking drain only collects bytes already buffered; it never waits
    // for a descendant that survived the group signal.
    kill_process_group(&mut child);
    let _ = child.wait();
    if !stdout_closed {
        drain_available(&mut stdout_pipe, &mut stdout);
    }
    if !stderr_closed {
        drain_available(&mut stderr_pipe, &mut stderr);
    }
    Ok(CommandOutcome {
        stdout,
        stderr,
        success: false,
        timed_out: true,
    })
}

/// Fallback for platforms without Unix process groups: kill only the direct
/// child and join its output-reader threads. Descendants that inherit the
/// pipes can still extend the deadline on these platforms.
#[cfg(not(unix))]
fn run_command_with_timeout_fallback(
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
                });
            }
        }
    }
}

/// Runs one syntactic SimpleFlow checkpoint. The default and `process`
/// runners execute a project checker script with the raw-report protocol; a
/// `cli` binding (frozen into the plan) asks an agent CLI for the same
/// declared rules in a single JSON verdict object and retains that verdict.
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

/// The shared prologue of every syntactic check: plan bytes hash and id, run
/// record binding, the current visit identity, and the recorded artifact
/// snapshot. Both execution paths refuse to proceed on any mismatch.
struct Prepared {
    plan_bytes: Vec<u8>,
    plan_json: serde_json::Value,
    record_bytes: Vec<u8>,
    record: serde_json::Value,
    visit_json: serde_json::Value,
    visit: RsStageVisit,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct AgentRuleVerdict {
    rule_id: String,
    verdict: String,
    #[serde(default)]
    failures: Vec<String>,
}

#[derive(Deserialize)]
struct AgentVerdict {
    overall_verdict: String,
    rules: Vec<AgentRuleVerdict>,
}

impl RsProcessCheck {
    #[allow(clippy::too_many_arguments)]
    fn preflight(
        plan_id: &str,
        plan_uri: &str,
        plan_sha256: &str,
        wish_model: &str,
        run_record: &str,
        stage: &str,
        artifact_revision: u32,
        snapshot_digest: &str,
    ) -> Result<Prepared, RsGateError> {
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
            .ok_or(RsGateError::CommandFailed)?
            .clone();
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
        Ok(Prepared {
            plan_bytes,
            plan_json,
            record_bytes,
            record,
            visit_json,
            visit,
        })
    }

    fn report_target(
        &self,
        stage: &str,
        artifact_revision: u32,
    ) -> Result<(String, Duration), RsGateError> {
        fs::create_dir_all(&self.report_directory).map_err(|_| RsGateError::CommandFailed)?;
        let report_uri = format!(
            "{}/{}_{}_{}.json",
            self.report_directory,
            stage,
            artifact_revision,
            Uuid::new_v4()
        );
        Ok((report_uri, checker_timeout()))
    }

    /// The deterministic and `process` runner: run the project checker script
    /// and validate its raw report exactly as before custom bindings existed.
    #[allow(clippy::too_many_arguments)]
    fn run_process_check(
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
        prepared: Prepared,
    ) -> Result<RsGateReport, RsGateError> {
        let Prepared {
            plan_json,
            record_bytes,
            record,
            visit_json,
            visit,
            plan_bytes: _,
        } = prepared;
        if !Path::new(&checker.command_uri).is_file() {
            return Err(RsGateError::CommandFailed);
        }
        let (report_uri, timeout) = self.report_target(stage, artifact_revision)?;
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
        Self::validate_raw_report(
            checker,
            plan_uri,
            plan_sha256,
            wish_model,
            run_record,
            stage,
            artifact_revision,
            snapshot_digest,
            &report_uri,
            &record_bytes,
            &record,
            &visit_json,
            &visit,
            &plan_json,
        )
    }

    /// The retained `cli` runner: ask one agent CLI for the declared rules in
    /// read-only mode, parse exactly one verdict object, and retain both the
    /// verdict report and the raw command logs.
    #[allow(clippy::too_many_arguments)]
    fn run_agent_check(
        &self,
        checker: &RsCheckerSpec,
        custom: &RsCustomChecker,
        plan_id: &str,
        plan_uri: &str,
        plan_sha256: &str,
        wish_model: &str,
        run_record: &str,
        stage: &str,
        artifact_revision: u32,
        snapshot_digest: &str,
        prepared: Prepared,
    ) -> Result<RsGateReport, RsGateError> {
        let Some(cli) = custom.cli.as_deref().filter(|cli| !cli.trim().is_empty()) else {
            return Err(RsGateError::CommandFailed);
        };
        let Some(model) = custom
            .model
            .as_deref()
            .filter(|model| !model.trim().is_empty())
        else {
            return Err(RsGateError::CommandFailed);
        };
        let (checkpoint_id, criteria) =
            checkpoint_criteria(&prepared.plan_json, stage, &checker.checker_id)?;
        let prompt = agent_prompt(
            plan_id,
            plan_uri,
            plan_sha256,
            wish_model,
            run_record,
            stage,
            artifact_revision,
            snapshot_digest,
            &checkpoint_id,
            &criteria,
            checker,
        );
        let program = agent_program(cli);
        let (report_uri, timeout) = self.report_target(stage, artifact_revision)?;
        let mut command = Command::new(&program);
        match cli {
            "codex" => {
                command.args([
                    "exec",
                    "--skip-git-repo-check",
                    "--sandbox",
                    "read-only",
                    "-m",
                    model,
                    &prompt,
                ]);
            }
            "kimi" => {
                command.args(["-m", model, "-p", &prompt]);
            }
            _ => return Err(RsGateError::CommandFailed),
        }
        let output = match run_command_with_timeout(&mut command, timeout) {
            Ok(output) => output,
            Err(_) => {
                let _ = fs::write(format!("{report_uri}.stdout.log"), "");
                let _ = fs::write(
                    format!("{report_uri}.stderr.log"),
                    format!("agent CLI {program} could not be started"),
                );
                return Err(RsGateError::CommandFailed);
            }
        };
        let _ = fs::write(format!("{report_uri}.stdout.log"), &output.stdout);
        let _ = fs::write(
            format!("{report_uri}.stderr.log"),
            if output.timed_out {
                format!(
                    "agent CLI {cli} ({program}) exceeded the {timeout:?} deadline and was killed"
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
        let verdict = parse_agent_verdict(&output.stdout)?;
        let rules = validate_agent_verdict(&verdict, checker)?;
        let holds = rules.iter().all(|rule| rule.verdict == "passed");
        let input_uris = [wish_model, run_record, plan_uri];
        let mut input_sha256 = serde_json::Map::new();
        input_sha256.insert(
            wish_model.to_owned(),
            serde_json::json!(sha256_bytes(
                &fs::read(wish_model).map_err(|_| RsGateError::CommandFailed)?
            )),
        );
        input_sha256.insert(
            run_record.to_owned(),
            serde_json::json!(sha256_bytes(&prepared.record_bytes)),
        );
        input_sha256.insert(
            plan_uri.to_owned(),
            serde_json::json!(sha256_bytes(&prepared.plan_bytes)),
        );
        let report_rules: Vec<_> = rules
            .iter()
            .map(|rule| {
                serde_json::json!({
                    "rule_id": rule.rule_id,
                    "verdict": rule.verdict,
                    "inputs": input_uris,
                    "observed": {
                        "runner": format!("{cli}:{model}"),
                        "verdict": rule.verdict,
                        "failures": rule.failures,
                    },
                    "failures": rule.failures,
                })
            })
            .collect();
        let report = serde_json::json!({
            "checker_id": checker.checker_id,
            "stage": stage,
            "artifact_revision": artifact_revision,
            "snapshot_digest": snapshot_digest,
            "report_uri": report_uri,
            "overall_verdict": if holds { "passed" } else { "failed" },
            "rules": report_rules,
            "input_sha256": input_sha256,
            "runner": {
                "kind": "cli",
                "cli": cli,
                "model": model,
                "program": program,
                "prompt_sha256": sha256_bytes(prompt.as_bytes()),
            },
        });
        let report_bytes =
            serde_json::to_vec_pretty(&report).map_err(|_| RsGateError::CommandFailed)?;
        fs::write(&report_uri, report_bytes).map_err(|_| RsGateError::CommandFailed)?;
        let reported_rules = rules
            .iter()
            .map(|rule| RsRuleReport {
                rule_id: rule.rule_id.clone(),
                holds: rule.verdict == "passed",
                diagnostic: if rule.failures.is_empty() {
                    None
                } else {
                    Some(rule.failures.join("; "))
                },
            })
            .collect();
        Ok(RsGateReport {
            holds,
            artifact_revision,
            artifact_snapshot_digest: snapshot_digest.to_owned(),
            report_uri,
            rules: reported_rules,
        })
    }

    /// Validate the raw checker report exactly as the default contract did.
    #[allow(clippy::too_many_arguments)]
    fn validate_raw_report(
        checker: &RsCheckerSpec,
        plan_uri: &str,
        plan_sha256: &str,
        wish_model: &str,
        run_record: &str,
        stage: &str,
        artifact_revision: u32,
        snapshot_digest: &str,
        report_uri: &str,
        record_bytes: &[u8],
        record: &serde_json::Value,
        visit_json: &serde_json::Value,
        visit: &RsStageVisit,
        plan_json: &serde_json::Value,
    ) -> Result<RsGateReport, RsGateError> {
        let idx = record["current_visit"]
            .as_u64()
            .ok_or(RsGateError::CommandFailed)? as usize;
        let raw_bytes = fs::read(report_uri).map_err(|_| RsGateError::CommandFailed)?;
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
                != Some(sha256_bytes(record_bytes).as_str())
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
            report_uri: report_uri.to_owned(),
            rules,
        })
    }
}

fn checkpoint_criteria(
    plan_json: &serde_json::Value,
    stage: &str,
    checker_id: &str,
) -> Result<(String, Vec<String>), RsGateError> {
    let gate = plan_json["checkpoints"]
        .as_array()
        .ok_or(RsGateError::CommandFailed)?
        .iter()
        .find(|gate| gate["stage_id"] == stage && gate["checker"]["checker_id"] == checker_id)
        .ok_or(RsGateError::CommandFailed)?;
    let checkpoint_id = gate["checkpoint_id"]
        .as_str()
        .ok_or(RsGateError::CommandFailed)?
        .to_owned();
    let criteria: Vec<String> = gate["criteria"]
        .as_array()
        .ok_or(RsGateError::CommandFailed)?
        .iter()
        .filter_map(|criterion| criterion.as_str().map(str::to_owned))
        .collect();
    if criteria.is_empty() {
        return Err(RsGateError::CommandFailed);
    }
    Ok((checkpoint_id, criteria))
}

/// The resolved agent program: the development override when set, else the
/// CLI name. Tests point these at stub executables and never run a real agent.
fn agent_program(cli: &str) -> String {
    let variable = match cli {
        "codex" => "XDESIGN_AGENT_CODEX_BIN",
        "kimi" => "XDESIGN_AGENT_KIMI_BIN",
        _ => return cli.to_owned(),
    };
    std::env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| cli.to_owned())
}

/// The read-only conformance prompt: repository paths, plan identity, the
/// run-record and artifact bindings, the checkpoint criteria read from the
/// frozen plan, and every declared rule.
#[allow(clippy::too_many_arguments)]
fn agent_prompt(
    plan_id: &str,
    plan_uri: &str,
    plan_sha256: &str,
    wish_model: &str,
    run_record: &str,
    stage: &str,
    artifact_revision: u32,
    snapshot_digest: &str,
    checkpoint_id: &str,
    criteria: &[String],
    checker: &RsCheckerSpec,
) -> String {
    let project_root = std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| ".".into());
    let mut prompt = String::new();
    prompt.push_str(
        "You are a read-only verification agent for the xdesign SimpleFlow stage gate.\n",
    );
    prompt.push_str("Review the repository and answer every declared checker rule exactly once.\n");
    prompt.push_str(
        "Do not modify, create, or delete any file. Do not run formatting or write commands.\n\n",
    );
    prompt.push_str(&format!("Project root: {project_root}\n"));
    prompt.push_str("All paths below are project-relative unless the project root is absolute.\n");
    prompt.push_str(&format!("Frozen plan id: {plan_id}\n"));
    prompt.push_str(&format!("Frozen plan path: {plan_uri}\n"));
    prompt.push_str(&format!("Frozen plan sha256: {plan_sha256}\n"));
    prompt.push_str(&format!("Wish model: {wish_model}\n"));
    prompt.push_str(&format!("Retained run record snapshot: {run_record}\n"));
    prompt.push_str(&format!("Stage: {stage}\n"));
    prompt.push_str(&format!("Artifact revision: {artifact_revision}\n"));
    prompt.push_str(&format!("Artifact snapshot digest: {snapshot_digest}\n"));
    prompt.push_str(&format!("Checkpoint: {checkpoint_id}\n"));
    prompt.push_str(&format!("Checker: {}\n", checker.checker_id));
    prompt.push_str("\nCheckpoint criteria:\n");
    for criterion in criteria {
        prompt.push_str(&format!("- {criterion}\n"));
    }
    prompt.push_str("\nDeclared rules:\n");
    for rule in &checker.rules {
        prompt.push_str(&format!("- rule_id: {}\n", rule.rule_id));
        prompt.push_str(&format!("  condition: {}\n", rule.condition));
        prompt.push_str(&format!("  reads: {}\n", rule.reads.join(", ")));
    }
    prompt.push_str(
        "\nAnswer the rules against the recorded artifact revision and snapshot digest.\n",
    );
    prompt.push_str("The checkpoint passes only when every declared rule passes; report a rule as failed with at least one nonempty failure otherwise.\n");
    prompt.push_str("Output exactly one JSON object and nothing else, in this shape:\n");
    prompt.push_str(
        "{\"overall_verdict\":\"passed|failed\",\"rules\":[{\"rule_id\":\"<declared rule id>\",\"verdict\":\"passed|failed\",\"failures\":[\"<failure detail>\"]}]}\n",
    );
    prompt
}

/// Parse exactly one JSON verdict object from the agent's stdout. A single
/// object wrapped in prose or a Markdown code fence is accepted (agent CLIs
/// often narrate); multiple or malformed objects are rejected.
fn parse_agent_verdict(stdout: &[u8]) -> Result<AgentVerdict, RsGateError> {
    let text = std::str::from_utf8(stdout).map_err(|_| RsGateError::CommandFailed)?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(RsGateError::CommandFailed);
    }
    if let Ok(verdict) = serde_json::from_str::<AgentVerdict>(trimmed) {
        return Ok(verdict);
    }
    let candidate = single_json_object(trimmed).ok_or(RsGateError::CommandFailed)?;
    serde_json::from_str::<AgentVerdict>(candidate).map_err(|_| RsGateError::CommandFailed)
}

/// Return the single balanced `{...}` object in `text`. Braces before it are
/// impossible by construction, and a brace after it means a second object.
fn single_json_object(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let start = bytes.iter().position(|byte| *byte == b'{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, byte) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    let end = offset + 1;
                    if text[end..].contains('{') {
                        return None;
                    }
                    return Some(&text[start..end]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Validate the reported rule set against the declared rules: exact set
/// equality, passed/failed verdicts, failed failures nonempty (and passed
/// failures empty), and the overall verdict equal to the conjunction.
fn validate_agent_verdict(
    verdict: &AgentVerdict,
    checker: &RsCheckerSpec,
) -> Result<Vec<AgentRuleVerdict>, RsGateError> {
    if !matches!(verdict.overall_verdict.as_str(), "passed" | "failed") {
        return Err(RsGateError::CommandFailed);
    }
    let mut reported: HashMap<&str, &AgentRuleVerdict> = HashMap::new();
    for rule in &verdict.rules {
        if reported.insert(rule.rule_id.as_str(), rule).is_some() {
            return Err(RsGateError::CommandFailed);
        }
    }
    if reported.len() != checker.rules.len() {
        return Err(RsGateError::CommandFailed);
    }
    let mut ordered = Vec::new();
    for declared in &checker.rules {
        let Some(rule) = reported.get(declared.rule_id.as_str()).copied() else {
            return Err(RsGateError::CommandFailed);
        };
        if !matches!(rule.verdict.as_str(), "passed" | "failed")
            || rule
                .failures
                .iter()
                .any(|failure| failure.trim().is_empty())
            || (rule.verdict == "passed") != rule.failures.is_empty()
        {
            return Err(RsGateError::CommandFailed);
        }
        ordered.push(rule.clone());
    }
    let holds = ordered.iter().all(|rule| rule.verdict == "passed");
    if holds != (verdict.overall_verdict == "passed") {
        return Err(RsGateError::CommandFailed);
    }
    Ok(ordered)
}

impl RsCheckCommand for RsProcessCheck {
    #[allow(clippy::too_many_arguments)]
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
        let prepared = Self::preflight(
            plan_id,
            plan_uri,
            plan_sha256,
            wish_model,
            run_record,
            stage,
            artifact_revision,
            snapshot_digest,
        )?;
        if let Some(custom) = checker
            .custom
            .as_ref()
            .filter(|custom| custom.runner == RsCustomRunner::Cli)
        {
            self.run_agent_check(
                checker,
                custom,
                plan_id,
                plan_uri,
                plan_sha256,
                wish_model,
                run_record,
                stage,
                artifact_revision,
                snapshot_digest,
                prepared,
            )
        } else {
            self.run_process_check(
                checker,
                plan_id,
                plan_uri,
                plan_sha256,
                wish_model,
                run_record,
                stage,
                artifact_revision,
                snapshot_digest,
                prepared,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simple_flow::designed_simple_flow_plan;

    fn requirements_checker() -> RsCheckerSpec {
        designed_simple_flow_plan().checkpoints[0]
            .checker
            .clone()
            .unwrap()
    }

    fn verdict_object(overall: &str, rules: &[(&str, &str, &[&str])]) -> Vec<u8> {
        let rules: Vec<serde_json::Value> = rules
            .iter()
            .map(|(rule_id, verdict, failures)| {
                serde_json::json!({
                    "rule_id": rule_id,
                    "verdict": verdict,
                    "failures": failures,
                })
            })
            .collect();
        serde_json::to_vec(&serde_json::json!({
            "overall_verdict": overall,
            "rules": rules,
        }))
        .unwrap()
    }

    #[test]
    fn agent_verdict_accepts_a_complete_passed_object() {
        let checker = requirements_checker();
        let stdout = verdict_object(
            "passed",
            &[
                ("requirements-carry-rationale", "passed", &[]),
                ("satisfy-links-resolve", "passed", &[]),
            ],
        );
        let verdict = parse_agent_verdict(&stdout).unwrap();
        let rules = validate_agent_verdict(&verdict, &checker).unwrap();
        assert_eq!(
            rules
                .iter()
                .map(|rule| rule.rule_id.as_str())
                .collect::<Vec<_>>(),
            vec!["requirements-carry-rationale", "satisfy-links-resolve"]
        );
        assert!(rules.iter().all(|rule| rule.verdict == "passed"));
    }

    #[test]
    fn agent_verdict_tolerates_fenced_or_prefixed_output() {
        let checker = requirements_checker();
        let body = String::from_utf8(verdict_object(
            "failed",
            &[
                ("requirements-carry-rationale", "failed", &["no rationale"]),
                ("satisfy-links-resolve", "passed", &[]),
            ],
        ))
        .unwrap();
        let stdout = format!("Here is my read-only verdict:\n```json\n{body}\n```\nDone.");
        let verdict = parse_agent_verdict(stdout.as_bytes()).unwrap();
        let rules = validate_agent_verdict(&verdict, &checker).unwrap();
        assert_eq!(rules[0].verdict, "failed");
        assert_eq!(rules[0].failures, vec!["no rationale"]);
        assert_eq!(verdict.overall_verdict, "failed");
    }

    #[test]
    fn agent_verdict_rejects_missing_or_extra_rules() {
        let checker = requirements_checker();
        let missing = verdict_object("passed", &[("requirements-carry-rationale", "passed", &[])]);
        let verdict = parse_agent_verdict(&missing).unwrap();
        assert_eq!(
            validate_agent_verdict(&verdict, &checker),
            Err(RsGateError::CommandFailed)
        );
        let extra = verdict_object(
            "passed",
            &[
                ("requirements-carry-rationale", "passed", &[]),
                ("satisfy-links-resolve", "passed", &[]),
                ("not-a-declared-rule", "passed", &[]),
            ],
        );
        let verdict = parse_agent_verdict(&extra).unwrap();
        assert_eq!(
            validate_agent_verdict(&verdict, &checker),
            Err(RsGateError::CommandFailed)
        );
    }

    #[test]
    fn agent_verdict_rejects_inconsistent_overall_and_failures() {
        let checker = requirements_checker();
        let inconsistent = verdict_object(
            "passed",
            &[
                ("requirements-carry-rationale", "failed", &["missing"]),
                ("satisfy-links-resolve", "passed", &[]),
            ],
        );
        let verdict = parse_agent_verdict(&inconsistent).unwrap();
        assert_eq!(
            validate_agent_verdict(&verdict, &checker),
            Err(RsGateError::CommandFailed)
        );
        let failed_without_failure = verdict_object(
            "failed",
            &[
                ("requirements-carry-rationale", "failed", &[]),
                ("satisfy-links-resolve", "passed", &[]),
            ],
        );
        let verdict = parse_agent_verdict(&failed_without_failure).unwrap();
        assert_eq!(
            validate_agent_verdict(&verdict, &checker),
            Err(RsGateError::CommandFailed)
        );
    }
}
