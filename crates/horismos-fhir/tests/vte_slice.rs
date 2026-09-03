use std::collections::BTreeMap;

use horismos_fhir::{
    CapabilityStatement, CdsHookRequest, RecordedVteBooleanEvaluator, VtePrototype,
};
use horismos_host::ClosureState;
use serde_json::{Value, json};

const ONTOLOGY: &str = include_str!("../../../examples/vte/vte.ttl");
const BUNDLE: &str = include_str!("../../../examples/vte/recorded-bundle.json");
const PLAN: &str = include_str!("../../../examples/vte/plan-definition.json");
const PROFILE: &str = include_str!("../../../examples/vte/profile.json");
const MAPPING: &str = include_str!("../../../examples/vte/mapping.json");
const TERMINOLOGY: &str = include_str!("../../../examples/vte/terminology-expansion.json");
const MANIFEST: &str = include_str!("../../../examples/vte/manifest.json");

fn request(bundle: Value) -> CdsHookRequest {
    CdsHookRequest {
        hook: "patient-view".into(),
        hook_instance: "vte-run-1".into(),
        context: json!({"patientId":"patient-1","encounterId":"encounter-1"}),
        prefetch: BTreeMap::from([("vte-closure".into(), bundle)]),
    }
}

fn states() -> BTreeMap<String, ClosureState> {
    BTreeMap::from([
        ("Patient/patient-1".into(), ClosureState::Present),
        ("Observation/mobility".into(), ClosureState::Present),
        ("Observation/cancer".into(), ClosureState::Present),
        ("Observation/history".into(), ClosureState::Present),
        ("Observation/bleeding".into(), ClosureState::Present),
        ("Observation/platelets".into(), ClosureState::VerifiedAbsent),
    ])
}

fn etags() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("Patient/patient-1".into(), "W/\"11\"".into()),
        ("Observation/mobility".into(), "W/\"7\"".into()),
        ("Observation/cancer".into(), "\"4\"".into()),
        ("Observation/history".into(), "W/\"2\"".into()),
        ("Observation/bleeding".into(), "\"3\"".into()),
    ])
}

fn capability() -> CapabilityStatement {
    CapabilityStatement {
        fhir_version: "4.0.1".into(),
        transaction: true,
        versioned_update: true,
    }
}

fn prototype() -> VtePrototype {
    VtePrototype::from_assets(
        ONTOLOGY,
        &serde_json::from_str(PLAN).expect("plan fixture"),
        &serde_json::from_str(PROFILE).expect("profile fixture"),
        &serde_json::from_str(MAPPING).expect("mapping fixture"),
        &serde_json::from_str(TERMINOLOGY).expect("terminology fixture"),
        &serde_json::from_str(MANIFEST).expect("manifest fixture"),
    )
    .expect("valid VTE assets")
}

#[test]
fn recorded_bundle_returns_a_proof_linked_cds_recommendation_and_commits() {
    let request = request(serde_json::from_str(BUNDLE).expect("bundle fixture"));
    let prototype = prototype();
    let evaluator = RecordedVteBooleanEvaluator::new(prototype.external_evaluator().clone(), true);
    let mut service = prototype
        .recorded_service(&request, capability(), states(), etags())
        .expect("recorded adapter");

    let mut outcome = service
        .invoke(&request, &evaluator)
        .expect("bounded VTE decision");

    assert_eq!(outcome.response().cards.len(), 1);
    assert_eq!(
        outcome.response().cards[0].summary,
        "Recommend pharmacologic VTE prophylaxis"
    );
    assert_eq!(outcome.response().cards[0].suggestions[0].actions.len(), 2);
    assert_eq!(outcome.proof().source_versions.entries().len(), 5);
    assert_eq!(outcome.trace().versions, outcome.proof().source_versions);
    assert_eq!(outcome.trace().profile, outcome.proof().profile);
    assert_eq!(outcome.trace().mapping, outcome.proof().mapping);
    assert_eq!(outcome.trace().proof_ref, outcome.proof().proof_ref);
    assert_eq!(outcome.trace().core.steps.len(), 1);
    assert_eq!(outcome.utility().declared, 6);
    assert_eq!(outcome.utility().present, 5);
    assert_eq!(outcome.utility().verified_absent, 1);
    assert_eq!(outcome.utility().fail_closed, 0);
    assert!(!outcome.proof().rendered.is_empty());

    let receipt = service
        .commit_outcome(&mut outcome)
        .expect("unchanged closure commits");
    assert_eq!(receipt.effects.len(), 1);
    assert_eq!(outcome.trace().commits.len(), 3);
    assert!(outcome.trace().commits[2].receipt.is_some());
    assert_eq!(service.adapter().transport().transactions().len(), 1);
}

