use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsFlowStatus {
    Pending,
    Active,
    Blocked,
    Completed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsStageStatus {
    Pending,
    Active,
    Blocked,
    Completed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsCheckKind {
    Syntactic,
    Semantic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsCheckpointOutcome {
    Unevaluated,
    Failed,
    Passed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsJudgeAuthority {
    Script,
    Agent,
    Human,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsAdvanceOutcome {
    EnterNextStage,
    CompleteRun,
    RemainBlocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RsConformanceKind {
    ArtifactPresence,
    IdentifierConformance,
    DefinitionCoverage,
    TraceConformance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsPlanError {
    MissingInitialStage,
    DuplicateStageId,
    UnknownStageReference,
    SyntacticSpecWithoutChecker,
    SemanticSpecWithChecker,
    MissingWishModelUri,
    MissingRunRecordUri,
    MissingCriteria,
    NoPermittedJudge,
    InvalidDesignedFlow,
    InvalidChecker,
    InvalidPlanFile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsGateError {
    CheckerMissing,
    CommandFailed,
    /// The checker subprocess exceeded its deadline and was killed.
    CheckerTimeout,
    JudgmentRefused,
    JudgeNotPermitted,
    MissingEvidence,
}
