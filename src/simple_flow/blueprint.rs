use crate::checkpoint::{RsCheckpointExecutorMode, RsCheckpointKind, RsCheckpointLibrary};
use crate::core::{
    valid_checker_id, valid_library_id, RsCheckKind, RsCheckerRuleSpec, RsCheckerSpec,
    RsCheckpointSpec, RsConformanceKind, RsCustomChecker, RsCustomRunner, RsFlowPlan, RsPlanError,
    RsStageSpec,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    path::Path,
};

/// The four syntactic SimpleFlow checkpoints that accept check bindings.
pub const SIMPLE_FLOW_CHECK_BINDING_IDS: [&str; 4] = [
    "simpleRequirementsAccepted",
    "simpleDesignAccepted",
    "simpleImplementationAccepted",
    "simpleRecordAccepted",
];

/// One per-checkpoint check binding supplied as strict JSON at initialization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCheckBinding {
    pub checkpoint_id: String,
    pub mode: RsCheckBindingMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_id: Option<String>,
}

/// How a bound checkpoint is executed: the built-in deterministic checker, an
/// agent CLI, or a project command that keeps the checker contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsCheckBindingMode {
    Default,
    Cli,
    Command,
    Library,
}

/// The strict binding configuration accepted by `xdesign init --checks FILE`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCheckBindingConfig {
    pub schema_version: u32,
    pub checks: Vec<RsCheckBinding>,
}

/// A project-relative `.py` file that may stand in for a covered checkpoint's
/// command. It must be normalized and exist in the repository.
pub fn valid_custom_command_uri(uri: &str) -> bool {
    !uri.is_empty()
        && !uri.contains('\\')
        && !uri.contains('\0')
        && uri
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && Path::new(uri)
            .components()
            .all(|part| matches!(part, std::path::Component::Normal(_)))
        && uri.ends_with(".py")
        && Path::new(uri).is_file()
}

fn checker_mut<'a>(plan: &'a mut RsFlowPlan, id: &str) -> Result<&'a mut RsCheckerSpec, String> {
    plan.checkpoints
        .iter_mut()
        .find(|gate| gate.checkpoint_id == id)
        .and_then(|gate| gate.checker.as_mut())
        .ok_or_else(|| format!("checkpoint {id} is not a syntactic gate of this plan"))
}

fn is_simple_flow_plan(plan: &RsFlowPlan) -> bool {
    SIMPLE_FLOW_CHECK_BINDING_IDS.iter().all(|id| {
        plan.checkpoints.iter().any(|gate| {
            gate.checkpoint_id == *id
                && gate.kind == RsCheckKind::Syntactic
                && gate.checker.is_some()
        })
    })
}

/// Apply validated non-library check bindings to an unfrozen plan. Default
/// bindings are a no-op; `Cli` keeps the covered checker's default command and
/// rules and only replaces the executor; `Command` swaps the command URI; a
/// library binding needs the project library and is refused here.
pub fn apply_check_bindings(
    plan: &mut RsFlowPlan,
    config: &RsCheckBindingConfig,
) -> Result<(), String> {
    apply_bindings(plan, config, None).map(|_| ())
}

/// Apply validated check bindings against the project checkpoint library.
/// Returns the frozen work inputs (stage id -> declared inputs) of every
/// library-bound syntactic gate, so initialization does not invoke that
/// checker's `--list-inputs`.
pub fn apply_check_bindings_with_library(
    plan: &mut RsFlowPlan,
    config: &RsCheckBindingConfig,
    library: &RsCheckpointLibrary,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    apply_bindings(plan, config, Some(library))
}

fn expected_binding_ids(library_mode: bool) -> Vec<&'static str> {
    if library_mode {
        SIMPLE_FLOW_CHECK_BINDING_IDS
            .iter()
            .copied()
            .chain(["verificationAccepted"])
            .collect()
    } else {
        SIMPLE_FLOW_CHECK_BINDING_IDS.to_vec()
    }
}

