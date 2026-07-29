//! Snapshots, structural diffs, and the snapshot binary form (U8).
//!
//! A [`Snapshot`] is an immutable, O(1)-to-take point-in-time view of a
//! [`World`]'s instance store — `world.snapshot()` costs a `Vector`/`OrdMap`
//! clone (persistent structural sharing, decision D4) plus an `Arc` bump for
//! the interner, none of which touches the store's contents. See
//! [`crate::world::World`]'s `interner` field for why the world's [`Interner`]
//! had to move behind an `Arc` to make that true: a bare `Interner::clone`
//! deep-copies its arena, which would make every `snapshot()` an O(n) copy.
//!
//! [`World::diff`] compares a live world against an earlier snapshot of
//! itself (or of a world that shares structure with it) fact-by-fact, using
//! `imbl`'s own `diff()` at both the instance-map and the per-property-map
//! level so shared subtrees are skipped rather than walked (`OrdMap::diff`
//! short-circuits on `ptr_eq` and shares cursors past identical nodes).
//!
//! [`Snapshot::to_bytes`]/[`Snapshot::from_bytes`] are the postcard binary
//! form (decision D4): a small versioned header, then the fact set with
//! *resolved* IRI strings and lexical values — never raw `Iri` tokens, which
//! are only meaningful relative to the interner that minted them and so
//! cannot survive a process boundary. `from_bytes` re-validates every
//! restored fact through the same gate a live assertion goes through
//! (`Provenance::Api`) — "the validation gate has no back door" (CLAUDE.md)
//! applies to bytes on disk exactly as it does to a caller's `assert_*` call,
//! so a tampered snapshot cannot smuggle an invalid fact into a world.
//! `World::from_snapshot`, by contrast, is in-memory reconstruction of a
//! snapshot this same process already validated (its own `snapshot()`, or the
//! result of a `from_bytes` that already ran the gate) — it trusts that and
//! skips re-validation.
//!
//! Spec: `docs/framework/03-world.md` § Snapshots · Owner: PRP-001

use std::sync::Arc;

use imbl::OrdMap;
use imbl::ordmap::DiffItem;
use serde::{Deserialize, Serialize};

use crate::error::HorismosError;
use crate::ontology::Ontology;
use crate::ontology::term::{Datatype, Interner, Iri, Ref, Value};
use crate::world::gate::{self, Subject};
use crate::world::{Instance, Provenance, SKOLEM_NAMESPACE, World};

/// Identifies the byte stream as a Horismos snapshot ("HSNP") before any
/// version-specific decoding is attempted.
const SNAPSHOT_MAGIC: [u8; 4] = *b"HSNP";
/// The wire format this build writes and reads. Bump on any breaking change
/// to [`WireSnapshot`]'s shape; [`Snapshot::from_bytes`] rejects every other
/// version rather than guess at compatibility.
const SNAPSHOT_VERSION: u16 = 1;
/// `magic` (4 bytes) + `version` (2 bytes, little-endian).
const HEADER_LEN: usize = SNAPSHOT_MAGIC.len() + 2;

// -- Snapshot ----------------------------------------------------------------

/// An immutable, point-in-time view of a [`World`]'s instance store. Cheap to
/// take ([`World::snapshot`]) and cheap to clone — both are persistent
/// structural sharing plus an `Arc` bump, never a deep copy.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub(super) instances: OrdMap<Iri, Instance>,
    pub(super) next_skolem: u64,
    pub(super) interner: Arc<Interner>,
}

impl Snapshot {
    /// Serializes to postcard bytes behind a small versioned header (decision
    /// D4): `to_bytes`/`from_bytes` are the wire form doc 03 calls "a compact
    /// binary form for replay". Every `Iri` is written out resolved (its
    /// string), and every literal value in its XSD lexical form — an
    /// interned token means nothing outside the process that minted it — so
    /// the bytes are portable across processes, not just across worlds
    /// sharing one interner.
    pub fn to_bytes(&self) -> Vec<u8> {
        let wire = WireSnapshot {
            next_skolem: self.next_skolem,
            instances: self
                .instances
                .iter()
                .map(|(&subject, instance)| WireInstance {
                    subject: self.interner.resolve(subject).to_owned(),
                    class: self.interner.resolve(instance.class()).to_owned(),
                    properties: instance
                        .properties()
                        .map(|property| {
                            let values = instance
                                .values(property)
                                .map(|v| WireValue::from_value(v, &self.interner))
                                .collect();
                            (self.interner.resolve(property).to_owned(), values)
                        })
                        .collect(),
                })
                .collect(),
        };
        encode(&wire)
    }

