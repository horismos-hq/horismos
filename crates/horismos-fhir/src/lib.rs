//! FHIR R4 adapter for bounded, versioned Horismos projections.
//!
//! The adapter owns FHIR JSON and transport concerns. It always materializes a
//! complete [`horismos_host::ExternalSnapshot`] before core evaluation.
//!
//! Spec: `docs/framework/03-world.md`, `07-tools.md`, and
//! `14-architecture.md` · Owner: PRP-008.

use std::collections::{BTreeMap, BTreeSet};

use horismos_host::{
    ArtifactIdentity, ClosureMember, ClosureState, CommitReceipt, Effect, EffectCommitState,
    EffectReceipt, EffectVerification, ExternalFact, ExternalSnapshot, ExternalValue, HostAdapter,
    HostError, ProposedEffectSet, Reversibility, SourceVersion, SourceVersionVector,
    TerminologyPackage,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub mod cds;
pub mod plan;
pub mod vte;

pub use cds::*;
pub use plan::*;
pub use vte::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityStatement {
    pub fhir_version: String,
    pub transaction: bool,
    pub versioned_update: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FhirClosureQuery {
    pub key: String,
    pub required: bool,
}

impl FhirClosureQuery {
    pub fn required(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            required: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FhirHydrationRequest {
    pub snapshot_id: String,
    pub closure: Vec<FhirClosureQuery>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FhirValueKind {
    Boolean,
    Integer,
    String,
    Reference,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FhirFieldMapping {
    pub closure_key: String,
    pub json_pointer: String,
    pub property: String,
    pub kind: FhirValueKind,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterConfig {
    pub profile: ArtifactIdentity,
    pub mapping: ArtifactIdentity,
    pub package: ArtifactIdentity,
    pub terminology: Vec<TerminologyPackage>,
    pub supported_subset: Vec<String>,
    pub fact_subject: String,
    pub fact_class: String,
    pub fields: Vec<FhirFieldMapping>,
    pub allowed_write_resource_types: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitPosture {
    ConditionalWrites,
    RecommendationOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FhirAuthorizationEvidence {
    proof_ref: String,
    effect_ids: Vec<String>,
    posture: String,
}

impl FhirAuthorizationEvidence {
    pub fn proof_ref(&self) -> &str {
        &self.proof_ref
    }
}

#[derive(Debug, Clone)]
pub struct AuthorizedFhirCommit {
    effects: ProposedEffectSet,
    evidence: FhirAuthorizationEvidence,
}

/// A policy-validated proposed-resource envelope suitable for a recommendation card.
/// Unlike [`AuthorizedFhirCommit`], it conveys no permission to write remotely.
#[derive(Debug, Clone)]
pub struct AuthorizedFhirProposal {
    effects: ProposedEffectSet,
    evidence: FhirAuthorizationEvidence,
}

impl AuthorizedFhirProposal {
    pub fn evidence(&self) -> &FhirAuthorizationEvidence {
        &self.evidence
    }

    pub(crate) fn effects(&self) -> &ProposedEffectSet {
        &self.effects
    }
}

impl AuthorizedFhirCommit {
    pub fn evidence(&self) -> &FhirAuthorizationEvidence {
        &self.evidence
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FhirFetch {
    pub bundle: Value,
    pub states: BTreeMap<String, ClosureState>,
    pub etags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FhirTransportError {
    pub status: Option<u16>,
    pub detail: String,
}

pub trait FhirTransport {
    fn capability_statement(&self) -> Result<CapabilityStatement, FhirTransportError>;

    fn fetch(&self, closure: &[FhirClosureQuery]) -> Result<FhirFetch, FhirTransportError>;

    fn current_versions(
        &self,
        resources: &[String],
    ) -> Result<Vec<SourceVersion>, FhirTransportError>;

    /// Apply the Bundle only if the complete expected read vector still matches,
    /// atomically with the write. Returning 409 or 412 reports that precondition drift.
    fn conditional_transaction(
        &mut self,
        bundle: &Value,
        expected: &SourceVersionVector,
    ) -> Result<String, FhirTransportError>;
}

/// Deterministic, zero-network transport for fixtures and standalone replay.
#[derive(Debug, Clone)]
pub struct RecordedFhirTransport {
    pub capability: CapabilityStatement,
    bundle: Value,
    states: BTreeMap<String, ClosureState>,
    etags: BTreeMap<String, String>,
    current: BTreeMap<String, SourceVersion>,
    next_commit_status: Option<u16>,
    drift_during_next_commit: Option<SourceVersion>,
    transactions: Vec<Value>,
}

impl RecordedFhirTransport {
    pub fn new(
        capability: CapabilityStatement,
        bundle: Value,
        states: BTreeMap<String, ClosureState>,
        etags: BTreeMap<String, String>,
    ) -> Self {
        let current = bundle_resources(&bundle)
            .into_iter()
            .filter_map(|(key, resource)| {
                let version_id = resource.pointer("/meta/versionId")?.as_str()?.to_owned();
                Some((
                    key.clone(),
                    SourceVersion {
                        resource: key.clone(),
                        version_id,
                        etag: etags.get(&key).cloned(),
                    },
                ))
            })
            .collect();
        Self {
            capability,
            bundle,
            states,
            etags,
            current,
            next_commit_status: None,
            drift_during_next_commit: None,
            transactions: Vec::new(),
        }
    }

    pub fn set_current_version(&mut self, resource: &str, version_id: &str, etag: &str) {
        self.current.insert(
            resource.into(),
            SourceVersion {
                resource: resource.into(),
                version_id: version_id.into(),
                etag: Some(etag.into()),
            },
        );
    }

    pub fn set_closure_state(&mut self, resource: &str, state: ClosureState) {
        self.states.insert(resource.into(), state);
    }

    pub fn fail_next_commit_with(&mut self, status: u16) {
        self.next_commit_status = Some(status);
    }

    pub fn remove_current_version(&mut self, resource: &str) {
        self.current.remove(resource);
    }

    pub fn drift_during_next_commit(&mut self, resource: &str, version_id: &str, etag: &str) {
        self.drift_during_next_commit = Some(SourceVersion {
            resource: resource.into(),
            version_id: version_id.into(),
            etag: Some(etag.into()),
        });
    }

    pub fn transactions(&self) -> &[Value] {
        &self.transactions
    }
}

impl FhirTransport for RecordedFhirTransport {
    fn capability_statement(&self) -> Result<CapabilityStatement, FhirTransportError> {
        Ok(self.capability.clone())
    }

    fn fetch(&self, _closure: &[FhirClosureQuery]) -> Result<FhirFetch, FhirTransportError> {
        Ok(FhirFetch {
            bundle: self.bundle.clone(),
            states: self.states.clone(),
            etags: self.etags.clone(),
        })
    }

    fn current_versions(
        &self,
        resources: &[String],
    ) -> Result<Vec<SourceVersion>, FhirTransportError> {
        Ok(resources
            .iter()
            .filter_map(|resource| self.current.get(resource).cloned())
            .collect())
    }

    fn conditional_transaction(
        &mut self,
        bundle: &Value,
        expected: &SourceVersionVector,
    ) -> Result<String, FhirTransportError> {
        if let Some(status) = self.next_commit_status.take() {
            return Err(FhirTransportError {
                status: Some(status),
                detail: format!("FHIR transaction returned {status}"),
            });
        }
        if let Some(drift) = self.drift_during_next_commit.take() {
            self.current.insert(drift.resource.clone(), drift);
        }
        if !version_vector_matches(expected.entries(), &self.current) {
            return Err(FhirTransportError {
                status: Some(412),
                detail: "FHIR transaction source precondition failed".into(),
            });
        }
        self.transactions.push(bundle.clone());
        Ok(format!("recorded-transaction-{}", self.transactions.len()))
    }
}

#[derive(Debug)]
pub struct FhirAdapter<T> {
    transport: T,
    config: AdapterConfig,
    commit_posture: CommitPosture,
}

impl<T: FhirTransport> FhirAdapter<T> {
    pub fn authorize_proposal(
        &self,
        effects: &ProposedEffectSet,
    ) -> Result<AuthorizedFhirProposal, HostError> {
        if effects.proof_ref.is_empty() {
            return Err(fhir_error(
                "horis:authorizationGranted",
                "FHIR proposal",
                "proof_ref is required",
            ));
        }
        if effects.effects.is_empty() {
            return Err(fhir_error(
                "horis:authorizationGranted",
                &effects.proof_ref,
                "at least one proposed FHIR resource is required",
            ));
        }
        for effect in &effects.effects {
            validate_proposed_effect(effect, &effects.proof_ref, &self.config)?;
        }
        Ok(AuthorizedFhirProposal {
            effects: effects.clone(),
            evidence: FhirAuthorizationEvidence {
                proof_ref: effects.proof_ref.clone(),
                effect_ids: effects
                    .effects
                    .iter()
                    .map(|effect| effect.id.clone())
                    .collect(),
                posture: "recommendation-authorized".into(),
            },
        })
    }

    pub fn new(transport: T, config: AdapterConfig) -> Result<Self, HostError> {
        let capability = transport.capability_statement().map_err(transport_error)?;
        if capability.fhir_version != "4.0.1" {
            return Err(HostError::boundary(
                "horis:fhirCapability requires FHIR R4 4.0.1 JSON",
                "horis:fhirCapability",
                capability.fhir_version,
                Some("unsupported".into()),
                "adapter accepts only the declared R4 baseline",
            ));
        }
        let commit_posture = if capability.transaction && capability.versioned_update {
            CommitPosture::ConditionalWrites
        } else {
            CommitPosture::RecommendationOnly
        };
        Ok(Self {
            transport,
            config,
            commit_posture,
        })
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    pub fn commit_posture(&self) -> CommitPosture {
        self.commit_posture
    }

    pub fn authorize_commit(
        &self,
        effects: &ProposedEffectSet,
    ) -> Result<AuthorizedFhirCommit, HostError> {
        if self.commit_posture != CommitPosture::ConditionalWrites {
            return Err(HostError::boundary(
                "horis:fhirCapability permits recommendations only; conditional commit is unavailable",
                "horis:fhirCapability",
                effects.proof_ref.clone(),
                Some("recommendation-only".into()),
                "server lacks transaction or versioned-update support",
            ));
        }
        let proposal = self.authorize_proposal(effects)?;
        Ok(AuthorizedFhirCommit {
            effects: proposal.effects,
            evidence: FhirAuthorizationEvidence {
                proof_ref: effects.proof_ref.clone(),
                effect_ids: proposal.evidence.effect_ids,
                posture: "conditional-write-authorized".into(),
            },
        })
    }

    pub fn commit_authorized(
        &mut self,
        authorized: &AuthorizedFhirCommit,
    ) -> Result<CommitReceipt, HostError> {
        let rebound = self.authorize_commit(&authorized.effects)?;
        if rebound.evidence != authorized.evidence {
            return Err(fhir_error(
                "horis:authorizationGranted",
                &authorized.effects.proof_ref,
                "authorization evidence does not match this adapter policy",
            ));
        }
        self.commit_effects(&authorized.effects)
    }

    fn map_fetch(
        &self,
        request: &FhirHydrationRequest,
        fetch: FhirFetch,
    ) -> Result<ExternalSnapshot, HostError> {
        if fetch.bundle.get("resourceType").and_then(Value::as_str) != Some("Bundle")
            || fetch.bundle.get("type").and_then(Value::as_str) != Some("collection")
        {
            return Err(fhir_error(
                "horis:inputClosure",
                &request.snapshot_id,
                "FHIR hydration must return a collection Bundle",
            ));
        }
        let resources = bundle_resources(&fetch.bundle);
        let entry_count = fetch
            .bundle
            .get("entry")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        if resources.len() != entry_count {
            return Err(fhir_error(
                "horis:inputClosure",
                &request.snapshot_id,
                "Bundle entries require unique resourceType/id identities",
            ));
        }
        let declared: BTreeSet<&str> = request
            .closure
            .iter()
            .map(|query| query.key.as_str())
            .collect();
        if declared.len() != request.closure.len() {
            return Err(fhir_error(
                "horis:inputClosure",
                &request.snapshot_id,
                "declared closure contains duplicate members",
            ));
        }
        if resources.keys().any(|key| !declared.contains(key.as_str())) {
            return Err(fhir_error(
                "horis:inputClosure",
                &request.snapshot_id,
                "Bundle contains a resource outside the declared closure",
            ));
        }
        if fetch
            .states
            .keys()
            .any(|key| !declared.contains(key.as_str()))
        {
            return Err(fhir_error(
                "horis:inputClosure",
                &request.snapshot_id,
                "transport returned an undeclared closure member",
            ));
        }
        if fetch
            .etags
            .keys()
            .any(|key| !declared.contains(key.as_str()))
        {
            return Err(fhir_error(
                "horis:inputClosure",
                &request.snapshot_id,
                "transport returned an ETag outside the declared closure",
            ));
        }

        let mut closure = Vec::with_capacity(request.closure.len());
        let mut versions = Vec::new();
        for query in &request.closure {
            let state = fetch
                .states
                .get(&query.key)
                .copied()
                .unwrap_or(ClosureState::NotFetched);
            let resource_is_present = resources.contains_key(&query.key);
            if resource_is_present != (state == ClosureState::Present) {
                return Err(fhir_error(
                    "horis:inputClosure",
                    &query.key,
                    &format!(
                        "closure state {} contradicts Bundle presence {}",
                        state.as_str(),
                        resource_is_present
                    ),
                ));
            }
            let etag_is_present = fetch.etags.contains_key(&query.key);
            if etag_is_present != (state == ClosureState::Present) {
                return Err(fhir_error(
                    "horis:sourceVersion",
                    &query.key,
                    &format!(
                        "ETag presence {} contradicts closure state {}",
                        etag_is_present,
                        state.as_str()
                    ),
                ));
            }
            if state == ClosureState::Present {
                let resource = resources.get(&query.key).ok_or_else(|| {
                    fhir_error(
                        "horis:inputClosure",
                        &query.key,
                        "present resource is absent from Bundle",
                    )
                })?;
                reject_modifier_extensions(resource, &query.key)?;
                let version_id = required_string(resource, "/meta/versionId", &query.key)?;
                let etag = fetch.etags.get(&query.key).cloned();
                if let Some(tag) = &etag
                    && normalize_etag(tag) != version_id
                {
                    return Err(fhir_error(
                        "horis:sourceVersion",
                        &query.key,
                        "ETag does not match meta.versionId",
                    ));
                }
                versions.push(SourceVersion {
                    resource: query.key.clone(),
                    version_id: version_id.into(),
                    etag,
                });
            }
            closure.push(ClosureMember {
                key: query.key.clone(),
                required: query.required,
                state,
            });
        }

        let mut properties: BTreeMap<String, Vec<ExternalValue>> = BTreeMap::new();
        let mut mapped_paths: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for field in &self.config.fields {
            if !declared.contains(field.closure_key.as_str()) {
                return Err(fhir_error(
                    "horis:mappingLoss",
                    &field.closure_key,
                    "mapping reads outside the declared closure",
                ));
            }
            let state = fetch
                .states
                .get(&field.closure_key)
                .copied()
                .unwrap_or(ClosureState::NotFetched);
            if state != ClosureState::Present {
                continue;
            }
            let resource = resources
                .get(&field.closure_key)
                .expect("present resources checked above");
            match resource.pointer(&field.json_pointer) {
                Some(value) => {
                    let value =
                        external_value(value, field.kind, &field.closure_key, &field.json_pointer)?;
                    properties
                        .entry(field.property.clone())
                        .or_default()
                        .push(value);
                    mapped_paths
                        .entry(&field.closure_key)
                        .or_default()
                        .insert(&field.json_pointer);
                }
                None if field.required => {
                    return Err(fhir_error(
                        "horis:mappingLoss",
                        &field.closure_key,
                        &format!("required path {} is missing", field.json_pointer),
                    ));
                }
                None => {}
            }
        }

        let mut losses = Vec::new();
        for (key, resource) in &resources {
            let known = mapped_paths.get(key.as_str()).cloned().unwrap_or_default();
            for path in leaf_paths(resource) {
                if !matches!(path.as_str(), "/resourceType" | "/id" | "/meta/versionId")
                    && !known.contains(path.as_str())
                {
                    losses.push(format!("horis:mappingLoss ignored {key}{path}"));
                }
            }
        }
        losses.sort();

        Ok(ExternalSnapshot {
            snapshot_id: request.snapshot_id.clone(),
            closure,
            versions: SourceVersionVector::new(versions)?,
            profile: self.config.profile.clone(),
            mapping: self.config.mapping.clone(),
            terminology: self.config.terminology.clone(),
            package: self.config.package.clone(),
            supported_subset: self.config.supported_subset.clone(),
            facts: vec![ExternalFact {
                source: "FHIR R4 declared closure".into(),
                subject: self.config.fact_subject.clone(),
                class: self.config.fact_class.clone(),
                properties,
            }],
            losses,
        })
    }
}

impl<T: FhirTransport> FhirAdapter<T> {
    fn hydrate_snapshot(
        &self,
        request: &FhirHydrationRequest,
    ) -> Result<ExternalSnapshot, HostError> {
        let fetch = self
            .transport
            .fetch(&request.closure)
            .map_err(transport_error)?;
        self.map_fetch(request, fetch)
    }

    fn commit_effects(&mut self, effects: &ProposedEffectSet) -> Result<CommitReceipt, HostError> {
        let resource_names = effects
            .source_versions
            .entries()
            .iter()
            .map(|version| version.resource.clone())
            .collect::<Vec<_>>();
        let current = SourceVersionVector::new(
            self.transport
                .current_versions(&resource_names)
                .map_err(transport_error)?,
        )?;
        for hydrated in effects.source_versions.entries() {
            let Some(now) = current
                .entries()
                .iter()
                .find(|now| now.resource == hydrated.resource)
            else {
                return Err(missing_version_error(hydrated));
            };
            if !same_version(hydrated, now) {
                return Err(drift_error(hydrated, now));
            }
        }
        if current.entries().len() != effects.source_versions.entries().len() {
            return Err(HostError::boundary(
                "horis:sourceVersion current vector differs from the hydrated closure; commit aborted",
                "horis:sourceVersion",
                effects.proof_ref.clone(),
                Some("conflicted".into()),
                "source version vector cardinality changed",
            ));
        }

        let entries = effects
            .effects
            .iter()
            .flat_map(|effect| {
                let resource = effect
                    .payload
                    .get("resource")
                    .cloned()
                    .unwrap_or(Value::Null);
                let provenance = effect
                    .payload
                    .get("provenance")
                    .cloned()
                    .unwrap_or(Value::Null);
                [
                    json!({
                        "fullUrl": proposal_full_url(&effect.id),
                        "resource": resource,
                        "request": {
                            "method": "POST",
                            "url": effect.target.split('/').next().unwrap_or(""),
                            "ifNoneExist": format!(
                                "identifier={}|{}",
                                IDEMPOTENCY_IDENTIFIER_SYSTEM,
                                effect.idempotency_key
                            )
                        }
                    }),
                    json!({
                        "resource": provenance,
                        "request": {"method": "POST", "url": "Provenance"}
                    }),
                ]
            })
            .collect::<Vec<_>>();
        let transaction = json!({
            "resourceType": "Bundle",
            "type": "transaction",
            "entry": entries
        });
        let receipt_id = self
            .transport
            .conditional_transaction(&transaction, &effects.source_versions)
            .map_err(|error| {
                if matches!(error.status, Some(409 | 412)) {
                    HostError::boundary(
                        format!("horis:sourceVersion drift during conditional commit: {}; commit aborted", error.detail),
                        "horis:sourceVersion",
                        effects.proof_ref.clone(),
                        Some("conflicted".into()),
                        "FHIR 409 and 412 are equivalent source drift",
                    )
                } else {
                    transport_error(error)
                }
            })?;
        Ok(CommitReceipt {
            receipt_id,
            source_versions: current,
            effects: effects
                .effects
                .iter()
                .map(|effect| EffectReceipt {
                    effect_id: effect.id.clone(),
                    target: effect.target.clone(),
                    state: EffectCommitState::Accepted,
                    evidence: Some("FHIR conditional transaction accepted".into()),
                })
                .collect(),
        })
    }
}

impl<T: FhirTransport> HostAdapter for FhirAdapter<T> {
    type HydrationRequest = FhirHydrationRequest;

    fn hydrate(&self, request: &Self::HydrationRequest) -> Result<ExternalSnapshot, HostError> {
        self.hydrate_snapshot(request)
    }

    fn commit(&mut self, effects: &ProposedEffectSet) -> Result<CommitReceipt, HostError> {
        let authorized = self.authorize_commit(effects)?;
        self.commit_authorized(&authorized)
    }
}

const PROPOSAL_TAG_SYSTEM: &str = "https://w3id.org/horismos/proposal-state";
const PROOF_IDENTIFIER_SYSTEM: &str = "https://w3id.org/horismos/proof";
const IDEMPOTENCY_IDENTIFIER_SYSTEM: &str = "https://w3id.org/horismos/idempotency";

/// Build the only effect envelope accepted by the FHIR conditional-write path.
pub fn proposed_fhir_effect(
    id: impl Into<String>,
    target: impl Into<String>,
    idempotency_key: impl Into<String>,
    proof_ref: impl Into<String>,
    mut resource: Value,
    reversibility: Reversibility,
) -> Result<Effect, HostError> {
    let id = id.into();
    let target = target.into();
    let idempotency_key = idempotency_key.into();
    let proof_ref = proof_ref.into();
    let proposal_ref = proposal_full_url(&id);
    reject_modifier_extensions(&resource, &target)?;
    let object = resource.as_object_mut().ok_or_else(|| {
        fhir_error(
            "horis:proposedResource",
            &target,
            "FHIR proposal must be a JSON resource object",
        )
    })?;
    let meta = object.entry("meta").or_insert_with(|| json!({}));
    let meta = meta.as_object_mut().ok_or_else(|| {
        fhir_error(
            "horis:proposedResource",
            &target,
            "resource meta must be an object",
        )
    })?;
    let tags = meta.entry("tag").or_insert_with(|| json!([]));
    let tags = tags.as_array_mut().ok_or_else(|| {
        fhir_error(
            "horis:proposedResource",
            &target,
            "resource meta.tag must be an array",
        )
    })?;
    tags.push(json!({"system": PROPOSAL_TAG_SYSTEM, "code": "proposed"}));
    let identifiers = object.entry("identifier").or_insert_with(|| json!([]));
    let identifiers = identifiers.as_array_mut().ok_or_else(|| {
        fhir_error(
            "horis:proposedResource",
            &target,
            "resource identifier must be an array",
        )
    })?;
    identifiers.push(json!({
        "system": IDEMPOTENCY_IDENTIFIER_SYSTEM,
        "value": idempotency_key
    }));
    let provenance = json!({
        "resourceType": "Provenance",
        "id": format!("{}-proof", id),
        "target": [{"reference": proposal_ref}],
        "entity": [{
            "role": "source",
            "what": {"identifier": {"system": PROOF_IDENTIFIER_SYSTEM, "value": proof_ref}}
        }]
    });
    Ok(Effect {
        id,
        target: target.clone(),
        reversibility,
        idempotency_key,
        payload: json!({"resource": resource, "provenance": provenance}),
        verification: EffectVerification {
            postcondition: Some(format!(
                "{target} exists with its Horismos proof provenance"
            )),
            verify_before_retry: true,
        },
    })
}

fn validate_proposed_effect(
    effect: &Effect,
    proof_ref: &str,
    config: &AdapterConfig,
) -> Result<(), HostError> {
    let resource = effect.payload.get("resource").ok_or_else(|| {
        fhir_error(
            "horis:authorizationGranted",
            &effect.id,
            "proof-linked proposed resource envelope is required",
        )
    })?;
    if effect.payload.as_object().map_or(0, serde_json::Map::len) != 2 {
        return Err(fhir_error(
            "horis:authorizationGranted",
            &effect.id,
            "proposal envelope supports exactly resource and provenance",
        ));
    }
    reject_modifier_extensions(resource, &effect.target)?;
    let resource_type = required_string(resource, "/resourceType", &effect.target)?;
    let resource_id = required_string(resource, "/id", &effect.target)?;
    if effect.target != format!("{resource_type}/{resource_id}")
        || !config.allowed_write_resource_types.contains(resource_type)
    {
        return Err(fhir_error(
            "horis:authorizationGranted",
            &effect.id,
            "proposal target is outside the configured FHIR write scope",
        ));
    }
    let labeled = resource
        .pointer("/meta/tag")
        .and_then(Value::as_array)
        .is_some_and(|tags| {
            tags.iter().any(|tag| {
                tag.get("system").and_then(Value::as_str) == Some(PROPOSAL_TAG_SYSTEM)
                    && tag.get("code").and_then(Value::as_str) == Some("proposed")
            })
        });
    let idempotency_linked = resource
        .get("identifier")
        .and_then(Value::as_array)
        .is_some_and(|identifiers| {
            identifiers.iter().any(|identifier| {
                identifier.get("system").and_then(Value::as_str)
                    == Some(IDEMPOTENCY_IDENTIFIER_SYSTEM)
                    && identifier.get("value").and_then(Value::as_str)
                        == Some(effect.idempotency_key.as_str())
            })
        });
    let provenance = effect.payload.get("provenance");
    if let Some(provenance) = provenance {
        reject_modifier_extensions(provenance, &effect.id)?;
    }
    let provenance_shape = provenance.and_then(Value::as_object).is_some_and(|object| {
        object.len() == 4
            && object.get("resourceType").and_then(Value::as_str) == Some("Provenance")
            && object
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| !id.is_empty())
            && object
                .get("target")
                .and_then(Value::as_array)
                .is_some_and(|targets| targets.len() == 1)
            && object
                .get("entity")
                .and_then(Value::as_array)
                .is_some_and(|entities| entities.len() == 1)
    });
    let proof_linked = provenance
        .and_then(|value| value.pointer("/entity/0/what/identifier/value"))
        .and_then(Value::as_str)
        == Some(proof_ref);
    let proof_system = provenance
        .and_then(|value| value.pointer("/entity/0/what/identifier/system"))
        .and_then(Value::as_str)
        == Some(PROOF_IDENTIFIER_SYSTEM);
    let target_linked = provenance
        .and_then(|value| value.pointer("/target/0/reference"))
        .and_then(Value::as_str)
        == Some(proposal_full_url(&effect.id).as_str());
    let bounded_target = provenance
        .and_then(|value| value.pointer("/target/0"))
        .and_then(Value::as_object)
        .is_some_and(|target| target.len() == 1);
    let bounded_entity = provenance
        .and_then(|value| value.pointer("/entity/0"))
        .and_then(Value::as_object)
        .is_some_and(|entity| {
            entity.len() == 2
                && entity.get("role").and_then(Value::as_str) == Some("source")
                && entity
                    .get("what")
                    .and_then(Value::as_object)
                    .is_some_and(|what| what.len() == 1)
                && entity
                    .get("what")
                    .and_then(|what| what.get("identifier"))
                    .and_then(Value::as_object)
                    .is_some_and(|identifier| identifier.len() == 2)
        });
    if !labeled
        || !idempotency_linked
        || !provenance_shape
        || !proof_linked
        || !proof_system
        || !target_linked
        || !bounded_target
        || !bounded_entity
    {
        return Err(fhir_error(
            "horis:authorizationGranted",
            &effect.id,
            "proposal label and Provenance proof/target links are required",
        ));
    }
    Ok(())
}

fn proposal_full_url(effect_id: &str) -> String {
    format!("urn:horismos:{effect_id}")
}

pub fn normalize_etag(etag: &str) -> &str {
    etag.strip_prefix("W/").unwrap_or(etag).trim_matches('"')
}

fn same_version(left: &SourceVersion, right: &SourceVersion) -> bool {
    left.resource == right.resource
        && left.version_id == right.version_id
        && left.etag.as_deref().map(normalize_etag) == right.etag.as_deref().map(normalize_etag)
}

fn version_vector_matches(
    expected: &[SourceVersion],
    current: &BTreeMap<String, SourceVersion>,
) -> bool {
    expected.len() == current.len()
        && expected.iter().all(|version| {
            current
                .get(&version.resource)
                .is_some_and(|now| same_version(version, now))
        })
}

fn bundle_resources(bundle: &Value) -> BTreeMap<String, &Value> {
    bundle
        .get("entry")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("resource"))
        .filter_map(|resource| {
            let resource_type = resource.get("resourceType")?.as_str()?;
            let id = resource.get("id")?.as_str()?;
            Some((format!("{resource_type}/{id}"), resource))
        })
        .collect()
}

fn reject_modifier_extensions(value: &Value, resource: &str) -> Result<(), HostError> {
    match value {
        Value::Object(fields) => {
            if fields.contains_key("modifierExtension") {
                return Err(HostError::boundary(
                    format!(
                        "horis:mappingLoss rejected {resource}: modifierExtension is unsupported"
                    ),
                    "horis:mappingLoss",
                    resource,
                    Some("unsupported".into()),
                    "FHIR modifier semantics cannot be discarded",
                ));
            }
            for child in fields.values() {
                reject_modifier_extensions(child, resource)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                reject_modifier_extensions(child, resource)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn required_string<'a>(
    value: &'a Value,
    pointer: &str,
    resource: &str,
) -> Result<&'a str, HostError> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            fhir_error(
                "horis:sourceVersion",
                resource,
                &format!("{pointer} is required"),
            )
        })
}

fn external_value(
    value: &Value,
    kind: FhirValueKind,
    resource: &str,
    path: &str,
) -> Result<ExternalValue, HostError> {
    let mapped = match kind {
        FhirValueKind::Boolean => value.as_bool().map(ExternalValue::Boolean),
        FhirValueKind::Integer => value.as_i64().map(ExternalValue::Integer),
        FhirValueKind::String => value
            .as_str()
            .map(|value| ExternalValue::String(value.into())),
        FhirValueKind::Reference => value
            .as_str()
            .map(|value| ExternalValue::Reference(value.into())),
    };
    mapped.ok_or_else(|| {
        fhir_error(
            "horis:mappingLoss",
            resource,
            &format!("{path} has the wrong FHIR value shape"),
        )
    })
}

fn leaf_paths(value: &Value) -> Vec<String> {
    fn visit(value: &Value, prefix: &str, out: &mut Vec<String>) {
        match value {
            Value::Object(fields) => {
                for (key, child) in fields {
                    visit(child, &format!("{prefix}/{key}"), out);
                }
            }
            Value::Array(values) => {
                for (index, child) in values.iter().enumerate() {
                    visit(child, &format!("{prefix}/{index}"), out);
                }
            }
            _ => out.push(prefix.into()),
        }
    }
    let mut paths = Vec::new();
    visit(value, "", &mut paths);
    paths
}

fn drift_error(hydrated: &SourceVersion, current: &SourceVersion) -> HostError {
    HostError::boundary(
        format!(
            "horis:sourceVersion drift on {}: hydrated {}, current {}; commit aborted",
            hydrated.resource, hydrated.version_id, current.version_id
        ),
        "horis:sourceVersion",
        hydrated.resource.clone(),
        Some("conflicted".into()),
        "source changed after hydration",
    )
}

fn missing_version_error(hydrated: &SourceVersion) -> HostError {
    HostError::boundary(
        format!(
            "horis:sourceVersion drift on {}: hydrated {}, current missing; commit aborted",
            hydrated.resource, hydrated.version_id
        ),
        "horis:sourceVersion",
        hydrated.resource.clone(),
        Some("conflicted".into()),
        "current source version vector is incomplete",
    )
}

fn fhir_error(constraint: &str, subject: &str, detail: &str) -> HostError {
    HostError::boundary(
        format!("{constraint} rejected {subject}: {detail}"),
        constraint,
        subject,
        Some("rejected".into()),
        detail,
    )
}

fn transport_error(error: FhirTransportError) -> HostError {
    HostError::boundary(
        format!("horis:fhirTransport rejected: {}", error.detail),
        "horis:fhirTransport",
        "FHIR endpoint",
        error.status.map(|status| status.to_string()),
        error.detail,
    )
}
