//! Format-neutral snapshot/effect values and the host adapter contract.
//!
//! Spec: `docs/framework/03-world.md`, `07-tools.md`, and
//! `14-architecture.md` · Owner: PRP-008.

use std::collections::{BTreeMap, BTreeSet};

use horismos_core::{Datatype, Ontology, Ref, Value, World};
use serde::{Deserialize, Deserializer, Serialize};

use crate::assumptions::AssumptionManifest;
use crate::error::HostError;

/// Whether one declared member of the closed input boundary was available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClosureState {
    Present,
    VerifiedAbsent,
    NotAuthorized,
    NotFetched,
}

impl ClosureState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::VerifiedAbsent => "verified-absent",
            Self::NotAuthorized => "not-authorized",
            Self::NotFetched => "not-fetched",
        }
    }

    fn satisfies_required(self) -> bool {
        matches!(self, Self::Present | Self::VerifiedAbsent)
    }
}

/// One explicit member/query in a decision's declared input closure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureMember {
    pub key: String,
    pub required: bool,
    pub state: ClosureState,
}

/// Stable identity of a profile, mapping, terminology expansion, or package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIdentity {
    pub canonical: String,
    pub version: String,
    pub hash: String,
}

impl ArtifactIdentity {
    pub fn new(
        canonical: impl Into<String>,
        version: impl Into<String>,
        hash: impl Into<String>,
    ) -> Self {
        Self {
            canonical: canonical.into(),
            version: version.into(),
            hash: hash.into(),
        }
    }
}

/// Exact identity of one source object at hydration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceVersion {
    pub resource: String,
    pub version_id: String,
    pub etag: Option<String>,
}

/// A sorted, duplicate-free read set used by conditional commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SourceVersionVector(Vec<SourceVersion>);

impl SourceVersionVector {
    pub fn new(mut versions: Vec<SourceVersion>) -> Result<Self, HostError> {
        versions.sort_by(|left, right| left.resource.cmp(&right.resource));
        let mut seen = BTreeSet::new();
        for version in &versions {
            if version.resource.is_empty() || version.version_id.is_empty() {
                return Err(HostError::new(
                    "horis:sourceVersion requires a resource and versionId",
                    "horis:sourceVersion",
                    version.resource.clone(),
                    None,
                    "resource and versionId must be non-empty",
                ));
            }
            if !seen.insert(version.resource.clone()) {
                return Err(HostError::new(
                    format!(
                        "horis:sourceVersion names {} more than once",
                        version.resource
                    ),
                    "horis:sourceVersion",
                    version.resource.clone(),
                    None,
                    "duplicate source resource",
                ));
            }
        }
        Ok(Self(versions))
    }

    pub fn entries(&self) -> &[SourceVersion] {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SourceVersionVector {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let versions = Vec::<SourceVersion>::deserialize(deserializer)?;
        Self::new(versions).map_err(|error| serde::de::Error::custom(error.domain_text()))
    }
}

/// Offline terminology expansion evidence required by D8.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TerminologyPackage {
    canonical: String,
    code_system_version: String,
    expansion_timestamp: String,
    parameters: BTreeMap<String, String>,
    hash: String,
}

impl<'de> Deserialize<'de> for TerminologyPackage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            canonical: String,
            code_system_version: String,
            expansion_timestamp: String,
            parameters: BTreeMap<String, String>,
            hash: String,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(
            wire.canonical,
            wire.code_system_version,
            wire.expansion_timestamp,
            wire.parameters,
            wire.hash,
        )
        .map_err(|error| serde::de::Error::custom(error.domain_text()))
    }
}

impl TerminologyPackage {
    pub fn new(
        canonical: impl Into<String>,
        code_system_version: impl Into<String>,
        expansion_timestamp: impl Into<String>,
        parameters: BTreeMap<String, String>,
        hash: impl Into<String>,
    ) -> Result<Self, HostError> {
        let package = Self {
            canonical: canonical.into(),
            code_system_version: code_system_version.into(),
            expansion_timestamp: expansion_timestamp.into(),
            parameters,
            hash: hash.into(),
        };
        if package.canonical.is_empty()
            || package.code_system_version.is_empty()
            || package.expansion_timestamp.is_empty()
            || package.hash.is_empty()
            || !package.parameters.contains_key("check-system-version")
        {
            return Err(HostError::new(
                format!(
                    "horis:terminologyPackage {} requires a code-system version, expansion timestamp, hash, and check-system-version parameter",
                    package.canonical
                ),
                "horis:terminologyPackage",
                package.canonical,
                Some("incomplete".into()),
                "missing pinned terminology expansion evidence",
            ));
        }
        Ok(package)
    }

    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    pub fn code_system_version(&self) -> &str {
        &self.code_system_version
    }

