//! Runtime half of the candidate-artifact boundary.
//!
//! Generated content remains opaque. This module proves only deterministic
//! policy legality, declared sandbox-check outcomes, and symbolic authorization;
//! it never claims semantic correctness.
//!
//! Spec: `docs/framework/06-llm-boundaries.md` and `07-tools.md` · Owner:
//! PRP-008 (runtime evidence only; PRP-007 owns authoring semantics).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CANDIDATE_POLICY_EVIDENCE: &str = "https://w3id.org/horismos#CandidatePolicyEvidence";
pub const SANDBOX_EVIDENCE: &str = "https://w3id.org/horismos#SandboxEvidence";
pub const AUTHORIZATION_EVIDENCE: &str = "https://w3id.org/horismos#AuthorizationEvidence";
pub const POLICY_ACCEPTED: &str = "https://w3id.org/horismos#policyAccepted";
pub const SANDBOX_PASSED: &str = "https://w3id.org/horismos#sandboxPassed";
pub const AUTHORIZATION_GRANTED: &str = "https://w3id.org/horismos#authorizationGranted";
pub const EVIDENCE_FOR: &str = "https://w3id.org/horismos#evidenceFor";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    Patch,
    File,
    Command,
}

/// The runtime treats each content variant as bytes/arguments, never meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum ArtifactContent {
    Patch(String),
    File(String),
    Command { program: String, args: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredCheck {
    pub id: String,
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateArtifact {
    pub proposal_id: String,
    pub kind: ArtifactKind,
    pub target_path: String,
    pub base_hash: String,
    pub content: ArtifactContent,
    pub required_permissions: BTreeSet<String>,
    pub declared_checks: Vec<DeclaredCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPolicy {
    pub allowed_path_prefixes: Vec<String>,
    pub current_base_hashes: BTreeMap<String, String>,
    pub allowed_permissions: BTreeSet<String>,
    pub allowed_executables: BTreeSet<String>,
    pub authorizable_kinds: BTreeSet<ArtifactKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,
    pub status: CheckStatus,
    pub evidence_hash: String,
}

/// Execution remains deployer-owned; the pipeline consumes typed results only.
pub trait Sandbox {
    fn run(&self, checks: &[DeclaredCheck]) -> Vec<CheckResult>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RejectionStage {
    TargetPath,
    BaseHash,
    Syntax,
    Permission,
    Sandbox,
    Authorization,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{text}")]
pub struct ArtifactRejection {
    pub proposal_id: String,
    pub stage: RejectionStage,
    pub constraint: String,
    text: String,
}

impl ArtifactRejection {
    fn new(
        artifact: &CandidateArtifact,
        stage: RejectionStage,
        constraint: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            proposal_id: artifact.proposal_id.clone(),
            stage,
            constraint: constraint.into(),
            text: text.into(),
        }
    }

    pub fn domain_text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PolicyEvidence {
    evidence_type: String,
    evidence_for: String,
    policy_accepted: bool,
    target_path: String,
    base_hash: String,
    permissions: Vec<String>,
}

impl PolicyEvidence {
    pub fn accepted(&self) -> bool {
        self.policy_accepted
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SandboxEvidence {
    evidence_type: String,
    evidence_for: String,
    sandbox_passed: bool,
    checks: Vec<CheckResult>,
}

impl SandboxEvidence {
    pub fn passed(&self) -> bool {
        self.sandbox_passed
    }

    pub fn checks(&self) -> &[CheckResult] {
        &self.checks
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthorizationEvidence {
    evidence_type: String,
    evidence_for: String,
    authorization_granted: bool,
    artifact_kind: ArtifactKind,
}

impl AuthorizationEvidence {
    pub fn granted(&self) -> bool {
        self.authorization_granted
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactAuthorization {
    proposal_id: String,
    claim: String,
    policy: PolicyEvidence,
    sandbox: SandboxEvidence,
    authorization: AuthorizationEvidence,
}

impl ArtifactAuthorization {
    pub fn proposal_id(&self) -> &str {
        &self.proposal_id
    }

    pub fn claim(&self) -> &str {
        &self.claim
    }

    pub fn policy(&self) -> &PolicyEvidence {
        &self.policy
    }

    pub fn sandbox(&self) -> &SandboxEvidence {
        &self.sandbox
    }

    pub fn authorization(&self) -> &AuthorizationEvidence {
        &self.authorization
    }
}

pub struct ArtifactPipeline {
    policy: ArtifactPolicy,
}

impl ArtifactPipeline {
    pub fn new(policy: ArtifactPolicy) -> Self {
        Self { policy }
    }

    pub fn evaluate(
        &self,
        artifact: &CandidateArtifact,
        sandbox: &impl Sandbox,
    ) -> Result<ArtifactAuthorization, ArtifactRejection> {
        self.validate_target(artifact)?;
        self.validate_base_hash(artifact)?;
        self.validate_syntax(artifact)?;
        self.validate_permissions(artifact)?;

        let policy = PolicyEvidence {
            evidence_type: CANDIDATE_POLICY_EVIDENCE.into(),
            evidence_for: artifact.proposal_id.clone(),
            policy_accepted: true,
            target_path: artifact.target_path.clone(),
            base_hash: artifact.base_hash.clone(),
            permissions: artifact.required_permissions.iter().cloned().collect(),
        };
        let checks = sandbox.run(&artifact.declared_checks);
        if checks.len() != artifact.declared_checks.len()
            || checks
                .iter()
                .zip(&artifact.declared_checks)
                .any(|(actual, declared)| actual.id != declared.id)
        {
            return Err(ArtifactRejection::new(
                artifact,
                RejectionStage::Sandbox,
                "horis:sandboxPassed",
                format!(
                    "horis:sandboxPassed is false for {}: sandbox results do not match declared checks",
                    artifact.proposal_id
                ),
            ));
        }
        if let Some(failed) = checks
            .iter()
            .find(|result| result.status == CheckStatus::Failed)
        {
            return Err(ArtifactRejection::new(
                artifact,
                RejectionStage::Sandbox,
                "horis:sandboxPassed",
                format!(
                    "horis:sandboxPassed is false for {}: check {} failed",
                    artifact.proposal_id, failed.id
                ),
            ));
        }
        let sandbox_evidence = SandboxEvidence {
            evidence_type: SANDBOX_EVIDENCE.into(),
            evidence_for: artifact.proposal_id.clone(),
            sandbox_passed: true,
            checks,
        };

        let authorization = authorize(
            &artifact.proposal_id,
            artifact.kind,
            &policy,
            &sandbox_evidence,
            &self.policy.authorizable_kinds,
        )
        .ok_or_else(|| {
            ArtifactRejection::new(
                artifact,
                RejectionStage::Authorization,
                "horis:authorizationGranted",
                format!(
                    "horis:authorizationGranted is false for {}: {:?} artifacts are not authorizable",
                    artifact.proposal_id, artifact.kind
                ),
            )
        })?;
        Ok(ArtifactAuthorization {
            proposal_id: artifact.proposal_id.clone(),
            claim: "proposal was policy-legal and passed its declared checks; semantic correctness is not proven".into(),
            policy,
            sandbox: sandbox_evidence,
            authorization,
        })
    }

    fn validate_target(&self, artifact: &CandidateArtifact) -> Result<(), ArtifactRejection> {
        let path = Path::new(&artifact.target_path);
        let structurally_relative = !artifact.target_path.is_empty()
            && !path.is_absolute()
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_)));
        let in_scope = self.policy.allowed_path_prefixes.iter().any(|prefix| {
            artifact.target_path == *prefix
                || artifact
                    .target_path
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('/'))
        });
        if structurally_relative && in_scope {
            return Ok(());
        }
        Err(ArtifactRejection::new(
            artifact,
            RejectionStage::TargetPath,
            "horis:artifactPolicy",
            format!(
                "horis:artifactPolicy rejected {} at targetPath: {} is outside allowed paths",
                artifact.proposal_id, artifact.target_path
            ),
        ))
    }

    fn validate_base_hash(&self, artifact: &CandidateArtifact) -> Result<(), ArtifactRejection> {
        let current = self.policy.current_base_hashes.get(&artifact.target_path);
        if current.is_some_and(|hash| hash == &artifact.base_hash) {
            return Ok(());
        }
        let current = current.map_or("absent", String::as_str);
        Err(ArtifactRejection::new(
            artifact,
            RejectionStage::BaseHash,
            "horis:artifactPolicy",
            format!(
                "horis:artifactPolicy rejected {} at baseHash: {} expected {} but current source is {}",
                artifact.proposal_id, artifact.target_path, artifact.base_hash, current
            ),
        ))
    }

    fn validate_syntax(&self, artifact: &CandidateArtifact) -> Result<(), ArtifactRejection> {
        let valid = match (&artifact.kind, &artifact.content) {
            (ArtifactKind::Patch, ArtifactContent::Patch(text)) => {
                valid_patch_shape(text, &artifact.target_path)
            }
            (ArtifactKind::File, ArtifactContent::File(text)) => !text.is_empty(),
            (ArtifactKind::Command, ArtifactContent::Command { program, args }) => {
                self.policy.allowed_executables.contains(program)
                    && is_typed_executable(program, args)
            }
            _ => false,
        };
        if valid {
            return Ok(());
        }
        Err(ArtifactRejection::new(
            artifact,
            RejectionStage::Syntax,
            "horis:artifactPolicy",
            format!(
                "horis:artifactPolicy rejected {} at syntax: content does not match {:?}",
                artifact.proposal_id, artifact.kind
            ),
        ))
    }

    fn validate_permissions(&self, artifact: &CandidateArtifact) -> Result<(), ArtifactRejection> {
        let denied: Vec<&str> = artifact
            .required_permissions
            .difference(&self.policy.allowed_permissions)
            .map(String::as_str)
            .collect();
        if denied.is_empty() {
            for check in &artifact.declared_checks {
                if !self.policy.allowed_executables.contains(&check.program)
                    || !is_typed_executable(&check.program, &check.args)
                {
                    return Err(ArtifactRejection::new(
                        artifact,
                        RejectionStage::Permission,
                        "horis:artifactPolicy",
                        format!(
                            "horis:artifactPolicy rejected {} at permission: declared check {} executable {} is not allowed",
                            artifact.proposal_id, check.id, check.program
                        ),
                    ));
                }
            }
            return Ok(());
        }
        Err(ArtifactRejection::new(
            artifact,
            RejectionStage::Permission,
            "horis:artifactPolicy",
            format!(
                "horis:artifactPolicy rejected {} at permission: {} is not allowed",
                artifact.proposal_id,
                denied.join(", ")
            ),
        ))
    }
}

fn valid_patch_shape(text: &str, target_path: &str) -> bool {
    if !text.starts_with("*** Begin Patch\n") || !text.ends_with("*** End Patch") {
        return false;
    }
    let mut operation_paths = Vec::new();
    for line in text.lines() {
        if let Some(path) = line.strip_prefix("*** Update File: ") {
            operation_paths.push(path);
        } else if line.starts_with("*** Add File: ")
            || line.starts_with("*** Delete File: ")
            || line.starts_with("*** Move to: ")
        {
            return false;
        }
    }
    operation_paths == [target_path]
}

fn is_typed_executable(program: &str, args: &[String]) -> bool {
    const SHELLS: &[&str] = &["sh", "bash", "dash", "zsh", "cmd", "powershell", "pwsh"];
    if program.is_empty() || program.chars().any(char::is_whitespace) {
        return false;
    }
    let executable = Path::new(program)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(program);
    !SHELLS.contains(&executable)
        && !args
            .iter()
            .any(|arg| matches!(arg.as_str(), "-c" | "/c" | "-Command" | "-EncodedCommand"))
}

/// Pure symbolic step over runtime-minted evidence and the declared authorization input.
fn authorize(
    proposal_id: &str,
    kind: ArtifactKind,
    policy: &PolicyEvidence,
    sandbox: &SandboxEvidence,
    authorizable_kinds: &BTreeSet<ArtifactKind>,
) -> Option<AuthorizationEvidence> {
    if !policy.accepted() || !sandbox.passed() || !authorizable_kinds.contains(&kind) {
        return None;
    }
    Some(AuthorizationEvidence {
        evidence_type: AUTHORIZATION_EVIDENCE.into(),
        evidence_for: proposal_id.into(),
        authorization_granted: true,
        artifact_kind: kind,
    })
}