    /// Decodes `bytes` produced by [`Snapshot::to_bytes`] (or any
    /// process — the format is portable), rejecting an unrecognized magic
    /// header or an unsupported format version before attempting to decode
    /// the payload at all.
    ///
    /// Every restored fact is re-validated through `ontology`'s gate with
    /// [`Provenance::Api`] — the controller pin for U8 (`docs/framework/03-world.md`):
    /// bytes are untrusted input, so a tampered snapshot (a `heartRate` above
    /// `horis:max`, say) is rejected exactly as the same assertion would be
    /// live, in the same domain terms
    /// ([`HorismosError::ValidationRejected`]). A decode failure that never
    /// reaches the gate — bad magic, an unsupported version, a corrupt
    /// postcard payload, or a literal whose lexical form doesn't parse —
    /// surfaces as [`HorismosError::SnapshotFormat`] instead.
    pub fn from_bytes(ontology: &Ontology, bytes: &[u8]) -> Result<Snapshot, HorismosError> {
        if bytes.len() < HEADER_LEN || bytes[..SNAPSHOT_MAGIC.len()] != SNAPSHOT_MAGIC {
            return Err(HorismosError::SnapshotFormat {
                reason: "not a horismos snapshot: bad or missing magic header".to_owned(),
            });
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != SNAPSHOT_VERSION {
            return Err(HorismosError::SnapshotFormat {
                reason: format!(
                    "snapshot format version {version} is not supported by this build \
                     (reads version {SNAPSHOT_VERSION})"
                ),
            });
        }
        let wire: WireSnapshot = postcard::from_bytes(&bytes[HEADER_LEN..]).map_err(|e| {
            HorismosError::SnapshotFormat {
                reason: format!("corrupt snapshot payload: {e}"),
            }
        })?;

        let mut world = World::new(ontology);
        // Restoration is two-phase because wire order is interning order,
        // not dependency order, and a legitimate world may hold reference
        // cycles: phase 1 runs every gate check except unresolvable object
        // references (deferred), phase 2 settles those against the complete
        // store. The gate's guarantee holds at `Ok`: nothing is returned
        // until every deferred reference has passed the same checks a live
        // assertion would have run.
        let mut deferred_by_subject = Vec::new();
        let mut max_restored_skolem: Option<u64> = None;
        for wi in wire.instances {
            let subject = world.intern(&wi.subject);
            let class = world.intern(&wi.class);
            let mut properties = Vec::new();
            for (property, values) in wi.properties {
                let property = world.intern(&property);
                for value in values {
                    properties.push((property, value.into_value(&mut world)?));
                }
            }
            if let Some(index) = wi
                .subject
                .strip_prefix(SKOLEM_NAMESPACE)
                .and_then(|suffix| suffix.parse::<u64>().ok())
            {
                max_restored_skolem = Some(max_restored_skolem.map_or(index, |m| m.max(index)));
            }
            let deferred = world.restore_instance(subject, class, properties)?;
            if !deferred.is_empty() {
                deferred_by_subject.push((subject, class, deferred));
            }
        }
        for (subject, class, deferred) in deferred_by_subject {
            for d in deferred {
                gate::check_restored_reference(&world, Provenance::Api, subject, class, &d)
                    .map_err(|witness| HorismosError::ValidationRejected { witness })?;
            }
        }

        // The counter is world state like any other: a wire counter at or
        // below a restored skolem subject would let a later, legitimate
        // anonymous assertion re-mint that subject's IRI and silently merge
        // into it — the exact corruption check 1's reserved-namespace rule
        // exists to prevent. Refuse the snapshot rather than repair it:
        // bytes that inconsistent were tampered with or corrupted.
        if let Some(max) = max_restored_skolem
            && wire.next_skolem <= max
        {
            return Err(HorismosError::SnapshotFormat {
                reason: format!(
                    "snapshot skolem counter {} is not beyond its own restored \
                     instance {SKOLEM_NAMESPACE}{max} — the snapshot is corrupt \
                     or tampered with",
                    wire.next_skolem
                ),
            });
        }
        world.next_skolem = wire.next_skolem;
        Ok(world.snapshot())
    }
}

/// Serializes `wire` to postcard bytes behind the magic+version header —
/// shared by [`Snapshot::to_bytes`] and this module's own tests, which build
/// a [`WireSnapshot`] directly to exercise `from_bytes`'s gate re-validation
/// against payload content no legitimate `to_bytes` call would ever produce
/// (see the "tampered" test below).
fn encode(wire: &WireSnapshot) -> Vec<u8> {
    let body = postcard::to_allocvec(wire)
        .expect("a Snapshot's wire form always serializes: plain data, no cycles");
    let mut out = Vec::with_capacity(HEADER_LEN + body.len());
    out.extend_from_slice(&SNAPSHOT_MAGIC);
    out.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
    out.extend_from_slice(&body);
    out
}

// -- Diff ----------------------------------------------------------------

/// One fact a [`Diff`] reports as added or removed. Class assignment counts
/// as a fact for diff purposes (doc 03/U8 brief): an instance created since
/// the snapshot appears as its `Class` fact plus one `Property` fact per
/// asserted value, and a fully-retracted instance's disappearance would
/// appear the same way in `removed` — though full-instance retraction is not
/// itself possible yet (retraction is per-fact only, U6).
#[derive(Debug, Clone, PartialEq)]
pub enum Fact {
    /// `subject`'s class, at assertion.
    Class { subject: Iri, class: Iri },
    /// One `(subject, property, value)` triple.
    Property {
        subject: Iri,
        property: Iri,
        value: Value,
    },
}

/// The added/removed facts between a [`World`] and an earlier [`Snapshot`] of
/// it ([`World::diff`]). Deterministic order: subjects in `Iri`-token order,
/// each subject's own facts class-first then properties in `Iri`-token order,
/// each property's values in assertion order — the same order every other
/// store iteration in this crate commits to.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Diff {
    added: Vec<Fact>,
    removed: Vec<Fact>,
}

