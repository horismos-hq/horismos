//! Adapter-layer trace envelope for source and remote-commit evidence.
//!
//! This embeds an unchanged core trace rather than changing the meaning of
//! `horismos.trace/1` or `/2`.
//!
//! Spec: `docs/framework/09-traces.md` · Owner: PRP-008.

use horismos_core::trace::Trace;
use serde::{Deserialize, Serialize};

use crate::{
    ArtifactIdentity, AssumptionManifest, CommitReceipt, ProposedEffectSet, SourceVersionVector,
    TerminologyPackage,
};

pub const INTEGRATION_TRACE_V1: &str = "horismos.integration-trace/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommitState {
    DecisionProduced,
    CommitAttempted,
    CommitAccepted,
    CommitConflicted,
    CommitFailed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitEvent {
    pub state: CommitState,
    pub receipt: Option<CommitReceipt>,
    pub domain_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationTrace {
    pub schema: String,
    pub run_id: String,
    pub snapshot_id: String,
    pub versions: SourceVersionVector,
    pub profile: ArtifactIdentity,
    pub mapping: ArtifactIdentity,
    pub package: ArtifactIdentity,
    pub terminology: Vec<TerminologyPackage>,
    pub assumptions: AssumptionManifest,
    pub proof_ref: String,
    pub effects: ProposedEffectSet,
    pub core: Trace,
    pub commits: Vec<CommitEvent>,
}

impl IntegrationTrace {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("IntegrationTrace contains serializable data")
    }
}