#[test]
fn drift_after_decision_aborts_the_conditional_commit() {
    let request = request(serde_json::from_str(BUNDLE).expect("bundle fixture"));
    let prototype = prototype();
    let evaluator = RecordedVteBooleanEvaluator::new(prototype.external_evaluator().clone(), true);
    let mut service = prototype
        .recorded_service(&request, capability(), states(), etags())
        .expect("recorded adapter");
    let mut outcome = service.invoke(&request, &evaluator).expect("decision");

    service.inject_recorded_drift("Observation/mobility", "8", "W/\"8\"");
    let error = service
        .commit_outcome(&mut outcome)
        .expect_err("source drift must fail closed");

    assert_eq!(
        error.domain_text(),
        "horis:sourceVersion drift on Observation/mobility: hydrated 7, current 8; commit aborted"
    );
    let witness = error.witness().expect("machine witness");
    assert_eq!(witness.constraint, "horis:sourceVersion");
    assert_eq!(witness.subject, "Observation/mobility");
    assert_eq!(witness.state.as_deref(), Some("conflicted"));
    assert_eq!(outcome.trace().commits.len(), 3);
    assert!(outcome.trace().commits[2].domain_text.is_some());
    assert!(service.adapter().transport().transactions().is_empty());
}

#[test]
fn incomplete_required_closure_fails_in_domain_terms() {
    let prototype = prototype();
    let evaluator = RecordedVteBooleanEvaluator::new(prototype.external_evaluator().clone(), true);
    let mut incomplete = states();
    incomplete.insert("Observation/bleeding".into(), ClosureState::NotAuthorized);
    let mut bundle: Value = serde_json::from_str(BUNDLE).expect("bundle fixture");
    bundle["entry"]
        .as_array_mut()
        .expect("entries")
        .retain(|entry| entry.pointer("/resource/id").and_then(Value::as_str) != Some("bleeding"));
    let request = request(bundle);
    let mut missing_etag = etags();
    missing_etag.remove("Observation/bleeding");
    let service = prototype
        .recorded_service(&request, capability(), incomplete, missing_etag)
        .expect("recorded adapter");

    let error = service
        .invoke(&request, &evaluator)
        .expect_err("required authorization gap must fail closed");

    assert_eq!(
        error.domain_text(),
        "horis:inputClosure requires Observation/bleeding, but it is not-authorized"
    );
    let witness = error.witness().expect("machine witness");
    assert_eq!(witness.constraint, "horis:inputClosure");
    assert_eq!(witness.subject, "Observation/bleeding");
    assert_eq!(witness.state.as_deref(), Some("not-authorized"));
}

#[test]
fn replay_is_byte_deterministic() {
    let run = || {
        let request = request(serde_json::from_str(BUNDLE).expect("bundle fixture"));
        let prototype = prototype();
        let evaluator =
            RecordedVteBooleanEvaluator::new(prototype.external_evaluator().clone(), true);
        let service = prototype
            .recorded_service(&request, capability(), states(), etags())
            .expect("recorded adapter");
        service
            .invoke(&request, &evaluator)
            .expect("bounded VTE decision")
            .trace()
            .to_json()
    };

    assert_eq!(run(), run());
}

