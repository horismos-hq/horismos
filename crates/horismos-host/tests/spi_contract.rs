use std::collections::BTreeMap;

use horismos_core::Ontology;
use horismos_host::{
    ArtifactIdentity, ClosureMember, ClosureState, CommitEvent, CommitReceipt, CommitState, Effect,
    EffectCommitState, EffectReceipt, EffectVerification, ExternalFact, ExternalSnapshot,
    ExternalValue, INTEGRATION_TRACE_V1, IntegrationTrace, ProposedEffectSet, Reversibility,
    SourceVersion, SourceVersionVector, TerminologyPackage,
};

const ONTOLOGY: &str = r#"
@prefix : <https://example.org/host#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix horis: <https://w3id.org/horismos#> .

:Case a rdfs:Class .
:status a rdf:Property ; rdfs:domain :Case ; rdfs:range xsd:string ;
  horis:cardinality "1" .
"#;

fn snapshot(state: ClosureState) -> ExternalSnapshot {
    ExternalSnapshot {
        snapshot_id: "repo-snapshot-7".into(),
        closure: vec![ClosureMember {
            key: "repo:branch/main".into(),
            required: true,
            state,
        }],
        versions: SourceVersionVector::new(vec![SourceVersion {
            resource: "repo:file/Cargo.toml".into(),
            version_id: "sha256:bbb".into(),
            etag: None,
        }])
        .expect("fixture vector is valid"),
        profile: ArtifactIdentity::new("repo-profile", "1", "sha256:profile"),
        mapping: ArtifactIdentity::new("repo-map", "3", "sha256:mapping"),
        terminology: vec![],
        package: ArtifactIdentity::new("repo-package", "7", "sha256:package"),
        supported_subset: vec!["branch".into(), "file-hash".into()],
        facts: vec![ExternalFact {
            source: "repo:file/Cargo.toml".into(),
            subject: "https://example.org/host#case-1".into(),
            class: "https://example.org/host#Case".into(),
            properties: BTreeMap::from([(
                "https://example.org/host#status".into(),
                vec![ExternalValue::String("clean".into())],
            )]),
        }],
        losses: vec![],
    }
}

#[test]
fn a_format_neutral_snapshot_admits_deterministically() {
    let ontology = Ontology::load_from_str(ONTOLOGY, |_| None).expect("fixture ontology loads");
    let first = snapshot(ClosureState::Present);
    let second = snapshot(ClosureState::Present);
    assert_eq!(first.to_json(), second.to_json());

    let admitted = first.admit(&ontology).expect("complete snapshot admits");
    assert_eq!(admitted.world().instance_count(), 1);
}

#[test]
fn an_incomplete_closure_fails_in_domain_terms() {
    let ontology = Ontology::load_from_str(ONTOLOGY, |_| None).expect("fixture ontology loads");
    let error = snapshot(ClosureState::NotFetched)
        .admit(&ontology)
        .expect_err("not-fetched required data must fail closed");
    assert_eq!(
        error.domain_text(),
        "horis:inputClosure requires repo:branch/main, but it is not-fetched"
    );
    let witness = error.witness().expect("structured witness");
    assert_eq!(witness.constraint, "horis:inputClosure");
    assert_eq!(witness.subject, "repo:branch/main");
}

#[test]
fn a_bad_fact_never_returns_a_partially_admitted_world() {
    let ontology = Ontology::load_from_str(ONTOLOGY, |_| None).expect("fixture ontology loads");
    let mut candidate = snapshot(ClosureState::Present);
    candidate.facts.push(ExternalFact {
        source: "repo:file/invalid".into(),
        subject: "https://example.org/host#case-2".into(),
        class: "https://example.org/host#Case".into(),
        properties: BTreeMap::new(),
    });
    let error = candidate
        .admit(&ontology)
        .expect_err("missing required status must reject the snapshot");
    assert_eq!(
        error.domain_text(),
        "horis:externalSnapshot rejected repo:file/invalid: status is required by horis:cardinality \"1\" but no value is asserted on :Case"
    );
    assert!(error.rejected_world().is_none());
}

