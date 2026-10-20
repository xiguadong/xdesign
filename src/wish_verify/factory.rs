use super::blueprint::validate_wish_verify_plan;
use crate::core::{
    RsCheckCommand, RsCheckerSpec, RsFlowPlan, RsFlowRun, RsGateError, RsGateReport, RsJudge,
    RsPlanError,
};

/// The check-command stub of an all-semantic flow: the plan declares no
/// syntactic gate, so any invocation of this boundary is an error.
pub struct RsUnusedCheck;

impl RsCheckCommand for RsUnusedCheck {
    fn run_check(
        &self,
        _checker: &RsCheckerSpec,
        _plan_id: &str,
        _plan_uri: &str,
        _plan_sha256: &str,
        _wish_model: &str,
        _run_record: &str,
        _stage: &str,
        _artifact_revision: u32,
        _snapshot_digest: &str,
    ) -> Result<RsGateReport, RsGateError> {
        Err(RsGateError::CommandFailed)
    }
}

/// Blueprint conformance, then generic construction of the designed run.
pub fn create_wish_verify_run(
    plan: RsFlowPlan,
    wish_model_uri: &str,
    run_record_uri: &str,
    judgment: impl RsJudge + Send + 'static,
) -> Result<RsFlowRun, RsPlanError> {
    validate_wish_verify_plan(&plan)?;
    crate::core::create_run(
        plan,
        wish_model_uri.to_owned(),
        run_record_uri.to_owned(),
        Box::new(RsUnusedCheck),
        Box::new(judgment),
    )
}
