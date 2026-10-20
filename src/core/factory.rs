use super::{
    plan::{valid_relative_path, RsFlowPlan},
    ports::{RsCheckCommand, RsJudge},
    record::RsStageVisit,
    run::RsFlowRun,
    status::{RsFlowStatus, RsPlanError, RsStageStatus},
};
use std::path::Path;
use uuid::Uuid;

pub fn create_run(
    plan: RsFlowPlan,
    wish_model_uri: String,
    run_record_uri: String,
    check: Box<dyn RsCheckCommand>,
    judgment: Box<dyn RsJudge>,
) -> Result<RsFlowRun, RsPlanError> {
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
    Ok(RsFlowRun {
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