#[test]
fn proposed_effects_are_typed_by_reversibility() {
    let versions = snapshot(ClosureState::Present).versions;
    let set = ProposedEffectSet {
        proof_ref: "proof:run-7/vte".into(),
        source_versions: versions,
        effects: vec![Effect {
            id: "effect-1".into(),
            target: "repo:file/src/lib.rs".into(),
            reversibility: Reversibility::Reversible,
            idempotency_key: "run-7/effect-1".into(),
            payload: serde_json::json!({"patch": "*** Begin Patch"}),
            verification: EffectVerification {
                postcondition: Some("cargo test".into()),
                verify_before_retry: true,
            },
        }],
    };
    assert_eq!(set.effects[0].reversibility, Reversibility::Reversible);
    assert!(set.to_json().contains("reversible"));
}

#[test]
fn integration_trace_keeps_the_five_assumptions_and_commit_lifecycle_together() {
    let snapshot = snapshot(ClosureState::Present);
    let effects = ProposedEffectSet {
        proof_ref: "proof:run-7/vte".into(),
        source_versions: snapshot.versions.clone(),
        effects: vec![],
    };
    let assumptions = snapshot.assumptions(vec!["repo:patch-application".into()]);
    let trace = IntegrationTrace {
        schema: INTEGRATION_TRACE_V1.into(),
        run_id: "run-7".into(),
        snapshot_id: snapshot.snapshot_id.clone(),
        versions: snapshot.versions.clone(),
        profile: snapshot.profile.clone(),
        mapping: snapshot.mapping.clone(),
        package: snapshot.package.clone(),
        terminology: snapshot.terminology.clone(),
        assumptions,
        proof_ref: effects.proof_ref.clone(),
        effects,
        core: horismos_core::trace::Trace::new_v2("run-7"),
        commits: vec![
            CommitEvent {
                state: CommitState::DecisionProduced,
                receipt: None,
                domain_text: None,
            },
            CommitEvent {
                state: CommitState::CommitConflicted,
                receipt: Some(CommitReceipt {
                    receipt_id: "receipt-7".into(),
                    source_versions: snapshot.versions.clone(),
                    effects: vec![
                        EffectReceipt {
                            effect_id: "effect-1".into(),
                            target: "repo:file/a".into(),
                            state: EffectCommitState::Accepted,
                            evidence: Some("sha256:after".into()),
                        },
                        EffectReceipt {
                            effect_id: "effect-2".into(),
                            target: "repo:file/b".into(),
                            state: EffectCommitState::Conflicted,
                            evidence: Some("sha256:drifted".into()),
                        },
                    ],
                }),
                domain_text: Some("horis:sourceVersion drifted".into()),
            },
        ],
    };

    let json = trace.to_json();
    let decoded: IntegrationTrace = serde_json::from_str(&json).expect("trace round-trips");
    assert_eq!(decoded, trace);
    assert_eq!(decoded.assumptions.input_closure.len(), 1);
    assert_eq!(decoded.assumptions.external_versions.entries().len(), 1);
    assert_eq!(decoded.assumptions.mapping_version, "3");
    assert_eq!(decoded.assumptions.supported_subset.len(), 2);
    assert_eq!(decoded.assumptions.unverified_effects.len(), 1);
    assert_eq!(
        decoded.commits[1]
            .receipt
            .as_ref()
            .expect("structured receipt")
            .effects[1]
            .state,
        EffectCommitState::Conflicted
    );
}

