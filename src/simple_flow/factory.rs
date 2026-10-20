use super::blueprint::validate_designed_plan;
use crate::core::{RsCheckCommand, RsFlowPlan, RsFlowRun, RsJudge, RsPlanError};

/// Blueprint conformance, then generic construction of the designed run.
pub fn create_designed_run(
    plan: RsFlowPlan,
    wish_model_uri: String,
    run_record_uri: String,
    check: Box<dyn RsCheckCommand>,
    judgment: Box<dyn RsJudge>,
) -> Result<RsFlowRun, RsPlanError> {
    validate_designed_plan(&plan)?;
    crate::core::create_run(plan, wish_model_uri, run_record_uri, check, judgment)
}
