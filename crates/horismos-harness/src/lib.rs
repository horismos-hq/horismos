//! Coding-harness adapter for a bounded, versioned repository projection.
//!
//! This crate is the second-domain proof for `horismos-host`: it materializes all
//! repository inputs before evaluation and composes the candidate-artifact runtime
//! pipeline. It does not run commands or mutate a filesystem; a successful commit is a
//! deterministic authorization handoff to the host. Spec: `docs/framework/03-world.md`,
//! `06-llm-boundaries.md`, `07-tools.md`, and `14-architecture.md` · Owner: PRP-008 U6.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use horismos_host::{
    ArtifactAuthorization, ArtifactIdentity, ArtifactKind, ArtifactPipeline, ArtifactPolicy,
    ArtifactRejection, CandidateArtifact, ClosureMember, ClosureState, CommitReceipt, Effect,
    EffectCommitState, EffectReceipt, EffectVerification, ExternalFact, ExternalSnapshot,
    ExternalValue, HostAdapter, HostError, ProposedEffectSet, Reversibility, Sandbox,
    SourceVersion, SourceVersionVector,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepoCheckStatus {
    Passed,
    Failed,
    NotRun,
}

impl RepoCheckStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NotRun => "not-run",
        }
    }
}

/// Complete recorded repository state for one decision epoch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoState {
    pub branch: String,
    pub head: String,
    pub file_hashes: BTreeMap<String, String>,
    pub dirty_paths: BTreeSet<String>,
    pub test_statuses: BTreeMap<String, RepoCheckStatus>,
    pub path_scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessConfig {
    pub profile: ArtifactIdentity,
    pub mapping: ArtifactIdentity,
    pub package: ArtifactIdentity,
    pub fact_subject: String,
    pub fact_class: String,
    pub branch_property: String,
    pub head_property: String,
    pub dirty_path_property: String,
    pub test_status_property: String,
    pub path_scope_property: String,
    pub allowed_permissions: BTreeSet<String>,
    pub allowed_executables: BTreeSet<String>,
    pub authorizable_kinds: BTreeSet<ArtifactKind>,
}

