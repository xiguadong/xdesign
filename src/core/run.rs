use super::{
    gate::evaluate_checkpoint,
    plan::{valid_relative_path, RsFlowPlan},
    ports::{RsCheckCommand, RsJudge},
    record::{RsActivityEntry, RsAssessmentEntry, RsStageVisit},
    status::{
        RsAdvanceOutcome, RsCheckKind, RsFlowStatus, RsGateError, RsJudgeAuthority, RsStageStatus,
    },
};
use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};

pub struct RsFlowRun {
    pub(crate) run_id: String,
    pub(crate) plan_id: String,
    pub(crate) plan_uri: String,
    pub(crate) plan_sha256: String,
    pub(crate) wish_model_uri: String,
    pub(crate) run_record_uri: String,
    pub(crate) plan: RsFlowPlan,
    pub(crate) status: RsFlowStatus,
    pub(crate) visits: Vec<RsStageVisit>,
    pub(crate) current_visit: usize,
    pub(crate) check: Box<dyn RsCheckCommand>,
    pub(crate) judgment: Box<dyn RsJudge>,
    pub(crate) persistence_error: bool,
    pub(crate) last_error: Option<String>,
    pub(crate) artifact_snapshot_directory: Option<String>,
}

#[derive(Serialize)]
struct RunRecord<'a> {
    run_id: &'a str,
    execution_type: &'static str,
    plan_id: &'a str,
    plan_uri: &'a str,
    plan_sha256: &'a str,
    wish_model_uri: &'a str,
    status: RsFlowStatus,
    current_visit: usize,
    visits: &'a [RsStageVisit],
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_snapshot_directory: &'a Option<String>,
}