fn expected_command_uri(checkpoint_id: &str) -> String {
    designed_simple_flow_plan()
        .checkpoints
        .iter()
        .find(|gate| gate.checkpoint_id == checkpoint_id)
        .and_then(|gate| gate.checker.as_ref())
        .map(|checker| checker.command_uri.clone())
        .expect("designed SimpleFlow gate has a checker")
}

fn apply_bindings(
    plan: &mut RsFlowPlan,
    config: &RsCheckBindingConfig,
    library: Option<&RsCheckpointLibrary>,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    if config.schema_version != 1 {
        return Err(format!(
            "unsupported check binding schema_version {} (expected 1)",
            config.schema_version
        ));
    }
    let mut seen = HashSet::new();
    for binding in &config.checks {
        if !seen.insert(binding.checkpoint_id.as_str()) {
            return Err(format!(
                "duplicate check binding for checkpoint {}",
                binding.checkpoint_id
            ));
        }
    }
    let simple_flow = is_simple_flow_plan(plan);
    let mut library_inputs = BTreeMap::new();
    for binding in &config.checks {
        let library_mode = binding.mode == RsCheckBindingMode::Library;
        let expected = expected_binding_ids(library_mode);
        if !expected.contains(&binding.checkpoint_id.as_str()) {
            return Err(format!(
                "unknown checkpoint in check bindings: {} (expected one of {})",
                binding.checkpoint_id,
                expected.join(", ")
            ));
        }
        match binding.mode {
            RsCheckBindingMode::Default => {}
            RsCheckBindingMode::Cli => {
                if !simple_flow {
                    return Err("custom checks are supported for simple-flow only".into());
                }
                let cli = binding
                    .cli
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                let Some(cli) = cli else {
                    return Err(format!(
                        "checkpoint {}: cli mode requires cli codex or kimi",
                        binding.checkpoint_id
                    ));
                };
                if !matches!(cli, "codex" | "kimi") {
                    return Err(format!(
                        "checkpoint {}: cli mode requires cli codex or kimi",
                        binding.checkpoint_id
                    ));
                }
                let model = binding
                    .model
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                let Some(model) = model else {
                    return Err(format!(
                        "checkpoint {}: cli mode requires a nonempty model",
                        binding.checkpoint_id
                    ));
                };
                let checker = checker_mut(plan, &binding.checkpoint_id)?;
                checker.custom = Some(RsCustomChecker {
                    runner: RsCustomRunner::Cli,
                    cli: Some(cli.to_owned()),
                    model: Some(model.to_owned()),
                });
            }
            RsCheckBindingMode::Command => {
                if !simple_flow {
                    return Err("custom checks are supported for simple-flow only".into());
                }
                let Some(command_uri) = binding.command_uri.as_deref() else {
                    return Err(format!(
                        "checkpoint {}: command mode requires a command_uri",
                        binding.checkpoint_id
                    ));
                };
                if !valid_custom_command_uri(command_uri) {
                    return Err(format!(
                        "checkpoint {}: command mode requires a normalized project-relative .py command_uri that exists: {command_uri}",
                        binding.checkpoint_id
                    ));
                }
                let checker = checker_mut(plan, &binding.checkpoint_id)?;
                checker.command_uri = command_uri.to_owned();
                checker.custom = Some(RsCustomChecker {
                    runner: RsCustomRunner::Process,
                    cli: None,
                    model: None,
                });
            }
            RsCheckBindingMode::Library => {
                if !simple_flow {
                    return Err("custom checks are supported for simple-flow only".into());
                }
                let Some(library) = library else {
                    return Err(format!(
                        "checkpoint {}: library bindings require the project checkpoint library",
                        binding.checkpoint_id
                    ));
                };
                let library_id = binding
                    .library_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        format!(
                            "checkpoint {}: library mode requires a library_id",
                            binding.checkpoint_id
                        )
                    })?;
                if !valid_library_id(library_id) {
                    return Err(format!(
                        "checkpoint {}: invalid library_id: {library_id}",
                        binding.checkpoint_id
                    ));
                }
                let Some((entry, _builtin)) = library.entry(library_id) else {
                    return Err(format!(
                        "checkpoint {}: unknown library checkpoint: {library_id}",
                        binding.checkpoint_id
                    ));
                };
                if binding.checkpoint_id == "verificationAccepted" {
                    if entry.kind != RsCheckpointKind::Semantic {
                        return Err(format!(
                            "checkpoint {}: kind mismatch; library checkpoint {library_id} is syntactic but the gate is semantic",
                            binding.checkpoint_id
                        ));
                    }
                    let gate = plan
                        .checkpoints
                        .iter_mut()
                        .find(|gate| gate.checkpoint_id == binding.checkpoint_id)
                        .ok_or_else(|| {
                            format!(
                                "checkpoint {}: plan has no semantic verification gate",
                                binding.checkpoint_id
                            )
                        })?;
                    if gate.checker.is_some() {
                        return Err(format!(
                            "checkpoint {}: library semantic gate cannot carry a checker",
                            binding.checkpoint_id
                        ));
                    }
                    let Some(judges) = entry.judges.as_ref() else {
                        return Err(format!(
                            "checkpoint {library_id}: semantic library entry has no judges"
                        ));
                    };
                    if !(judges.agent || judges.human) {
                        return Err(format!(
                            "checkpoint {library_id}: semantic library entry permits no judge"
                        ));
                    }
                    gate.library = Some(entry.checkpoint_id.clone());
                    gate.criteria = entry.criteria.clone();
                    gate.agent_may_judge = judges.agent;
                    gate.human_may_judge = judges.human;
                } else {
                    if entry.kind != RsCheckpointKind::Syntactic {
                        return Err(format!(
                            "checkpoint {}: kind mismatch; library checkpoint {library_id} is semantic but the gate is syntactic",
                            binding.checkpoint_id
                        ));
                    }
                    let Some(executor) = entry.executor.as_ref() else {
                        return Err(format!(
                            "checkpoint {library_id}: syntactic library entry has no executor"
                        ));
                    };
                    let gate = plan
                        .checkpoints
                        .iter_mut()
                        .find(|gate| gate.checkpoint_id == binding.checkpoint_id)
                        .ok_or_else(|| {
                            format!(
                                "checkpoint {}: plan has no such syntactic gate",
                                binding.checkpoint_id
                            )
                        })?;
                    let checker = gate.checker.as_mut().ok_or_else(|| {
                        format!(
                            "checkpoint {}: plan gate has no checker to bind",
                            binding.checkpoint_id
                        )
                    })?;
                    checker.checker_id = entry
                        .checker_id
                        .clone()
                        .unwrap_or_else(|| entry.checkpoint_id.clone());
                    checker.rules = entry.rules.clone();
                    match executor.mode {
                        RsCheckpointExecutorMode::Script => {
                            let command_uri = executor
                                .command_uri
                                .as_deref()
                                .ok_or_else(|| {
                                    format!(
                                        "checkpoint {library_id}: script executor has no command_uri"
                                    )
                                })?
                                .to_owned();
                            if !valid_custom_command_uri(&command_uri) {
                                return Err(format!(
                                    "checkpoint {library_id}: library script does not exist: {command_uri}"
                                ));
                            }
                            checker.command_uri = command_uri;
                            checker.custom = Some(RsCustomChecker {
                                runner: RsCustomRunner::Process,
                                cli: None,
                                model: None,
                            });
                        }
                        RsCheckpointExecutorMode::Cli => {
                            let cli = executor
                                .cli
                                .as_deref()
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .ok_or_else(|| {
                                    format!("checkpoint {library_id}: cli executor has no cli")
                                })?;
                            if !matches!(cli, "codex" | "kimi") {
                                return Err(format!(
                                    "checkpoint {library_id}: cli executor requires codex or kimi"
                                ));
                            }
                            let model = executor
                                .model
                                .as_deref()
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .ok_or_else(|| {
                                    format!("checkpoint {library_id}: cli executor has no model")
                                })?;
                            checker.command_uri = expected_command_uri(&binding.checkpoint_id);
                            checker.custom = Some(RsCustomChecker {
                                runner: RsCustomRunner::Cli,
                                cli: Some(cli.to_owned()),
                                model: Some(model.to_owned()),
                            });
                        }
                    }
                    gate.library = Some(entry.checkpoint_id.clone());
                    library_inputs.insert(gate.stage_id.clone(), entry.inputs.clone());
                }
            }
        }
    }
    Ok(library_inputs)
}

