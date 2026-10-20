use crate::{
    plan::{
        valid_relative_path, RsCheckerRuleSpec, RsCheckerSpec, RsCheckpointSpec, RsFlowPlan,
        RsStageSpec,
    },
    ports::{RsCheckCommand, RsJudge},
    record::RsStageVisit,
    run::RsSimpleFlowRun,
    status::{RsCheckKind, RsConformanceKind, RsFlowStatus, RsPlanError, RsStageStatus},
};
use std::path::Path;
use uuid::Uuid;

pub fn create_run(
    plan: RsFlowPlan,
    wish_model_uri: String,
    run_record_uri: String,
    check: Box<dyn RsCheckCommand>,
    judgment: Box<dyn RsJudge>,
) -> Result<RsSimpleFlowRun, RsPlanError> {
    plan.validate()?;
    if !valid_relative_path(&wish_model_uri)
        || !wish_model_uri.ends_with(".sysml")
        || !Path::new(&wish_model_uri).is_file()
    {
        return Err(RsPlanError::MissingWishModelUri);
    }
    if !valid_relative_path(&run_record_uri) {
        return Err(RsPlanError::MissingRunRecordUri);
    }
    let mut visits: Vec<_> = plan
        .stages
        .iter()
        .map(|s| RsStageVisit::pending(s.stage_id.clone(), 0))
        .collect();
    let current_visit = plan
        .stages
        .iter()
        .position(|s| s.stage_id == plan.initial_stage_id)
        .ok_or(RsPlanError::MissingInitialStage)?;
    visits[current_visit].status = RsStageStatus::Active;
    Ok(RsSimpleFlowRun {
        run_id: Uuid::new_v4().to_string(),
        plan_id: plan.plan_id.clone(),
        plan_uri: plan.plan_uri.clone(),
        plan_sha256: plan.plan_sha256.clone(),
        plan,
        wish_model_uri,
        run_record_uri,
        status: RsFlowStatus::Active,
        visits,
        current_visit,
        check,
        judgment,
        persistence_error: false,
        last_error: None,
    })
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
        }
    }
    let req = checker("simpleRequirementsCheck", "tools/checks/simple_requirements.py",
        vec![RsConformanceKind::IdentifierConformance, RsConformanceKind::TraceConformance], vec![
            rule("requirements-carry-rationale", "Every requirement declaration inside XDesignRequirements carries a nonempty rationale attribute.", &["mbse/model/requirements.sysml"]),
            rule("satisfy-links-resolve", "Every satisfy link names a requirement declared inside XDesignRequirements.", &["mbse/model/requirements.sysml", "mbse/model/traceability.sysml"]),
        ]);
    let design = checker("simpleDesignCheck", "tools/checks/simple_design.py",
        vec![RsConformanceKind::ArtifactPresence, RsConformanceKind::DefinitionCoverage], vec![
            rule("design-declarations-present", "The run, factory, gate, advance, designed flow, checkpoint class, both adaptations, and checkpoint evaluation action are declared.", &["mbse/model/detailedDesign.sysml"]),
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
            rule("flow-is-supported", "The run record has plan_id designedSimpleFlow and plan_sha256 equal to the SHA-256 of the retained frozen plan serialization.", &["runtime run record JSON"]),
            rule("every-stage-visited", "Every planned stage has at least one visit and every visit names a stage in that plan.", &["runtime run record JSON"]),
            rule("completed-visits-authorized", "For every completed visit, the latest assessment of each required exit gate passed at that visit's artifact revision and snapshot digest, and every recorded artifact still matches its file digest.", &["runtime run record JSON", "recorded artifact URIs"]),
            rule("assessments-name-exit-gates", "Every passed assessment names an exit gate of its visit's stage.", &["runtime run record JSON"]),
            rule("passed-assessments-carry-evidence", "Every passed assessment names at least one evidence URI.", &["runtime run record JSON"]),
            rule("evidence-exists", "Every evidence URI of every passed assessment on current or completed visits resolves to retained evidence.", &["runtime run record JSON", "recorded evidence URIs"]),
            rule("designed-runs-are-typed-by-the-design-class", "A run on the designed SimpleFlow records its execution as SimpleFlowRun.", &["runtime run record JSON"]),
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
                checker: None, agent_may_judge: true, human_may_judge: true },
        ],
    }
}
