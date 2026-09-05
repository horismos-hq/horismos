use std::collections::{BTreeMap, BTreeSet};

use horismos_core::Ontology;
use horismos_fhir::{
    AdapterConfig, CapabilityStatement, CommitPosture, FhirAdapter, FhirClosureQuery,
    FhirFieldMapping, FhirValueKind, RecordedFhirTransport, proposed_fhir_effect,
};
use horismos_host::{
    ArtifactIdentity, ClosureState, ExternalSnapshot, HostAdapter, ProposedEffectSet,
    Reversibility, TerminologyPackage,
};
use serde_json::json;

const ONTOLOGY: &str = r#"
@prefix : <https://example.org/vte#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

:Case a rdfs:Class .
:reducedMobility a rdf:Property ; rdfs:domain :Case ; rdfs:range xsd:boolean .
:bleedingRisk a rdf:Property ; rdfs:domain :Case ; rdfs:range xsd:boolean .
"#;

fn bundle(modifier: bool) -> serde_json::Value {
    let mut observation = json!({
        "resourceType": "Observation",
        "id": "mobility",
        "meta": {"versionId": "7"},
        "status": "final",
        "valueBoolean": true
    });
    if modifier {
        observation["modifierExtension"] = json!([{"url": "https://unsafe.example/mod"}]);
    }
    json!({
        "resourceType": "Bundle",
        "type": "collection",
        "entry": [
            {"resource": observation},
            {"resource": {
                "resourceType": "Observation",
                "id": "bleeding",
                "meta": {"versionId": "3"},
                "status": "final",
                "valueBoolean": false
            }}
        ]
    })
}

fn config() -> AdapterConfig {
    AdapterConfig {
        profile: ArtifactIdentity::new("https://example.org/vte-r4", "1", "sha256:profile"),
        mapping: ArtifactIdentity::new("https://example.org/vte-map", "1", "sha256:mapping"),
        package: ArtifactIdentity::new("https://example.org/vte", "1", "sha256:package"),
        terminology: vec![
            TerminologyPackage::new(
                "http://snomed.info/sct?fhir_vs=vte",
                "http://snomed.info/sct/900000000000207008/version/20260731",
                "2026-08-01T00:00:00Z",
                BTreeMap::from([("check-system-version".into(), "true".into())]),
                "sha256:terms",
            )
            .expect("terminology fixture is pinned"),
        ],
        supported_subset: vec![
            "FHIR R4 4.0.1 JSON".into(),
            "Observation.valueBoolean".into(),
        ],
        fact_subject: "https://example.org/vte#case-1".into(),
        fact_class: "https://example.org/vte#Case".into(),
        fields: vec![
            FhirFieldMapping {
                closure_key: "Observation/mobility".into(),
                json_pointer: "/valueBoolean".into(),
                property: "https://example.org/vte#reducedMobility".into(),
                kind: FhirValueKind::Boolean,
                required: true,
            },
            FhirFieldMapping {
                closure_key: "Observation/bleeding".into(),
                json_pointer: "/valueBoolean".into(),
                property: "https://example.org/vte#bleedingRisk".into(),
                kind: FhirValueKind::Boolean,
                required: true,
            },
        ],
        allowed_write_resource_types: BTreeSet::from(["MedicationRequest".into()]),
    }
}

fn effects(snapshot: &ExternalSnapshot) -> ProposedEffectSet {
    let proof_ref = "proof:vte-run-1";
    ProposedEffectSet {
        proof_ref: proof_ref.into(),
        source_versions: snapshot.versions.clone(),
        effects: vec![
            proposed_fhir_effect(
                "effect-medication-request",
                "MedicationRequest/vte-1",
                "vte-run-1:medication-request",
                proof_ref,
                json!({"resourceType": "MedicationRequest", "id": "vte-1"}),
                Reversibility::Bufferable,
            )
            .expect("valid labeled proposal"),
        ],
    }
}

fn request() -> horismos_fhir::FhirHydrationRequest {
    horismos_fhir::FhirHydrationRequest {
        snapshot_id: "vte-run-1".into(),
        closure: vec![
            FhirClosureQuery::required("Observation/mobility"),
            FhirClosureQuery::required("Observation/bleeding"),
        ],
    }
}

