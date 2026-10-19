use crate::status::{RsCheckKind, RsConformanceKind, RsPlanError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashSet, fs, path::Path};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsStageSpec {
    pub stage_id: String,
    pub next_stage_ids: Vec<String>,
    pub exit_checkpoint_ids: Vec<String>,
    pub completes_flow: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckerRuleSpec {
    pub rule_id: String,
    pub condition: String,
    pub reads: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckerSpec {
    pub checker_id: String,
    pub command_uri: String,
    pub establishes: Vec<RsConformanceKind>,
    pub rules: Vec<RsCheckerRuleSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckpointSpec {
    pub checkpoint_id: String,
    pub stage_id: String,
    pub kind: RsCheckKind,
    pub criteria: Vec<String>,
    pub checker: Option<RsCheckerSpec>,
    pub agent_may_judge: bool,
    pub human_may_judge: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RsFlowPlan {
    pub plan_id: String,
    pub plan_uri: String,
    pub plan_sha256: String,
    pub initial_stage_id: String,
    pub stages: Vec<RsStageSpec>,
    pub checkpoints: Vec<RsCheckpointSpec>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
struct FrozenPlan {
    plan_id: String,
    initial_stage_id: String,
    stages: Vec<RsStageSpec>,
    checkpoints: Vec<RsCheckpointSpec>,
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn valid_digest(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn valid_relative_path(s: &str) -> bool {
    let path = Path::new(s);
    !s.is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
}

impl RsFlowPlan {
    fn frozen(&self) -> FrozenPlan {
        FrozenPlan {
            plan_id: self.plan_id.clone(),
            initial_stage_id: self.initial_stage_id.clone(),
            stages: self.stages.clone(),
            checkpoints: self.checkpoints.clone(),
        }
    }

    /// Freeze the plan bytes once and store their hash outside the hashed file.
    pub fn freeze(&mut self, uri: impl Into<String>) -> Result<(), RsPlanError> {
        let uri = uri.into();
        if !valid_relative_path(&uri) {
            return Err(RsPlanError::InvalidPlanFile);
        }
        let bytes =
            serde_json::to_vec_pretty(&self.frozen()).map_err(|_| RsPlanError::InvalidPlanFile)?;
        if let Some(parent) = Path::new(&uri).parent() {
            fs::create_dir_all(parent).map_err(|_| RsPlanError::InvalidPlanFile)?;
        }
        fs::write(&uri, &bytes).map_err(|_| RsPlanError::InvalidPlanFile)?;
        self.plan_uri = uri;
        self.plan_sha256 = sha256_bytes(&bytes);
        self.validate()
    }

    pub fn validate(&self) -> Result<(), RsPlanError> {
        let mut stage_ids = HashSet::new();
        for stage in &self.stages {
            if !stage_ids.insert(stage.stage_id.as_str()) || stage.stage_id.is_empty() {
                return Err(RsPlanError::DuplicateStageId);
            }
        }
        if self.stages.is_empty() || !stage_ids.contains(self.initial_stage_id.as_str()) {
            return Err(RsPlanError::MissingInitialStage);
        }
        for stage in &self.stages {
            if stage
                .next_stage_ids
                .iter()
                .any(|id| !stage_ids.contains(id.as_str()))
            {
                return Err(RsPlanError::UnknownStageReference);
            }
        }
        let mut gate_ids = HashSet::new();
        let mut checker_ids = HashSet::new();
        for gate in &self.checkpoints {
            if !stage_ids.contains(gate.stage_id.as_str()) {
                return Err(RsPlanError::UnknownStageReference);
            }
            match (gate.kind, gate.checker.as_ref()) {
                (RsCheckKind::Syntactic, None) => {
                    return Err(RsPlanError::SyntacticSpecWithoutChecker)
                }
                (RsCheckKind::Semantic, Some(_)) => {
                    return Err(RsPlanError::SemanticSpecWithChecker)
                }
                _ => (),
            }
            if gate.criteria.is_empty() || gate.criteria.iter().any(|s| s.trim().is_empty()) {
                return Err(RsPlanError::MissingCriteria);
            }
            if gate.kind == RsCheckKind::Semantic && !(gate.agent_may_judge || gate.human_may_judge)
            {
                return Err(RsPlanError::NoPermittedJudge);
            }
            if let Some(checker) = &gate.checker {
                if gate.agent_may_judge
                    || gate.human_may_judge
                    || !checker_ids.insert(checker.checker_id.as_str())
                    || checker.checker_id.trim().is_empty()
                    || !valid_relative_path(&checker.command_uri)
                    || checker.establishes.is_empty()
                    || checker.rules.is_empty()
                {
                    return Err(RsPlanError::InvalidChecker);
                }
                let mut rule_ids = HashSet::new();
                if checker.rules.iter().any(|rule| {
                    !rule_ids.insert(rule.rule_id.as_str())
                        || rule.rule_id.trim().is_empty()
                        || rule.condition.trim().is_empty()
                        || rule.reads.is_empty()
                        || rule.reads.iter().any(|x| x.trim().is_empty())
                }) {
                    return Err(RsPlanError::InvalidChecker);
                }
            }
            if !gate_ids.insert(gate.checkpoint_id.as_str()) || gate.checkpoint_id.is_empty() {
                return Err(RsPlanError::InvalidDesignedFlow);
            }
        }
        let expected_stages = ["requirements", "design", "implement", "verify"];
        let expected_gates: [&[&str]; 4] = [
            &["simpleRequirementsAccepted"],
            &["simpleDesignAccepted"],
            &["simpleImplementationAccepted"],
            &["simpleRecordAccepted", "verificationAccepted"],
        ];
        if self.initial_stage_id != expected_stages[0]
            || self.stages.len() != 4
            || self.checkpoints.len() != 5
        {
            return Err(RsPlanError::InvalidDesignedFlow);
        }
        for (i, stage) in self.stages.iter().enumerate() {
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
        for (i, gate) in self.checkpoints.iter().enumerate() {
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
            self.checkpoints.iter().take(4).zip(expected_checkers)
        {
            let Some(checker) = gate.checker.as_ref() else {
                return Err(RsPlanError::InvalidChecker);
            };
            if checker.checker_id != checker_id
                || checker.command_uri != command_uri
                || checker
                    .rules
                    .iter()
                    .map(|r| r.rule_id.as_str())
                    .collect::<Vec<_>>()
                    != rule_ids
            {
                return Err(RsPlanError::InvalidChecker);
            }
        }
        let blueprint = crate::factory::designed_simple_flow_plan();
        if self.stages != blueprint.stages || self.checkpoints != blueprint.checkpoints {
            return Err(RsPlanError::InvalidDesignedFlow);
        }
        if self.plan_id != "designedSimpleFlow"
            || !valid_relative_path(&self.plan_uri)
            || !valid_digest(&self.plan_sha256)
        {
            return Err(RsPlanError::InvalidPlanFile);
        }
        let bytes = fs::read(&self.plan_uri).map_err(|_| RsPlanError::InvalidPlanFile)?;
        if sha256_bytes(&bytes) != self.plan_sha256 {
            return Err(RsPlanError::InvalidPlanFile);
        }
        let decoded: FrozenPlan =
            serde_json::from_slice(&bytes).map_err(|_| RsPlanError::InvalidPlanFile)?;
        if decoded != self.frozen() {
            return Err(RsPlanError::InvalidPlanFile);
        }
        Ok(())
    }

    pub fn initial_stage(&self) -> &RsStageSpec {
        self.stage(&self.initial_stage_id).expect("validated plan")
    }
    pub fn stages(&self) -> &[RsStageSpec] {
        &self.stages
    }
    pub fn stage(&self, id: &str) -> Option<&RsStageSpec> {
        self.stages.iter().find(|s| s.stage_id == id)
    }
    pub fn next_stage(&self, id: &str) -> Option<&RsStageSpec> {
        self.stage(id)
            .and_then(|s| s.next_stage_ids.first())
            .and_then(|id| self.stage(id))
    }
    pub fn checkpoints_of(&self, stage_id: &str) -> Vec<&RsCheckpointSpec> {
        self.checkpoints
            .iter()
            .filter(|c| c.stage_id == stage_id)
            .collect()
    }
}
