//! Confirmed VTE-prophylaxis vertical slice over the generic host boundary.
//!
//! All clinical data enters as one recorded or host-supplied declared closure. The
//! symbolic decision is independent of FHIR, while this module composes the pinned
//! artifacts into a CDS Hooks-shaped service response. Owner: PRP-008 U5.

use std::collections::{BTreeMap, BTreeSet};

use horismos_core::{Ontology, Ref, Value, trace};
use horismos_host::{
    ArtifactIdentity, ClosureState, CommitEvent, CommitReceipt, CommitState, ExternalSnapshot,
    HostAdapter, HostError, INTEGRATION_TRACE_V1, IntegrationTrace, ProposedEffectSet,
    Reversibility, SourceVersionVector, TerminologyPackage,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use sha2::{Digest, Sha256};

use crate::{
    AdapterConfig, CapabilityStatement, CdsCard, CdsHookRequest, CdsHooksResponse,
    ExternalBooleanResult, FhirAdapter, FhirFieldMapping, FhirHydrationRequest, FhirTransport,
    FhirValueKind, ImportedPlanDefinition, RecordedFhirTransport, proposed_fhir_effect,
};

const VTE_CLASS: &str = "https://example.org/vte#VteCase";
const VTE_RECOMMENDATION: &str = "https://example.org/vte#recommendation";
const PHARMACOLOGIC: &str = "https://example.org/vte#PharmacologicProphylaxis";
const HOLD: &str = "https://example.org/vte#HoldProphylaxis";
const PREFETCH_KEY: &str = "vte-closure";
const PATIENT_CLOSURE_MEMBER: &str = "Patient/{{context.patientId}}";
const EXTERNAL_RESULT: &str = "EligibleForVteAssessment";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VteManifest {
    fhir_baseline: String,
    ontology: ArtifactIdentity,
    profile: ArtifactIdentity,
    mapping: ArtifactIdentity,
    package: ArtifactIdentity,
    plan_definition: ArtifactIdentity,
    terminology: VteTerminology,
    external_evaluator: ArtifactIdentity,
    closure: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VteTerminology {
    canonical: String,
    code_system_version: String,
    expansion_timestamp: String,
    parameters: BTreeMap<String, String>,
    hash: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VteMappingAsset {
    canonical: String,
    version: String,
    fields: Vec<FhirFieldMapping>,
}

/// Immutable, validated assets used to instantiate a VTE host service.
pub struct VtePrototype {
    ontology: Ontology,
    ontology_identity: ArtifactIdentity,
    plan: ImportedPlanDefinition,
    plan_identity: ArtifactIdentity,
    external_evaluator: ArtifactIdentity,
    adapter_config: AdapterConfig,
    closure: Vec<String>,
}

impl VtePrototype {
    pub fn from_assets(
        ontology_source: &str,
        plan_json: &JsonValue,
        profile_json: &JsonValue,
        mapping_json: &JsonValue,
        terminology_json: &JsonValue,
        manifest_json: &JsonValue,
    ) -> Result<Self, HostError> {
        let manifest: VteManifest =
            serde_json::from_value(manifest_json.clone()).map_err(|error| {
                vte_error("horis:artifactIdentity", "VTE manifest", &error.to_string())
            })?;
        if manifest.fhir_baseline != "4.0.1" {
            return Err(vte_error(
                "horis:fhirCapability",
                &manifest.fhir_baseline,
                "the VTE package requires FHIR R4 4.0.1",
            ));
        }
        verify_hash(
            &manifest.ontology,
            ontology_source.as_bytes(),
            "VTE ontology",
        )?;
        verify_hash(
            &manifest.profile,
            &canonical_json(profile_json)?,
            "VTE FHIR profile",
        )?;
        verify_hash(
            &manifest.mapping,
            &canonical_json(mapping_json)?,
            "VTE mapping",
        )?;
        verify_hash(
            &manifest.plan_definition,
            &canonical_json(plan_json)?,
            "VTE PlanDefinition",
        )?;
        verify_terminology_asset(&manifest.terminology, terminology_json)?;
        verify_package_hash(&manifest)?;
        validate_profile_asset(&manifest.profile, profile_json)?;
        let plan = ImportedPlanDefinition::from_r4_json(plan_json)?;
        if plan.canonical() != manifest.plan_definition.canonical
            || plan.version() != manifest.plan_definition.version
        {
            return Err(vte_error(
                "horis:artifactIdentity",
                plan.canonical(),
                "PlanDefinition identity differs from the pinned VTE manifest",
            ));
        }
        let ontology = Ontology::load_from_str(ontology_source, |_| None).map_err(|error| {
            vte_error(
                "horis:ontology",
                "VTE ontology",
                &format!("ontology load failed: {error}; {error:?}"),
            )
        })?;
        let terminology = TerminologyPackage::new(
            manifest.terminology.canonical,
            manifest.terminology.code_system_version,
            manifest.terminology.expansion_timestamp,
            manifest.terminology.parameters,
            manifest.terminology.hash,
        )?;
        let expected_closure = [
            PATIENT_CLOSURE_MEMBER,
            "Observation/mobility",
            "Observation/cancer",
            "Observation/history",
            "Observation/bleeding",
            "Observation/platelets",
        ];
        if manifest
            .closure
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != expected_closure
        {
            return Err(vte_error(
                "horis:inputClosure",
                "VTE manifest",
                "the pinned VTE closure is incomplete or out of canonical order",
            ));
        }
        let mapping: VteMappingAsset = serde_json::from_value(mapping_json.clone())
            .map_err(|error| vte_error("horis:mappingLoss", "VTE mapping", &error.to_string()))?;
        if mapping.canonical != manifest.mapping.canonical
            || mapping.version != manifest.mapping.version
            || mapping.fields.len() != 4
            || mapping
                .fields
                .iter()
                .any(|field| field.kind != FhirValueKind::Boolean || !field.required)
        {
            return Err(vte_error(
                "horis:mappingLoss",
                "VTE mapping",
                "mapping identity or bounded boolean field set differs from the manifest",
            ));
        }
        Ok(Self {
            ontology,
            ontology_identity: manifest.ontology,
            plan,
            plan_identity: manifest.plan_definition,
            external_evaluator: manifest.external_evaluator,
            adapter_config: AdapterConfig {
                profile: manifest.profile,
                mapping: manifest.mapping,
                package: manifest.package,
                terminology: vec![terminology],
                supported_subset: vec![
                    "FHIR R4 4.0.1 JSON".into(),
                    "flat PlanDefinition applicability actions".into(),
                    "named external CQL boolean results".into(),
                    "VTE prophylaxis rules v1".into(),
                ],
                fact_subject: String::new(),
                fact_class: VTE_CLASS.into(),
                fields: mapping.fields,
                allowed_write_resource_types: BTreeSet::from([
                    "MedicationRequest".into(),
                    "DetectedIssue".into(),
                ]),
            },
            closure: manifest.closure,
        })
    }

    pub fn external_evaluator(&self) -> &ArtifactIdentity {
        &self.external_evaluator
    }

    pub fn recorded_service(
        self,
        request: &CdsHookRequest,
        capability: CapabilityStatement,
        states: BTreeMap<String, ClosureState>,
        etags: BTreeMap<String, String>,
    ) -> Result<VteService<RecordedFhirTransport>, HostError> {
        let bundle = request.prefetch.get(PREFETCH_KEY).cloned().ok_or_else(|| {
            vte_error(
                "horis:inputClosure",
                PREFETCH_KEY,
                "CDS Hooks prefetch is missing the declared VTE closure Bundle",
            )
        })?;
        let patient_id = request_patient_id(request)?;
        validate_patient_bound_bundle(&bundle, patient_id)?;
        validate_vte_bundle_profile(&bundle, patient_id)?;
        let closure = bind_closure(&self.closure, patient_id);
        if let Some(key) = states
            .keys()
            .chain(etags.keys())
            .find(|key| !closure.contains(key))
        {
            return Err(vte_error(
                "horis:inputClosure",
                key,
                &format!(
                    "recorded closure member is outside the declared closure for Patient/{patient_id}"
                ),
            ));
        }
        let fact_subject = projection_subject(patient_id);
        let mut adapter_config = self.adapter_config;
        adapter_config.fact_subject.clone_from(&fact_subject);
        let adapter = FhirAdapter::new(
            RecordedFhirTransport::new(capability, bundle.clone(), states, etags),
            adapter_config,
        )?;
        Ok(VteService {
            adapter,
            ontology: self.ontology,
            ontology_identity: self.ontology_identity,
            plan: self.plan,
            plan_identity: self.plan_identity,
            external_evaluator: self.external_evaluator,
            closure,
            fact_subject,
            recorded_prefetch: Some(bundle),
        })
    }
}

/// External CQL boundary. Implementations return only typed, version-bound results.
pub trait VteBooleanEvaluator {
    fn evaluate(
        &self,
        snapshot: &ExternalSnapshot,
    ) -> Result<BTreeMap<String, ExternalBooleanResult>, HostError>;
}

/// Deterministic external-result fixture; it performs no CQL parsing, I/O, or model call.
#[derive(Debug, Clone)]
pub struct RecordedVteBooleanEvaluator {
    identity: ArtifactIdentity,
    eligible: bool,
}

impl RecordedVteBooleanEvaluator {
    pub fn new(identity: ArtifactIdentity, eligible: bool) -> Self {
        Self { identity, eligible }
    }
}

impl VteBooleanEvaluator for RecordedVteBooleanEvaluator {
    fn evaluate(
        &self,
        snapshot: &ExternalSnapshot,
    ) -> Result<BTreeMap<String, ExternalBooleanResult>, HostError> {
        Ok(BTreeMap::from([(
            EXTERNAL_RESULT.into(),
            ExternalBooleanResult {
                name: EXTERNAL_RESULT.into(),
                value: self.eligible,
                evaluator: self.identity.clone(),
                source_versions: snapshot.versions.clone(),
            },
        )]))
    }
}

/// Host-visible measure of the declared-closure cost for one decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureUtility {
    pub declared: usize,
    pub present: usize,
    pub verified_absent: usize,
    pub fail_closed: usize,
    pub mapping_losses: usize,
}

impl ClosureUtility {
    fn from_snapshot(snapshot: &ExternalSnapshot) -> Self {
        Self {
            declared: snapshot.closure.len(),
            present: snapshot
                .closure
                .iter()
                .filter(|member| member.state == ClosureState::Present)
                .count(),
            verified_absent: snapshot
                .closure
                .iter()
                .filter(|member| member.state == ClosureState::VerifiedAbsent)
                .count(),
            fail_closed: snapshot
                .closure
                .iter()
                .filter(|member| {
                    matches!(
                        member.state,
                        ClosureState::NotAuthorized | ClosureState::NotFetched
                    )
                })
                .count(),
            mapping_losses: snapshot.losses.len(),
        }
    }
}

/// Proof artifact returned beside the CDS response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VteDecisionProof {
    pub proof_ref: String,
    pub recommendation: String,
    pub rendered: String,
    pub source_versions: SourceVersionVector,
    pub ontology: ArtifactIdentity,
    pub profile: ArtifactIdentity,
    pub mapping: ArtifactIdentity,
    pub package: ArtifactIdentity,
    pub plan_definition: ArtifactIdentity,
    pub external_evaluator: ArtifactIdentity,
    pub terminology: Vec<TerminologyPackage>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VteOutcome {
    response: CdsHooksResponse,
    proof: VteDecisionProof,
    trace: IntegrationTrace,
    effects: ProposedEffectSet,
    utility: ClosureUtility,
}

impl VteOutcome {
    pub fn response(&self) -> &CdsHooksResponse {
        &self.response
    }

    pub fn proof(&self) -> &VteDecisionProof {
        &self.proof
    }

    pub fn trace(&self) -> &IntegrationTrace {
        &self.trace
    }

    pub fn effects(&self) -> &ProposedEffectSet {
        &self.effects
    }

    pub fn utility(&self) -> &ClosureUtility {
        &self.utility
    }
}

pub struct VteService<T> {
    adapter: FhirAdapter<T>,
    ontology: Ontology,
    ontology_identity: ArtifactIdentity,
    plan: ImportedPlanDefinition,
    plan_identity: ArtifactIdentity,
    external_evaluator: ArtifactIdentity,
    closure: Vec<String>,
    fact_subject: String,
    recorded_prefetch: Option<JsonValue>,
}

impl<T: FhirTransport> VteService<T> {
    pub fn adapter(&self) -> &FhirAdapter<T> {
        &self.adapter
    }

    pub fn invoke(
        &self,
        request: &CdsHookRequest,
        evaluator: &impl VteBooleanEvaluator,
    ) -> Result<VteOutcome, HostError> {
        validate_request(request, self.recorded_prefetch.as_ref())?;
        let hydration = FhirHydrationRequest {
            snapshot_id: request.hook_instance.clone(),
            closure: self
                .closure
                .iter()
                .map(|key| crate::FhirClosureQuery {
                    key: key.clone(),
                    required: key != "Observation/platelets",
                })
                .collect(),
        };
        let snapshot = self.adapter.hydrate(&hydration)?;
        let results = evaluator.evaluate(&snapshot)?;
        let selected = self
            .plan
            .execute(&results, &snapshot.versions, &self.external_evaluator)?;
        if selected.len() != 1 {
            return Err(vte_error(
                "horis:externalResult",
                EXTERNAL_RESULT,
                "VTE PlanDefinition must select exactly one bounded assessment action",
            ));
        }

        let mut admitted = snapshot.admit(&self.ontology)?;
        let (subject, property) = {
            let world = admitted.world_mut();
            (
                Ref::new(world.intern(&self.fact_subject)),
                world.intern(VTE_RECOMMENDATION),
            )
        };
        let recommendation = admitted
            .world()
            .values(subject, property)
            .next()
            .and_then(|value| match value {
                Value::Ref(reference) => Some(admitted.world().resolve(reference.iri()).to_owned()),
                _ => None,
            })
            .ok_or_else(|| {
                vte_error(
                    "horis:recommendation",
                    &self.fact_subject,
                    "the VTE rules did not establish one typed recommendation",
                )
            })?;
        let rendered = admitted
            .world()
            .explain(subject, property)
            .map(|proof| proof.render(admitted.world()))
            .ok_or_else(|| {
                vte_error(
                    "horis:proofRef",
                    &self.fact_subject,
                    "a rule-derived VTE recommendation requires a proof",
                )
            })?;
        let record = admitted.world().last_reasoning().ok_or_else(|| {
            vte_error(
                "horis:proofRef",
                &self.fact_subject,
                "the admitted decision has no reasoning record",
            )
        })?;
        let mut core_trace = trace::Trace::new(&request.hook_instance);
        core_trace.ontology = Some(trace::schema::OntologyRef {
            iri: self.ontology_identity.canonical.clone(),
            version: Some(self.ontology_identity.version.clone()),
            hash: Some(self.ontology_identity.hash.clone()),
        });
        core_trace.steps.push(trace::reasoning_step(
            admitted.world(),
            Some("vte-prophylaxis"),
            record,
        ));
        let proof_ref = format!("urn:horismos:proof:{}", request.hook_instance);
        let patient_id = request
            .context
            .get("patientId")
            .and_then(JsonValue::as_str)
            .expect("request validation guarantees patientId");
        let (summary, effect) = recommendation_effect(
            &request.hook_instance,
            patient_id,
            &recommendation,
            &proof_ref,
        )?;
        let effects = ProposedEffectSet {
            proof_ref: proof_ref.clone(),
            source_versions: snapshot.versions.clone(),
            effects: vec![effect],
        };
        let authorized = self.adapter.authorize_proposal(&effects)?;
        let response = CdsHooksResponse {
            cards: vec![CdsCard::recommendation(summary, &authorized)],
        };
        let proof = VteDecisionProof {
            proof_ref: proof_ref.clone(),
            recommendation,
            rendered,
            source_versions: snapshot.versions.clone(),
            ontology: self.ontology_identity.clone(),
            profile: snapshot.profile.clone(),
            mapping: snapshot.mapping.clone(),
            package: snapshot.package.clone(),
            plan_definition: self.plan_identity.clone(),
            external_evaluator: self.external_evaluator.clone(),
            terminology: snapshot.terminology.clone(),
        };
        let utility = ClosureUtility::from_snapshot(&snapshot);
        let trace = IntegrationTrace {
            schema: INTEGRATION_TRACE_V1.into(),
            run_id: request.hook_instance.clone(),
            snapshot_id: snapshot.snapshot_id.clone(),
            versions: snapshot.versions.clone(),
            profile: snapshot.profile.clone(),
            mapping: snapshot.mapping.clone(),
            package: snapshot.package.clone(),
            terminology: snapshot.terminology.clone(),
            assumptions: snapshot.assumptions(
                effects
                    .effects
                    .iter()
                    .map(|effect| effect.id.clone())
                    .collect(),
            ),
            proof_ref,
            effects: effects.clone(),
            core: core_trace,
            commits: vec![CommitEvent {
                state: CommitState::DecisionProduced,
                receipt: None,
                domain_text: None,
            }],
        };
        Ok(VteOutcome {
            response,
            proof,
            trace,
            effects,
            utility,
        })
    }

    /// Attempt the conditional write and append its externally observable lifecycle.
    pub fn commit_outcome(&mut self, outcome: &mut VteOutcome) -> Result<CommitReceipt, HostError> {
        if outcome.effects != outcome.trace.effects
            || outcome.effects.proof_ref != outcome.proof.proof_ref
            || outcome.effects.source_versions != outcome.proof.source_versions
            || outcome.trace.proof_ref != outcome.proof.proof_ref
            || outcome.trace.versions != outcome.proof.source_versions
        {
            return Err(vte_error(
                "horis:authorizationGranted",
                &outcome.proof.proof_ref,
                "decision proof, trace, and proposed effects are not coherent",
            ));
        }
        outcome.trace.commits.push(CommitEvent {
            state: CommitState::CommitAttempted,
            receipt: None,
            domain_text: None,
        });
        match self.adapter.commit(&outcome.effects) {
            Ok(receipt) => {
                outcome.trace.commits.push(CommitEvent {
                    state: CommitState::CommitAccepted,
                    receipt: Some(receipt.clone()),
                    domain_text: None,
                });
                Ok(receipt)
            }
            Err(error) => {
                let state = if error
                    .witness()
                    .is_some_and(|witness| witness.constraint == "horis:sourceVersion")
                {
                    CommitState::CommitConflicted
                } else {
                    CommitState::CommitFailed
                };
                outcome.trace.commits.push(CommitEvent {
                    state,
                    receipt: None,
                    domain_text: Some(error.domain_text().to_owned()),
                });
                Err(error)
            }
        }
    }
}

impl VteService<RecordedFhirTransport> {
    /// Fixture-only source mutation used to prove source-drift handling.
    pub fn inject_recorded_drift(&mut self, resource: &str, version_id: &str, etag: &str) {
        self.adapter
            .transport_mut()
            .set_current_version(resource, version_id, etag);
    }
}

fn canonical_json(value: &JsonValue) -> Result<Vec<u8>, HostError> {
    serde_json::to_vec(value).map_err(|error| {
        vte_error(
            "horis:artifactIdentity",
            "VTE JSON asset",
            &format!("canonical serialization failed: {error}"),
        )
    })
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn verify_hash(identity: &ArtifactIdentity, bytes: &[u8], subject: &str) -> Result<(), HostError> {
    let actual = sha256(bytes);
    if identity.hash != actual {
        return Err(vte_error(
            "horis:artifactIdentity",
            subject,
            &format!(
                "content digest {actual} differs from pinned {}",
                identity.hash
            ),
        ));
    }
    Ok(())
}

fn verify_package_hash(manifest: &VteManifest) -> Result<(), HostError> {
    let components = format!(
        "{}\n{}\n{}\n{}\n{}\n",
        manifest.ontology.hash,
        manifest.profile.hash,
        manifest.mapping.hash,
        manifest.plan_definition.hash,
        manifest.terminology.hash,
    );
    verify_hash(&manifest.package, components.as_bytes(), "VTE package")
}

fn validate_profile_asset(
    identity: &ArtifactIdentity,
    profile: &JsonValue,
) -> Result<(), HostError> {
    let valid = profile.get("resourceType").and_then(JsonValue::as_str)
        == Some("StructureDefinition")
        && profile.get("url").and_then(JsonValue::as_str) == Some(identity.canonical.as_str())
        && profile.get("version").and_then(JsonValue::as_str) == Some(identity.version.as_str())
        && profile.get("fhirVersion").and_then(JsonValue::as_str) == Some("4.0.1")
        && profile.get("type").and_then(JsonValue::as_str) == Some("Bundle")
        && profile.get("derivation").and_then(JsonValue::as_str) == Some("constraint")
        && profile
            .pointer("/differential/element")
            .and_then(JsonValue::as_array)
            .is_some_and(|elements| !elements.is_empty());
    if !valid {
        return Err(vte_error(
            "horis:fhirProfile",
            &identity.canonical,
            "local StructureDefinition identity or R4 constraint shape is invalid",
        ));
    }
    Ok(())
}

fn verify_terminology_asset(
    manifest: &VteTerminology,
    expansion: &JsonValue,
) -> Result<(), HostError> {
    let identity = ArtifactIdentity::new(
        &manifest.canonical,
        &manifest.code_system_version,
        &manifest.hash,
    );
    verify_hash(
        &identity,
        &canonical_json(expansion)?,
        "VTE terminology expansion",
    )?;
    let system_version_is_pinned = expansion
        .pointer("/expansion/contains")
        .and_then(JsonValue::as_array)
        .is_some_and(|contains| {
            !contains.is_empty()
                && contains.iter().all(|concept| {
                    concept.get("version").and_then(JsonValue::as_str)
                        == Some(manifest.code_system_version.as_str())
                })
        });
    let check_version = expansion
        .pointer("/expansion/parameter")
        .and_then(JsonValue::as_array)
        .is_some_and(|parameters| {
            parameters.iter().any(|parameter| {
                parameter.get("name").and_then(JsonValue::as_str) == Some("check-system-version")
                    && parameter.get("valueBoolean").and_then(JsonValue::as_bool) == Some(true)
            })
        });
    if expansion.get("resourceType").and_then(JsonValue::as_str) != Some("ValueSet")
        || expansion.get("url").and_then(JsonValue::as_str) != Some(manifest.canonical.as_str())
        || expansion.get("version").and_then(JsonValue::as_str)
            != Some(manifest.code_system_version.as_str())
        || expansion
            .pointer("/expansion/timestamp")
            .and_then(JsonValue::as_str)
            != Some(manifest.expansion_timestamp.as_str())
        || !system_version_is_pinned
        || !check_version
    {
        return Err(vte_error(
            "horis:terminologyPackage",
            &manifest.canonical,
            "offline expansion differs from its pinned version/timestamp/parameters",
        ));
    }
    Ok(())
}

fn validate_vte_bundle_profile(bundle: &JsonValue, patient_id: &str) -> Result<(), HostError> {
    if bundle.get("resourceType").and_then(JsonValue::as_str) != Some("Bundle")
        || bundle.get("type").and_then(JsonValue::as_str) != Some("collection")
    {
        return Err(vte_error(
            "horis:fhirProfile",
            "VTE closure Bundle",
            "local R4 profile requires a collection Bundle",
        ));
    }
    let entries = bundle
        .get("entry")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            vte_error(
                "horis:fhirProfile",
                "VTE closure Bundle",
                "local R4 profile requires Bundle.entry",
            )
        })?;
    for entry in entries {
        let resource = entry.get("resource").ok_or_else(|| {
            vte_error(
                "horis:fhirProfile",
                "Bundle.entry",
                "every entry requires one resource",
            )
        })?;
        let resource_type = resource
            .get("resourceType")
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        let id = resource.get("id").and_then(JsonValue::as_str).unwrap_or("");
        if resource
            .pointer("/meta/versionId")
            .and_then(JsonValue::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(vte_error(
                "horis:fhirProfile",
                &format!("{resource_type}/{id}"),
                "local R4 profile requires meta.versionId",
            ));
        }
        match (resource_type, id) {
            ("Patient", actual) if actual == patient_id => {
                if resource.get("active").and_then(JsonValue::as_bool) != Some(true) {
                    return Err(vte_error(
                        "horis:fhirProfile",
                        &format!("Patient/{actual}"),
                        "VTE slice requires an active Patient",
                    ));
                }
            }
            ("Observation", "mobility" | "cancer" | "history" | "bleeding") => {
                let valid = resource.get("status").and_then(JsonValue::as_str) == Some("final")
                    && resource
                        .pointer("/code/text")
                        .and_then(JsonValue::as_str)
                        .is_some_and(|text| !text.is_empty())
                    && resource
                        .get("valueBoolean")
                        .and_then(JsonValue::as_bool)
                        .is_some();
                if !valid {
                    return Err(vte_error(
                        "horis:fhirProfile",
                        &format!("Observation/{id}"),
                        "VTE risk Observations require final status, code text, and valueBoolean",
                    ));
                }
            }
            ("Observation", "platelets") => {
                if resource
                    .get("valueInteger")
                    .and_then(JsonValue::as_i64)
                    .is_none()
                {
                    return Err(vte_error(
                        "horis:fhirProfile",
                        "Observation/platelets",
                        "present platelet Observation requires valueInteger",
                    ));
                }
            }
            _ => {
                return Err(vte_error(
                    "horis:fhirProfile",
                    &format!("{resource_type}/{id}"),
                    "resource is outside the local VTE closure profile",
                ));
            }
        }
    }
    Ok(())
}

fn validate_request(
    request: &CdsHookRequest,
    recorded_prefetch: Option<&JsonValue>,
) -> Result<(), HostError> {
    if request.hook != "patient-view" {
        return Err(vte_error(
            "horis:supportedSubset",
            &request.hook,
            "the VTE service supports only the patient-view hook",
        ));
    }
    if request.hook_instance.is_empty() || request_patient_id(request).is_err() {
        return Err(vte_error(
            "horis:inputClosure",
            &request.hook_instance,
            "hookInstance and context.patientId are required",
        ));
    }
    if request.prefetch.len() != 1 || !request.prefetch.contains_key(PREFETCH_KEY) {
        return Err(vte_error(
            "horis:inputClosure",
            PREFETCH_KEY,
            "the request must contain exactly the declared VTE prefetch Bundle",
        ));
    }
    if recorded_prefetch
        .is_some_and(|expected| request.prefetch.get(PREFETCH_KEY) != Some(expected))
    {
        return Err(vte_error(
            "horis:inputClosure",
            PREFETCH_KEY,
            "invocation prefetch differs from the Bundle bound to this recorded service",
        ));
    }
    validate_patient_bound_bundle(
        request
            .prefetch
            .get(PREFETCH_KEY)
            .expect("prefetch shape checked above"),
        request_patient_id(request)?,
    )?;
    validate_vte_bundle_profile(
        request
            .prefetch
            .get(PREFETCH_KEY)
            .expect("prefetch shape checked above"),
        request_patient_id(request)?,
    )?;
    Ok(())
}

fn request_patient_id(request: &CdsHookRequest) -> Result<&str, HostError> {
    request
        .context
        .get("patientId")
        .and_then(JsonValue::as_str)
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.'))
        })
        .ok_or_else(|| {
            vte_error(
                "horis:inputClosure",
                &request.hook_instance,
                "context.patientId must be a valid non-empty FHIR id",
            )
        })
}