fn transport(bundle: serde_json::Value) -> RecordedFhirTransport {
    RecordedFhirTransport::new(
        CapabilityStatement {
            fhir_version: "4.0.1".into(),
            transaction: true,
            versioned_update: true,
        },
        bundle,
        BTreeMap::from([
            ("Observation/mobility".into(), ClosureState::Present),
            ("Observation/bleeding".into(), ClosureState::Present),
        ]),
        BTreeMap::from([
            ("Observation/mobility".into(), "W/\"7\"".into()),
            ("Observation/bleeding".into(), "\"3\"".into()),
        ]),
    )
}

#[test]
fn recorded_r4_closure_maps_and_admits_atomically() {
    let adapter = FhirAdapter::new(transport(bundle(false)), config()).expect("capable fixture");
    let snapshot = adapter.hydrate(&request()).expect("complete fixture");
    let replay = FhirAdapter::new(transport(bundle(false)), config())
        .expect("capable replay fixture")
        .hydrate(&request())
        .expect("complete replay fixture");
    assert_eq!(snapshot.to_json(), replay.to_json());
    assert_eq!(
        snapshot
            .versions
            .entries()
            .iter()
            .map(|version| (version.resource.as_str(), version.version_id.as_str()))
            .collect::<Vec<_>>(),
        vec![("Observation/bleeding", "3"), ("Observation/mobility", "7")]
    );
    assert!(snapshot.losses.iter().any(|loss| loss.contains("/status")));
    snapshot
        .admit(&Ontology::load_from_str(ONTOLOGY, |_| None).expect("valid test ontology"))
        .expect("whole mapped snapshot passes the World gate");
}

#[test]
fn missing_required_mapped_data_fails_with_a_machine_witness() {
    let mut missing = bundle(false);
    missing["entry"][0]["resource"]
        .as_object_mut()
        .expect("resource is an object")
        .remove("valueBoolean");
    let adapter = FhirAdapter::new(transport(missing), config()).expect("capable fixture");
    let error = adapter
        .hydrate(&request())
        .expect_err("required mapping is total");
    assert_eq!(
        error.domain_text(),
        "horis:mappingLoss rejected Observation/mobility: required path /valueBoolean is missing"
    );
    assert_eq!(
        error.witness().expect("structured witness").subject,
        "Observation/mobility"
    );
}

#[test]
fn closure_state_cannot_contradict_bundle_presence() {
    let mut recorded = transport(bundle(false));
    recorded.set_closure_state("Observation/mobility", ClosureState::VerifiedAbsent);
    let adapter = FhirAdapter::new(recorded, config()).expect("capable fixture");
    let error = adapter
        .hydrate(&request())
        .expect_err("absence labels cannot hide returned data");
    assert_eq!(
        error.domain_text(),
        "horis:inputClosure rejected Observation/mobility: closure state verified-absent contradicts Bundle presence true"
    );
}

#[test]
fn etag_presence_must_match_a_present_closure_member() {
    let mut recorded_bundle = bundle(false);
    recorded_bundle["entry"]
        .as_array_mut()
        .expect("Bundle entries")
        .remove(0);
    let recorded = RecordedFhirTransport::new(
        CapabilityStatement {
            fhir_version: "4.0.1".into(),
            transaction: true,
            versioned_update: true,
        },
        recorded_bundle,
        BTreeMap::from([
            ("Observation/mobility".into(), ClosureState::VerifiedAbsent),
            ("Observation/bleeding".into(), ClosureState::Present),
        ]),
        BTreeMap::from([
            ("Observation/mobility".into(), "W/\"7\"".into()),
            ("Observation/bleeding".into(), "\"3\"".into()),
        ]),
    );
    let adapter = FhirAdapter::new(recorded, config()).expect("capable fixture");
    let error = adapter
        .hydrate(&request())
        .expect_err("absent resources cannot carry version evidence");
    assert_eq!(
        error.domain_text(),
        "horis:sourceVersion rejected Observation/mobility: ETag presence true contradicts closure state verified-absent"
    );
}