impl HarnessConfig {
    pub fn prototype() -> Self {
        let namespace = "https://example.org/harness#";
        Self {
            profile: ArtifactIdentity::new(
                "https://example.org/horismos/profile/repository",
                "1.0.0",
                "sha256:repository-profile-1",
            ),
            mapping: ArtifactIdentity::new(
                "https://example.org/horismos/mapping/repository",
                "1.0.0",
                "sha256:repository-mapping-1",
            ),
            package: ArtifactIdentity::new(
                "https://example.org/horismos/package/repository",
                "1.0.0",
                "sha256:repository-package-1",
            ),
            fact_subject: format!("{namespace}projection"),
            fact_class: format!("{namespace}Projection"),
            branch_property: format!("{namespace}branch"),
            head_property: format!("{namespace}head"),
            dirty_path_property: format!("{namespace}dirtyPath"),
            test_status_property: format!("{namespace}testStatus"),
            path_scope_property: format!("{namespace}pathScope"),
            allowed_permissions: BTreeSet::from(["workspace-write".into()]),
            allowed_executables: BTreeSet::from(["cargo".into(), "make".into()]),
            authorizable_kinds: BTreeSet::from([ArtifactKind::Patch, ArtifactKind::File]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessHydrationRequest {
    pub snapshot_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum HarnessCandidateError {
    #[error(transparent)]
    Policy(#[from] ArtifactRejection),
    #[error(transparent)]
    Host(#[from] HostError),
}

impl HarnessCandidateError {
    pub fn domain_text(&self) -> &str {
        match self {
            Self::Policy(error) => error.domain_text(),
            Self::Host(error) => error.domain_text(),
        }
    }
}

pub struct HarnessAdapter {
    state: RepoState,
    config: HarnessConfig,
    authorized: BTreeMap<String, ProposedEffectSet>,
    receipts: BTreeMap<String, CommitReceipt>,
    committed_proposals: Vec<String>,
}

impl HarnessAdapter {
    pub fn new(mut state: RepoState, config: HarnessConfig) -> Result<Self, HostError> {
        validate_state(&state)?;
        state.path_scopes.sort();
        state.path_scopes.dedup();
        Ok(Self {
            state,
            config,
            authorized: BTreeMap::new(),
            receipts: BTreeMap::new(),
            committed_proposals: Vec::new(),
        })
    }

    pub fn authorize_candidate(
        &mut self,
        snapshot: &ExternalSnapshot,
        candidate: &CandidateArtifact,
        sandbox: &impl Sandbox,
    ) -> Result<ProposedEffectSet, HarnessCandidateError> {
        let expected_snapshot = self.snapshot(&HarnessHydrationRequest {
            snapshot_id: snapshot.snapshot_id.clone(),
        })?;
        if snapshot != &expected_snapshot {
            return Err(harness_error(
                "horis:externalSnapshot",
                &snapshot.snapshot_id,
                "candidate authorization requires this adapter's current complete projection",
                "stale",
            )
            .into());
        }
        let proof_ref = format!(
            "urn:horismos:artifact:{}:{}",
            snapshot.snapshot_id, candidate.proposal_id
        );
        if self.authorized.contains_key(&proof_ref) || self.receipts.contains_key(&proof_ref) {
            return Err(harness_error(
                "horis:authorizationGranted",
                &proof_ref,
                "proposal_id must be unique within one repository snapshot",
                "duplicate",
            )
            .into());
        }
        let policy = ArtifactPolicy {
            allowed_path_prefixes: self.state.path_scopes.clone(),
            current_base_hashes: self.state.file_hashes.clone(),
            allowed_permissions: self.config.allowed_permissions.clone(),
            allowed_executables: self.config.allowed_executables.clone(),
            authorizable_kinds: self.config.authorizable_kinds.clone(),
        };
        let authorization = ArtifactPipeline::new(policy).evaluate(candidate, sandbox)?;
        let effect = authorized_effect(candidate, &authorization, &proof_ref)?;
        let effects = ProposedEffectSet {
            proof_ref: proof_ref.clone(),
            source_versions: snapshot.versions.clone(),
            effects: vec![effect],
        };
        self.authorized.insert(proof_ref, effects.clone());
        Ok(effects)
    }

    pub fn committed_proposals(&self) -> &[String] {
        &self.committed_proposals
    }

    /// Fixture-only repository drift injection.
    pub fn set_recorded_file_hash(&mut self, path: &str, hash: &str) {
        if let Some(current) = self.state.file_hashes.get_mut(path) {
            current.clear();
            current.push_str(hash);
        }
    }

    /// Fixture-only repository file appearance used to prove vector cardinality drift.
    pub fn insert_recorded_file_hash(&mut self, path: &str, hash: &str) {
        self.state.file_hashes.insert(path.into(), hash.into());
    }

    fn current_versions(&self) -> Result<SourceVersionVector, HostError> {
        let mut versions = vec![
            SourceVersion {
                resource: "repo:branch".into(),
                version_id: self.state.branch.clone(),
                etag: None,
            },
            SourceVersion {
                resource: "repo:head".into(),
                version_id: self.state.head.clone(),
                etag: None,
            },
            SourceVersion {
                resource: "repo:dirty-paths".into(),
                version_id: digest_value(&self.state.dirty_paths)?,
                etag: None,
            },
            SourceVersion {
                resource: "repo:test-statuses".into(),
                version_id: digest_value(&self.state.test_statuses)?,
                etag: None,
            },
            SourceVersion {
                resource: "repo:path-scopes".into(),
                version_id: digest_value(&self.state.path_scopes)?,
                etag: None,
            },
        ];
        versions.extend(
            self.state
                .file_hashes
                .iter()
                .map(|(path, hash)| SourceVersion {
                    resource: format!("repo:file/{path}"),
                    version_id: hash.clone(),
                    etag: None,
                }),
        );
        SourceVersionVector::new(versions)
    }

    fn snapshot(&self, request: &HarnessHydrationRequest) -> Result<ExternalSnapshot, HostError> {
        if request.snapshot_id.is_empty() {
            return Err(harness_error(
                "horis:externalSnapshot",
                "repository projection",
                "snapshot_id is required",
                "rejected",
            ));
        }
        let properties = BTreeMap::from([
            (
                self.config.branch_property.clone(),
                vec![ExternalValue::String(self.state.branch.clone())],
            ),
            (
                self.config.head_property.clone(),
                vec![ExternalValue::String(self.state.head.clone())],
            ),
            (
                self.config.dirty_path_property.clone(),
                self.state
                    .dirty_paths
                    .iter()
                    .cloned()
                    .map(ExternalValue::String)
                    .collect(),
            ),
            (
                self.config.test_status_property.clone(),
                self.state
                    .test_statuses
                    .iter()
                    .map(|(name, status)| {
                        ExternalValue::String(format!("{name}={}", status.as_str()))
                    })
                    .collect(),
            ),
            (
                self.config.path_scope_property.clone(),
                self.state
                    .path_scopes
                    .iter()
                    .cloned()
                    .map(ExternalValue::String)
                    .collect(),
            ),
        ]);
        Ok(ExternalSnapshot {
            snapshot_id: request.snapshot_id.clone(),
            closure: [
                "branch",
                "head",
                "dirty-paths",
                "test-statuses",
                "path-scopes",
            ]
            .into_iter()
            .map(|name| ClosureMember {
                key: format!("repo:{name}"),
                required: true,
                state: ClosureState::Present,
            })
            .collect(),
            versions: self.current_versions()?,
            profile: self.config.profile.clone(),
            mapping: self.config.mapping.clone(),
            terminology: vec![],
            package: self.config.package.clone(),
            supported_subset: vec![
                "branch/head identity".into(),
                "relative file hashes".into(),
                "dirty paths".into(),
                "typed test status".into(),
                "authorized path scopes".into(),
            ],
            facts: vec![ExternalFact {
                source: "recorded coding harness state".into(),
                subject: self.config.fact_subject.clone(),
                class: self.config.fact_class.clone(),
                properties,
            }],
            losses: vec![],
        })
    }

    fn commit_handoff(&mut self, effects: &ProposedEffectSet) -> Result<CommitReceipt, HostError> {
        if self.authorized.get(&effects.proof_ref) != Some(effects) {
            return Err(harness_error(
                "horis:authorizationGranted",
                &effects.proof_ref,
                "effect set was not produced by this adapter's candidate pipeline",
                "rejected",
            ));
        }
        if let Some(receipt) = self.receipts.get(&effects.proof_ref) {
            return Ok(receipt.clone());
        }
        let current = self.current_versions()?;
        if let Some((resource, hydrated, now)) = version_drift(&effects.source_versions, &current) {
            return Err(HostError::boundary(
                format!(
                    "horis:sourceVersion drift on {}: hydrated {}, current {}; commit aborted",
                    resource, hydrated, now
                ),
                "horis:sourceVersion",
                resource,
                Some("conflicted".into()),
                "repository changed after hydration",
            ));
        }
        let receipts = effects
            .effects
            .iter()
            .map(|effect| EffectReceipt {
                effect_id: effect.id.clone(),
                target: effect.target.clone(),
                state: EffectCommitState::Accepted,
                evidence: Some("candidate authorized for host-controlled application".into()),
            })
            .collect();
        let receipt = CommitReceipt {
            receipt_id: format!("harness-handoff:{}", effects.proof_ref),
            source_versions: current,
            effects: receipts,
        };
        self.committed_proposals
            .extend(effects.effects.iter().map(|effect| effect.id.clone()));
        self.receipts
            .insert(effects.proof_ref.clone(), receipt.clone());
        Ok(receipt)
    }
}

impl HostAdapter for HarnessAdapter {
    type HydrationRequest = HarnessHydrationRequest;

    fn hydrate(&self, request: &Self::HydrationRequest) -> Result<ExternalSnapshot, HostError> {
        self.snapshot(request)
    }

    fn commit(&mut self, effects: &ProposedEffectSet) -> Result<CommitReceipt, HostError> {
        self.commit_handoff(effects)
    }
}

fn authorized_effect(
    candidate: &CandidateArtifact,
    authorization: &ArtifactAuthorization,
    proof_ref: &str,
) -> Result<Effect, HostError> {
    let bytes = serde_json::to_vec(candidate).map_err(|error| {
        harness_error(
            "horis:candidateArtifact",
            &candidate.proposal_id,
            &format!("candidate serialization failed: {error}"),
            "rejected",
        )
    })?;
    let digest = format!("sha256:{:x}", Sha256::digest(bytes));
    Ok(Effect {
        id: candidate.proposal_id.clone(),
        target: format!("repo:file/{}", candidate.target_path),
        reversibility: match candidate.kind {
            ArtifactKind::Patch | ArtifactKind::File => Reversibility::Reversible,
            ArtifactKind::Command => Reversibility::Irreversible,
        },
        idempotency_key: proof_ref.into(),
        payload: serde_json::json!({
            "candidate": candidate,
            "authorization": authorization,
            "candidateDigest": digest,
        }),
        verification: EffectVerification {
            postcondition: Some("host verifies applied content and declared checks".into()),
            verify_before_retry: true,
        },
    })
}

fn digest_value(value: &impl Serialize) -> Result<String, HostError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        harness_error(
            "horis:sourceVersion",
            "repository projection",
            &format!("version digest serialization failed: {error}"),
            "rejected",
        )
    })?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn version_drift(
    expected: &SourceVersionVector,
    current: &SourceVersionVector,
) -> Option<(String, String, String)> {
    for hydrated in expected.entries() {
        match current
            .entries()
            .iter()
            .find(|now| now.resource == hydrated.resource)
        {
            Some(now) if now.version_id == hydrated.version_id => {}
            Some(now) => {
                return Some((
                    hydrated.resource.clone(),
                    hydrated.version_id.clone(),
                    now.version_id.clone(),
                ));
            }
            None => {
                return Some((
                    hydrated.resource.clone(),
                    hydrated.version_id.clone(),
                    "missing".into(),
                ));
            }
        }
    }
    if let Some(extra) = current.entries().iter().find(|now| {
        !expected
            .entries()
            .iter()
            .any(|expected| expected.resource == now.resource)
    }) {
        return Some((
            extra.resource.clone(),
            "missing".into(),
            extra.version_id.clone(),
        ));
    }
    None
}

fn validate_state(state: &RepoState) -> Result<(), HostError> {
    if state.branch.is_empty() || state.head.is_empty() {
        return Err(harness_error(
            "horis:inputClosure",
            "repository identity",
            "branch and head are required",
            "incomplete",
        ));
    }
    if state.file_hashes.is_empty()
        || state.test_statuses.is_empty()
        || state.path_scopes.is_empty()
    {
        return Err(harness_error(
            "horis:inputClosure",
            "repository projection",
            "file hashes, test statuses, and path scopes are required",
            "incomplete",
        ));
    }
    for path in state
        .file_hashes
        .keys()
        .chain(&state.dirty_paths)
        .chain(&state.path_scopes)
    {
        if !normal_relative_path(path) {
            return Err(harness_error(
                "horis:inputClosure",
                path,
                "repository paths must be relative normal paths",
                "rejected",
            ));
        }
    }
    if let Some(path) = state
        .dirty_paths
        .iter()
        .find(|path| !state.file_hashes.contains_key(*path))
    {
        return Err(harness_error(
            "horis:inputClosure",
            path,
            "dirty path requires a materialized file hash",
            "incomplete",
        ));
    }
    if let Some((path, _)) = state.file_hashes.iter().find(|(_, hash)| hash.is_empty()) {
        return Err(harness_error(
            "horis:sourceVersion",
            path,
            "file hash is required",
            "incomplete",
        ));
    }
    Ok(())
}

fn normal_relative_path(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn harness_error(constraint: &str, subject: &str, detail: &str, state: &str) -> HostError {
    HostError::boundary(
        format!("{constraint} rejected {subject}: {detail}"),
        constraint,
        subject,
        Some(state.into()),
        detail,
    )
}
