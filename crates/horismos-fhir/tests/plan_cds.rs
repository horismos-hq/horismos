use std::collections::{BTreeMap, BTreeSet};

use horismos_fhir::{
    AdapterConfig, CapabilityStatement, CdsCard, CdsHookRequest, CdsHooksResponse,
    ExternalBooleanResult, FhirAdapter, ImportedPlanDefinition, RecordedFhirTransport,
    proposed_fhir_effect,
};
use horismos_host::{
    ArtifactIdentity, ProposedEffectSet, Reversibility, SourceVersion, SourceVersionVector,
};
use serde_json::json;

fn versions() -> SourceVersionVector {
    SourceVersionVector::new(vec![SourceVersion {
        resource: "Observation/mobility".into(),
        version_id: "7".into(),
        etag: Some("W/\"7\"".into()),
    }])
    .expect("valid version fixture")
}

fn evaluator() -> ArtifactIdentity {
    ArtifactIdentity::new(
        "https://example.org/cql-engine",
        "2.4.0",
        "sha256:evaluator",
    )
}

fn plan() -> serde_json::Value {
    json!({
        "resourceType": "PlanDefinition",
        "id": "vte-plan",
        "url": "https://example.org/PlanDefinition/vte",
        "version": "1.0.0",
        "status": "active",
        "title": "VTE prophylaxis",
        "action": [{
            "id": "assess-vte",
            "title": "Assess VTE prophylaxis",
            "description": "Run bounded VTE rules",
            "condition": [{
                "kind": "applicability",
                "expression": {
                    "language": "text/cql-identifier",
                    "expression": "EligibleForVteAssessment"
                }
            }]
        }]
    })
}

#[test]
fn bounded_plan_executes_only_named_pinned_boolean_results() {
    let imported = ImportedPlanDefinition::from_r4_json(&plan()).expect("bounded plan imports");
    let selected = imported
        .execute(
            &BTreeMap::from([(
                "EligibleForVteAssessment".into(),
                ExternalBooleanResult {
                    name: "EligibleForVteAssessment".into(),
                    value: true,
                    evaluator: evaluator(),
                    source_versions: versions(),
                },
            )]),
            &versions(),
            &evaluator(),
        )
        .expect("pinned result selects action");
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id(), "assess-vte");
}

#[test]
fn unsupported_plan_semantics_fail_at_load() {
    let mut unsupported = plan();
    unsupported["action"][0]["action"] = json!([{"id": "nested"}]);
    let error = ImportedPlanDefinition::from_r4_json(&unsupported)
        .expect_err("nested orchestration is outside v1");
    assert_eq!(
        error.domain_text(),
        "horis:supportedSubset rejected PlanDefinition.action[0].action: nested actions are unsupported"
    );
    assert_eq!(
        error.witness().expect("structured witness").constraint,
        "horis:supportedSubset"
    );
}

#[test]
fn missing_or_stale_external_cql_results_fail_closed() {
    let imported = ImportedPlanDefinition::from_r4_json(&plan()).expect("bounded plan imports");
    let error = imported
        .execute(&BTreeMap::new(), &versions(), &evaluator())
        .expect_err("missing external result is not false");
    assert_eq!(
        error.domain_text(),
        "horis:externalResult requires EligibleForVteAssessment, but no typed boolean result was supplied"
    );

    let stale = SourceVersionVector::new(vec![SourceVersion {
        resource: "Observation/mobility".into(),
        version_id: "8".into(),
        etag: Some("\"8\"".into()),
    }])
    .expect("valid stale fixture");
    let error = imported
        .execute(
            &BTreeMap::from([(
                "EligibleForVteAssessment".into(),
                ExternalBooleanResult {
                    name: "EligibleForVteAssessment".into(),
                    value: true,
                    evaluator: evaluator(),
                    source_versions: stale,
                },
            )]),
            &versions(),
            &evaluator(),
        )
        .expect_err("stale CQL result is not consumed");
    assert_eq!(
        error
            .witness()
            .expect("structured witness")
            .state
            .as_deref(),
        Some("stale")
    );
}

