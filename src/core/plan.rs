use super::status::{RsCheckKind, RsConformanceKind, RsPlanError};
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

/// How a checkpoint's custom binding executes: either the project's checker
/// script (the default contract) or an agent CLI that answers the declared
/// rules directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsCustomRunner {
    Process,
    Cli,
}

/// The non-default execution binding frozen into a plan. It never redefines
/// the declared rules or the default command URI; it only chooses who
/// executes them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCustomChecker {
    pub runner: RsCustomRunner,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Exact executor files retained by a store-backed registration (D111/U110).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RsCheckerBundle {
    pub source: String,
    pub entry_uri: String,
    pub files: Vec<crate::core::RsArtifactDigest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
}

impl RsCheckerBundle {
    pub(crate) fn valid(&self) -> bool {
        let mut seen = HashSet::new();
        matches!(self.source.as_str(), "builtin-v1" | "project")
            && valid_relative_path(&self.entry_uri)
            && self.entry_uri.ends_with(".py")
            && !self.files.is_empty()
            && self.files.len() <= 256
            && self.files.iter().all(|f| {
                valid_relative_path(&f.uri) && valid_digest(&f.sha256) && seen.insert(&f.uri)
            })
            && self.files.iter().any(|f| f.uri == self.entry_uri)
            && self
                .python
                .as_deref()
                .is_none_or(|p| valid_relative_path(p) && p.starts_with("cache/"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsCheckerSpec {
    pub checker_id: String,
    pub command_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle: Option<RsCheckerBundle>,
    pub establishes: Vec<RsConformanceKind>,
    pub rules: Vec<RsCheckerRuleSpec>,
    /// Absent for the deterministic default binding, so default plan bytes
    /// stay exactly as before custom bindings were introduced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<RsCustomChecker>,
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
    /// The project checkpoint-library entry this gate was frozen from, if the
    /// run bound a library checkpoint at initialization. The frozen rules or
    /// criteria are the durable definition; this id only names the origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library: Option<String>,
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

/// A checker identity a script may report, and the identity frozen into a
/// library-bound plan: `[A-Za-z0-9][A-Za-z0-9_-]{0,127}`.
pub fn valid_checker_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && s.bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// Checkpoint-library ids are lowercase slugs: `[a-z0-9][a-z0-9_-]{0,63}`.
pub fn valid_library_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && s.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
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

    /// Read and verify a frozen plan back from its retained bytes, then
    /// structurally validate it. Flow layers add their own conformance rules.
    pub fn load_frozen(plan_uri: &str, plan_sha256: &str) -> Result<RsFlowPlan, RsPlanError> {
        if !valid_relative_path(plan_uri) || !valid_digest(plan_sha256) {
            return Err(RsPlanError::InvalidPlanFile);
        }
        let bytes = fs::read(plan_uri).map_err(|_| RsPlanError::InvalidPlanFile)?;
        if sha256_bytes(&bytes) != plan_sha256 {
            return Err(RsPlanError::InvalidPlanFile);
        }
        let decoded: FrozenPlan =
            serde_json::from_slice(&bytes).map_err(|_| RsPlanError::InvalidPlanFile)?;
        let plan = Self {
            plan_id: decoded.plan_id,
            plan_uri: plan_uri.to_owned(),
            plan_sha256: plan_sha256.to_owned(),
            initial_stage_id: decoded.initial_stage_id,
            stages: decoded.stages,
            checkpoints: decoded.checkpoints,
        };
        plan.validate()?;
        Ok(plan)
    }

    /// Structural validation only: a flow layer adds its own conformance rules.
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
            if gate
                .library
                .as_deref()
                .is_some_and(|library| !valid_library_id(library))
            {
                return Err(RsPlanError::InvalidDesignedFlow);
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
                    || checker
                        .bundle
                        .as_ref()
                        .is_some_and(|bundle| !bundle.valid())
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
                match checker.custom.as_ref() {
                    None => {}
                    Some(custom) => match custom.runner {
                        RsCustomRunner::Cli => {
                            if custom
                                .cli
                                .as_deref()
                                .is_none_or(|cli| cli.trim().is_empty())
                                || custom
                                    .model
                                    .as_deref()
                                    .is_none_or(|model| model.trim().is_empty())
                            {
                                return Err(RsPlanError::InvalidChecker);
                            }
                        }
                        RsCustomRunner::Process => {
                            if custom.cli.is_some() || custom.model.is_some() {
                                return Err(RsPlanError::InvalidChecker);
                            }
                        }
                    },
                }
            }
            if !gate_ids.insert(gate.checkpoint_id.as_str()) || gate.checkpoint_id.is_empty() {
                return Err(RsPlanError::InvalidDesignedFlow);
            }
        }
        if self.plan_id.trim().is_empty()
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
