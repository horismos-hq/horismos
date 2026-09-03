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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = CdsHookRequest {
        hook: "patient-view".into(),
        hook_instance: "vte-standalone-1".into(),
        context: json!({"patientId":"patient-1","encounterId":"encounter-1"}),
        prefetch: BTreeMap::from([("vte-closure".into(), serde_json::from_str::<Value>(BUNDLE)?)]),
    };
    let prototype = VtePrototype::from_assets(
        ONTOLOGY,
        &serde_json::from_str(PLAN)?,
        &serde_json::from_str(PROFILE)?,
        &serde_json::from_str(MAPPING)?,
        &serde_json::from_str(TERMINOLOGY)?,
        &serde_json::from_str(MANIFEST)?,
    )?;
    let evaluator = RecordedVteBooleanEvaluator::new(prototype.external_evaluator().clone(), true);
    let states = BTreeMap::from([
        ("Patient/patient-1".into(), ClosureState::Present),
        ("Observation/mobility".into(), ClosureState::Present),
        ("Observation/cancer".into(), ClosureState::Present),
        ("Observation/history".into(), ClosureState::Present),
        ("Observation/bleeding".into(), ClosureState::Present),
        ("Observation/platelets".into(), ClosureState::VerifiedAbsent),
    ]);
    let etags = BTreeMap::from([
        ("Patient/patient-1".into(), "W/\"11\"".into()),
        ("Observation/mobility".into(), "W/\"7\"".into()),
        ("Observation/cancer".into(), "\"4\"".into()),
        ("Observation/history".into(), "W/\"2\"".into()),
        ("Observation/bleeding".into(), "\"3\"".into()),
    ]);
    let service = prototype.recorded_service(
        &request,
        CapabilityStatement {
            fhir_version: "4.0.1".into(),
            transaction: true,
            versioned_update: true,
        },
        states,
        etags,
    )?;
    let outcome = service.invoke(&request, &evaluator)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "request": request,
            "response": outcome.response(),
            "proof": outcome.proof(),
            "closureUtility": outcome.utility(),
            "integrationTrace": outcome.trace()
        }))?
    );
    Ok(())
}