#[test]
fn incomplete_current_vector_and_atomic_drift_both_abort() {
    let mut adapter =
        FhirAdapter::new(transport(bundle(false)), config()).expect("capable fixture");
    let snapshot = adapter.hydrate(&request()).expect("complete fixture");
    let proposed = effects(&snapshot);
    adapter
        .transport_mut()
        .remove_current_version("Observation/mobility");
    let error = adapter
        .commit(&proposed)
        .expect_err("truncated current vector fails closed");
    assert_eq!(
        error.domain_text(),
        "horis:sourceVersion drift on Observation/mobility: hydrated 7, current missing; commit aborted"
    );

    let mut adapter =
        FhirAdapter::new(transport(bundle(false)), config()).expect("capable fixture");
    let snapshot = adapter.hydrate(&request()).expect("complete fixture");
    adapter
        .transport_mut()
        .drift_during_next_commit("Observation/mobility", "8", "W/\"8\"");
    let error = adapter
        .commit(&effects(&snapshot))
        .expect_err("transport rechecks atomically");
    assert_eq!(
        error.domain_text(),
        "horis:sourceVersion drift during conditional commit: FHIR transaction source precondition failed; commit aborted"
    );
}

#[test]
fn commit_requires_a_labeled_proof_linked_authorization() {
    let mut adapter =
        FhirAdapter::new(transport(bundle(false)), config()).expect("capable fixture");
    let snapshot = adapter.hydrate(&request()).expect("complete fixture");
    let mut proposed = effects(&snapshot);
    proposed.effects[0].payload = json!({
        "resource": {"resourceType": "MedicationRequest", "id": "vte-1"},
        "provenance": {"resourceType": "Provenance"}
    });
    let error = adapter
        .commit(&proposed)
        .expect_err("unlabeled writes are never accepted");
    assert_eq!(
        error.domain_text(),
        "horis:authorizationGranted rejected effect-medication-request: proposal label and Provenance proof/target links are required"
    );
}

#[test]
fn authorization_is_rebound_to_the_committing_adapter_policy() {
    let source = transport(bundle(false));
    let adapter = FhirAdapter::new(source.clone(), config()).expect("capable fixture");
    let snapshot = adapter.hydrate(&request()).expect("complete fixture");
    let authorized = adapter
        .authorize_commit(&effects(&snapshot))
        .expect("source policy allows MedicationRequest");

    let mut restricted = config();
    restricted.allowed_write_resource_types.clear();
    let mut receiving = FhirAdapter::new(source, restricted).expect("capable fixture");
    let error = receiving
        .commit_authorized(&authorized)
        .expect_err("tokens are not transferable across policies");
    assert_eq!(
        error.domain_text(),
        "horis:authorizationGranted rejected effect-medication-request: proposal target is outside the configured FHIR write scope"
    );
}

#[test]
fn malformed_provenance_is_not_authorized() {
    let adapter = FhirAdapter::new(transport(bundle(false)), config()).expect("capable fixture");
    let snapshot = adapter.hydrate(&request()).expect("complete fixture");
    let mut proposed = effects(&snapshot);
    proposed.effects[0].payload["provenance"]["resourceType"] = json!("Basic");
    let error = adapter
        .authorize_commit(&proposed)
        .expect_err("only bounded Provenance is accepted");
    assert_eq!(
        error.domain_text(),
        "horis:authorizationGranted rejected effect-medication-request: proposal label and Provenance proof/target links are required"
    );

    let mut overbroad = effects(&snapshot);
    overbroad.effects[0].payload["provenance"]["target"]
        .as_array_mut()
        .expect("target array")
        .push(json!({"reference": "Patient/unrelated"}));
    let error = adapter
        .authorize_commit(&overbroad)
        .expect_err("extra provenance targets fail closed");
    assert_eq!(
        error.domain_text(),
        "horis:authorizationGranted rejected effect-medication-request: proposal label and Provenance proof/target links are required"
    );
}

#[test]
fn both_fhir_conflict_statuses_are_source_drift() {
    for status in [409, 412] {
        let mut adapter =
            FhirAdapter::new(transport(bundle(false)), config()).expect("capable fixture");
        let snapshot = adapter.hydrate(&request()).expect("complete fixture");
        adapter.transport_mut().fail_next_commit_with(status);
        let error = adapter
            .commit(&effects(&snapshot))
            .expect_err("conflicts are never retried");
        assert_eq!(
            error.witness().expect("structured witness").constraint,
            "horis:sourceVersion"
        );
        assert_eq!(
            error
                .witness()
                .expect("structured witness")
                .state
                .as_deref(),
            Some("conflicted")
        );
        assert_eq!(
            error.domain_text(),
            format!(
                "horis:sourceVersion drift during conditional commit: FHIR transaction returned {status}; commit aborted"
            )
        );
    }
}