    pub fn expansion_timestamp(&self) -> &str {
        &self.expansion_timestamp
    }

    pub fn parameters(&self) -> &BTreeMap<String, String> {
        &self.parameters
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }
}

/// A serializable core value before it is interned into one World.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum ExternalValue {
    Decimal(String),
    Integer(i64),
    String(String),
    Boolean(bool),
    DateTime(String),
    Duration(String),
    Reference(String),
}

impl ExternalValue {
    fn into_core(self, world: &mut World) -> Result<Value, HostError> {
        let parsed = match self {
            Self::Decimal(value) => Value::parse_typed(&value, Datatype::Decimal),
            Self::Integer(value) => return Ok(Value::Integer(value)),
            Self::String(value) => return Ok(Value::String(value)),
            Self::Boolean(value) => return Ok(Value::Boolean(value)),
            Self::DateTime(value) => Value::parse_typed(&value, Datatype::DateTime),
            Self::Duration(value) => Value::parse_typed(&value, Datatype::Duration),
            Self::Reference(value) => return Ok(Value::Ref(Ref::new(world.intern(&value)))),
        };
        parsed.map_err(|error| {
            HostError::new(
                format!("horis:externalValue rejected: {}", error.domain_text()),
                "horis:externalValue",
                "literal",
                None,
                error.domain_text(),
            )
        })
    }
}

/// One source-attributed typed instance in an external snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalFact {
    pub source: String,
    pub subject: String,
    pub class: String,
    pub properties: BTreeMap<String, Vec<ExternalValue>>,
}

/// A complete format-neutral hydration, ready for atomic admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalSnapshot {
    pub snapshot_id: String,
    pub closure: Vec<ClosureMember>,
    pub versions: SourceVersionVector,
    pub profile: ArtifactIdentity,
    pub mapping: ArtifactIdentity,
    pub terminology: Vec<TerminologyPackage>,
    pub package: ArtifactIdentity,
    pub supported_subset: Vec<String>,
    pub facts: Vec<ExternalFact>,
    pub losses: Vec<String>,
}

