use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use horismos_core::Ontology;
use horismos_harness::{
    HarnessAdapter, HarnessConfig, HarnessHydrationRequest, RepoCheckStatus, RepoState,
};
use horismos_host::{
    ArtifactContent, ArtifactKind, CandidateArtifact, CheckResult, CheckStatus, DeclaredCheck,
    HostAdapter, Reversibility, Sandbox,
};

const ONTOLOGY: &str = r#"
@prefix : <https://example.org/harness#> .
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
:Projection a rdfs:Class .
:branch a rdf:Property ; rdfs:domain :Projection ; rdfs:range xsd:string ; horis:cardinality "1" .
:head a rdf:Property ; rdfs:domain :Projection ; rdfs:range xsd:string ; horis:cardinality "1" .
:dirtyPath a rdf:Property ; rdfs:domain :Projection ; rdfs:range xsd:string ; horis:cardinality "0..*" .
:testStatus a rdf:Property ; rdfs:domain :Projection ; rdfs:range xsd:string ; horis:cardinality "1..*" .
:pathScope a rdf:Property ; rdfs:domain :Projection ; rdfs:range xsd:string ; horis:cardinality "1..*" .
"#;

fn state() -> RepoState {
    RepoState {
        branch: "fm/example".into(),
        head: "abc123".into(),
        file_hashes: BTreeMap::from([
            ("src/lib.rs".into(), "sha256:lib".into()),
            ("tests/smoke.rs".into(), "sha256:test".into()),
        ]),
        dirty_paths: BTreeSet::from(["tests/smoke.rs".into()]),
        test_statuses: BTreeMap::from([("make-test".into(), RepoCheckStatus::Passed)]),
        path_scopes: vec!["src".into(), "tests".into()],
    }
}

fn request() -> HarnessHydrationRequest {
    HarnessHydrationRequest {
        snapshot_id: "repo-run-1".into(),
    }
}

fn patch(base_hash: &str) -> CandidateArtifact {
    CandidateArtifact {
        proposal_id: "patch-1".into(),
        kind: ArtifactKind::Patch,
        target_path: "src/lib.rs".into(),
        base_hash: base_hash.into(),
        content: ArtifactContent::Patch(
            "*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-old\n+new\n*** End Patch".into(),
        ),
        required_permissions: BTreeSet::from(["workspace-write".into()]),
        declared_checks: vec![DeclaredCheck {
            id: "make-test".into(),
            program: "make".into(),
            args: vec!["test".into()],
        }],
    }
}

struct PassingSandbox(Cell<usize>);

impl Sandbox for PassingSandbox {
    fn run(&self, checks: &[DeclaredCheck]) -> Vec<CheckResult> {
        self.0.set(self.0.get() + 1);
        checks
            .iter()
            .map(|check| CheckResult {
                id: check.id.clone(),
                status: CheckStatus::Passed,
                evidence_hash: "sha256:make-test-output".into(),
            })
            .collect()
    }
}

#[test]
fn repository_projection_hydrates_and_admits_deterministically() {
    let adapter = HarnessAdapter::new(state(), HarnessConfig::prototype()).expect("valid state");
    let first = adapter.hydrate(&request()).expect("complete projection");
    let second = adapter.hydrate(&request()).expect("repeat projection");
    assert_eq!(first.to_json(), second.to_json());
    assert_eq!(first.versions.entries().len(), 7);
    assert_eq!(first.closure.len(), 5);

    let ontology = Ontology::load_from_str(ONTOLOGY, |_| None).expect("harness ontology");
    let admitted = first
        .admit(&ontology)
        .expect("projection passes World gate");
    assert_eq!(admitted.world().instance_count(), 1);
}

#[test]
fn candidate_pipeline_authorizes_then_commits_one_reversible_handoff() {
    let mut adapter =
        HarnessAdapter::new(state(), HarnessConfig::prototype()).expect("valid state");
    let sandbox = PassingSandbox(Cell::new(0));
    let snapshot = adapter.hydrate(&request()).expect("snapshot");
    let effects = adapter
        .authorize_candidate(&snapshot, &patch("sha256:lib"), &sandbox)
        .expect("legal tested patch");

    assert_eq!(sandbox.0.get(), 1);
    assert_eq!(effects.effects.len(), 1);
    assert_eq!(effects.effects[0].reversibility, Reversibility::Reversible);
    let receipt = adapter
        .commit(&effects)
        .expect("authorized handoff commits");
    assert_eq!(receipt.effects.len(), 1);
    assert_eq!(adapter.committed_proposals(), &["patch-1"]);
}

