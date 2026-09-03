use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use horismos_core::Ontology;
use horismos_host::{
    ArtifactContent, ArtifactKind, ArtifactPipeline, ArtifactPolicy, CandidateArtifact,
    CheckResult, CheckStatus, DeclaredCheck, RejectionStage, Sandbox,
};

fn patch(base_hash: &str, target_path: &str) -> CandidateArtifact {
    CandidateArtifact {
        proposal_id: "patch-1".into(),
        kind: ArtifactKind::Patch,
        target_path: target_path.into(),
        base_hash: base_hash.into(),
        content: ArtifactContent::Patch(
            "*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-old\n+new\n*** End Patch".into(),
        ),
        required_permissions: BTreeSet::from(["workspace-write".into()]),
        declared_checks: vec![DeclaredCheck {
            id: "cargo-test".into(),
            program: "cargo".into(),
            args: vec!["test".into()],
        }],
    }
}

fn policy() -> ArtifactPolicy {
    ArtifactPolicy {
        allowed_path_prefixes: vec!["src".into()],
        current_base_hashes: BTreeMap::from([("src/lib.rs".into(), "sha256:new".into())]),
        allowed_permissions: BTreeSet::from(["workspace-write".into()]),
        allowed_executables: BTreeSet::from(["cargo".into()]),
        authorizable_kinds: BTreeSet::from([ArtifactKind::Patch]),
    }
}

struct RecordingSandbox {
    calls: Cell<usize>,
    status: CheckStatus,
}

impl RecordingSandbox {
    fn passing() -> Self {
        Self {
            calls: Cell::new(0),
            status: CheckStatus::Passed,
        }
    }
}

impl Sandbox for RecordingSandbox {
    fn run(&self, checks: &[DeclaredCheck]) -> Vec<CheckResult> {
        self.calls.set(self.calls.get() + 1);
        checks
            .iter()
            .map(|check| CheckResult {
                id: check.id.clone(),
                status: self.status,
                evidence_hash: "sha256:check-output".into(),
            })
            .collect()
    }
}

#[test]
fn a_seeded_patch_passes_policy_sandbox_and_symbolic_authorization() {
    let sandbox = RecordingSandbox::passing();
    let proof = ArtifactPipeline::new(policy())
        .evaluate(&patch("sha256:new", "src/lib.rs"), &sandbox)
        .expect("legal tested patch is authorized");
    assert_eq!(sandbox.calls.get(), 1);
    assert!(proof.policy().accepted());
    assert!(proof.sandbox().passed());
    assert!(proof.authorization().granted());
    assert_eq!(
        proof.claim(),
        "proposal was policy-legal and passed its declared checks; semantic correctness is not proven"
    );
    assert_eq!(
        proof,
        ArtifactPipeline::new(policy())
            .evaluate(
                &patch("sha256:new", "src/lib.rs"),
                &RecordingSandbox::passing()
            )
            .expect("identical inputs authorize identically")
    );
}

#[test]
fn an_embedded_patch_path_must_match_the_validated_target() {
    let sandbox = RecordingSandbox::passing();
    let mut candidate = patch("sha256:new", "src/lib.rs");
    candidate.content = ArtifactContent::Patch(
        "*** Begin Patch\n*** Update File: ../../outside\n@@\n-old\n+new\n*** End Patch".into(),
    );
    let error = ArtifactPipeline::new(policy())
        .evaluate(&candidate, &sandbox)
        .expect_err("embedded traversal fails closed");
    assert_eq!(sandbox.calls.get(), 0);
    assert_eq!(error.stage, RejectionStage::Syntax);
    assert_eq!(error.constraint, "horis:artifactPolicy");
    assert_eq!(
        error.domain_text(),
        "horis:artifactPolicy rejected patch-1 at syntax: content does not match Patch"
    );
}

#[test]
fn denied_permissions_and_kinds_have_distinct_structured_stages() {
    let sandbox = RecordingSandbox::passing();
    let mut denied = patch("sha256:new", "src/lib.rs");
    denied.required_permissions.insert("network".into());
    let error = ArtifactPipeline::new(policy())
        .evaluate(&denied, &sandbox)
        .expect_err("permission fails closed");
    assert_eq!(error.stage, RejectionStage::Permission);
    assert_eq!(sandbox.calls.get(), 0);
    assert_eq!(
        error.domain_text(),
        "horis:artifactPolicy rejected patch-1 at permission: network is not allowed"
    );

    let mut not_authorizable = policy();
    not_authorizable.authorizable_kinds.clear();
    let error = ArtifactPipeline::new(not_authorizable)
        .evaluate(&patch("sha256:new", "src/lib.rs"), &sandbox)
        .expect_err("kind fails at policy before any sandbox execution");
    assert_eq!(error.stage, RejectionStage::Kind);
    assert_eq!(sandbox.calls.get(), 0);
    assert_eq!(
        error.domain_text(),
        "horis:artifactPolicy rejected patch-1 at kind: Patch artifacts are not authorizable"
    );
}