impl ExternalSnapshot {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ExternalSnapshot contains only serializable data")
    }

    pub fn assumptions(&self, unverified_effects: Vec<String>) -> AssumptionManifest {
        AssumptionManifest {
            input_closure: self.closure.clone(),
            external_versions: self.versions.clone(),
            mapping_version: self.mapping.version.clone(),
            supported_subset: self.supported_subset.clone(),
            unverified_effects,
        }
    }

    /// Builds and returns a World only after the whole snapshot is admitted.
    pub fn admit(&self, ontology: &Ontology) -> Result<AdmittedSnapshot, HostError> {
        for member in &self.closure {
            if member.required && !member.state.satisfies_required() {
                return Err(HostError::new(
                    format!(
                        "horis:inputClosure requires {}, but it is {}",
                        member.key,
                        member.state.as_str()
                    ),
                    "horis:inputClosure",
                    member.key.clone(),
                    Some(member.state.as_str().to_owned()),
                    "required closure member is incomplete",
                ));
            }
        }

        let mut facts_by_subject = BTreeMap::new();
        for fact in &self.facts {
            if facts_by_subject
                .insert(fact.subject.clone(), fact)
                .is_some()
            {
                return Err(HostError::new(
                    format!(
                        "horis:externalSnapshot names {} more than once",
                        fact.subject
                    ),
                    "horis:externalSnapshot",
                    fact.subject.clone(),
                    Some("duplicate".into()),
                    "duplicate external fact subject",
                ));
            }
        }
        let subjects: BTreeSet<&str> = facts_by_subject.keys().map(String::as_str).collect();
        let mut remaining: BTreeMap<&str, BTreeSet<&str>> = facts_by_subject
            .iter()
            .map(|(subject, fact)| {
                let dependencies = fact
                    .properties
                    .values()
                    .flatten()
                    .filter_map(|value| match value {
                        ExternalValue::Reference(target) if subjects.contains(target.as_str()) => {
                            Some(target.as_str())
                        }
                        _ => None,
                    })
                    .collect();
                (subject.as_str(), dependencies)
            })
            .collect();
        let mut ordered = Vec::with_capacity(self.facts.len());
        while !remaining.is_empty() {
            let ready: Vec<&str> = remaining
                .iter()
                .filter(|(_, dependencies)| dependencies.is_empty())
                .map(|(subject, _)| *subject)
                .collect();
            if ready.is_empty() {
                let cycle = remaining.keys().copied().collect::<Vec<_>>().join(" -> ");
                return Err(HostError::new(
                    format!("horis:externalSnapshot reference cycle: {cycle}"),
                    "horis:externalSnapshot",
                    cycle,
                    Some("reference-cycle".into()),
                    "cyclic object references cannot be admitted through the World gate",
                ));
            }
            for subject in ready {
                remaining.remove(subject);
                for dependencies in remaining.values_mut() {
                    dependencies.remove(subject);
                }
                ordered.push(facts_by_subject[subject]);
            }
        }

        let mut world = World::new(ontology);
        for fact in ordered {
            let subject = world.intern(&fact.subject);
            let class = world.intern(&fact.class);
            let mut properties = Vec::new();
            for (property, values) in &fact.properties {
                let property = world.intern(property);
                for value in values.clone() {
                    let value = value.into_core(&mut world).map_err(|error| {
                        HostError::new(
                            format!(
                                "horis:externalSnapshot rejected {}: {}",
                                fact.source,
                                error.domain_text()
                            ),
                            "horis:externalSnapshot",
                            fact.source.clone(),
                            Some("rejected".into()),
                            error.domain_text(),
                        )
                    })?;
                    properties.push((property, value));
                }
            }
            if let Err(error) = world.assert_named_instance(subject, class, properties) {
                let detail = error.domain_text();
                return Err(HostError::new(
                    format!("horis:externalSnapshot rejected {}: {detail}", fact.source),
                    "horis:externalSnapshot",
                    fact.source.clone(),
                    Some("rejected".into()),
                    detail,
                ));
            }
        }
        Ok(AdmittedSnapshot {
            external: self.clone(),
            world,
        })
    }
}

/// A snapshot and its fully admitted fixed-point World.
#[derive(Debug, Clone)]
pub struct AdmittedSnapshot {
    external: ExternalSnapshot,
    world: World,
}

impl AdmittedSnapshot {
    pub fn external(&self) -> &ExternalSnapshot {
        &self.external
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

/// Commit/retry semantics of an external effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reversibility {
    Bufferable,
    Reversible,
    Irreversible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectVerification {
    pub postcondition: Option<String>,
    pub verify_before_retry: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effect {
    pub id: String,
    pub target: String,
    pub reversibility: Reversibility,
    pub idempotency_key: String,
    pub payload: serde_json::Value,
    pub verification: EffectVerification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedEffectSet {
    pub proof_ref: String,
    pub source_versions: SourceVersionVector,
    pub effects: Vec<Effect>,
}

impl ProposedEffectSet {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("ProposedEffectSet contains serializable data")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EffectCommitState {
    Accepted,
    Conflicted,
    Failed,
    Compensated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectReceipt {
    pub effect_id: String,
    pub target: String,
    pub state: EffectCommitState,
    pub evidence: Option<String>,
}

/// Format-neutral reconciliation of every effect in one remote attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitReceipt {
    pub receipt_id: String,
    pub source_versions: SourceVersionVector,
    pub effects: Vec<EffectReceipt>,
}

/// Minimal format-neutral adapter behavior. Transports remain adapter-local.
pub trait HostAdapter {
    type HydrationRequest;

    fn hydrate(&self, request: &Self::HydrationRequest) -> Result<ExternalSnapshot, HostError>;

    fn commit(&mut self, effects: &ProposedEffectSet) -> Result<CommitReceipt, HostError>;
}