fn bind_closure(closure: &[String], patient_id: &str) -> Vec<String> {
    closure
        .iter()
        .map(|member| {
            if member == PATIENT_CLOSURE_MEMBER {
                format!("Patient/{patient_id}")
            } else {
                member.clone()
            }
        })
        .collect()
}

fn projection_subject(patient_id: &str) -> String {
    format!("https://example.org/vte/patient/{patient_id}")
}

fn validate_patient_bound_bundle(bundle: &JsonValue, patient_id: &str) -> Result<(), HostError> {
    let expected_patient = format!("Patient/{patient_id}");
    let resources = bundle
        .get("entry")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| {
            vte_error(
                "horis:inputClosure",
                &expected_patient,
                "VTE prefetch must be a Bundle with entries",
            )
        })?;
    let patient_count = resources
        .iter()
        .filter(|entry| {
            entry
                .pointer("/resource/resourceType")
                .and_then(JsonValue::as_str)
                == Some("Patient")
                && entry.pointer("/resource/id").and_then(JsonValue::as_str) == Some(patient_id)
        })
        .count();
    if patient_count != 1 {
        return Err(vte_error(
            "horis:inputClosure",
            &expected_patient,
            "the declared closure must contain exactly the requested Patient",
        ));
    }
    for entry in resources.iter().filter(|entry| {
        entry
            .pointer("/resource/resourceType")
            .and_then(JsonValue::as_str)
            == Some("Observation")
    }) {
        if entry
            .pointer("/resource/subject/reference")
            .and_then(JsonValue::as_str)
            != Some(expected_patient.as_str())
        {
            let id = entry
                .pointer("/resource/id")
                .and_then(JsonValue::as_str)
                .unwrap_or("unknown");
            return Err(vte_error(
                "horis:inputClosure",
                &format!("Observation/{id}"),
                "VTE input does not reference context.patientId",
            ));
        }
    }
    Ok(())
}