#[test]
fn capability_probe_and_modifier_extensions_fail_in_domain_terms() {
    let mut incapable = transport(bundle(false));
    incapable.capability.versioned_update = false;
    let mut adapter = FhirAdapter::new(incapable, config()).expect("reads remain useful");
    assert_eq!(adapter.commit_posture(), CommitPosture::RecommendationOnly);
    let snapshot = adapter
        .hydrate(&request())
        .expect("hydration remains enabled");
    let error = adapter
        .commit(&effects(&snapshot))
        .expect_err("writes require conditional support");
    assert_eq!(
        error.domain_text(),
        "horis:fhirCapability permits recommendations only; conditional commit is unavailable"
    );

    let mut wrong_version = transport(bundle(false));
    wrong_version.capability.fhir_version = "5.0.0".into();
    let error = FhirAdapter::new(wrong_version, config()).expect_err("wrong wire version refused");
    assert_eq!(
        error.domain_text(),
        "horis:fhirCapability requires FHIR R4 4.0.1 JSON"
    );
    assert_eq!(
        error.witness().expect("structured witness").constraint,
        "horis:fhirCapability"
    );

    let mut nested_modifier = bundle(true);
    let modifier = nested_modifier["entry"][0]["resource"]
        .as_object_mut()
        .expect("resource is an object")
        .remove("modifierExtension")
        .expect("modifier fixture exists");
    nested_modifier["entry"][0]["resource"]["component"] = json!([{"modifierExtension": modifier}]);
    let adapter = FhirAdapter::new(transport(nested_modifier), config()).expect("capable fixture");
    let error = adapter
        .hydrate(&request())
        .expect_err("modifier semantics are never discarded");
    assert_eq!(
        error.domain_text(),
        "horis:mappingLoss rejected Observation/mobility: modifierExtension is unsupported"
    );
    assert_eq!(
        error.witness().expect("structured witness").constraint,
        "horis:mappingLoss"
    );
}

#[test]
fn conditional_commit_normalizes_weak_etags_and_fails_closed_on_drift() {
    let mut adapter =
        FhirAdapter::new(transport(bundle(false)), config()).expect("capable fixture");
    let snapshot = adapter.hydrate(&request()).expect("complete fixture");
    let effects = effects(&snapshot);
    let receipt = adapter
        .commit(&effects)
        .expect("unchanged weak ETag commits");
    assert_eq!(receipt.effects[0].effect_id, effects.effects[0].id);
    let transaction = &adapter.transport().transactions()[0];
    assert_eq!(transaction["entry"].as_array().map(Vec::len), Some(2));
    assert_eq!(
        transaction["entry"][0]["resource"]["meta"]["tag"][0]["code"],
        "proposed"
    );
    assert_eq!(transaction["entry"][0]["request"]["method"], "POST");
    assert_eq!(
        transaction["entry"][0]["request"]["url"],
        "MedicationRequest"
    );
    assert_eq!(
        transaction["entry"][0]["fullUrl"],
        "urn:horismos:effect-medication-request"
    );
    assert_eq!(
        transaction["entry"][1]["resource"]["target"][0]["reference"],
        transaction["entry"][0]["fullUrl"]
    );
    assert_eq!(
        transaction["entry"][1]["resource"]["entity"][0]["what"]["identifier"]["value"],
        "proof:vte-run-1"
    );

    adapter
        .transport_mut()
        .set_current_version("Observation/mobility", "8", "W/\"8\"");
    let error = adapter.commit(&effects).expect_err("drift is not retried");
    assert_eq!(
        error.domain_text(),
        "horis:sourceVersion drift on Observation/mobility: hydrated 7, current 8; commit aborted"
    );
    assert_eq!(
        error
            .witness()
            .expect("structured witness")
            .state
            .as_deref(),
        Some("conflicted")
    );
}