impl Diff {
    /// Facts present now that were not present at the snapshot.
    pub fn added(&self) -> &[Fact] {
        &self.added
    }

    /// Facts present at the snapshot that are not present now.
    pub fn removed(&self) -> &[Fact] {
        &self.removed
    }
}

/// Every fact `instance` (named by `subject`) carries — its class first, then
/// each property's values in assertion order — appended to `out`. Used for a
/// subject that appears on only one side of a [`Diff`] (a whole instance
/// added, or one removed).
fn facts_of(subject: Iri, instance: &Instance, out: &mut Vec<Fact>) {
    out.push(Fact::Class {
        subject,
        class: instance.class(),
    });
    for property in instance.properties() {
        for value in instance.values(property) {
            out.push(Fact::Property {
                subject,
                property,
                value: value.clone(),
            });
        }
    }
}

/// The property-level half of a [`Diff`] for a subject present on both sides
/// (an `imbl::ordmap::DiffItem::Update`): diffs `old`'s and `new`'s property
/// maps with `imbl`'s own `OrdMap::diff` (sharing shortcuts included), then —
/// for a property whose `Vector<Value>` changed rather than appearing or
/// disappearing whole — the per-value set difference in each side's own
/// assertion order.
///
/// A subject's class is immutable once asserted (the gate's `ClassConflict`
/// check, U6) — `old` and `new` always agree on it, so no `Fact::Class` is
/// ever emitted here.
fn diff_properties(
    subject: Iri,
    old: &Instance,
    new: &Instance,
    added: &mut Vec<Fact>,
    removed: &mut Vec<Fact>,
) {
    debug_assert_eq!(
        old.class(),
        new.class(),
        "the gate enforces a subject's class as immutable once asserted"
    );
    for item in old.properties.diff(&new.properties) {
        match item {
            DiffItem::Add(&property, values) => {
                for value in values {
                    added.push(Fact::Property {
                        subject,
                        property,
                        value: value.clone(),
                    });
                }
            }
            DiffItem::Remove(&property, values) => {
                for value in values {
                    removed.push(Fact::Property {
                        subject,
                        property,
                        value: value.clone(),
                    });
                }
            }
            DiffItem::Update {
                old: (&property, old_values),
                new: (_, new_values),
            } => {
                for value in new_values {
                    if !old_values.contains(value) {
                        added.push(Fact::Property {
                            subject,
                            property,
                            value: value.clone(),
                        });
                    }
                }
                for value in old_values {
                    if !new_values.contains(value) {
                        removed.push(Fact::Property {
                            subject,
                            property,
                            value: value.clone(),
                        });
                    }
                }
            }
        }
    }
}

// -- World: snapshot / from_snapshot / diff / restore -----------------------

