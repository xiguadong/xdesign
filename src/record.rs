use crate::{
    plan::{sha256_bytes, valid_digest, valid_relative_path, RsFlowPlan},
    status::{RsCheckpointOutcome, RsGateError, RsJudgeAuthority, RsStageStatus},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::Path,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsArtifactDigest {
    pub uri: String,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsActivityEntry {
    pub description: String,
    pub performed_by: String,
    pub occurred_at: String,
    pub artifact_uris: Vec<String>,
    pub artifacts_changed: bool,
    pub artifact_digests: Vec<RsArtifactDigest>,
    pub removed_artifact_uris: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsAssessmentEntry {
    pub checkpoint_id: String,
    pub evaluated_revision: u32,
    pub evaluated_snapshot_digest: String,
    pub outcome: RsCheckpointOutcome,
    pub judged_by: RsJudgeAuthority,
    pub assessor: String,
    pub rationale: String,
    pub evidence_uris: Vec<String>,
}

impl RsAssessmentEntry {
    pub fn unevaluated(
        id: String,
        revision: u32,
        digest: String,
        error: RsGateError,
        authority: RsJudgeAuthority,
    ) -> Self {
        Self {
            checkpoint_id: id,
            evaluated_revision: revision,
            evaluated_snapshot_digest: digest,
            outcome: RsCheckpointOutcome::Unevaluated,
            judged_by: authority,
            assessor: "unavailable".into(),
            rationale: format!("{error:?}"),
            evidence_uris: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RsStageVisit {
    stage_id: String,
    visit_index: u32,
    pub(crate) status: RsStageStatus,
    pub(crate) artifact_revision: u32,
    pub(crate) artifact_snapshot_digest: String,
    pub(crate) artifact_digests: Vec<RsArtifactDigest>,
    activities: Vec<RsActivityEntry>,
    assessments: Vec<RsAssessmentEntry>,
}

/// Canonical snapshot: sorted URI, NUL, lowercase SHA-256, LF.
pub fn snapshot_digest(artifacts: &[RsArtifactDigest]) -> String {
    let mut entries: Vec<_> = artifacts.iter().collect();
    entries.sort_by(|a, b| a.uri.cmp(&b.uri));
    let mut bytes = Vec::new();
    for artifact in entries {
        bytes.extend_from_slice(artifact.uri.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(artifact.sha256.as_bytes());
        bytes.push(b'\n');
    }
    sha256_bytes(&bytes)
}

pub fn hash_artifact(uri: &str) -> Result<RsArtifactDigest, String> {
    if !valid_relative_path(uri) {
        return Err(format!("invalid artifact URI: {uri}"));
    }
    let bytes = fs::read(uri).map_err(|e| format!("unreadable artifact {uri}: {e}"))?;
    Ok(RsArtifactDigest {
        uri: uri.to_owned(),
        sha256: sha256_bytes(&bytes),
    })
}

impl RsStageVisit {
    pub(crate) fn pending(stage_id: String, visit_index: u32) -> Self {
        Self {
            stage_id,
            visit_index,
            status: RsStageStatus::Pending,
            artifact_revision: 0,
            artifact_snapshot_digest: sha256_bytes(&[]),
            artifact_digests: vec![],
            activities: vec![],
            assessments: vec![],
        }
    }
    pub fn stage_id(&self) -> &str {
        &self.stage_id
    }
    pub fn visit_index(&self) -> u32 {
        self.visit_index
    }
    pub fn status(&self) -> RsStageStatus {
        self.status
    }
    pub fn artifact_revision(&self) -> u32 {
        self.artifact_revision
    }
    pub fn artifact_snapshot_digest(&self) -> &str {
        &self.artifact_snapshot_digest
    }
    pub fn artifact_digests(&self) -> &[RsArtifactDigest] {
        &self.artifact_digests
    }
    pub fn activities(&self) -> &[RsActivityEntry] {
        &self.activities
    }
    pub fn assessments(&self) -> &[RsAssessmentEntry] {
        &self.assessments
    }
    pub fn evidence_uris(&self) -> Vec<String> {
        let mut uris: Vec<String> = self
            .artifact_digests
            .iter()
            .map(|a| a.uri.clone())
            .collect();
        for assessment in &self.assessments {
            if assessment.evaluated_revision == self.artifact_revision
                && assessment.evaluated_snapshot_digest == self.artifact_snapshot_digest
            {
                uris.extend(assessment.evidence_uris.iter().cloned());
            }
        }
        uris.sort();
        uris.dedup();
        uris
    }
    pub fn verify_artifacts(&self) -> bool {
        if snapshot_digest(&self.artifact_digests) != self.artifact_snapshot_digest {
            return false;
        }
        let mut uris = HashSet::new();
        self.artifact_digests.iter().all(|a| {
            uris.insert(&a.uri)
                && valid_digest(&a.sha256)
                && hash_artifact(&a.uri)
                    .map(|actual| actual.sha256 == a.sha256)
                    .unwrap_or(false)
        })
    }
    pub(crate) fn verify_passed_evidence(&self) -> bool {
        self.assessments
            .iter()
            .filter(|assessment| assessment.outcome == RsCheckpointOutcome::Passed)
            .all(|assessment| {
                !assessment.evidence_uris.is_empty()
                    && assessment
                        .evidence_uris
                        .iter()
                        .all(|uri| valid_relative_path(uri) && Path::new(uri).is_file())
            })
    }
    pub(crate) fn record_activity(&mut self, activity: RsActivityEntry) -> bool {
        let changed = activity.artifacts_changed;
        if changed {
            let mut map: BTreeMap<String, String> = self
                .artifact_digests
                .iter()
                .map(|a| (a.uri.clone(), a.sha256.clone()))
                .collect();
            for uri in &activity.removed_artifact_uris {
                map.remove(uri);
            }
            for digest in &activity.artifact_digests {
                map.insert(digest.uri.clone(), digest.sha256.clone());
            }
            self.artifact_digests = map
                .into_iter()
                .map(|(uri, sha256)| RsArtifactDigest { uri, sha256 })
                .collect();
            self.artifact_snapshot_digest = snapshot_digest(&self.artifact_digests);
            self.artifact_revision += 1;
        }
        self.activities.push(activity);
        changed
    }
    pub(crate) fn append_assessment(&mut self, assessment: RsAssessmentEntry) {
        debug_assert_eq!(assessment.evaluated_revision, self.artifact_revision);
        debug_assert_eq!(
            assessment.evaluated_snapshot_digest,
            self.artifact_snapshot_digest
        );
        self.assessments.push(assessment);
    }
    pub fn completion_authorized(&self, plan: &RsFlowPlan) -> bool {
        let required = plan.checkpoints_of(&self.stage_id);
        if required.is_empty() {
            return false;
        }
        let latest: HashMap<&str, &RsAssessmentEntry> = self
            .assessments
            .iter()
            .map(|a| (a.checkpoint_id.as_str(), a))
            .collect();
        required.iter().all(|gate| {
            latest.get(gate.checkpoint_id.as_str()).is_some_and(|a| {
                a.outcome == RsCheckpointOutcome::Passed
                    && a.evaluated_revision == self.artifact_revision
                    && a.evaluated_snapshot_digest == self.artifact_snapshot_digest
            })
        })
    }
}