fn recommendation_effect(
    run_id: &str,
    patient_id: &str,
    recommendation: &str,
    proof_ref: &str,
) -> Result<(String, horismos_host::Effect), HostError> {
    match recommendation {
        PHARMACOLOGIC => {
            let id = format!("{run_id}-pharmacologic");
            let target = format!("MedicationRequest/{id}");
            let resource = json!({
                "resourceType":"MedicationRequest",
                "id":id,
                "status":"draft",
                "intent":"proposal",
                "medicationCodeableConcept":{"text":"Pharmacologic VTE prophylaxis (agent selection remains a host action)"},
                "subject":{"reference":format!("Patient/{patient_id}")}
            });
            let effect = proposed_fhir_effect(
                format!("vte-effect-{run_id}"),
                target,
                format!("vte:{run_id}:pharmacologic"),
                proof_ref,
                resource,
                Reversibility::Bufferable,
            )?;
            Ok(("Recommend pharmacologic VTE prophylaxis".into(), effect))
        }
        HOLD => {
            let id = format!("{run_id}-contraindication");
            let target = format!("DetectedIssue/{id}");
            let resource = json!({
                "resourceType":"DetectedIssue",
                "id":id,
                "status":"preliminary",
                "severity":"high",
                "code":{"text":"Bleeding contraindication to pharmacologic VTE prophylaxis"},
                "patient":{"reference":format!("Patient/{patient_id}")}
            });
            let effect = proposed_fhir_effect(
                format!("vte-effect-{run_id}"),
                target,
                format!("vte:{run_id}:hold"),
                proof_ref,
                resource,
                Reversibility::Bufferable,
            )?;
            Ok(("Hold pharmacologic VTE prophylaxis".into(), effect))
        }
        other => Err(vte_error(
            "horis:recommendation",
            other,
            "recommendation is outside the VTE package enumeration",
        )),
    }
}

fn vte_error(constraint: &str, subject: &str, detail: &str) -> HostError {
    HostError::boundary(
        format!("{constraint} rejected {subject}: {detail}"),
        constraint,
        subject,
        Some("rejected".into()),
        detail,
    )
}