#[test]
fn stale_candidate_rejects_before_sandbox_and_source_drift_rejects_commit() {
    let mut adapter =
        HarnessAdapter::new(state(), HarnessConfig::prototype()).expect("valid state");
    let sandbox = PassingSandbox(Cell::new(0));
    let snapshot = adapter.hydrate(&request()).expect("snapshot");
    let rejection = adapter
        .authorize_candidate(&snapshot, &patch("sha256:stale"), &sandbox)
        .expect_err("stale base fails policy");
    assert_eq!(sandbox.0.get(), 0);
    assert_eq!(
        rejection.domain_text(),
        "horis:artifactPolicy rejected patch-1 at baseHash: src/lib.rs expected sha256:stale but current source is sha256:lib"
    );

    let effects = adapter
        .authorize_candidate(&snapshot, &patch("sha256:lib"), &sandbox)
        .expect("current patch authorizes");
    adapter.set_recorded_file_hash("src/lib.rs", "sha256:changed");
    let error = adapter
        .commit(&effects)
        .expect_err("repository drift fails closed");
    assert_eq!(
        error.domain_text(),
        "horis:sourceVersion drift on repo:file/src/lib.rs: hydrated sha256:lib, current sha256:changed; commit aborted"
    );
    let witness = error.witness().expect("machine witness");
    assert_eq!(witness.constraint, "horis:sourceVersion");
    assert_eq!(witness.state.as_deref(), Some("conflicted"));
    assert!(adapter.committed_proposals().is_empty());
}

#[test]
fn invalid_repository_state_fails_in_domain_terms() {
    let mut invalid = state();
    invalid.dirty_paths.insert("../outside".into());
    let error = HarnessAdapter::new(invalid, HarnessConfig::prototype())
        .err()
        .expect("traversal must fail closed");
    assert_eq!(
        error.domain_text(),
        "horis:inputClosure rejected ../outside: repository paths must be relative normal paths"
    );
    assert_eq!(
        error.witness().expect("machine witness").constraint,
        "horis:inputClosure"
    );
}

#[test]
fn forged_snapshot_and_mutated_idempotent_replay_are_rejected() {
    let mut adapter =
        HarnessAdapter::new(state(), HarnessConfig::prototype()).expect("valid state");
    let sandbox = PassingSandbox(Cell::new(0));
    let mut snapshot = adapter.hydrate(&request()).expect("snapshot");
    snapshot.facts[0].source = "forged".into();
    let error = adapter
        .authorize_candidate(&snapshot, &patch("sha256:lib"), &sandbox)
        .expect_err("only the exact adapter projection can authorize");
    assert_eq!(
        error.domain_text(),
        "horis:externalSnapshot rejected repo-run-1: candidate authorization requires this adapter's current complete projection"
    );
    assert_eq!(sandbox.0.get(), 0);

    let snapshot = adapter.hydrate(&request()).expect("snapshot");
    let effects = adapter
        .authorize_candidate(&snapshot, &patch("sha256:lib"), &sandbox)
        .expect("candidate authorizes");
    adapter.commit(&effects).expect("first handoff");
    let error = adapter
        .authorize_candidate(&snapshot, &patch("sha256:lib"), &sandbox)
        .expect_err("proposal identity cannot be rebound after commit");
    assert_eq!(
        error.domain_text(),
        "horis:authorizationGranted rejected urn:horismos:artifact:repo-run-1:patch-1: proposal_id must be unique within one repository snapshot"
    );
    let mut mutated = effects;
    mutated.effects[0].id = "different-proposal".into();
    let error = adapter
        .commit(&mutated)
        .expect_err("receipt replay must remain bound to exact effects");
    assert_eq!(
        error.domain_text(),
        "horis:authorizationGranted rejected urn:horismos:artifact:repo-run-1:patch-1: effect set was not produced by this adapter's candidate pipeline"
    );
}

#[test]
fn an_added_file_is_exact_vector_drift() {
    let mut adapter =
        HarnessAdapter::new(state(), HarnessConfig::prototype()).expect("valid state");
    let sandbox = PassingSandbox(Cell::new(0));
    let snapshot = adapter.hydrate(&request()).expect("snapshot");
    let effects = adapter
        .authorize_candidate(&snapshot, &patch("sha256:lib"), &sandbox)
        .expect("candidate authorizes");
    adapter.insert_recorded_file_hash("src/new.rs", "sha256:new");

    let error = adapter
        .commit(&effects)
        .expect_err("newly appearing file changes the exact vector");
    assert_eq!(
        error.domain_text(),
        "horis:sourceVersion drift on repo:file/src/new.rs: hydrated missing, current sha256:new; commit aborted"
    );
}
