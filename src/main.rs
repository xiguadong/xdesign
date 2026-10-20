use simpflow::{
    create_run, designed_simple_flow_plan, hash_artifact, RsActivityEntry, RsAdvanceOutcome,
    RsCheckpointOutcome, RsFlowStatus, RsJudgeAuthority, RsProcessCheck, RsSuppliedJudge,
};
use std::{collections::BTreeSet, env, fs, path::Path, process::Command};

fn option(args: &[String], name: &str) -> Result<String, String> {
    let index = args
        .iter()
        .position(|x| x == name)
        .ok_or_else(|| format!("missing {name}"))?;
    args.get(index + 1)
        .filter(|x| !x.starts_with("--"))
        .cloned()
        .ok_or_else(|| format!("missing value for {name}"))
}

fn list_checker_inputs(command_uri: &str) -> Result<Vec<String>, String> {
    let output = Command::new("python3")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg(command_uri)
        .arg("--list-inputs")
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "{command_uri} --list-inputs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice::<Vec<String>>(&output.stdout)
        .map_err(|e| format!("{command_uri} returned invalid input list: {e}"))
}

fn timestamp() -> String {
    let output = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output();
    output
        .ok()
        .filter(|x| x.status.success())
        .map(|x| String::from_utf8_lossy(&x.stdout).trim().to_owned())
        .unwrap_or_else(|| "unknown".into())
}

fn stage_artifact_paths(
    stage: &str,
    verification_evidence: &str,
) -> Result<BTreeSet<String>, String> {
    let mut paths = BTreeSet::new();
    match stage {
        "requirements" => {
            paths.extend(list_checker_inputs("tools/checks/simple_requirements.py")?);
        }
        "design" => paths.extend(list_checker_inputs("tools/checks/simple_design.py")?),
        "implement" => {
            paths.insert("Cargo.toml".into());
            for entry in fs::read_dir("src").map_err(|e| e.to_string())? {
                let path = entry.map_err(|e| e.to_string())?.path();
                if path.extension().is_some_and(|x| x == "rs") {
                    paths.insert(path.to_string_lossy().to_string());
                }
            }
        }
        "verify" => {
            paths.insert(verification_evidence.into());
        }
        _ => return Err(format!("unknown stage: {stage}")),
    }
    Ok(paths)
}

fn run_demo(args: &[String]) -> Result<(), String> {
    let wish = option(args, "--wish-model")?;
    let plan_uri = option(args, "--plan")?;
    let record_uri = option(args, "--record")?;
    let verification_evidence = option(args, "--verify-evidence")?;
    let judge_authority = match option(args, "--judge-authority")?.as_str() {
        "agent" => RsJudgeAuthority::Agent,
        "human" => RsJudgeAuthority::Human,
        _ => return Err("--judge-authority must be agent or human".into()),
    };
    let judge_outcome = match option(args, "--judge-outcome")?.as_str() {
        "passed" => RsCheckpointOutcome::Passed,
        "failed" => RsCheckpointOutcome::Failed,
        "unevaluated" => RsCheckpointOutcome::Unevaluated,
        _ => return Err("--judge-outcome must be passed, failed, or unevaluated".into()),
    };
    let judge_identity = option(args, "--judge-identity")?;
    let judge_rationale = option(args, "--judge-rationale")?;
    if judge_identity.trim().is_empty() || judge_rationale.trim().is_empty() {
        return Err("judge identity and rationale must be nonempty".into());
    }
    if !plan_uri.starts_with("cache/") || !record_uri.starts_with("cache/") {
        return Err("plan and record output must be under cache/".into());
    }
    if !Path::new(&wish).is_file() || !wish.ends_with(".sysml") {
        return Err("wish model must be an existing .sysml file".into());
    }
    if !Path::new(&verification_evidence).is_file() {
        return Err("verification evidence file is missing".into());
    }
    let mut plan = designed_simple_flow_plan();
    plan.freeze(plan_uri).map_err(|e| format!("plan: {e:?}"))?;
    let report_directory = format!(
        "{}/reports",
        Path::new(&record_uri)
            .parent()
            .unwrap_or_else(|| Path::new("cache"))
            .display()
    );
    let check = Box::new(RsProcessCheck { report_directory });
    let judge = Box::new(RsSuppliedJudge {
        authority: judge_authority,
        identity: judge_identity,
        outcome: judge_outcome,
        rationale: judge_rationale,
    });
    let mut run = create_run(plan, wish.clone(), record_uri.clone(), check, judge)
        .map_err(|e| format!("create run: {e:?}"))?;
    run.write_record()?;
    for stage in ["requirements", "design", "implement", "verify"] {
        if run.current().stage_id() != stage {
            return Err(format!(
                "expected {stage}, got {}",
                run.current().stage_id()
            ));
        }
        let paths = stage_artifact_paths(stage, &verification_evidence)?;
        let digests = paths
            .iter()
            .map(|p| hash_artifact(p))
            .collect::<Result<Vec<_>, _>>()?;
        let activity = RsActivityEntry {
            description: format!("Record {stage} stage input artifacts"),
            performed_by: "simpflow demo caller".into(),
            occurred_at: timestamp(),
            artifact_uris: paths.into_iter().collect(),
            artifacts_changed: true,
            artifact_digests: digests,
            removed_artifact_uris: vec![],
        };
        run.record_activity(activity)?;
        let entries = run.evaluate_gates();
        println!(
            "{stage}: {}",
            serde_json::to_string(&entries).map_err(|e| e.to_string())?
        );
        let outcome = run.advance();
        println!("{stage} advance: {outcome:?}");
        if outcome == RsAdvanceOutcome::RemainBlocked {
            return Err(format!(
                "run blocked at {stage}; record retained at {record_uri}"
            ));
        }
    }
    if run.status() != RsFlowStatus::Completed {
        return Err("run did not complete".into());
    }
    println!("completed run {}: {record_uri}", run.run_id());
    Ok(())
}

#[cfg(test)]
mod main_wish_tests;

fn main() {
    let args: Vec<_> = env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("demo") => run_demo(&args[2..]),
        _ => Err("usage: simpflow demo --wish-model PATH --plan cache/PLAN.json --record cache/RECORD.json --verify-evidence PATH --judge-authority agent|human --judge-identity NAME --judge-outcome passed|failed|unevaluated --judge-rationale TEXT".into()),
    };
    if let Err(error) = result {
        eprintln!("simpflow: {error}");
        std::process::exit(1);
    }
}