impl World {
    /// Captures the world's current instance store as an immutable
    /// [`Snapshot`] — O(1): an `OrdMap` clone (persistent structural sharing,
    /// decision D4) plus an `Arc` bump for the interner. Nothing here walks
    /// the store.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            instances: self.instances.clone(),
            next_skolem: self.next_skolem,
            interner: Arc::clone(&self.interner),
        }
    }

    /// Reconstructs a live world from `snapshot`. Also O(1) — the returned
    /// world *shares* structure with `snapshot` until one of them mutates, at
    /// which point `imbl`'s copy-on-write (for the instance store) and
    /// `Arc::make_mut`'s clone-on-write (for the interner, in
    /// [`World::intern`]) each copy exactly the piece that changed, leaving
    /// `snapshot` and any other world sharing it untouched either way.
    ///
    /// Trusts `snapshot` — no re-validation. A `Snapshot` value only ever
    /// comes from this same process's own [`World::snapshot`] or from
    /// [`Snapshot::from_bytes`], and `from_bytes` already ran every fact
    /// through the gate; re-checking here would just repeat that work.
    pub fn from_snapshot(ontology: &Ontology, snapshot: Snapshot) -> World {
        World {
            ontology: ontology.clone(),
            interner: snapshot.interner,
            instances: snapshot.instances,
            next_skolem: snapshot.next_skolem,
        }
    }

    /// The added/removed facts between this world and an earlier `snapshot`
    /// of it (doc 03: `world.diff(s)`). Compares the instance maps with
    /// `imbl`'s own `OrdMap::diff`, which skips subtrees the two maps still
    /// share by pointer rather than walking every entry — efficient in the
    /// common case (a handful of transactions since the snapshot) without
    /// giving up correctness for the uncommon one (everything changed).
    pub fn diff(&self, snapshot: &Snapshot) -> Diff {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        for item in snapshot.instances.diff(&self.instances) {
            match item {
                DiffItem::Add(&subject, instance) => facts_of(subject, instance, &mut added),
                DiffItem::Remove(&subject, instance) => facts_of(subject, instance, &mut removed),
                DiffItem::Update {
                    old: (&subject, old),
                    new: (_, new),
                } => diff_properties(subject, old, new, &mut added, &mut removed),
            }
        }
        Diff { added, removed }
    }

    /// Restores one instance from a snapshot's wire form: runs the *same*
    /// gate a live assertion would (`Provenance::Api`), so a fact that could
    /// never have been asserted legitimately cannot be smuggled back in
    /// through bytes ([`Snapshot::from_bytes`]'s controller pin). The one
    /// deviation is deliberate and temporary: an object reference whose
    /// referent is not yet in the store is returned as a
    /// [`gate::DeferredRef`] for `from_bytes`'s post-restore pass instead of
    /// being refused — wire order is interning order, not dependency order.
    ///
    /// A restored subject may legitimately fall inside [`SKOLEM_NAMESPACE`] —
    /// it was minted there by the *original* world's own counter — so this
    /// tags it [`Subject::Skolem`] rather than [`Subject::Named`], the one
    /// place outside `assert_instance_as`'s anonymous path that constructs
    /// that variant. That skips check 1's caller-may-not-squat rule (which
    /// exists to keep *callers* out of the space the world mints into, not to
    /// keep the world from repopulating its own past assertions) while still
    /// running every other check.
    fn restore_instance(
        &mut self,
        subject: Iri,
        class: Iri,
        properties: Vec<(Iri, Value)>,
    ) -> Result<Vec<gate::DeferredRef>, HorismosError> {
        let kind = if self.resolve(subject).starts_with(SKOLEM_NAMESPACE) {
            Subject::Skolem(subject)
        } else {
            Subject::Named(subject)
        };
        let (instance, deferred) =
            gate::check_assertion_restoring(self, Provenance::Api, kind, class, &properties)
                .map_err(|witness| HorismosError::ValidationRejected { witness })?;
        self.instances.insert(subject, instance);
        Ok(deferred)
    }
}

// -- Wire format ---------------------------------------------------------

/// The postcard-serialized payload (after the magic+version header). Every
/// `Iri` is a resolved string and every literal a lexical form — see the
/// module doc for why raw tokens can't travel in this format.
#[derive(Debug, Serialize, Deserialize)]
struct WireSnapshot {
    next_skolem: u64,
    instances: Vec<WireInstance>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WireInstance {
    subject: String,
    class: String,
    properties: Vec<(String, Vec<WireValue>)>,
}

/// One value, portable across processes: `Integer`/`String`/`Boolean` travel
/// as themselves, `Decimal`/`DateTime`/`Duration` as their XSD lexical form
/// (round-tripped through [`Value::lexical_form`]/[`Value::parse_typed`],
/// the same functions `ontology::term`'s own round-trip property tests
/// pin), and `Ref` as the referent's resolved IRI, re-interned on load.
#[derive(Debug, Serialize, Deserialize)]
enum WireValue {
    Decimal(String),
    Integer(i64),
    String(String),
    Boolean(bool),
    DateTime(String),
    Duration(String),
    Ref(String),
}

impl WireValue {
    fn from_value(value: &Value, interner: &Interner) -> WireValue {
        match value {
            Value::Decimal(_) => WireValue::Decimal(
                value
                    .lexical_form()
                    .expect("every non-Ref value has a lexical form"),
            ),
            Value::Integer(i) => WireValue::Integer(*i),
            Value::String(s) => WireValue::String(s.clone()),
            Value::Boolean(b) => WireValue::Boolean(*b),
            Value::DateTime(_) => WireValue::DateTime(
                value
                    .lexical_form()
                    .expect("every non-Ref value has a lexical form"),
            ),
            Value::Duration(_) => WireValue::Duration(
                value
                    .lexical_form()
                    .expect("every non-Ref value has a lexical form"),
            ),
            Value::Ref(r) => WireValue::Ref(interner.resolve(r.iri()).to_owned()),
        }
    }