#[test]
fn external_cql_evaluator_identity_is_exactly_pinned() {
    let imported = ImportedPlanDefinition::from_r4_json(&plan()).expect("bounded plan imports");
    let error = imported
        .execute(
            &BTreeMap::from([(
                "EligibleForVteAssessment".into(),
                ExternalBooleanResult {
                    name: "EligibleForVteAssessment".into(),
                    value: true,
                    evaluator: ArtifactIdentity::new(
                        "https://example.org/cql-engine",
                        "2.4.1",
                        "sha256:different",
                    ),
                    source_versions: versions(),
                },
            )]),
            &versions(),
            &evaluator(),
        )
        .expect_err("different evaluator cannot select an action");
    assert_eq!(
        error.domain_text(),
        "horis:externalResult rejected EligibleForVteAssessment: identity mismatch"
    );
}

#[test]
fn cds_hooks_wire_shape_round_trips_without_model_fields() {
    let request = CdsHookRequest {
        hook: "patient-view".into(),
        hook_instance: "hook-vte-1".into(),
        context: json!({"patientId": "patient-1"}),
        prefetch: BTreeMap::from([("vte".into(), json!({"resourceType": "Bundle"}))]),
    };
    let adapter = FhirAdapter::new(
        RecordedFhirTransport::new(
            CapabilityStatement {
                fhir_version: "4.0.1".into(),
                transaction: true,
                versioned_update: true,
            },
            json!({"resourceType": "Bundle", "type": "collection", "entry": []}),
            BTreeMap::new(),
            BTreeMap::new(),
        ),
        AdapterConfig {
            profile: ArtifactIdentity::new("profile", "1", "sha256:profile"),
            mapping: ArtifactIdentity::new("mapping", "1", "sha256:mapping"),
            package: ArtifactIdentity::new("package", "1", "sha256:package"),
            terminology: vec![],
            supported_subset: vec![],
            fact_subject: "case".into(),
            fact_class: "Case".into(),
            fields: vec![],
            allowed_write_resource_types: BTreeSet::from(["MedicationRequest".into()]),
        },
    )
    .expect("capable adapter");
    let proof_ref = "proof:vte-run-1";
    let proposal = ProposedEffectSet {
        proof_ref: proof_ref.into(),
        source_versions: versions(),
        effects: vec![
            proposed_fhir_effect(
                "effect-vte",
                "MedicationRequest/vte-1",
                "vte-1",
                proof_ref,
                json!({"resourceType": "MedicationRequest", "id": "vte-1"}),
                Reversibility::Bufferable,
            )
            .expect("bounded proposal"),
            proposed_fhir_effect(
                "effect-vte-followup",
                "MedicationRequest/vte-2",
                "vte-2",
                proof_ref,
                json!({"resourceType": "MedicationRequest", "id": "vte-2"}),
                Reversibility::Bufferable,
            )
            .expect("second bounded proposal"),
        ],
    };
    let authorized = adapter
        .authorize_commit(&proposal)
        .expect("proposal is policy-authorized");
    let response = CdsHooksResponse {
        cards: vec![CdsCard::recommendation(
            "Pharmacologic VTE prophylaxis is recommended",
            &authorized,
        )],
    };
    let request_json = serde_json::to_string(&request).expect("request serializes");
    let response_json = serde_json::to_string(&response).expect("response serializes");
    assert!(request_json.contains("hookInstance"));
    assert!(response_json.contains("create"));
    assert!(response_json.contains("proof:vte-run-1"));
    assert!(response_json.contains("Provenance"));
    assert!(response_json.contains("proposal-state"));
    assert!(!response_json.contains("model"));
    assert_eq!(response.cards[0].suggestions[0].actions.len(), 4);
    assert_eq!(
        response.cards[0].suggestions[0].actions[1].resource["target"][0]["reference"],
        "MedicationRequest/vte-1"
    );
    assert_eq!(
        response.cards[0].suggestions[0].actions[3].resource["target"][0]["reference"],
        "MedicationRequest/vte-2"
    );
    assert_eq!(
        serde_json::from_str::<CdsHooksResponse>(&response_json).expect("response parses"),
        response
    );
}
