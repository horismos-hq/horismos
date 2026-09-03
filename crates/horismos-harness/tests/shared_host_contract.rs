use std::collections::{BTreeMap, BTreeSet};

use horismos_fhir::{
    AdapterConfig, CapabilityStatement, FhirAdapter, FhirClosureQuery, FhirFieldMapping,
    FhirHydrationRequest, FhirValueKind, RecordedFhirTransport,
};
use horismos_harness::{
    HarnessAdapter, HarnessConfig, HarnessHydrationRequest, RepoCheckStatus, RepoState,
};
use horismos_host::{ArtifactIdentity, ClosureState, HostAdapter};
use serde_json::json;

fn assert_shared_hydration_contract<A: HostAdapter>(adapter: &A, request: &A::HydrationRequest) {
    let first = adapter.hydrate(request).expect("first complete snapshot");
    let second = adapter.hydrate(request).expect("repeat complete snapshot");
    assert_eq!(first.to_json(), second.to_json());
    assert!(!first.closure.is_empty());
    assert!(first.closure.iter().all(|member| matches!(
        member.state,
        ClosureState::Present | ClosureState::VerifiedAbsent
    )));
    assert!(!first.versions.entries().is_empty());
    let assumptions = first.assumptions(vec![]);
    assert_eq!(assumptions.input_closure, first.closure);
    assert_eq!(assumptions.external_versions, first.versions);
}

#[test]
fn fhir_and_coding_harness_satisfy_the_same_format_neutral_contract() {
    let fhir_request = FhirHydrationRequest {
        snapshot_id: "fhir-contract".into(),
        closure: vec![FhirClosureQuery::required("Observation/status")],
    };
    let fhir = FhirAdapter::new(
        RecordedFhirTransport::new(
            CapabilityStatement {
                fhir_version: "4.0.1".into(),
                transaction: false,
                versioned_update: false,
            },
            json!({"resourceType":"Bundle","type":"collection","entry":[{"resource":{"resourceType":"Observation","id":"status","meta":{"versionId":"1"},"valueString":"ready"}}]}),
            BTreeMap::from([("Observation/status".into(), ClosureState::Present)]),
            BTreeMap::from([("Observation/status".into(), "W/\"1\"".into())]),
        ),
        AdapterConfig {
            profile: ArtifactIdentity::new("fhir-profile", "1", "sha256:fhir-profile"),
            mapping: ArtifactIdentity::new("fhir-map", "1", "sha256:fhir-map"),
            package: ArtifactIdentity::new("fhir-package", "1", "sha256:fhir-package"),
            terminology: vec![],
            supported_subset: vec!["FHIR R4 string".into()],
            fact_subject: "https://example.org/shared#case".into(),
            fact_class: "https://example.org/shared#Case".into(),
            fields: vec![FhirFieldMapping {
                closure_key: "Observation/status".into(),
                json_pointer: "/valueString".into(),
                property: "https://example.org/shared#status".into(),
                kind: FhirValueKind::String,
                required: true,
            }],
            allowed_write_resource_types: BTreeSet::new(),
        },
    )
    .expect("recorded FHIR adapter");
    assert_shared_hydration_contract(&fhir, &fhir_request);

    let harness_request = HarnessHydrationRequest {
        snapshot_id: "harness-contract".into(),
    };
    let harness = HarnessAdapter::new(
        RepoState {
            branch: "fm/shared".into(),
            head: "abc123".into(),
            file_hashes: BTreeMap::from([("src/lib.rs".into(), "sha256:lib".into())]),
            dirty_paths: BTreeSet::new(),
            test_statuses: BTreeMap::from([("make-test".into(), RepoCheckStatus::Passed)]),
            path_scopes: vec!["src".into()],
        },
        HarnessConfig::prototype(),
    )
    .expect("recorded harness adapter");
    assert_shared_hydration_contract(&harness, &harness_request);
}