#[test]
fn server_without_conditional_writes_still_returns_a_recommendation() {
    let request = request(serde_json::from_str(BUNDLE).expect("bundle fixture"));
    let prototype = prototype();
    let evaluator = RecordedVteBooleanEvaluator::new(prototype.external_evaluator().clone(), true);
    let mut service = prototype
        .recorded_service(
            &request,
            CapabilityStatement {
                fhir_version: "4.0.1".into(),
                transaction: false,
                versioned_update: false,
            },
            states(),
            etags(),
        )
        .expect("recommendation-only adapter");

    let mut outcome = service
        .invoke(&request, &evaluator)
        .expect("recommendation does not require write capability");
    assert_eq!(outcome.response().cards.len(), 1);
    let error = service
        .commit_outcome(&mut outcome)
        .expect_err("write remains unavailable");
    assert!(error.domain_text().contains("recommendations only"));
}

#[test]
fn bleeding_contraindication_selects_the_higher_priority_hold_rule() {
    let mut bundle: Value = serde_json::from_str(BUNDLE).expect("bundle fixture");
    let bleeding = bundle["entry"]
        .as_array_mut()
        .expect("entries")
        .iter_mut()
        .find(|entry| entry.pointer("/resource/id").and_then(Value::as_str) == Some("bleeding"))
        .expect("bleeding fixture");
    bleeding["resource"]["valueBoolean"] = json!(true);
    let request = request(bundle);
    let prototype = prototype();
    let evaluator = RecordedVteBooleanEvaluator::new(prototype.external_evaluator().clone(), true);
    let service = prototype
        .recorded_service(&request, capability(), states(), etags())
        .expect("recorded adapter");

    let outcome = service.invoke(&request, &evaluator).expect("hold decision");

    assert_eq!(
        outcome.response().cards[0].summary,
        "Hold pharmacologic VTE prophylaxis"
    );
    assert!(
        outcome
            .proof()
            .rendered
            .contains("bleeding-contraindication")
    );
    assert_eq!(
        outcome.effects().effects[0].payload["resource"]["resourceType"],
        "DetectedIssue"
    );
}

#[test]
fn prefetch_resources_must_be_bound_to_the_requested_patient() {
    let mut bundle: Value = serde_json::from_str(BUNDLE).expect("bundle fixture");
    bundle["entry"][1]["resource"]["subject"]["reference"] = json!("Patient/patient-2");
    let request = request(bundle);

    let error = prototype()
        .recorded_service(&request, capability(), states(), etags())
        .err()
        .expect("cross-patient evidence must fail closed");

    assert_eq!(
        error.domain_text(),
        "horis:inputClosure rejected Observation/mobility: VTE input does not reference context.patientId"
    );
    let witness = error.witness().expect("machine witness");
    assert_eq!(witness.constraint, "horis:inputClosure");
    assert_eq!(witness.subject, "Observation/mobility");
    assert_eq!(witness.state.as_deref(), Some("rejected"));
}

#[test]
fn artifact_content_must_match_the_pinned_digest() {
    let mut plan: Value = serde_json::from_str(PLAN).expect("plan fixture");
    plan["title"] = json!("Tampered VTE plan");
    let error = VtePrototype::from_assets(
        ONTOLOGY,
        &plan,
        &serde_json::from_str(PROFILE).expect("profile fixture"),
        &serde_json::from_str(MAPPING).expect("mapping fixture"),
        &serde_json::from_str(TERMINOLOGY).expect("terminology fixture"),
        &serde_json::from_str(MANIFEST).expect("manifest fixture"),
    )
    .err()
    .expect("tampered plan cannot retain its pinned identity");

    assert!(
        error
            .domain_text()
            .starts_with("horis:artifactIdentity rejected VTE PlanDefinition: content digest")
    );
    assert_eq!(
        error.witness().expect("machine witness").constraint,
        "horis:artifactIdentity"
    );
}

#[test]
fn local_profile_rejects_an_invalid_observation() {
    let mut bundle: Value = serde_json::from_str(BUNDLE).expect("bundle fixture");
    bundle["entry"][1]["resource"]["status"] = json!("preliminary");
    let request = request(bundle);

    let error = prototype()
        .recorded_service(&request, capability(), states(), etags())
        .err()
        .expect("profile-invalid Observation must fail closed");

    assert_eq!(
        error.domain_text(),
        "horis:fhirProfile rejected Observation/mobility: VTE risk Observations require final status, code text, and valueBoolean"
    );
    assert_eq!(
        error.witness().expect("machine witness").constraint,
        "horis:fhirProfile"
    );
}
