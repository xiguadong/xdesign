use crate::core::{RsCheckKind, RsCheckpointSpec, RsFlowPlan, RsPlanError, RsStageSpec};

/// The modeled designedWishVerifyFlow as plan data, before `freeze` binds its bytes.
pub fn wish_verify_flow_plan() -> RsFlowPlan {
    fn gate(id: &str, stage: &str, criteria: &[&str]) -> RsCheckpointSpec {
        RsCheckpointSpec {
            checkpoint_id: id.into(),
            stage_id: stage.into(),
            kind: RsCheckKind::Semantic,
            criteria: criteria.iter().map(|x| x.to_string()).collect(),
            checker: None,
            agent_may_judge: true,
            human_may_judge: true,
        }
    }
    RsFlowPlan {
        plan_id: "designedWishVerifyFlow".into(), plan_uri: String::new(), plan_sha256: String::new(),
        initial_stage_id: "wish".into(),
        stages: vec![
            RsStageSpec { stage_id: "wish".into(), next_stage_ids: vec!["verify".into()], exit_checkpoint_ids: vec!["wishAccepted".into()], completes_flow: false },
            RsStageSpec { stage_id: "verify".into(), next_stage_ids: vec![], exit_checkpoint_ids: vec!["outcomeAccepted".into()], completes_flow: true },
        ],
        checkpoints: vec![
            gate("wishAccepted", "wish", &[
                "The wish statement is recorded, and the artifacts that answer it are named in the current visit.",
                "A reader can see from the recorded artifacts what the wish asked and what was produced for it, without re-asking the requester.",
            ]),
            gate("outcomeAccepted", "verify", &[
                "The delivered artifacts let the requester see the outcome from them, without reading the model or re-asking.",
                "Diagrams carry the explanation where a diagram is clearer than prose, no statement goes beyond what the evidence supports, and nothing is added that the requester did not ask for.",
            ]),
        ],
    }
}

/// Blueprint conformance: reject any plan that differs from the designed WishVerifyFlow.
pub fn validate_wish_verify_plan(plan: &RsFlowPlan) -> Result<(), RsPlanError> {
    let expected_stages = ["wish", "verify"];
    let expected_gates: [&[&str]; 2] = [&["wishAccepted"], &["outcomeAccepted"]];
    if plan.plan_id != "designedWishVerifyFlow" {
        return Err(RsPlanError::InvalidPlanFile);
    }
    if plan.initial_stage_id != expected_stages[0]
        || plan.stages.len() != 2
        || plan.checkpoints.len() != 2
    {
        return Err(RsPlanError::InvalidDesignedFlow);
    }
    for (i, stage) in plan.stages.iter().enumerate() {
        let next: Vec<String> = if i == 1 {
            vec![]
        } else {
            vec![expected_stages[i + 1].to_owned()]
        };
        if stage.stage_id != expected_stages[i]
            || stage.next_stage_ids != next
            || stage.completes_flow != (i == 1)
            || stage.exit_checkpoint_ids
                != expected_gates[i]
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
        {
            return Err(RsPlanError::InvalidDesignedFlow);
        }
    }
    for (i, gate) in plan.checkpoints.iter().enumerate() {
        if gate.checkpoint_id != expected_gates[i][0]
            || gate.stage_id != expected_stages[i]
            || gate.kind != RsCheckKind::Semantic
            || !gate.agent_may_judge
            || !gate.human_may_judge
        {
            return Err(RsPlanError::InvalidDesignedFlow);
        }
        if gate.checker.is_some() {
            return Err(RsPlanError::InvalidChecker);
        }
    }
    let blueprint = wish_verify_flow_plan();
    if plan.stages != blueprint.stages || plan.checkpoints != blueprint.checkpoints {
        return Err(RsPlanError::InvalidDesignedFlow);
    }
    Ok(())
}