/// The modeled designedSimpleFlow as plan data, before `freeze` binds its bytes.
pub fn designed_simple_flow_plan() -> RsFlowPlan {
    fn rule(id: &str, condition: &str, reads: &[&str]) -> RsCheckerRuleSpec {
        RsCheckerRuleSpec {
            rule_id: id.into(),
            condition: condition.into(),
            reads: reads.iter().map(|x| x.to_string()).collect(),
        }
    }
    fn gate(id: &str, stage: &str, criterion: &str, checker: RsCheckerSpec) -> RsCheckpointSpec {
        RsCheckpointSpec {
            checkpoint_id: id.into(),
            stage_id: stage.into(),
            kind: RsCheckKind::Syntactic,
            criteria: vec![criterion.into()],
            checker: Some(checker),
            agent_may_judge: false,
            human_may_judge: false,
            library: None,
        }
    }
    fn checker(
        id: &str,
        path: &str,
        establishes: Vec<RsConformanceKind>,
        rules: Vec<RsCheckerRuleSpec>,
    ) -> RsCheckerSpec {
        RsCheckerSpec {
            checker_id: id.into(),
            command_uri: path.into(),
            establishes,
            rules,
            custom: None,
        }
    }
    let req = checker("simpleRequirementsCheck", "tools/checks/simple_requirements.py",
        vec![RsConformanceKind::IdentifierConformance, RsConformanceKind::TraceConformance], vec![
            rule("requirements-carry-rationale", "XDesignRequirements declares at least one requirement, and every requirement declaration inside it carries a nonempty rationale attribute.", &["mbse/model/requirements.sysml"]),
            rule("satisfy-links-resolve", "The declared traceability source has at least one satisfy link, and every satisfy link names a requirement declared inside XDesignRequirements.", &["mbse/model/requirements.sysml", "mbse/model/traceability.sysml"]),
        ]);
    let design = checker("simpleDesignCheck", "tools/checks/simple_design.py",
        vec![RsConformanceKind::ArtifactPresence, RsConformanceKind::DefinitionCoverage], vec![
            rule("design-declarations-present", "The run, factory, gate, advance, designed flow definition and xdesign-owned usage, checkpoint class, both adaptations, and checkpoint evaluation action are declared.", &["mbse/model/detailedDesign.sysml", "mbse/model/composition.sysml"]),
            rule("every-stage-gated", "Every stage names nonempty exit gates declared by this flow and typed by SimpleFlowCheckpoint; each syntactic exit binds its designated one of the four fixed SimpleFlow checkers.", &["mbse/model/detailedDesign.sysml"]),
            rule("design-elements-carry-doc-sections", "Every top-level detailed design element has a DocSection that resolves to a document and section.", &["mbse/model/detailedDesign.sysml", "docs/"]),
            rule("checker-commands-exist", "Every declared SimpleFlow checker command URI names an existing repository file.", &["mbse/model/detailedDesign.sysml", "tools/checks/"]),
            rule("checker-rules-are-declared", "All four fixed SimpleFlow checkers are declared, each with at least one rule with a nonempty ID, condition and reads value.", &["mbse/model/detailedDesign.sysml"]),
            rule("rule-sets-agree-with-commands", "Each of the four fixed checkers' declared rule IDs exactly equal its command's rule ID list.", &["mbse/model/detailedDesign.sysml", "tools/checks/"]),
            rule("structure-graphs-are-carried", "Each of the nine primary SimpleFlow views names a distinct model figure, and every required figure exists and is embedded in the document named by its DocFigure annotation.", &["mbse/model/detailedDesign.sysml", "docs/"]),
        ]);
    let implementation = checker("simpleImplementationCheck", "tools/checks/simple_implementation.py",
        vec![RsConformanceKind::ArtifactPresence], vec![
            rule("visit-exists", "The checked implementation stage has a current visit in the run record.", &["runtime run record JSON"]),
            rule("artifacts-of-current-visit-exist", "The current implementation visit records at least one artifact URI and every recorded artifact exists at that URI.", &["runtime run record JSON", "recorded artifact URIs"]),
        ]);
    let record = checker("simpleRecordCheck", "tools/checks/simple_record.py",
        vec![RsConformanceKind::TraceConformance, RsConformanceKind::ArtifactPresence], vec![
            rule("flow-is-supported", "The run record has plan_id designedSimpleFlow and plan_sha256 equal to the SHA-256 of the retained frozen plan serialization.", &["runtime run record JSON", "retained plan serialization"]),
            rule("every-stage-visited", "Every planned stage has at least one visit and every visit names a stage in that plan.", &["runtime run record JSON"]),
            rule("completed-visits-authorized", "For every completed visit, the latest assessment of each required exit gate passed at that visit's artifact revision and snapshot digest, and every recorded artifact still matches its file digest.", &["runtime run record JSON", "recorded artifact URIs"]),
            rule("assessments-name-exit-gates", "Every passed assessment names an exit gate of its visit's stage.", &["runtime run record JSON"]),
            rule("passed-assessments-carry-evidence", "Every passed assessment names at least one evidence URI.", &["runtime run record JSON"]),
            rule("evidence-exists", "Every evidence URI of every passed assessment on current or completed visits resolves to retained evidence.", &["runtime run record JSON", "recorded evidence URIs"]),
            rule("designed-runs-are-typed-by-the-design-class", "A run on the designed SimpleFlow records its execution as a FlowRun of plan designedSimpleFlow.", &["runtime run record JSON"]),
        ]);
    RsFlowPlan {
        plan_id: "designedSimpleFlow".into(), plan_uri: String::new(), plan_sha256: String::new(),
        initial_stage_id: "requirements".into(),
        stages: vec![
            RsStageSpec { stage_id: "requirements".into(), next_stage_ids: vec!["design".into()], exit_checkpoint_ids: vec!["simpleRequirementsAccepted".into()], completes_flow: false },
            RsStageSpec { stage_id: "design".into(), next_stage_ids: vec!["implement".into()], exit_checkpoint_ids: vec!["simpleDesignAccepted".into()], completes_flow: false },
            RsStageSpec { stage_id: "implement".into(), next_stage_ids: vec!["verify".into()], exit_checkpoint_ids: vec!["simpleImplementationAccepted".into()], completes_flow: false },
            RsStageSpec { stage_id: "verify".into(), next_stage_ids: vec![], exit_checkpoint_ids: vec!["simpleRecordAccepted".into(), "verificationAccepted".into()], completes_flow: true },
        ],
        checkpoints: vec![
            gate("simpleRequirementsAccepted", "requirements", "Every requirement declared in the project model carries its rationale, and every satisfy link resolves to a requirement that is declared.", req),
            gate("simpleDesignAccepted", "design", "The design declares the run class, the factory, the functions that evaluate gates and advance the run, and the checkpoint class with its syntactic and semantic adaptations and its evaluate operation; every exit checkpoint of this flow is an instance of that class; every stage of this flow declares its exit checkpoint; every design element names its documentation section; and all nine SysML diagram views are declared, drawn and carried by their document.", design),
            gate("simpleImplementationAccepted", "implement", "The stage's current visit records at least one artifact, and every recorded artifact exists at its URI.", implementation),
            gate("simpleRecordAccepted", "verify", "The run records at least one visit per stage; every completed visit has a latest passed assessment of each exit gate at its artifact revision; and evidence of every passed assessment on current or completed visits is retained.", record),
            RsCheckpointSpec { checkpoint_id: "verificationAccepted".into(), stage_id: "verify".into(), kind: RsCheckKind::Semantic,
                criteria: vec!["Verification evidence demonstrates that the implementation satisfies the requirements.".into(),
                    "The record and the evidence stand on their own: a reader can re-examine the judgment from them without re-asking.".into()],
                checker: None, agent_may_judge: true, human_may_judge: true, library: None },
        ],
    }
}

