use super::{
    plan::RsCheckerSpec,
    status::{RsCheckpointOutcome, RsGateError, RsJudgeAuthority},
};

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