    /// Reconstructs a [`Value`], interning a `Ref`'s IRI into `world` fresh —
    /// the one case that needs a live world rather than a bare `Interner`.
    fn into_value(self, world: &mut World) -> Result<Value, HorismosError> {
        let parse = |lexical: &str, datatype: Datatype| {
            Value::parse_typed(lexical, datatype).map_err(|e| HorismosError::SnapshotFormat {
                reason: e.domain_text(),
            })
        };
        match self {
            WireValue::Decimal(s) => parse(&s, Datatype::Decimal),
            WireValue::Integer(i) => Ok(Value::Integer(i)),
            WireValue::String(s) => Ok(Value::String(s)),
            WireValue::Boolean(b) => Ok(Value::Boolean(b)),
            WireValue::DateTime(s) => parse(&s, Datatype::DateTime),
            WireValue::Duration(s) => parse(&s, Datatype::Duration),
            WireValue::Ref(iri) => Ok(Value::Ref(Ref::new(world.intern(&iri)))),
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;
    use crate::world::gate::ConstraintKind;

    const PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    /// A world over an inline ontology spanning every literal datatype plus
    /// an object property — `:` = `https://example.org/t#`.
    fn spread_world() -> World {
        let onto = Ontology::load_from_str(
            &format!(
                "{PRELUDE}\
                 :Patient a rdfs:Class .\n\
                 :Encounter a rdfs:Class .\n\
                 :heartRate a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:decimal ;\n\
                 \x20   horis:min 0 ; horis:max 300 ; horis:unit \"bpm\" ; horis:cardinality \"0..1\" .\n\
                 :age a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:integer .\n\
                 :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string .\n\
                 :admitted a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean .\n\
                 :seenAt a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:dateTime .\n\
                 :stayLength a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:duration .\n\
                 :seenIn a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Encounter ;\n\
                 \x20   horis:cardinality \"0..1\" .\n"
            ),
            |_| None,
        )
        .expect("the spread ontology must load");
        World::new(&onto)
    }

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    /// A world over `examples/triage/triage.ttl`, for the tests that need its
    /// real `horis:max`/`horis:unit` wording.
    fn triage_world() -> World {
        let onto = Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load");
        World::new(&onto)
    }

    fn tri(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/triage#{local}"))
    }

    /// A value rendered as text comparable *across* two worlds' interners —
    /// `Value::PartialEq` alone isn't safe for that, since a `Value::Ref`'s
    /// `Iri` token is only meaningful relative to the interner that minted
    /// it (two independently-built worlds over the same ontology need not
    /// assign instance IRIs the same tokens even when they hold the same
    /// facts).
    fn render(world: &World, value: &Value) -> String {
        match value {
            Value::Ref(r) => world.resolve(r.iri()).to_owned(),
            other => other
                .lexical_form()
                .expect("every non-Ref value has a lexical form"),
        }
    }

    fn rendered_values(world: &World, subject: Ref, property: Iri) -> Vec<String> {
        world
            .values(subject, property)
            .map(|v| render(world, v))
            .collect()
    }

    // -- snapshot() is O(1)/immutable ---------------------------------------

    #[test]
    fn snapshot_is_unaffected_by_later_mutation_of_the_source_world() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p1 = tri(&mut world, "p1");
        let p = world
            .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(80))])
            .expect("accepted");

        let snapshot = world.snapshot();

        world
            .assert_named_instance(p.iri(), patient, [(heart_rate, Value::Integer(150))])
            .expect("re-assertion must succeed");
        world
            .assert_instance(patient, [(heart_rate, Value::Integer(60))])
            .expect("a second, unrelated instance");