impl RsFlowRun {
    /// Opt in before any work. Legacy records keep workspace-bound verification.
    pub fn retain_completed_artifacts(&mut self, directory: String) -> Result<(), String> {
        if !valid_relative_path(&directory)
            || self.visits.iter().any(|v| !v.activities().is_empty())
        {
            return Err("artifact retention must be configured before recording work".into());
        }
        self.artifact_snapshot_directory = Some(directory);
        Ok(())
    }
    pub fn artifact_snapshot_directory(&self) -> Option<&str> {
        self.artifact_snapshot_directory.as_deref()
    }
    fn retain_current_artifacts(&mut self) -> Result<(), String> {
        let Some(directory) = &self.artifact_snapshot_directory else {
            return Ok(());
        };
        // No links in the retention namespace, including existing ancestors.
        let mut prefix = std::path::PathBuf::new();
        for part in Path::new(directory).components() {
            prefix.push(part);
            match fs::symlink_metadata(&prefix) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                    return Err(format!(
                        "unsafe artifact retention directory: {}",
                        prefix.display()
                    ))
                }
                Ok(_) => (),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&prefix).map_err(|e| e.to_string())?
                }
                Err(error) => return Err(error.to_string()),
            }
        }
        let mut snapshots = std::collections::BTreeMap::new();
        for artifact in self.current().artifact_digests() {
            let bytes = fs::read(&artifact.uri).map_err(|e| e.to_string())?;
            if super::plan::sha256_bytes(&bytes) != artifact.sha256 {
                return Err(format!(
                    "artifact drifted before retention: {}",
                    artifact.uri
                ));
            }
            let uri = format!("{directory}/{}", artifact.sha256);
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&uri)
            {
                Ok(mut file) => {
                    use std::io::Write;
                    file.write_all(&bytes).map_err(|e| e.to_string())?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let metadata = fs::symlink_metadata(&uri).map_err(|e| e.to_string())?;
                    if !metadata.is_file()
                        || metadata.file_type().is_symlink()
                        || fs::read(&uri).map_err(|e| e.to_string())? != bytes
                    {
                        return Err(format!(
                            "retained artifact conflicts with assessed bytes: {uri}"
                        ));
                    }
                }
                Err(error) => return Err(error.to_string()),
            }
            snapshots.insert(artifact.uri.clone(), uri);
        }
        self.visits[self.current_visit].artifact_snapshots = snapshots;
        Ok(())
    }
    pub fn status(&self) -> RsFlowStatus {
        self.status
    }
    pub fn current(&self) -> &RsStageVisit {
        &self.visits[self.current_visit]
    }
    pub fn visits(&self) -> &[RsStageVisit] {
        &self.visits
    }
    pub fn plan(&self) -> &RsFlowPlan {
        &self.plan
    }
    pub fn run_id(&self) -> &str {
        &self.run_id
    }
    pub fn run_record_uri(&self) -> &str {
        &self.run_record_uri
    }
    pub fn current_visit_index(&self) -> usize {
        self.current_visit
    }
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Writes the re-readable JSON snapshot used as each checker invocation's input.
    pub fn write_record(&self) -> Result<(), String> {
        if !valid_relative_path(&self.run_record_uri) {
            return Err("invalid record URI".into());
        }
        let record = RunRecord {
            run_id: &self.run_id,
            execution_type: "FlowRun",
            plan_id: &self.plan_id,
            plan_uri: &self.plan_uri,
            plan_sha256: &self.plan_sha256,
            wish_model_uri: &self.wish_model_uri,
            status: self.status,
            current_visit: self.current_visit,
            visits: &self.visits,
            artifact_snapshot_directory: &self.artifact_snapshot_directory,
        };
        let bytes = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
        if let Some(parent) = Path::new(&self.run_record_uri).parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temporary = format!("{}.{}.tmp", self.run_record_uri, uuid::Uuid::new_v4());
        fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
        fs::rename(&temporary, &self.run_record_uri).map_err(|e| e.to_string())
    }

    fn persist(&mut self) -> bool {
        match self.write_record() {
            Ok(()) => {
                self.persistence_error = false;
                self.last_error = None;
                true
            }
            Err(error) => {
                self.persistence_error = true;
                self.last_error = Some(error);
                false
            }
        }
    }

    pub fn record_activity(&mut self, activity: RsActivityEntry) -> Result<(), String> {
        if self.status == RsFlowStatus::Completed {
            return Err("completed run must be reopened first".into());
        }
        if activity.description.trim().is_empty()
            || activity.performed_by.trim().is_empty()
            || activity.occurred_at.trim().is_empty()
        {
            return Err("activity description, actor, and time are required".into());
        }
        if !activity.artifacts_changed
            && (!activity.artifact_digests.is_empty() || !activity.removed_artifact_uris.is_empty())
            || activity.artifacts_changed
                && activity.artifact_digests.is_empty()
                && activity.removed_artifact_uris.is_empty()
        {
            return Err(
                "artifact change declaration disagrees with its digest/removal list".into(),
            );
        }
        let mut updated = HashSet::new();
        for digest in &activity.artifact_digests {
            if !updated.insert(digest.uri.as_str())
                || !valid_relative_path(&digest.uri)
                || super::record::hash_artifact(&digest.uri)
                    .map(|d| d.sha256 != digest.sha256)
                    .unwrap_or(true)
            {
                return Err(format!("invalid or stale digest for {}", digest.uri));
            }
        }
        let prior: HashSet<_> = self
            .current()
            .artifact_digests()
            .iter()
            .map(|d| d.uri.as_str())
            .collect();
        let mut removed = HashSet::new();
        for uri in &activity.removed_artifact_uris {
            if !removed.insert(uri.as_str())
                || !prior.contains(uri.as_str())
                || updated.contains(uri.as_str())
            {
                return Err(format!("invalid artifact removal: {uri}"));
            }
        }
        let mut named = HashSet::new();
        for uri in &activity.artifact_uris {
            if !valid_relative_path(uri) || !named.insert(uri.as_str()) {
                return Err(format!("invalid or duplicate activity artifact URI: {uri}"));
            }
        }
        if activity.artifacts_changed {
            let changed: HashSet<_> = updated.union(&removed).copied().collect();
            if named != changed {
                return Err(
                    "activity artifact URIs must match updated and removed artifacts".into(),
                );
            }
        } else if activity
            .artifact_uris
            .iter()
            .any(|uri| !Path::new(uri).is_file())
        {
            return Err("unchanged activity names a missing artifact".into());
        }
        if activity.artifacts_changed && self.current().artifact_revision() == u32::MAX {
            return Err("artifact revision exhausted".into());
        }
        self.visits[self.current_visit].record_activity(activity);
        if self.status == RsFlowStatus::Blocked {
            self.status = RsFlowStatus::Active;
            self.visits[self.current_visit].status = RsStageStatus::Active;
        }
        if self.persist() {
            Ok(())
        } else {
            Err(self
                .last_error
                .clone()
                .unwrap_or_else(|| "record persistence failed".into()))
        }
    }

    pub fn evaluate_gates(&mut self) -> Vec<RsAssessmentEntry> {
        if self.status == RsFlowStatus::Completed {
            return vec![];
        }
        let gates: Vec<_> = self
            .plan
            .checkpoints_of(self.current().stage_id())
            .into_iter()
            .cloned()
            .collect();
        let mut out = vec![];
        for spec in gates {
            // Refresh the snapshot after each gate so the semantic judge can cite the
            // record check report appended earlier in this same verify-stage loop.
            let visit = self.current().clone();
            let authority = if spec.kind == RsCheckKind::Syntactic {
                RsJudgeAuthority::Script
            } else {
                self.judgment.authority()
            };
            let result = if self.plan.validate().is_err() || !self.persist() {
                Err(RsGateError::CommandFailed)
            } else {
                evaluate_checkpoint(
                    &spec,
                    &visit,
                    &self.plan_id,
                    &self.plan_uri,
                    &self.plan_sha256,
                    &self.wish_model_uri,
                    &self.run_record_uri,
                    &*self.check,
                    &*self.judgment,
                )
            };
            let entry = result.unwrap_or_else(|error| {
                RsAssessmentEntry::unevaluated(
                    spec.checkpoint_id.clone(),
                    visit.artifact_revision(),
                    visit.artifact_snapshot_digest().to_owned(),
                    error,
                    authority,
                )
            });
            self.visits[self.current_visit].append_assessment(entry.clone());
            if self.persist() {
                out.push(entry);
            } else {
                let error = RsAssessmentEntry::unevaluated(
                    spec.checkpoint_id.clone(),
                    visit.artifact_revision(),
                    visit.artifact_snapshot_digest().to_owned(),
                    RsGateError::CommandFailed,
                    authority,
                );
                self.visits[self.current_visit].append_assessment(error.clone());
                out.push(error);
            }
        }
        let authorized = self.current().completion_authorized(&self.plan);
        self.status = if authorized {
            RsFlowStatus::Active
        } else {
            RsFlowStatus::Blocked
        };
        self.visits[self.current_visit].status = if authorized {
            RsStageStatus::Active
        } else {
            RsStageStatus::Blocked
        };
        if !self.persist() {
            self.status = RsFlowStatus::Blocked;
            self.visits[self.current_visit].status = RsStageStatus::Blocked;
            for entry in &mut out {
                let error = RsAssessmentEntry::unevaluated(
                    entry.checkpoint_id.clone(),
                    entry.evaluated_revision,
                    entry.evaluated_snapshot_digest.clone(),
                    RsGateError::CommandFailed,
                    entry.judged_by,
                );
                self.visits[self.current_visit].append_assessment(error.clone());
                *entry = error;
            }
        }
        out
    }

    fn passed_evidence_retained(&self) -> bool {
        self.visits.iter().enumerate().all(|(index, visit)| {
            (index != self.current_visit && visit.status() != RsStageStatus::Completed)
                || visit.verify_passed_evidence()
        })
    }

    fn visited_artifacts_match(&self) -> bool {
        self.visits.iter().enumerate().all(|(index, visit)| {
            (index != self.current_visit && visit.status() != RsStageStatus::Completed)
                || visit.verify_artifacts()
        })
    }

    fn remain_blocked(&mut self) -> RsAdvanceOutcome {
        self.status = RsFlowStatus::Blocked;
        self.visits[self.current_visit].artifact_snapshots.clear();
        self.visits[self.current_visit].status = RsStageStatus::Blocked;
        self.persist();
        RsAdvanceOutcome::RemainBlocked
    }

    pub fn advance(&mut self) -> RsAdvanceOutcome {
        if self.status == RsFlowStatus::Completed {
            if self.plan.validate().is_err()
                || !self.visited_artifacts_match()
                || !self.current().completion_authorized(&self.plan)
                || !self.passed_evidence_retained()
            {
                return self.remain_blocked();
            }
            return RsAdvanceOutcome::CompleteRun;
        }
        if self.persistence_error && !self.persist() {
            return self.remain_blocked();
        }
        if self.plan.validate().is_err()
            || !self.visited_artifacts_match()
            || !self.current().completion_authorized(&self.plan)
            || !self.passed_evidence_retained()
        {
            return self.remain_blocked();
        }
        let old_visits = self.visits.clone();
        let old_index = self.current_visit;
        let old_status = self.status;
        if let Err(error) = self.retain_current_artifacts() {
            let outcome = self.remain_blocked();
            self.last_error = Some(error);
            return outcome;
        }
        let stage = self.current().stage_id().to_owned();
        if self.plan.stage(&stage).is_some_and(|s| s.completes_flow) {
            self.status = RsFlowStatus::Completed;
            self.visits[self.current_visit].status = RsStageStatus::Completed;
            if self.persist() {
                return RsAdvanceOutcome::CompleteRun;
            }
            self.visits = old_visits;
            self.current_visit = old_index;
            self.status = old_status;
            return self.remain_blocked();
        }
        let Some(next_stage) = self.plan.next_stage(&stage).map(|s| s.stage_id.clone()) else {
            return self.remain_blocked();
        };
        let Some(index) = self
            .visits
            .iter()
            .position(|v| v.stage_id() == next_stage && v.status() == RsStageStatus::Pending)
        else {
            return self.remain_blocked();
        };
        self.visits[self.current_visit].status = RsStageStatus::Completed;
        self.current_visit = index;
        self.visits[index].status = RsStageStatus::Active;
        self.status = RsFlowStatus::Active;
        if self.persist() {
            RsAdvanceOutcome::EnterNextStage
        } else {
            self.visits = old_visits;
            self.current_visit = old_index;
            self.status = old_status;
            self.remain_blocked()
        }
    }

    pub fn reopen_stage(&mut self, stage_id: &str) -> bool {
        if self.status != RsFlowStatus::Completed
            || self.plan.stage(stage_id).is_none_or(|s| !s.completes_flow)
        {
            return false;
        }
        let Some(previous) = self.visits.iter().rev().find(|v| v.stage_id() == stage_id) else {
            return false;
        };
        let mut new_visit = RsStageVisit::pending(stage_id.to_owned(), previous.visit_index() + 1);
        new_visit.status = RsStageStatus::Active;
        self.visits.push(new_visit);
        self.current_visit = self.visits.len() - 1;
        self.status = RsFlowStatus::Active;
        if self.persist() {
            true
        } else {
            self.visits.pop();
            self.current_visit -= 1;
            self.status = RsFlowStatus::Completed;
            false
        }
    }
}