#[test]
fn version_vectors_canonicalize_wire_order_and_reject_invalid_entries() {
    let left = SourceVersionVector::new(vec![
        SourceVersion {
            resource: "z".into(),
            version_id: "2".into(),
            etag: None,
        },
        SourceVersion {
            resource: "a".into(),
            version_id: "1".into(),
            etag: None,
        },
    ])
    .expect("valid vector");
    let right = SourceVersionVector::new(vec![
        SourceVersion {
            resource: "a".into(),
            version_id: "1".into(),
            etag: None,
        },
        SourceVersion {
            resource: "z".into(),
            version_id: "2".into(),
            etag: None,
        },
    ])
    .expect("valid vector");
    assert_eq!(
        serde_json::to_string(&left).expect("left serializes"),
        serde_json::to_string(&right).expect("right serializes")
    );
    assert_eq!(
        serde_json::to_string(&left).expect("serializes"),
        r#"[{"resource":"a","version_id":"1","etag":null},{"resource":"z","version_id":"2","etag":null}]"#
    );

    for invalid in [
        r#"[{"resource":"a","version_id":"1","etag":null},{"resource":"a","version_id":"2","etag":null}]"#,
        r#"[{"resource":"a","version_id":"","etag":null}]"#,
    ] {
        assert!(serde_json::from_str::<SourceVersionVector>(invalid).is_err());
    }
}

#[test]
fn malformed_values_keep_their_source_in_the_domain_error() {
    let ontology = Ontology::load_from_str(ONTOLOGY, |_| None).expect("fixture ontology loads");
    let mut candidate = snapshot(ClosureState::Present);
    candidate.facts[0].properties.insert(
        "https://example.org/host#status".into(),
        vec![ExternalValue::Decimal("not-a-decimal".into())],
    );
    let error = candidate
        .admit(&ontology)
        .expect_err("malformed value rejects");
    assert_eq!(
        error.domain_text(),
        "horis:externalSnapshot rejected repo:file/Cargo.toml: horis:externalValue rejected: \"not-a-decimal\" is not a valid xsd:decimal lexical form"
    );
    assert_eq!(
        error.witness().expect("witness").subject,
        "repo:file/Cargo.toml"
    );
}

#[test]
fn forward_references_admit_independent_of_fact_order_and_cycles_fail_closed() {
    let ontology = Ontology::load_from_str(
        &ONTOLOGY.replace(
            ":status a rdf:Property",
            ":related a rdf:Property ; rdfs:domain :Case ; rdfs:range :Case ; horis:cardinality \"0..1\" .\n:status a rdf:Property",
        ),
        |_| None,
    )
    .expect("reference fixture ontology loads");
    let mut candidate = snapshot(ClosureState::Present);
    candidate.facts[0].properties.insert(
        "https://example.org/host#related".into(),
        vec![ExternalValue::Reference(
            "https://example.org/host#case-2".into(),
        )],
    );
    candidate.facts.push(ExternalFact {
        source: "repo:file/second".into(),
        subject: "https://example.org/host#case-2".into(),
        class: "https://example.org/host#Case".into(),
        properties: BTreeMap::from([(
            "https://example.org/host#status".into(),
            vec![ExternalValue::String("clean".into())],
        )]),
    });
    assert_eq!(
        candidate
            .admit(&ontology)
            .expect("forward ref admits")
            .world()
            .instance_count(),
        2
    );

    candidate.facts[1].properties.insert(
        "https://example.org/host#related".into(),
        vec![ExternalValue::Reference(
            "https://example.org/host#case-1".into(),
        )],
    );
    let error = candidate.admit(&ontology).expect_err("cycle rejects");
    assert_eq!(
        error.domain_text(),
        "horis:externalSnapshot reference cycle: https://example.org/host#case-1 -> https://example.org/host#case-2"
    );
    assert_eq!(
        error.witness().expect("witness").state.as_deref(),
        Some("reference-cycle")
    );
}

#[test]
fn terminology_packages_require_pinned_expansion_evidence() {
    let error = TerminologyPackage::new(
        "http://snomed.info/sct",
        "",
        "2026-09-02T00:00:00Z",
        BTreeMap::new(),
        "sha256:expansion",
    )
    .expect_err("unversioned terminology fails closed");
    assert_eq!(
        error.domain_text(),
        "horis:terminologyPackage http://snomed.info/sct requires a code-system version, expansion timestamp, hash, and check-system-version parameter"
    );
    assert_eq!(
        error.witness().expect("witness").constraint,
        "horis:terminologyPackage"
    );
}