        let onto = world.ontology().clone();
        let restored = World::from_snapshot(&onto, snapshot);
        assert_eq!(
            restored.instance_count(),
            1,
            "the snapshot must not see the instance asserted after it was taken"
        );
        assert_eq!(
            rendered_values(&restored, p, heart_rate),
            vec!["80".to_owned()],
            "the snapshot must not see the mutation to the instance it already held"
        );
    }

    // -- from_snapshot independence ------------------------------------------

    #[test]
    fn from_snapshot_independence_mutating_the_copy_does_not_touch_the_original() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p1 = tri(&mut world, "p1");
        let p = world
            .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(80))])
            .expect("accepted");

        let onto = world.ontology().clone();
        let mut copy = World::from_snapshot(&onto, world.snapshot());
        let p_text = copy.resolve(p.iri()).to_owned();
        let p_in_copy = Ref::new(copy.intern(&p_text));
        copy.assert_named_instance(
            p_in_copy.iri(),
            patient,
            [(heart_rate, Value::Integer(200))],
        )
        .expect("mutating the reconstructed world must succeed");

        assert_eq!(
            rendered_values(&world, p, heart_rate),
            vec!["80".to_owned()],
            "the original world must be untouched by mutating the copy"
        );
        assert_eq!(
            rendered_values(&copy, p_in_copy, heart_rate),
            vec!["200".to_owned()]
        );
    }

    #[test]
    fn from_snapshot_independence_mutating_the_original_does_not_touch_the_copy() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p1 = tri(&mut world, "p1");
        let p = world
            .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(80))])
            .expect("accepted");

        let onto = world.ontology().clone();
        let copy = World::from_snapshot(&onto, world.snapshot());

        world
            .assert_named_instance(p.iri(), patient, [(heart_rate, Value::Integer(200))])
            .expect("mutating the original must succeed");

        assert_eq!(
            rendered_values(&copy, p, heart_rate),
            vec!["80".to_owned()],
            "the copy must be untouched by mutating the original"
        );
    }

    // -- diff -----------------------------------------------------------------

    #[test]
    fn diff_is_empty_when_nothing_changed() {
        let world = spread_world();
        let snapshot = world.snapshot();
        let diff = world.diff(&snapshot);
        assert!(diff.added().is_empty());
        assert!(diff.removed().is_empty());
    }

    #[test]
    fn diff_reports_a_new_instance_as_its_class_fact_plus_its_property_facts() {
        let mut world = spread_world();
        let snapshot = world.snapshot();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(72))])
            .expect("accepted");

        let diff = world.diff(&snapshot);
        assert!(diff.removed().is_empty());
        assert_eq!(
            diff.added(),
            &[
                Fact::Class {
                    subject: p.iri(),
                    class: patient,
                },
                Fact::Property {
                    subject: p.iri(),
                    property: heart_rate,
                    value: Value::Integer(72),
                },
            ]
        );
    }

    #[test]
    fn diff_reports_a_retraction_as_removed() {
        let mut world = spread_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let mrn = t(&mut world, "mrn");
        let p = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(72)),
                    (mrn, Value::String("AB-1".to_owned())),
                ],
            )
            .expect("accepted");

        let snapshot = world.snapshot();
        world
            .retract(p, heart_rate, &Value::Integer(72))
            .expect("retraction must succeed");

        let diff = world.diff(&snapshot);
        assert!(diff.added().is_empty());
        assert_eq!(
            diff.removed(),
            &[Fact::Property {
                subject: p.iri(),
                property: heart_rate,
                value: Value::Integer(72),
            }]
        );
    }

    #[test]
    fn diff_reports_a_replaced_single_valued_property_as_removed_and_added() {
        let mut world = spread_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let p1 = t(&mut world, "p1");
        let p = world
            .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(72))])
            .expect("accepted");

        let snapshot = world.snapshot();
        world
            .assert_named_instance(p.iri(), patient, [(heart_rate, Value::Integer(90))])
            .expect("replacement must succeed");

        let diff = world.diff(&snapshot);
        assert_eq!(
            diff.removed(),
            &[Fact::Property {
                subject: p.iri(),
                property: heart_rate,
                value: Value::Integer(72),
            }]
        );
        assert_eq!(
            diff.added(),
            &[Fact::Property {
                subject: p.iri(),
                property: heart_rate,
                value: Value::Integer(90),
            }]
        );
    }

    // -- to_bytes / from_bytes round trip -------------------------------------

    #[test]
    fn bytes_round_trip_preserves_every_datatype_and_object_reference() {
        let mut world = spread_world();
        let patient = t(&mut world, "Patient");
        let encounter = t(&mut world, "Encounter");
        let heart_rate = t(&mut world, "heartRate");
        let age = t(&mut world, "age");
        let mrn = t(&mut world, "mrn");
        let admitted = t(&mut world, "admitted");
        let seen_at = t(&mut world, "seenAt");
        let stay_length = t(&mut world, "stayLength");
        let seen_in = t(&mut world, "seenIn");

        let e1_iri = t(&mut world, "e1");
        let e1 = world
            .assert_named_instance(e1_iri, encounter, [])
            .expect("the encounter must assert");
        let p1_iri = t(&mut world, "p1");
        // Looked up again below by resolved text (`p2`), not by this `Ref` —
        // subject identity across two independently-built worlds' interners
        // can only be compared that way (see `render`'s doc comment).
        let _p = world
            .assert_named_instance(
                p1_iri,
                patient,
                [
                    (heart_rate, Value::Decimal(Decimal::new(725, 1))),
                    (age, Value::Integer(42)),
                    (mrn, Value::String("AB-1234".to_owned())),
                    (admitted, Value::Boolean(true)),
                    (
                        seen_at,
                        Value::parse_typed("2024-04-12T13:20:00-05:00", Datatype::DateTime)
                            .expect("valid dateTime"),
                    ),
                    (
                        stay_length,
                        Value::parse_typed("P1DT2H", Datatype::Duration).expect("valid duration"),
                    ),
                    (seen_in, Value::Ref(e1)),
                ],
            )
            .expect("accepted");
        // Also cover an anonymous (skolem-named) instance in the same
        // snapshot, so restoration into the skolem namespace is exercised.
        let anon = world
            .assert_instance(patient, [(age, Value::Integer(7))])
            .expect("accepted");

        let bytes = world.snapshot().to_bytes();
        let onto = world.ontology().clone();
        let restored_snapshot =
            Snapshot::from_bytes(&onto, &bytes).expect("a legitimate snapshot's bytes must decode");
        let restored = World::from_snapshot(&onto, restored_snapshot);

        assert_eq!(restored.instance_count(), world.instance_count());

        let p2 = Ref::new(
            restored
                .interner()
                .get("https://example.org/t#p1")
                .expect("p1 must resolve"),
        );
        assert_eq!(
            rendered_values(&restored, p2, heart_rate),
            vec!["72.5".to_owned()]
        );
        assert_eq!(rendered_values(&restored, p2, age), vec!["42".to_owned()]);
        assert_eq!(
            rendered_values(&restored, p2, mrn),
            vec!["AB-1234".to_owned()]
        );
        assert_eq!(
            rendered_values(&restored, p2, admitted),
            vec!["true".to_owned()]
        );
        assert_eq!(
            rendered_values(&restored, p2, seen_at),
            vec!["2024-04-12T13:20:00-05:00".to_owned()]
        );
        assert_eq!(
            rendered_values(&restored, p2, stay_length),
            vec!["P1DT2H".to_owned()]
        );
        assert_eq!(
            rendered_values(&restored, p2, seen_in),
            vec!["https://example.org/t#e1".to_owned()]
        );

        let anon_text = world.resolve(anon.iri()).to_owned();
        let anon2 = Ref::new(
            restored
                .interner()
                .get(&anon_text)
                .expect("the skolem-named instance must resolve under the same IRI text"),
        );
        assert_eq!(rendered_values(&restored, anon2, age), vec!["7".to_owned()]);
        let restored_patient = restored
            .interner()
            .get("https://example.org/t#Patient")
            .expect("Patient must resolve");
        assert_eq!(
            restored.instance(anon2).map(Instance::class),
            Some(restored_patient)
        );
    }

    #[test]
    fn bytes_round_trip_yields_an_equal_diff_against_an_empty_snapshot() {
        // A structural cross-check on the round trip above: the restored
        // world's *diff against a fresh empty world's snapshot* must name
        // the same resolved facts as the original's, even though the two
        // worlds' `Iri` tokens differ.
        let mut world = spread_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let p1 = t(&mut world, "p1");
        world
            .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(72))])
            .expect("accepted");

        let onto = world.ontology().clone();
        let empty_original = World::new(&onto).snapshot();
        let original_diff = world.diff(&empty_original);

        let bytes = world.snapshot().to_bytes();
        let restored = World::from_snapshot(
            &onto,
            Snapshot::from_bytes(&onto, &bytes).expect("must decode"),
        );
        let empty_restored = World::new(&onto).snapshot();
        let restored_diff = restored.diff(&empty_restored);

        let render_fact = |world: &World, fact: &Fact| -> String {
            match fact {
                Fact::Class { subject, class } => {
                    format!("{} a {}", world.resolve(*subject), world.resolve(*class))
                }
                Fact::Property {
                    subject,
                    property,
                    value,
                } => format!(
                    "{} {} {}",
                    world.resolve(*subject),
                    world.resolve(*property),
                    render(world, value)
                ),
            }
        };
        let original_added: Vec<String> = original_diff
            .added()
            .iter()
            .map(|f| render_fact(&world, f))
            .collect();
        let restored_added: Vec<String> = restored_diff
            .added()
            .iter()
            .map(|f| render_fact(&restored, f))
            .collect();
        assert_eq!(original_added, restored_added);
    }

    // -- version-header rejection ----------------------------------------------

    #[test]
    fn from_bytes_rejects_a_bad_magic_header() {
        let onto = spread_world().ontology().clone();
        let err = Snapshot::from_bytes(&onto, &[0u8; 16]).expect_err("bad magic must be rejected");
        assert!(matches!(err, HorismosError::SnapshotFormat { .. }));
        assert!(err.domain_text().contains("magic"), "{}", err.domain_text());
    }

    #[test]
    fn from_bytes_rejects_an_unsupported_version() {
        let onto = spread_world().ontology().clone();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&SNAPSHOT_MAGIC);
        bytes.extend_from_slice(&99u16.to_le_bytes());
        let err = Snapshot::from_bytes(&onto, &bytes)
            .expect_err("an unsupported version must be rejected");
        assert!(matches!(err, HorismosError::SnapshotFormat { .. }));
        let text = err.domain_text();
        assert!(text.contains("99"), "{text}");
        assert!(text.contains("version"), "{text}");
    }

    #[test]
    fn from_bytes_rejects_a_corrupt_payload() {
        let onto = spread_world().ontology().clone();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&SNAPSHOT_MAGIC);
        bytes.extend_from_slice(&SNAPSHOT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&[0xFF, 0xFF, 0xFF]);
        let err =
            Snapshot::from_bytes(&onto, &bytes).expect_err("garbage payload must be rejected");
        assert!(matches!(err, HorismosError::SnapshotFormat { .. }));
    }

    // -- tampered payload rejected by the gate ----------------------------------

    #[test]
    fn from_bytes_re_validates_through_the_gate_in_the_documented_wording() {
        // A payload no legitimate `to_bytes` call would ever produce — built
        // directly from the wire types (this module's own private test
        // access) rather than by patching serialized bytes, since postcard's
        // varint encoding makes the latter both fragile and no more honest a
        // test of "bytes are untrusted input" than this is.
        let onto = triage_world().ontology().clone();
        let wire = WireSnapshot {
            next_skolem: 0,
            instances: vec![WireInstance {
                subject: "https://example.org/triage#p1".to_owned(),
                class: "https://example.org/triage#Patient".to_owned(),
                properties: vec![(
                    "https://example.org/triage#heartRate".to_owned(),
                    vec![WireValue::Integer(1280)],
                )],
            }],
        };
        let bytes = encode(&wire);

        let err = Snapshot::from_bytes(&onto, &bytes)
            .expect_err("1280 exceeds triage.ttl's horis:max 300 and must be rejected");
        match err {
            HorismosError::ValidationRejected { witness } => {
                assert_eq!(
                    witness.kind,
                    ConstraintKind::Max {
                        bound: "300".to_owned(),
                        unit: Some("bpm".to_owned()),
                    }
                );
                assert_eq!(
                    witness.text,
                    "heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"
                );
            }
            other => panic!("expected ValidationRejected, got {other:?}"),
        }
    }

    #[test]
    fn restoring_a_squatted_named_subject_still_runs_check_1s_class_declaration_rule() {
        // restore_instance only special-cases the reserved-namespace rule,
        // not the rest of check 1: an undeclared class must still be
        // rejected even though the subject looks skolem-shaped.
        let onto = triage_world().ontology().clone();
        let wire = WireSnapshot {
            next_skolem: 1,
            instances: vec![WireInstance {
                subject: "https://w3id.org/horismos/.well-known/genid/0".to_owned(),
                class: "https://example.org/triage#Ghost".to_owned(),
                properties: vec![],
            }],
        };
        let bytes = encode(&wire);
        let err = Snapshot::from_bytes(&onto, &bytes).expect_err(":Ghost is not declared");
        match err {
            HorismosError::ValidationRejected { witness } => {
                assert_eq!(witness.kind, ConstraintKind::ClassNotDeclared);
            }
            other => panic!("expected ValidationRejected, got {other:?}"),
        }
    }

    #[test]
    fn a_rolled_back_skolem_counter_is_rejected_as_corrupt() {
        // A wire counter at or below a restored skolem subject would let a
        // later legitimate anonymous assertion re-mint that subject's IRI
        // and silently merge into it — the corruption check 1's reserved-
        // namespace rule exists to prevent, reopened through the wire field.
        let onto = triage_world().ontology().clone();
        let wire = WireSnapshot {
            next_skolem: 2,
            instances: vec![WireInstance {
                subject: format!("{SKOLEM_NAMESPACE}3"),
                class: "https://example.org/triage#Patient".to_owned(),
                properties: vec![],
            }],
        };
        let bytes = encode(&wire);
        let err = Snapshot::from_bytes(&onto, &bytes)
            .expect_err("a counter behind its own skolem subject must be refused");
        match err {
            HorismosError::SnapshotFormat { reason } => {
                assert!(
                    reason.contains("skolem counter 2"),
                    "reason must name the counter: {reason}"
                );
                assert!(
                    reason.contains(&format!("{SKOLEM_NAMESPACE}3")),
                    "reason must name the conflicting subject: {reason}"
                );
            }
            other => panic!("expected SnapshotFormat, got {other:?}"),
        }
    }

    #[test]
    fn restoration_tolerates_an_object_reference_ahead_of_its_referent_in_wire_order() {
        // Wire order is interning order, not dependency order: a subject
        // may reference an instance that appears later in the wire. The
        // two-phase restore defers the reference and settles it once the
        // store is complete.
        let onto = spread_world().ontology().clone();
        let wire = WireSnapshot {
            next_skolem: 0,
            instances: vec![
                WireInstance {
                    subject: "https://example.org/t#p1".to_owned(),
                    class: "https://example.org/t#Patient".to_owned(),
                    properties: vec![(
                        "https://example.org/t#seenIn".to_owned(),
                        vec![WireValue::Ref("https://example.org/t#e1".to_owned())],
                    )],
                },
                WireInstance {
                    subject: "https://example.org/t#e1".to_owned(),
                    class: "https://example.org/t#Encounter".to_owned(),
                    properties: vec![],
                },
            ],
        };
        let bytes = encode(&wire);
        let snapshot = Snapshot::from_bytes(&onto, &bytes)
            .expect("a forward reference within the same snapshot is legitimate");
        let mut world = World::from_snapshot(&onto, snapshot);
        let p1 = t(&mut world, "p1");
        let seen_in = t(&mut world, "seenIn");
        let e1 = t(&mut world, "e1");
        assert_eq!(
            world.value(Ref::new(p1), seen_in),
            Some(&Value::Ref(Ref::new(e1)))
        );
    }

    #[test]
    fn a_dangling_object_reference_in_a_snapshot_is_still_rejected() {
        // Deferral is not forgiveness: a reference that no restored instance
        // (and no declared class) ever satisfies fails the post-restore pass
        // with the same witness a live assertion would have produced.
        let onto = spread_world().ontology().clone();
        let wire = WireSnapshot {
            next_skolem: 0,
            instances: vec![WireInstance {
                subject: "https://example.org/t#p1".to_owned(),
                class: "https://example.org/t#Patient".to_owned(),
                properties: vec![(
                    "https://example.org/t#seenIn".to_owned(),
                    vec![WireValue::Ref("https://example.org/t#nowhere".to_owned())],
                )],
            }],
        };
        let bytes = encode(&wire);
        let err = Snapshot::from_bytes(&onto, &bytes)
            .expect_err("a reference nothing in the snapshot satisfies must be refused");
        match err {
            HorismosError::ValidationRejected { witness } => {
                assert_eq!(witness.kind, ConstraintKind::UnknownReference);
            }
            other => panic!("expected ValidationRejected, got {other:?}"),
        }
    }
}