#[test]
fn shell_commands_are_not_typed_executables() {
    let sandbox = RecordingSandbox::passing();
    let mut command_policy = policy();
    command_policy.authorizable_kinds = BTreeSet::from([ArtifactKind::Command]);
    let mut command = patch("sha256:new", "src/lib.rs");
    command.kind = ArtifactKind::Command;
    command.content = ArtifactContent::Command {
        program: "sh".into(),
        args: vec!["-c".into(), "cargo test; curl example.test".into()],
    };
    let error = ArtifactPipeline::new(command_policy)
        .evaluate(&command, &sandbox)
        .expect_err("shell text fails closed");
    assert_eq!(error.stage, RejectionStage::Syntax);
    assert_eq!(sandbox.calls.get(), 0);
    assert_eq!(
        error.domain_text(),
        "horis:artifactPolicy rejected patch-1 at syntax: content does not match Command"
    );
}

#[test]
fn declared_checks_cannot_bypass_the_executable_policy() {
    let sandbox = RecordingSandbox::passing();
    let mut candidate = patch("sha256:new", "src/lib.rs");
    candidate.declared_checks[0].program = "sh".into();
    candidate.declared_checks[0].args = vec!["-c".into(), "curl example.test".into()];
    let error = ArtifactPipeline::new(policy())
        .evaluate(&candidate, &sandbox)
        .expect_err("shell check fails before sandbox");
    assert_eq!(sandbox.calls.get(), 0);
    assert_eq!(error.stage, RejectionStage::Permission);
    assert_eq!(
        error.domain_text(),
        "horis:artifactPolicy rejected patch-1 at permission: declared check cargo-test executable sh is not allowed"
    );
}

struct WrongShapeSandbox;

impl Sandbox for WrongShapeSandbox {
    fn run(&self, _checks: &[DeclaredCheck]) -> Vec<CheckResult> {
        vec![]
    }
}

#[test]
fn sandbox_must_return_one_ordered_result_per_declared_check() {
    let error = ArtifactPipeline::new(policy())
        .evaluate(&patch("sha256:new", "src/lib.rs"), &WrongShapeSandbox)
        .expect_err("missing result fails closed");
    assert_eq!(error.stage, RejectionStage::Sandbox);
    assert_eq!(error.constraint, "horis:sandboxPassed");
    assert_eq!(
        error.domain_text(),
        "horis:sandboxPassed is false for patch-1: sandbox results do not match declared checks"
    );
}

#[test]
fn stale_base_hash_rejects_before_the_sandbox() {
    let sandbox = RecordingSandbox::passing();
    let error = ArtifactPipeline::new(policy())
        .evaluate(&patch("sha256:old", "src/lib.rs"), &sandbox)
        .expect_err("stale proposal fails closed");
    assert_eq!(sandbox.calls.get(), 0);
    assert_eq!(error.stage, RejectionStage::BaseHash);
    assert_eq!(
        error.domain_text(),
        "horis:artifactPolicy rejected patch-1 at baseHash: src/lib.rs expected sha256:old but current source is sha256:new"
    );
}

#[test]
fn traversal_rejects_before_the_sandbox() {
    let sandbox = RecordingSandbox::passing();
    let error = ArtifactPipeline::new(policy())
        .evaluate(&patch("sha256:new", "../src/lib.rs"), &sandbox)
        .expect_err("traversal fails closed");
    assert_eq!(sandbox.calls.get(), 0);
    assert_eq!(error.stage, RejectionStage::TargetPath);
    assert_eq!(
        error.domain_text(),
        "horis:artifactPolicy rejected patch-1 at targetPath: ../src/lib.rs is outside allowed paths"
    );
}

#[test]
fn a_failed_declared_check_prevents_authorization() {
    let sandbox = RecordingSandbox {
        calls: Cell::new(0),
        status: CheckStatus::Failed,
    };
    let error = ArtifactPipeline::new(policy())
        .evaluate(&patch("sha256:new", "src/lib.rs"), &sandbox)
        .expect_err("failed check refuses authorization");
    assert_eq!(error.stage, RejectionStage::Sandbox);
    assert_eq!(
        error.domain_text(),
        "horis:sandboxPassed is false for patch-1: check cargo-test failed"
    );
}

#[test]
fn runtime_evidence_vocabulary_is_in_the_rdfs_plus_profile() {
    let source = r#"
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

horis:CandidatePolicyEvidence a rdfs:Class .
horis:SandboxEvidence a rdfs:Class .
horis:AuthorizationEvidence a rdfs:Class .
horis:policyAccepted a rdf:Property .
horis:sandboxPassed a rdf:Property .
horis:authorizationGranted a rdf:Property .
horis:evidenceFor a rdf:Property .
"#;
    Ontology::load_from_str(source, |_| None).expect("runtime evidence terms are accepted");
}