/// Blueprint conformance: reject any plan that differs from the designed SimpleFlow
/// except for the frozen library bindings a run chose at initialization. The
/// frozen rules/criteria are the durable definition; no library file is read here.
pub fn validate_designed_plan(plan: &RsFlowPlan) -> Result<(), RsPlanError> {
    let expected_stages = ["requirements", "design", "implement", "verify"];
    let expected_gates: [&[&str]; 4] = [
        &["simpleRequirementsAccepted"],
        &["simpleDesignAccepted"],
        &["simpleImplementationAccepted"],
        &["simpleRecordAccepted", "verificationAccepted"],
    ];
    if plan.initial_stage_id != expected_stages[0]
        || plan.stages.len() != 4
        || plan.checkpoints.len() != 5
    {
        return Err(RsPlanError::InvalidDesignedFlow);
    }
    for (i, stage) in plan.stages.iter().enumerate() {
        let next: Vec<String> = if i == 3 {
            vec![]
        } else {
            vec![expected_stages[i + 1].to_owned()]
        };
        if stage.stage_id != expected_stages[i]
            || stage.next_stage_ids != next
            || stage.completes_flow != (i == 3)
            || stage.exit_checkpoint_ids
                != expected_gates[i]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
        {
            return Err(RsPlanError::InvalidDesignedFlow);
        }
    }
    let expected_kinds = [
        RsCheckKind::Syntactic,
        RsCheckKind::Syntactic,
        RsCheckKind::Syntactic,
        RsCheckKind::Syntactic,
        RsCheckKind::Semantic,
    ];
    let expected_ids: Vec<&str> = expected_gates
        .iter()
        .flat_map(|x| x.iter().copied())
        .collect();
    for (i, gate) in plan.checkpoints.iter().enumerate() {
        let stage = if i < 4 { expected_stages[i] } else { "verify" };
        if gate.checkpoint_id != expected_ids[i]
            || gate.stage_id != stage
            || gate.kind != expected_kinds[i]
        {
            return Err(RsPlanError::InvalidDesignedFlow);
        }
    }
    let expected_checkers = [
        (
            "simpleRequirementsCheck",
            "tools/checks/simple_requirements.py",
            &["requirements-carry-rationale", "satisfy-links-resolve"][..],
        ),
        (
            "simpleDesignCheck",
            "tools/checks/simple_design.py",
            &[
                "design-declarations-present",
                "every-stage-gated",
                "design-elements-carry-doc-sections",
                "checker-commands-exist",
                "checker-rules-are-declared",
                "rule-sets-agree-with-commands",
                "structure-graphs-are-carried",
            ][..],
        ),
        (
            "simpleImplementationCheck",
            "tools/checks/simple_implementation.py",
            &["visit-exists", "artifacts-of-current-visit-exist"][..],
        ),
        (
            "simpleRecordCheck",
            "tools/checks/simple_record.py",
            &[
                "flow-is-supported",
                "every-stage-visited",
                "completed-visits-authorized",
                "assessments-name-exit-gates",
                "passed-assessments-carry-evidence",
                "evidence-exists",
                "designed-runs-are-typed-by-the-design-class",
            ][..],
        ),
    ];
    for (gate, (checker_id, command_uri, rule_ids)) in
        plan.checkpoints.iter().take(4).zip(expected_checkers)
    {
        let Some(checker) = gate.checker.as_ref() else {
            return Err(RsPlanError::InvalidChecker);
        };
        if let Some(library) = gate.library.as_deref() {
            // The frozen checker identity is the reused script identity when
            // the library entry declares one; the library id only names the
            // provenance, so the two need not be equal.
            if !valid_library_id(library) || !valid_checker_id(&checker.checker_id) {
                return Err(RsPlanError::InvalidChecker);
            }
            let mut rule_ids_seen = HashSet::new();
            if checker.rules.is_empty()
                || checker.rules.iter().any(|rule| {
                    rule.rule_id.trim().is_empty()
                        || rule.condition.trim().is_empty()
                        || rule.reads.is_empty()
                        || rule.reads.iter().any(|read| read.trim().is_empty())
                        || !rule_ids_seen.insert(rule.rule_id.as_str())
                })
            {
                return Err(RsPlanError::InvalidChecker);
            }
            match checker.custom.as_ref() {
                None => return Err(RsPlanError::InvalidChecker),
                Some(custom) => match custom.runner {
                    RsCustomRunner::Cli => {
                        let cli = custom.cli.as_deref().unwrap_or_default();
                        let model = custom.model.as_deref().unwrap_or_default();
                        if !matches!(cli, "codex" | "kimi")
                            || model.trim().is_empty()
                            || checker.command_uri != command_uri
                        {
                            return Err(RsPlanError::InvalidChecker);
                        }
                    }
                    RsCustomRunner::Process => {
                        if custom.cli.is_some()
                            || custom.model.is_some()
                            || !valid_custom_command_uri(&checker.command_uri)
                        {
                            return Err(RsPlanError::InvalidChecker);
                        }
                    }
                },
            }
            continue;
        }
        if checker.checker_id != checker_id
            || checker
                .rules
                .iter()
                .map(|r| r.rule_id.as_str())
                .collect::<Vec<_>>()
                != rule_ids
        {
            return Err(RsPlanError::InvalidChecker);
        }
        match checker.custom.as_ref() {
            None => {
                if checker.command_uri != command_uri {
                    return Err(RsPlanError::InvalidChecker);
                }
            }
            Some(custom) => match custom.runner {
                RsCustomRunner::Cli => {
                    let cli = custom.cli.as_deref().unwrap_or_default();
                    let model = custom.model.as_deref().unwrap_or_default();
                    if !matches!(cli, "codex" | "kimi")
                        || model.trim().is_empty()
                        || checker.command_uri != command_uri
                    {
                        return Err(RsPlanError::InvalidChecker);
                    }
                }
                RsCustomRunner::Process => {
                    if custom.cli.is_some()
                        || custom.model.is_some()
                        || !valid_custom_command_uri(&checker.command_uri)
                    {
                        return Err(RsPlanError::InvalidChecker);
                    }
                }
            },
        }
    }
    let semantic = &plan.checkpoints[4];
    if let Some(library) = semantic.library.as_deref() {
        if !valid_library_id(library)
            || semantic.checker.is_some()
            || semantic.criteria.is_empty()
            || semantic
                .criteria
                .iter()
                .any(|criterion| criterion.trim().is_empty())
            || !(semantic.agent_may_judge || semantic.human_may_judge)
        {
            return Err(RsPlanError::InvalidDesignedFlow);
        }
    }
    let blueprint = designed_simple_flow_plan();
    let mut normalized = plan.clone();
    for (index, gate) in normalized.checkpoints.iter_mut().take(4).enumerate() {
        let library_bound = gate.library.is_some();
        gate.library = None;
        if let Some(checker) = gate.checker.as_mut() {
            checker.custom = None;
            checker.command_uri = expected_checkers[index].1.to_owned();
            if library_bound {
                checker.checker_id = expected_checkers[index].0.to_owned();
                checker.rules = blueprint.checkpoints[index]
                    .checker
                    .as_ref()
                    .expect("designed gate carries a checker")
                    .rules
                    .clone();
            }
        }
    }
    if normalized.checkpoints[4].library.is_some() {
        normalized.checkpoints[4].library = None;
        normalized.checkpoints[4].criteria = blueprint.checkpoints[4].criteria.clone();
        normalized.checkpoints[4].agent_may_judge = blueprint.checkpoints[4].agent_may_judge;
        normalized.checkpoints[4].human_may_judge = blueprint.checkpoints[4].human_may_judge;
    }
    if normalized.stages != blueprint.stages || normalized.checkpoints != blueprint.checkpoints {
        return Err(RsPlanError::InvalidDesignedFlow);
    }
    if plan.plan_id != "designedSimpleFlow" {
        return Err(RsPlanError::InvalidPlanFile);
    }
    Ok(())
}
