//! The World: the typed instance store — the agent's memory of record.
//!
//! Instance graph, the validation gate (every mutation passes it; no back door),
//! snapshots with persistent structure sharing, transactions, pattern queries,
//! the SPARQL subset (behind the `sparql` feature), subscriptions.
//!
//! ## What U6 ships
//!
//! The store and the gate. A `World` is bound to exactly one [`Ontology`] and
//! holds facts as **adjacency maps** (decision D2): subject → property →
//! values, over `imbl`'s persistent maps (decision D4) so U8's `snapshot()`
//! is O(1) structural sharing rather than a copy. Every mutation — assertion
//! or retraction, whatever its provenance — goes through
//! [`gate::check_assertion`] / [`gate::check_retraction`] and is applied
//! atomically: the checks build the *candidate* instance and the store is
//! swapped only once every check has passed, so a rejected mutation cannot
//! leave a partial fact behind.
//!
//! ## What U8 adds
//!
//! [`snapshot::Snapshot`]: O(1) point-in-time views (`World::snapshot`),
//! reconstruction (`World::from_snapshot`), structural [`snapshot::Diff`]
//! (`World::diff`), and a postcard binary form with a versioned header. See
//! that module for why the world's [`Interner`] is behind an `Arc` — the
//! design constraint that makes O(1) snapshots possible without deep-copying
//! the arena on every call.
//!
//! ## What U9 adds
//!
//! Tier-1 pattern queries: [`World::instances`] (subsumption-aware) and
//! [`World::value`] (the single-valued sibling of [`World::values`]). The
//! SPARQL subset (behind the `sparql` feature) stays an empty stub —
//! PRP-003's job.
//!
//! Forward chaining and subscriptions (PRP-002/004) are not here yet;
//! [`World::assert_instance_as`] marks where the post-gate transaction stages
//! slot in.
//!
//! Spec: `docs/framework/03-world.md` · Owner: PRP-001

pub mod gate;
pub mod snapshot;

use std::sync::Arc;

use imbl::{OrdMap, Vector};

pub use snapshot::{Diff, Fact, Snapshot};

use crate::error::{HorismosError, OntologyError};
use crate::ontology::Ontology;
use crate::ontology::term::{Interner, Iri, Ref, Value};
use crate::reason::forward::{self, ReasoningRecord};
use crate::reason::matcher::MatcherKind;
use crate::reason::proof::ProofNode;

/// The namespace anonymous instances are skolemized into: RDF 1.1's
/// well-known `genid` space, under the `horis:` authority. Ids come from a
/// per-world counter that only advances when an assertion *commits*, so the
/// same sequence of operations against two fresh worlds mints exactly the
/// same IRIs — no randomness, no wall-clock (CLAUDE.md determinism rule).
///
/// The space is **reserved**: the gate refuses a caller-supplied subject IRI
/// under it ([`gate::ConstraintKind::ReservedSubject`]), because the
/// commit-only counter makes a squatter there wedge anonymous assertion
/// outright — see [`gate`]'s check 1.
pub(crate) const SKOLEM_NAMESPACE: &str = "https://w3id.org/horismos/.well-known/genid/";

/// Where a mutation came from. The validation gate matches it against each
/// property's `horis:assignedBy` policy — the property-level enforcement of
/// "the LLM never decides" (`docs/framework/02-ontology.md`).
///
/// Every mutating method takes one. The provenance-free conveniences
/// ([`World::assert_instance`], [`World::assert_named_instance`],
/// [`World::retract`]) state [`Provenance::Api`] — a direct API call is
/// exactly what they are; the `_as` forms take it explicitly, and are what
/// the perception, rule, and tool paths will call.
///
/// Serialization: JSON forms (traces, doc 09) write the doc-03 lowercase
/// name (`"perception" | "rule" | "tool" | "api"`). The postcard snapshot
/// wire form (v2) instead encodes the **variant index** — variant order is
/// part of that wire contract, so reordering variants is a snapshot-format
/// break (bump `SNAPSHOT_VERSION`), not a refactor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provenance {
    /// A transducer's typed output (doc 06). Never trusted with
    /// `horis:RulesOnly` properties.
    Perception,
    /// A rule conclusion (doc 04).
    Rule,
    /// A tool effect (doc 07).
    Tool,
    /// A direct call on the Rust/Python API — the default for the
    /// provenance-free convenience methods.
    #[default]
    Api,
}

impl Provenance {
    /// The word domain-term errors use ("… assertion came from perception").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Perception => "perception",
            Self::Rule => "a rule",
            Self::Tool => "a tool",
            Self::Api => "the api",
        }
    }
}

/// One instance: its class plus its property values.
///
/// Values are held per property in a persistent [`Vector`] in **assertion
/// order** — the order values were first accepted by the gate — rather than
/// sorted: `Value` has no total order (`XsdDuration` is only partially
/// ordered, see `ontology::term`), and assertion order is both deterministic
/// and the order a reader expects to see multi-valued properties in.
/// Duplicates are suppressed: re-asserting a value a property already holds
/// is a no-op, so each property behaves as an ordered set.
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    class: Iri,
    properties: OrdMap<Iri, Vector<Value>>,
}

impl Instance {
    /// The class this instance was asserted as.
    pub fn class(&self) -> Iri {
        self.class
    }

    /// This instance's values for `property`, in assertion order.
    pub fn values(&self, property: Iri) -> impl Iterator<Item = &Value> {
        self.properties.get(&property).into_iter().flatten()
    }

    /// Every property that holds at least one value, in `Iri`-token order.
    pub fn properties(&self) -> impl Iterator<Item = Iri> + '_ {
        self.properties.keys().copied()
    }
}

/// The typed instance store. Bound to one [`Ontology`]; single-writer by
/// design (doc 12 § Threading & determinism) — transactions are `&mut self`.
///
/// A world owns its own [`Interner`], cloned from the ontology's at
/// construction: every schema `Iri` keeps the token the schema knows it by,
/// and instance IRIs (caller-supplied or skolemized) are interned on top,
/// leaving the shared, immutable ontology untouched.
#[derive(Debug, Clone)]
pub struct World {
    ontology: Ontology,
    /// Behind an `Arc`, not held by value — the U8 design constraint.
    /// `Interner::clone` deep-copies its arena (see that type's docs), which
    /// would make [`World::snapshot`] an O(n) copy instead of the O(1)
    /// structural share decision D4 requires. `Arc` makes cloning the
    /// interner (what `snapshot()` and `World::clone` both do) a refcount
    /// bump; [`World::intern`] uses [`Arc::make_mut`] — clone-on-write — so a
    /// world that has *not* been snapshotted mutates its interner in place
    /// (the common case, no copy at all), and one that has, copies exactly
    /// once, on the first intern after the share, leaving every outstanding
    /// [`Snapshot`] untouched.
    interner: Arc<Interner>,
    /// subject → instance (decision D2's adjacency map, one level up).
    /// `OrdMap` keyed by the `Iri` token: point lookup is the gate's hot
    /// path, iteration order is token order — first-intern order, hence
    /// deterministic (CLAUDE.md) — and cloning is O(1) structural sharing,
    /// which is both what makes rejection atomic here and what
    /// [`World::snapshot`] (U8) is built on.
    instances: OrdMap<Iri, Instance>,
    /// The next skolem id. Advances only on a committed anonymous assertion.
    next_skolem: u64,
    /// Per-fact provenance (PRP-002 U4, adopting PRP-001's "stored fact
    /// provenance" gap for the in-memory world): subject → property → one
    /// [`Provenance`] per stored value, positionally parallel to the
    /// instance's value [`Vector`]. Proof-tree leaves read it ("asserted by
    /// perception" — doc 04's visible neuro-symbolic seam). Deliberately
    /// NOT part of the snapshot wire form yet — that half of the gap stays
    /// recorded (see the ledger's gap notes).
    fact_meta: OrdMap<Iri, OrdMap<Iri, Vector<Provenance>>>,
    /// (subject, property) → the recorded derivation of the standing
    /// rule-derived value (PRP-002 U4/U5): what `World::explain` replays.
    /// Base facts have no entry here.
    derivations: OrdMap<(Iri, Iri), Derivation>,
    /// The most recent transaction's chaining record — the reasoning-step
    /// payload traces consume (doc 09). `None` before the first assertion.
    last_reasoning: Option<ReasoningRecord>,
    /// The matching strategy chaining uses (PRP-002 Key decision 1's
    /// benchmark fixes the default; see [`MatcherKind`]). A pure
    /// performance choice — both strategies produce byte-identical records,
    /// and the differential tests pin that.
    matcher_kind: MatcherKind,
}

/// The recorded derivation of one standing rule-derived fact.
#[derive(Debug, Clone, PartialEq)]
pub struct Derivation {
    pub value: Value,
    /// The rule that concluded it — named by the D-5 retraction refusal.
    pub rule: Iri,
    pub proof: ProofNode,
}

impl World {
    /// A new, empty world over `ontology` (doc 12: `World::new(&onto)`).
    pub fn new(ontology: &Ontology) -> World {
        World {
            ontology: ontology.clone(),
            interner: Arc::new(ontology.interner().clone()),
            instances: OrdMap::new(),
            next_skolem: 0,
            fact_meta: OrdMap::new(),
            derivations: OrdMap::new(),
            last_reasoning: None,
            matcher_kind: MatcherKind::default(),
        }
    }

    /// Selects the matching strategy for this world's chaining
    /// ([`MatcherKind`]) — the D-1 escape valve and the differential tests'
    /// lever. Semantics are identical either way; only cost differs.
    pub fn use_matcher(&mut self, kind: MatcherKind) {
        self.matcher_kind = kind;
    }

    /// The ontology this world is bound to.
    pub fn ontology(&self) -> &Ontology {
        &self.ontology
    }

    /// This world's interner — the ontology's, plus every instance IRI
    /// minted since.
    pub fn interner(&self) -> &Interner {
        &self.interner
    }

    /// Interns `iri` into this world, returning its token: how callers name
    /// classes, properties, and instances. Interning an IRI the ontology
    /// already knows returns the ontology's own token.
    pub fn intern(&mut self, iri: &str) -> Iri {
        Arc::make_mut(&mut self.interner).intern(iri)
    }

    /// Resolves a token minted by this world (or by its ontology).
    ///
    /// # Panics
    /// Panics if `iri` came from a different world's interner.
    pub fn resolve(&self, iri: Iri) -> &str {
        self.interner.resolve(iri)
    }

    /// How many instances the world holds.
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// The instance `subject` names, if the world holds one. The raw store
    /// accessor the tier-1 pattern queries below (U9) are built on.
    pub fn instance(&self, subject: Ref) -> Option<&Instance> {
        self.instances.get(&subject.iri())
    }

    /// `subject`'s values for `property`, in assertion order; empty if the
    /// world holds no such instance or the property is unset.
    pub fn values(&self, subject: Ref, property: Iri) -> impl Iterator<Item = &Value> {
        self.instance(subject)
            .into_iter()
            .flat_map(move |i| i.values(property))
    }

    // -- Tier-1 pattern queries (U9) ----------------------------------------

    /// Every instance whose class is `class`, or a transitive
    /// `rdfs:subClassOf` descendant of it (doc 03: `world.instances(Patient)`).
    /// Subsumption-aware exactly as the gate's own domain/range checks are —
    /// [`crate::ontology::schema::Schema::is_subclass_of`] is `owl:sameAs`-bridged,
    /// so a class reached only through a `sameAs` anchor (U5) matches too.
    /// "is matches any subclass" (doc 04) applies to queries the same way it
    /// applies to assertion.
    ///
    /// Deterministic order: `Iri`-token order, the same first-intern order
    /// every other store iteration in this module commits to.
    ///
    /// No separate "direct instances only" query exists — `world.instance(r)
    /// .map(Instance::class) == Some(class)` is one comparison for a caller
    /// who wants that instead.
    pub fn instances(&self, class: Iri) -> impl Iterator<Item = Ref> + '_ {
        let schema = self.ontology.schema();
        self.instances
            .iter()
            .filter(move |(_, instance)| schema.is_subclass_of(instance.class(), class))
            .map(|(&subject, _)| Ref::new(subject))
    }

    /// `subject`'s first value for `property`, in assertion order (doc 03:
    /// `world.value(p, "heartRate")`) — `None` if the world holds no such
    /// instance or the property is unset. The single-valued sibling of
    /// [`World::values`]: for a property whose `horis:cardinality` admits at
    /// most one value this is *the* value; for a multi-valued property it is
    /// the one asserted first, not necessarily the only one.
    pub fn value(&self, subject: Ref, property: Iri) -> Option<&Value> {
        self.values(subject, property).next()
    }

    /// The tier-2 pattern query (doc 03 § Queries; PRP-003 U7, decision
    /// record `PRPs/PRP-003-python-bindings/03-decision-query.md`): every
    /// instance of `class` — subsumption-aware, exactly like
    /// [`World::instances`] — that holds, for *each* `(property, value)`
    /// pair, at least one value equal to `value` (typed [`Value`] equality;
    /// node refs compare by IRI). Conjunction only, one subject class per
    /// query, results in the store's deterministic order.
    ///
    /// A class or property the ontology does not declare is a domain-term
    /// *error*, never a silently empty result — a query cannot tell "no
    /// matches" from "you misspelled acuity" any other way.
    pub fn query_pattern(
        &self,
        class: Iri,
        patterns: &[(Iri, Value)],
    ) -> Result<Vec<Ref>, HorismosError> {
        let schema = self.ontology.schema();
        if schema.class_declared(class).is_none() {
            return Err(OntologyError::UndeclaredTerm {
                term: self.ontology.compact(self.resolve(class)),
            }
            .into());
        }
        for (property, _) in patterns {
            if schema.property(*property).is_none() {
                return Err(OntologyError::UndeclaredTerm {
                    term: self.ontology.compact(self.resolve(*property)),
                }
                .into());
            }
        }
        Ok(self
            .instances(class)
            .filter(|subject| {
                patterns
                    .iter()
                    .all(|(property, value)| self.values(*subject, *property).any(|v| v == value))
            })
            .collect())
    }

    // -- Assertion ---------------------------------------------------------

    /// Asserts an anonymous instance of `class`: the gate validates it, and
    /// on success it is stored under a freshly minted, deterministic skolem
    /// IRI (see [`SKOLEM_NAMESPACE`]). Provenance defaults to
    /// [`Provenance::Api`].
    pub fn assert_instance(
        &mut self,
        class: Iri,
        properties: impl IntoIterator<Item = (Iri, Value)>,
    ) -> Result<Ref, HorismosError> {
        self.assert_instance_as(Provenance::Api, None, class, properties)
    }

    /// Asserts an instance of `class` under the caller's own IRI. If the
    /// world already holds `subject`, the asserted properties are merged into
    /// it under the gate's cardinality rules (single-valued properties are
    /// replaced, multi-valued ones extended, `horis:immutable` ones
    /// rejected); re-asserting an existing subject as a *different* class is
    /// rejected. `subject` may not lie in the reserved [`SKOLEM_NAMESPACE`].
    /// Provenance defaults to [`Provenance::Api`].
    pub fn assert_named_instance(
        &mut self,
        subject: Iri,
        class: Iri,
        properties: impl IntoIterator<Item = (Iri, Value)>,
    ) -> Result<Ref, HorismosError> {
        self.assert_instance_as(Provenance::Api, Some(subject), class, properties)
    }

    /// The one assertion entry point — every other `assert_*` method is a
    /// thin wrapper. `subject` is `None` for an anonymous instance (skolem
    /// IRI minted here) or `Some(iri)` for a named one.
    ///
    /// One call is one transaction (doc 03 § Transactions): the five gate
    /// checks run in documented order against the *current* store and build
    /// a candidate instance; the store is swapped only after the last check
    /// passes, so a rejection leaves the world byte-identical.
    pub fn assert_instance_as(
        &mut self,
        provenance: Provenance,
        subject: Option<Iri>,
        class: Iri,
        properties: impl IntoIterator<Item = (Iri, Value)>,
    ) -> Result<Ref, HorismosError> {
        // The gate needs a token for the candidate subject, so an anonymous
        // assertion interns its skolem IRI *before* the checks run — and
        // must un-intern it on rejection: token order is observable through
        // `to_bytes` and `instances()` iteration, so a leftover slot would
        // make a rejected transaction distinguishable from one that never
        // happened. `next_skolem` — observable through the next assertion's
        // IRI — likewise advances only on commit below. Check 1 keeps
        // callers out of this namespace, so the id the counter is holding
        // is always free.
        let interner_len_before = self.interner.len();
        let subject = match subject {
            Some(iri) => gate::Subject::Named(iri),
            None => {
                let text = format!("{SKOLEM_NAMESPACE}{}", self.next_skolem);
                gate::Subject::Skolem(self.intern(&text))
            }
        };
        self.assert_as(provenance, subject, class, properties, interner_len_before)
    }

    /// Merges `properties` onto `subject`, an instance already in the world,
    /// through the same transactional five-check gate `assert_instance_as`
    /// uses — the write path for a provenance (`Provenance::Tool`) that adds
    /// facts back onto an instance the caller already holds a [`Ref`] to
    /// (returned by an earlier assertion in *this* `World`), rather than
    /// naming a new or anonymous one.
    ///
    /// `horismos-core`-internal only (`workflow::agent`'s tool-effect merge
    /// onto the case subject perception minted): unlike
    /// [`World::assert_instance_as`], this is never a public naming path, so
    /// it may safely re-tag an existing skolem-space subject as
    /// [`gate::Subject::Skolem`] — mirroring [`World::apply_rule_conclusion`]'s
    /// precedent — without reopening the squat hole check 1 exists to close
    /// for *external* callers (`gate_properties.rs`'s
    /// `a_rejected_operation_leaves_no_trace` pins that a caller naming a
    /// skolem IRI through `assert_named_instance` is refused regardless of
    /// whether it already exists). Panics if `subject` is not yet in the
    /// world — every call site holds a `Ref` this same `World` minted.
    pub(crate) fn assert_on(
        &mut self,
        provenance: Provenance,
        subject: Ref,
        properties: impl IntoIterator<Item = (Iri, Value)>,
    ) -> Result<Ref, HorismosError> {
        let interner_len_before = self.interner.len();
        let class = self
            .instance(subject)
            .map(Instance::class)
            .expect("assert_on's caller holds a Ref this World returned from an earlier assertion");
        let kind = if self.resolve(subject.iri()).starts_with(SKOLEM_NAMESPACE) {
            gate::Subject::Skolem(subject.iri())
        } else {
            gate::Subject::Named(subject.iri())
        };
        self.assert_as(provenance, kind, class, properties, interner_len_before)
    }

    /// The shared transactional body behind [`World::assert_instance_as`]
    /// and [`World::assert_on`]: gate check, commit, truth maintenance,
    /// chain to the fixed point. `interner_len_before` is captured by the
    /// caller, before it computes `subject` — an anonymous mint interns its
    /// skolem text as part of that computation, and a rejection must roll
    /// that back too.
    fn assert_as(
        &mut self,
        provenance: Provenance,
        subject: gate::Subject,
        class: Iri,
        properties: impl IntoIterator<Item = (Iri, Value)>,
        interner_len_before: usize,
    ) -> Result<Ref, HorismosError> {
        let properties: Vec<(Iri, Value)> = properties.into_iter().collect();
        // The transaction checkpoint (all O(1) — persistent maps + counters):
        // the iteration-cap tripwire below must roll the WHOLE transaction
        // back, base facts included, so a caller never observes a
        // half-chained world (doc 03 § Transactions).
        let checkpoint_instances = self.instances.clone();
        let checkpoint_meta = self.fact_meta.clone();
        let checkpoint_derivations = self.derivations.clone();
        let checkpoint_skolem = self.next_skolem;
        let subject_iri = subject.iri();

        let instance = match gate::check_assertion(self, provenance, subject, class, &properties) {
            Ok(instance) => instance,
            Err(witness) => {
                if self.interner.len() > interner_len_before {
                    Arc::make_mut(&mut self.interner).truncate_to(interner_len_before);
                }
                return Err(HorismosError::ValidationRejected { witness });
            }
        };

        // Commit the base facts.
        let old = self.instances.get(&subject_iri).cloned();
        self.instances.insert(subject_iri, instance);
        self.update_fact_meta(subject_iri, old.as_ref(), &properties, provenance);
        // Only a genuinely new mint (never a merge onto an already-existing
        // subject — `assert_on`'s Skolem-tagged merges included, `old` is
        // always `Some` there) advances the counter.
        if matches!(subject, gate::Subject::Skolem(_)) && old.is_none() {
            self.next_skolem += 1;
        }

        // Truth maintenance (doc 03 § Truth maintenance, decided per
        // `tm-derived-facts.md`): reset every currently-standing rule-derived
        // value before chaining, so the fixed point below is recomputed from
        // this transaction's *final* base facts alone — never left standing
        // from whatever an earlier transaction happened to derive.
        self.retract_all_derived();

        // Forward chain to the fixed point (PRP-002 U4, doc 03's transaction
        // stage between gate and commit-visibility). Every conclusion passes
        // the same gate with Provenance::Rule; a refused conclusion is a
        // recorded diagnostic, not a rollback — but the iteration-cap
        // tripwire rolls the whole transaction back. Subscriber notification
        // and the runtime trace append remain PRP-004's slots.
        let ontology = self.ontology.clone();
        let mut matcher = self.matcher_kind.build(self, ontology.rules());
        match forward::chain(self, matcher.as_mut()) {
            Ok(record) => self.last_reasoning = Some(record),
            Err(err) => {
                self.instances = checkpoint_instances;
                self.fact_meta = checkpoint_meta;
                self.derivations = checkpoint_derivations;
                self.next_skolem = checkpoint_skolem;
                if self.interner.len() > interner_len_before {
                    Arc::make_mut(&mut self.interner).truncate_to(interner_len_before);
                }
                return Err(err);
            }
        }
        Ok(Ref::new(subject_iri))
    }

    // -- The reasoner's write path (PRP-002 U4) -----------------------------

    /// Asserts one rule conclusion on an *existing* instance through the
    /// full five-check gate with [`Provenance::Rule`] — the reasoner's only
    /// write path; there is no back door. Does NOT re-enter chaining (the
    /// caller *is* the chaining loop).
    pub(crate) fn apply_rule_conclusion(
        &mut self,
        subject: Ref,
        property: Iri,
        value: Value,
    ) -> Result<(), Box<gate::Witness>> {
        let class = self
            .instance(subject)
            .map(Instance::class)
            .expect("chaining only ever matches instances the world holds");
        // A skolem-space subject is the world's own anonymous instance — tag
        // it as such so check 1's caller-may-not-squat rule (which exists to
        // keep *callers* out) does not refuse the world's own conclusion.
        let kind = if self.resolve(subject.iri()).starts_with(SKOLEM_NAMESPACE) {
            gate::Subject::Skolem(subject.iri())
        } else {
            gate::Subject::Named(subject.iri())
        };
        let properties = vec![(property, value)];
        let instance = gate::check_assertion(self, Provenance::Rule, kind, class, &properties)?;
        let old = self.instances.get(&subject.iri()).cloned();
        self.instances.insert(subject.iri(), instance);
        self.update_fact_meta(subject.iri(), old.as_ref(), &properties, Provenance::Rule);
        Ok(())
    }

    /// Truth maintenance (doc 03 § Truth maintenance): strips every
    /// currently-standing rule-derived value from every instance.
    ///
    /// The (subject, property) pairs to reset come from `derivations`, not
    /// from scanning `fact_meta` for a `Provenance::Rule` tag: `derivations`
    /// is the gate's own definition of "derived" ([`gate::check_retraction`]
    /// keys its refusal off it too), and the two are *not* interchangeable —
    /// [`World::assert_instance_as`] and [`World::apply_rule_conclusion`]
    /// both admit [`Provenance::Rule`] as a stated provenance on a direct
    /// call, without that call being the reasoner's own write path, so a
    /// fact can carry the Rule tag without ever being recorded as a
    /// derivation. Once a (subject, property) pair *is* in `derivations`,
    /// every value fact_meta tags `Rule` at that pair is stripped — not just
    /// the one value `derivations` last recorded — so a multi-valued
    /// `horis:RulesOnly` property does not leave earlier conclusions behind
    /// just because a later one overwrote the single recorded explanation.
    ///
    /// Called at the top of every external transaction's chaining stage
    /// (both [`World::assert_instance_as`] and [`World::retract_as`]) so the
    /// `forward::chain` call that follows rebuilds the fixed point from this
    /// transaction's final base facts alone. This is the "retract-derived +
    /// rechain" approach: a full reset, not incremental invalidation — see
    /// the PRP-002 decision record for the cost tradeoff.
    ///
    /// This does not go through [`gate::check_retraction`] — not a back
    /// door, because every value it removes was itself only ever admitted
    /// by an earlier gate-checked `Provenance::Rule` conclusion, removal
    /// alone cannot violate a domain/range/pattern/oneOf/cardinality-maximum
    /// constraint (`check_retraction`'s own doc makes the same argument for
    /// the public retraction path), and every value that lands afterward —
    /// via `forward::chain` → `apply_rule_conclusion` — passes the full gate
    /// again before it is visible. The transient, mid-transaction state in
    /// between is not observable outside this method, same as the
    /// checkpoint/rollback pattern above it.
    fn retract_all_derived(&mut self) {
        for (subject, property) in self.derivations.keys().copied().collect::<Vec<_>>() {
            let Some(instance) = self.instances.get(&subject).cloned() else {
                continue;
            };
            let values: Vec<Value> = instance.values(property).cloned().collect();
            let provenances: Vector<Provenance> = self
                .fact_meta
                .get(&subject)
                .and_then(|props| props.get(&property))
                .cloned()
                .unwrap_or_default();
            let mut kept_values = Vector::new();
            let mut kept_meta = Vector::new();
            for (index, value) in values.into_iter().enumerate() {
                let prov = provenances.get(index).copied().unwrap_or(Provenance::Api);
                if prov != Provenance::Rule {
                    kept_values.push_back(value);
                    kept_meta.push_back(prov);
                }
            }
            let mut new_instance = instance;
            if kept_values.is_empty() {
                new_instance.properties.remove(&property);
            } else {
                new_instance.properties.insert(property, kept_values);
            }
            self.instances.insert(subject, new_instance);
            self.fact_meta
                .entry(subject)
                .or_default()
                .insert(property, kept_meta);
        }
        self.derivations = OrdMap::new();
    }

    /// Records the derivation behind a standing rule-derived fact — what
    /// [`World::explain`] replays. A later derivation for the same
    /// (subject, property) replaces the record, mirroring the value it
    /// explains.
    pub(crate) fn record_derivation(
        &mut self,
        subject: Ref,
        property: Iri,
        value: Value,
        rule: Iri,
        proof: ProofNode,
    ) {
        self.derivations
            .insert((subject.iri(), property), Derivation { value, rule, proof });
    }

    /// Rebuilds the per-value provenance vector for every property touched
    /// by a mutation, position-matching kept values so their recorded
    /// provenance survives replaces, extends, and retractions.
    fn update_fact_meta(
        &mut self,
        subject: Iri,
        old: Option<&Instance>,
        touched: &[(Iri, Value)],
        provenance: Provenance,
    ) {
        let new_instance = match self.instances.get(&subject) {
            Some(instance) => instance.clone(),
            None => return,
        };
        let mut properties: Vec<Iri> = touched.iter().map(|(p, _)| *p).collect();
        properties.dedup();
        for property in properties {
            let old_values: Vec<Value> = old
                .map(|i| i.values(property).cloned().collect())
                .unwrap_or_default();
            let old_meta = self
                .fact_meta
                .get(&subject)
                .and_then(|m| m.get(&property))
                .cloned()
                .unwrap_or_default();
            let mut new_meta = Vector::new();
            for value in new_instance.values(property) {
                let kept = old_values
                    .iter()
                    .position(|o| o == value)
                    .and_then(|idx| old_meta.get(idx).copied());
                new_meta.push_back(kept.unwrap_or(provenance));
            }
            self.fact_meta
                .entry(subject)
                .or_default()
                .insert(property, new_meta);
        }
    }

    // -- Reasoning read surface (PRP-002 U4/U5) -----------------------------

    /// The recorded provenance of one stored fact, if the world holds it and
    /// its provenance was recorded (facts restored from a snapshot's wire
    /// form have none — the PRP-001 gap that remains open there).
    pub fn fact_provenance(
        &self,
        subject: Ref,
        property: Iri,
        value: &Value,
    ) -> Option<Provenance> {
        let index = self
            .instance(subject)?
            .values(property)
            .position(|v| v == value)?;
        self.fact_meta
            .get(&subject.iri())?
            .get(&property)?
            .get(index)
            .copied()
    }

    /// The most recent transaction's chaining record (fired rules with
    /// proofs, suppressed conclusions, gate diagnostics, pass count) — the
    /// reasoning-step payload of the trace (doc 09). `None` before the
    /// first assertion.
    pub fn last_reasoning(&self) -> Option<&ReasoningRecord> {
        self.last_reasoning.as_ref()
    }

    /// Goal-directed proving (doc 04, U6): searches for a derivation that
    /// *would* establish `(subject, property, value)` — from the facts, or
    /// through rules, recursively — without mutating this world. `None`
    /// when the goal is not establishable within the cycle/depth caps.
    pub fn prove(&self, subject: Ref, property: Iri, value: &Value) -> Option<ProofNode> {
        crate::reason::backward::prove(self, subject, property, value)
    }

    /// The recorded derivation of an existing rule-derived fact (doc 04:
    /// "why *is* this established?") — from the forward-chaining record,
    /// not a fresh search. `None` for base facts and unset properties;
    /// `prove` (U6) answers the "could it be?" question instead.
    pub fn explain(&self, subject: Ref, property: Iri) -> Option<&ProofNode> {
        let derivation = self.derivations.get(&(subject.iri(), property))?;
        // The record explains the *standing* value; a base re-assertion may
        // have replaced it since. Only answer when the fact still holds.
        let held = self
            .values(subject, property)
            .any(|v| v == &derivation.value);
        held.then_some(&derivation.proof)
    }

    // -- Retraction --------------------------------------------------------

    /// Retracts one fact — `subject`'s `property` value — with
    /// [`Provenance::Api`].
    pub fn retract(
        &mut self,
        subject: Ref,
        property: Iri,
        value: &Value,
    ) -> Result<(), HorismosError> {
        self.retract_as(Provenance::Api, subject, property, value)
    }

    /// Retracts one fact, stating where the retraction came from.
    ///
    /// The gate checks, in order — a rejection leaves the world untouched,
    /// exactly as on the assertion path:
    ///
    /// 1. the world holds `subject`;
    /// 2. `property` is declared by the ontology;
    /// 3. `subject` actually holds `value` for `property`;
    /// 4. `property` is not `horis:immutable` (decided, PRP-002: an
    ///    immutable fact can be neither re-asserted differently nor
    ///    retracted — the two-step bypass is closed);
    /// 5. the fact is not rule-derived, unless `provenance` is
    ///    [`Provenance::Rule`] (decided, PRP-002: a caller cannot retract a
    ///    derived fact directly — the rule path itself remains admitted per
    ///    PRP-001 U7's pinned semantics; the remedy for every other
    ///    provenance is to change the *base* facts the derivation rests on,
    ///    which now genuinely does invalidate it — see the truth-maintenance
    ///    paragraph below);
    /// 6. removing it would not leave `subject` below `property`'s
    ///    `horis:cardinality` minimum;
    /// 7. `property`'s `horis:assignedBy` policy admits `provenance` — the
    ///    same check 5, and the same errors, as an assertion.
    ///
    /// It is **not** the full assertion gate. `rdfs:domain`/`rdfs:range`,
    /// `horis:min`/`max`/`pattern`/`oneOf`, and cardinality *maxima* all
    /// constrain what may enter the world and cannot be violated by removal,
    /// so they do not run.
    ///
    /// The unit of retraction is one *fact*, deliberately: whole-instance
    /// retraction would have to answer what happens to facts referencing
    /// the removed instance, and nothing yet needs it.
    ///
    /// **Truth maintenance** (doc 03 § Truth maintenance, decided per
    /// `tm-derived-facts.md`): once the requested fact is gone, this method
    /// resets every currently-standing rule-derived value and re-chains to
    /// the resulting base-fact set's fixed point — precisely what
    /// [`World::assert_instance_as`] does after committing an assertion. A
    /// retracted `heartRate` that alone supported a derived `:Immediate`
    /// takes that derivation with it, cascading through anything derived
    /// from *it* in turn; the iteration cap is a tripwire here exactly as it
    /// is on assertion, and rolls back the whole transaction, requested
    /// retraction included.
    pub fn retract_as(
        &mut self,
        provenance: Provenance,
        subject: Ref,
        property: Iri,
        value: &Value,
    ) -> Result<(), HorismosError> {
        // The transaction checkpoint (doc 03 § Transactions): truth
        // maintenance now re-chains after every retraction, so a retraction
        // can trip the iteration cap exactly as an assertion can, and the
        // whole transaction — the requested retraction included — must roll
        // back together (mirrors `assert_instance_as`'s checkpoint).
        let checkpoint_instances = self.instances.clone();
        let checkpoint_meta = self.fact_meta.clone();
        let checkpoint_derivations = self.derivations.clone();
        let interner_len_before = self.interner.len();

        let instance = gate::check_retraction(self, provenance, subject, property, value)
            .map_err(|witness| HorismosError::ValidationRejected { witness })?;
        let old = self.instances.get(&subject.iri()).cloned();
        self.instances.insert(subject.iri(), instance);
        // Position-matching drops the removed value's provenance and keeps
        // the rest; `provenance` itself tags nothing (nothing was added).
        self.update_fact_meta(
            subject.iri(),
            old.as_ref(),
            &[(property, value.clone())],
            provenance,
        );

        // Truth maintenance (doc 03 § Truth maintenance): the retracted fact
        // may have been the sole support for any number of standing
        // derivations, directly or several rules removed (chained support).
        // Reset every rule-derived value and re-chain to this transaction's
        // correct fixed point, exactly as `assert_instance_as` does.
        self.retract_all_derived();
        let ontology = self.ontology.clone();
        let mut matcher = self.matcher_kind.build(self, ontology.rules());
        match forward::chain(self, matcher.as_mut()) {
            Ok(record) => self.last_reasoning = Some(record),
            Err(err) => {
                self.instances = checkpoint_instances;
                self.fact_meta = checkpoint_meta;
                self.derivations = checkpoint_derivations;
                if self.interner.len() > interner_len_before {
                    Arc::make_mut(&mut self.interner).truncate_to(interner_len_before);
                }
                return Err(err);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::schema::AssignedBy;
    use crate::world::gate::{ConstraintKind, Operation};
    use rust_decimal::Decimal;

    // -- fixtures ----------------------------------------------------------

    const PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";

    /// A world over the inline test ontology (`:` = `https://example.org/t#`).
    fn world_of(source: &str) -> World {
        let onto =
            Ontology::load_from_str_allow_incomplete(&format!("{PRELUDE}{source}\n"), |_| None)
                .expect("the test ontology must load");
        World::new(&onto)
    }

    /// A world over the flagship fixture, `examples/triage/triage.ttl`.
    fn triage_world() -> World {
        let onto = Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load");
        World::new(&onto)
    }

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    fn tri(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/triage#{local}"))
    }

    fn witness_of(err: &HorismosError) -> &gate::Witness {
        err.witness()
            .expect("a gate rejection must carry a witness")
    }

    // -- happy path --------------------------------------------------------

    #[test]
    fn query_pattern_filters_instances_by_conjunctive_value_equality() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let has_symptom = tri(&mut world, "hasSymptom");
        let acuity = tri(&mut world, "acuity");
        let immediate = Value::Ref(Ref::new(tri(&mut world, "Immediate")));
        let crushing = Value::Ref(Ref::new(tri(&mut world, "CrushingChestPain")));

        // r1 fires for p1 (tachycardic chest pain) but not p2 (calm).
        let p1 = world
            .assert_instance(
                patient,
                [(heart_rate, Value::Integer(128)), (has_symptom, crushing)],
            )
            .expect("p1 is gate-valid");
        let p2 = world
            .assert_instance(patient, [(heart_rate, Value::Integer(70))])
            .expect("p2 is gate-valid");

        assert_eq!(
            world
                .query_pattern(patient, &[(acuity, immediate.clone())])
                .expect("declared terms"),
            vec![p1],
            "only the immediate-acuity patient matches"
        );
        assert_eq!(
            world.query_pattern(patient, &[]).expect("declared terms"),
            vec![p1, p2],
            "an empty conjunction is the class scan, deterministic order"
        );
        assert_eq!(
            world
                .query_pattern(
                    patient,
                    &[(acuity, immediate), (heart_rate, Value::Integer(70))]
                )
                .expect("declared terms"),
            Vec::<Ref>::new(),
            "conjunction: no patient is both immediate and at 70 bpm"
        );

        let ghost = tri(&mut world, "Ghost");
        let err = world
            .query_pattern(ghost, &[])
            .expect_err("an undeclared class is an error, not an empty result");
        assert_eq!(
            err.domain_text(),
            ":Ghost names no term declared by the ontology"
        );
        let err = world
            .query_pattern(patient, &[(ghost, Value::Integer(1))])
            .expect_err("an undeclared property is an error, not an empty result");
        assert_eq!(
            err.domain_text(),
            ":Ghost names no term declared by the ontology"
        );
    }

    #[test]
    fn asserts_a_patient_with_a_heart_rate_from_triage_ttl() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");

        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("heartRate 128 is within horis:min 0 / horis:max 300");

        assert_eq!(world.instance_count(), 1);
        assert_eq!(world.instance(p).map(Instance::class), Some(patient));
        assert_eq!(
            world.values(p, heart_rate).cloned().collect::<Vec<_>>(),
            vec![Value::Integer(128)]
        );
    }

    #[test]
    fn accepts_a_subclass_where_the_range_expects_its_parent() {
        // hasSymptom rdfs:range :Symptom; :CrushingChestPain is a Symptom
        // two levels down (doc 03's own example).
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = tri(&mut world, "CrushingChestPain");

        let p = world
            .assert_instance(patient, [(has_symptom, Value::Ref(Ref::new(crushing)))])
            .expect("subsumption must make CrushingChestPain acceptable as a Symptom");
        assert_eq!(
            world.values(p, has_symptom).cloned().collect::<Vec<_>>(),
            vec![Value::Ref(Ref::new(crushing))]
        );
    }

    #[test]
    fn a_multi_valued_property_keeps_assertion_order_and_dedupes() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = Value::Ref(Ref::new(tri(&mut world, "CrushingChestPain")));
        let chest_pain = Value::Ref(Ref::new(tri(&mut world, "ChestPain")));

        let p = world
            .assert_instance(
                patient,
                [
                    (has_symptom, crushing.clone()),
                    (has_symptom, chest_pain.clone()),
                    (has_symptom, crushing.clone()),
                ],
            )
            .expect("horis:cardinality \"0..*\" admits several symptoms");
        assert_eq!(
            world.values(p, has_symptom).cloned().collect::<Vec<_>>(),
            vec![crushing, chest_pain],
            "assertion order, duplicates suppressed"
        );
    }

    // -- gate check 1: the class exists ------------------------------------

    #[test]
    fn rejects_an_undeclared_class() {
        let mut world = triage_world();
        let ghost = tri(&mut world, "Ghost");
        let err = world
            .assert_instance(ghost, [])
            .expect_err(":Ghost is not in the ontology");

        let w = witness_of(&err);
        assert_eq!(w.kind, ConstraintKind::ClassNotDeclared);
        assert_eq!(w.operation, Operation::Assert);
        assert_eq!(
            err.domain_text(),
            ":Ghost is not a declared class in the ontology"
        );
    }

    #[test]
    fn rejects_re_asserting_a_subject_as_a_different_class() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let symptom = tri(&mut world, "Symptom");
        let p1 = tri(&mut world, "p1");

        world
            .assert_named_instance(p1, patient, [])
            .expect("first assertion must be accepted");
        let err = world
            .assert_named_instance(p1, symptom, [])
            .expect_err("a subject cannot change class");
        assert!(matches!(
            witness_of(&err).kind,
            ConstraintKind::ClassConflict { .. }
        ));
    }

    // -- gate check 2: domain and range ------------------------------------

    #[test]
    fn rejects_an_undeclared_property() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let pulse = tri(&mut world, "pulse");
        let err = world
            .assert_instance(patient, [(pulse, Value::Integer(90))])
            .expect_err("pulse is not declared");
        assert_eq!(witness_of(&err).kind, ConstraintKind::PropertyNotDeclared);
        assert_eq!(
            err.domain_text(),
            "pulse is not a declared property in the ontology"
        );
    }

    #[test]
    fn rejects_a_property_outside_its_domain() {
        let mut world = triage_world();
        let symptom = tri(&mut world, "Symptom");
        let heart_rate = tri(&mut world, "heartRate");
        let err = world
            .assert_instance(symptom, [(heart_rate, Value::Integer(90))])
            .expect_err("heartRate's rdfs:domain is :Patient, not :Symptom");

        let w = witness_of(&err);
        assert_eq!(
            w.kind,
            ConstraintKind::Domain {
                expected: ":Patient".to_owned()
            }
        );
        assert_eq!(
            w.property.as_deref(),
            Some("https://example.org/triage#heartRate")
        );
        assert_eq!(
            err.domain_text(),
            "heartRate does not apply to :Symptom (rdfs:domain :Patient)"
        );
    }

    #[test]
    fn rejects_a_literal_of_the_wrong_datatype() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let err = world
            .assert_instance(patient, [(heart_rate, Value::String("fast".to_owned()))])
            .expect_err("heartRate's rdfs:range is xsd:decimal");

        assert_eq!(
            witness_of(&err).kind,
            ConstraintKind::Range {
                expected: "xsd:decimal".to_owned()
            }
        );
        assert_eq!(
            err.domain_text(),
            "heartRate \"fast\" does not match rdfs:range xsd:decimal on :Patient"
        );
    }

    #[test]
    fn accepts_an_integer_where_the_range_is_xsd_decimal() {
        // xsd:integer is derived from xsd:decimal — every integer is a decimal.
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        world
            .assert_instance(patient, [(heart_rate, Value::Integer(72))])
            .expect("an integer must satisfy an xsd:decimal range");
    }

    #[test]
    fn rejects_an_object_value_that_is_not_in_the_world() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :Encounter a rdfs:Class .\n\
             :seenIn a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Encounter .",
        );
        let patient = t(&mut world, "Patient");
        let seen_in = t(&mut world, "seenIn");
        let nowhere = t(&mut world, "e404");

        let err = world
            .assert_instance(patient, [(seen_in, Value::Ref(Ref::new(nowhere)))])
            .expect_err("the referenced instance does not exist");
        assert_eq!(witness_of(&err).kind, ConstraintKind::UnknownReference);
        assert_eq!(
            err.domain_text(),
            "seenIn :e404 names an instance that is not in the world on :Patient"
        );
    }

    #[test]
    fn accepts_an_object_value_that_is_an_instance_of_the_range_class() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :Encounter a rdfs:Class .\n\
             :EdEncounter rdfs:subClassOf :Encounter .\n\
             :seenIn a rdf:Property ; rdfs:domain :Patient ; rdfs:range :Encounter .",
        );
        let patient = t(&mut world, "Patient");
        let ed = t(&mut world, "EdEncounter");
        let seen_in = t(&mut world, "seenIn");
        let e1 = t(&mut world, "e1");

        world
            .assert_named_instance(e1, ed, [])
            .expect("the encounter must assert");
        world
            .assert_instance(patient, [(seen_in, Value::Ref(Ref::new(e1)))])
            .expect("an EdEncounter instance satisfies an :Encounter range");
    }

    // -- gate check 3: constraints -----------------------------------------

    #[test]
    fn rejects_a_value_above_horis_max_in_the_documented_wording() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");

        let err = world
            .assert_instance(patient, [(heart_rate, Value::Integer(1280))])
            .expect_err("1280 exceeds horis:max 300");

        let w = witness_of(&err);
        assert_eq!(
            w.kind,
            ConstraintKind::Max {
                bound: "300".to_owned(),
                unit: Some("bpm".to_owned()),
            }
        );
        assert_eq!(
            w.property.as_deref(),
            Some("https://example.org/triage#heartRate")
        );
        assert_eq!(w.value.as_deref(), Some("1280"));
        assert_eq!(w.provenance, Provenance::Api);
        // The template from docs/framework/03-world.md and CLAUDE.md, verbatim.
        assert_eq!(
            err.domain_text(),
            "heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"
        );
    }

    #[test]
    fn rejects_a_value_below_horis_min() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let err = world
            .assert_instance(patient, [(heart_rate, Value::Decimal(Decimal::new(-5, 0)))])
            .expect_err("-5 is below horis:min 0");
        assert_eq!(
            witness_of(&err).kind,
            ConstraintKind::Min {
                bound: "0".to_owned(),
                unit: Some("bpm".to_owned()),
            }
        );
        assert_eq!(
            err.domain_text(),
            "heartRate -5 is below horis:min 0 (bpm) on :Patient"
        );
    }

    #[test]
    fn rejects_a_string_that_does_not_match_horis_pattern() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:pattern \"[A-Z]{2}-[0-9]{4}\" .",
        );
        let patient = t(&mut world, "Patient");
        let mrn = t(&mut world, "mrn");

        world
            .assert_instance(patient, [(mrn, Value::String("AB-1234".to_owned()))])
            .expect("a matching value must be accepted");
        let err = world
            .assert_instance(patient, [(mrn, Value::String("nope".to_owned()))])
            .expect_err("\"nope\" does not match the pattern");
        assert_eq!(
            witness_of(&err).kind,
            ConstraintKind::Pattern {
                pattern: "[A-Z]{2}-[0-9]{4}".to_owned()
            }
        );
        assert_eq!(
            err.domain_text(),
            "mrn \"nope\" does not match horis:pattern \"[A-Z]{2}-[0-9]{4}\" on :Patient"
        );
    }

    #[test]
    fn rejects_a_value_outside_a_horis_one_of_enumeration() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let acuity = tri(&mut world, "acuity");
        let bogus = tri(&mut world, "Whenever");

        // acuity is horis:assignedBy horis:RulesOnly (U7), so state Rule
        // provenance — the enumeration is what this test is about.
        let err = world
            .assert_instance_as(
                Provenance::Rule,
                None,
                patient,
                [(acuity, Value::Ref(Ref::new(bogus)))],
            )
            .expect_err(":Whenever is not an AcuityLevel");

        assert_eq!(
            witness_of(&err).kind,
            ConstraintKind::OneOf {
                members: vec![
                    ":Immediate".to_owned(),
                    ":Urgent".to_owned(),
                    ":Standard".to_owned()
                ]
            }
        );
        assert_eq!(
            err.domain_text(),
            "acuity :Whenever is not one of horis:oneOf (:Immediate :Urgent :Standard) on :Patient"
        );
    }

    #[test]
    fn accepts_a_member_of_a_horis_one_of_enumeration() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let acuity = tri(&mut world, "acuity");
        let immediate = tri(&mut world, "Immediate");
        world
            .assert_instance_as(
                Provenance::Rule,
                None,
                patient,
                [(acuity, Value::Ref(Ref::new(immediate)))],
            )
            .expect(":Immediate is an enumerated AcuityLevel");
    }

    // -- gate check 4: cardinality ------------------------------------------

    #[test]
    fn rejects_two_values_for_a_single_valued_property_in_one_assertion() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let err = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(120)),
                    (heart_rate, Value::Integer(128)),
                ],
            )
            .expect_err("horis:cardinality \"0..1\" admits at most one heartRate");

        assert_eq!(
            witness_of(&err).kind,
            ConstraintKind::CardinalityMax {
                bound: "0..1".to_owned(),
                asserted: 2,
            }
        );
        assert_eq!(
            err.domain_text(),
            "heartRate was asserted with 2 values but horis:cardinality \"0..1\" admits at most one on :Patient"
        );
    }

    #[test]
    fn re_asserting_a_single_valued_property_replaces_the_value() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p1 = tri(&mut world, "p1");

        world
            .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(120))])
            .expect("first assertion");
        world
            .assert_named_instance(p1, patient, [(heart_rate, Value::Integer(128))])
            .expect("re-assertion must replace, not reject");

        assert_eq!(world.instance_count(), 1);
        assert_eq!(
            world
                .values(Ref::new(p1), heart_rate)
                .cloned()
                .collect::<Vec<_>>(),
            vec![Value::Integer(128)]
        );
    }

    #[test]
    fn rejects_re_asserting_an_immutable_property_with_a_different_value() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:immutable true .",
        );
        let patient = t(&mut world, "Patient");
        let mrn = t(&mut world, "mrn");
        let p1 = t(&mut world, "p1");

        world
            .assert_named_instance(p1, patient, [(mrn, Value::String("AB-1".to_owned()))])
            .expect("first assignment of an immutable property is fine");
        let err = world
            .assert_named_instance(p1, patient, [(mrn, Value::String("CD-2".to_owned()))])
            .expect_err("an immutable property cannot be reassigned");

        assert_eq!(
            witness_of(&err).kind,
            ConstraintKind::Immutable {
                existing: "\"AB-1\"".to_owned()
            }
        );
        assert_eq!(
            err.domain_text(),
            "mrn is horis:immutable and already holds \"AB-1\"; it cannot be re-asserted as \"CD-2\" on :Patient"
        );
    }

    #[test]
    fn re_asserting_an_immutable_property_with_the_same_value_is_a_no_op() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:immutable true .",
        );
        let patient = t(&mut world, "Patient");
        let mrn = t(&mut world, "mrn");
        let p1 = t(&mut world, "p1");

        let value = Value::String("AB-1".to_owned());
        world
            .assert_named_instance(p1, patient, [(mrn, value.clone())])
            .expect("first assignment");
        world
            .assert_named_instance(p1, patient, [(mrn, value.clone())])
            .expect("re-asserting the same value must be a no-op, not a rejection");
        assert_eq!(
            world.values(Ref::new(p1), mrn).cloned().collect::<Vec<_>>(),
            vec![value]
        );
    }

    #[test]
    fn rejects_an_instance_missing_a_required_property() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:cardinality \"1\" .",
        );
        let patient = t(&mut world, "Patient");
        let err = world
            .assert_instance(patient, [])
            .expect_err("horis:cardinality \"1\" requires a value");

        assert_eq!(
            witness_of(&err).kind,
            ConstraintKind::CardinalityMin {
                bound: "1".to_owned()
            }
        );
        assert_eq!(
            err.domain_text(),
            "mrn is required by horis:cardinality \"1\" but no value is asserted on :Patient"
        );
    }

    // -- atomicity ----------------------------------------------------------

    #[test]
    fn a_rejected_assertion_leaves_the_world_untouched() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = tri(&mut world, "CrushingChestPain");

        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("the valid assertion must be accepted");
        let before = world.clone();

        // Valid symptom, valid class, but the last property blows the bound:
        // nothing at all may survive.
        let err = world
            .assert_instance(
                patient,
                [
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                    (heart_rate, Value::Integer(9001)),
                ],
            )
            .expect_err("9001 exceeds horis:max 300");
        assert!(matches!(witness_of(&err).kind, ConstraintKind::Max { .. }));

        assert_eq!(world.instance_count(), 1, "no partial instance was stored");
        assert_eq!(world.instance(p), before.instance(p));
        assert_eq!(
            world.values(p, has_symptom).count(),
            0,
            "the valid property of a rejected assertion must not survive"
        );
    }

    #[test]
    fn a_rejected_anonymous_assertion_does_not_burn_a_skolem_id() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");

        world
            .assert_instance(patient, [(heart_rate, Value::Integer(9001))])
            .expect_err("rejected");
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("accepted");
        assert_eq!(
            world.resolve(p.iri()),
            "https://w3id.org/horismos/.well-known/genid/0"
        );
    }

    #[test]
    fn skolem_iris_are_deterministic_across_worlds() {
        let onto = Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load");

        let run = |world: &mut World| {
            let patient = tri(world, "Patient");
            let heart_rate = tri(world, "heartRate");
            let a = world
                .assert_instance(patient, [(heart_rate, Value::Integer(80))])
                .expect("accepted");
            let b = world
                .assert_instance(patient, [(heart_rate, Value::Integer(90))])
                .expect("accepted");
            (
                world.resolve(a.iri()).to_owned(),
                world.resolve(b.iri()).to_owned(),
            )
        };

        let mut first = World::new(&onto);
        let mut second = World::new(&onto);
        assert_eq!(run(&mut first), run(&mut second));
        assert_eq!(
            run(&mut second).0,
            "https://w3id.org/horismos/.well-known/genid/2",
            "ids keep advancing within one world"
        );
    }

    #[test]
    fn a_caller_may_not_name_an_instance_in_the_skolem_namespace() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let squat = world.intern("https://w3id.org/horismos/.well-known/genid/0");

        let err = world
            .assert_named_instance(squat, patient, [])
            .expect_err("the skolem namespace is the world's to mint into");

        let w = witness_of(&err);
        assert_eq!(
            w.kind,
            ConstraintKind::ReservedSubject {
                namespace: "https://w3id.org/horismos/.well-known/genid/".to_owned()
            }
        );
        assert_eq!(w.operation, Operation::Assert);
        assert_eq!(
            err.domain_text(),
            "<https://w3id.org/horismos/.well-known/genid/0> is in \
             https://w3id.org/horismos/.well-known/genid/, the namespace reserved for \
             anonymous instances; only the world may name instances there"
        );
        assert_eq!(world.instance_count(), 0);
    }

    #[test]
    fn a_squatted_skolem_iri_cannot_wedge_anonymous_assertion() {
        // The counter advances only on commit, so an occupied next-id would
        // make *every* later anonymous assertion re-mint the same taken IRI
        // and fail identically — forever. Check 1 refuses the squat instead.
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let encounter = tri(&mut world, "Symptom");
        let heart_rate = tri(&mut world, "heartRate");
        let squat = world.intern("https://w3id.org/horismos/.well-known/genid/0");

        world
            .assert_named_instance(squat, encounter, [])
            .expect_err("refused, so genid/0 stays free");

        let first = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("anonymous assertion must still work");
        let second = world
            .assert_instance(patient, [(heart_rate, Value::Integer(96))])
            .expect("and keep working");

        assert_eq!(
            world.resolve(first.iri()),
            "https://w3id.org/horismos/.well-known/genid/0"
        );
        assert_eq!(
            world.resolve(second.iri()),
            "https://w3id.org/horismos/.well-known/genid/1"
        );
    }

    #[test]
    fn a_squatted_skolem_iri_cannot_silently_absorb_an_anonymous_instance() {
        // The same-class variant of the wedge: without the check, the caller's
        // instance is already at genid/0, so the anonymous assertion merges
        // into it and *replaces* its single-valued heartRate.
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let squat = world.intern("https://w3id.org/horismos/.well-known/genid/0");

        world
            .assert_named_instance(squat, patient, [(heart_rate, Value::Integer(200))])
            .expect_err("refused even when the class matches");

        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("accepted");

        assert_eq!(world.instance_count(), 1, "the squat stored nothing");
        assert_eq!(world.resolve(p.iri()), world.resolve(squat));
        assert_eq!(
            world.values(p, heart_rate).cloned().collect::<Vec<_>>(),
            vec![Value::Integer(128)],
            "the anonymous instance is its own, not a merge into a caller's"
        );
    }

    // -- retraction ---------------------------------------------------------

    #[test]
    fn retracts_one_fact() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = Value::Ref(Ref::new(tri(&mut world, "CrushingChestPain")));
        let chest_pain = Value::Ref(Ref::new(tri(&mut world, "ChestPain")));

        let p = world
            .assert_instance(
                patient,
                [
                    (has_symptom, crushing.clone()),
                    (has_symptom, chest_pain.clone()),
                ],
            )
            .expect("accepted");
        world
            .retract(p, has_symptom, &crushing)
            .expect("retracting an asserted fact must succeed");
        assert_eq!(
            world.values(p, has_symptom).cloned().collect::<Vec<_>>(),
            vec![chest_pain]
        );
    }

    #[test]
    fn rejects_retracting_a_fact_that_is_not_asserted() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("accepted");

        let err = world
            .retract(p, heart_rate, &Value::Integer(60))
            .expect_err("60 was never asserted");
        let w = witness_of(&err);
        assert_eq!(w.kind, ConstraintKind::FactNotPresent);
        assert_eq!(w.operation, Operation::Retract);
    }

    #[test]
    fn rejects_retracting_an_unknown_subject() {
        let mut world = triage_world();
        let heart_rate = tri(&mut world, "heartRate");
        let ghost = tri(&mut world, "p404");
        let err = world
            .retract(Ref::new(ghost), heart_rate, &Value::Integer(60))
            .expect_err("no such instance");
        assert_eq!(witness_of(&err).kind, ConstraintKind::SubjectNotPresent);
    }

    #[test]
    fn rejects_a_retraction_that_would_break_a_min_cardinality_bound() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:cardinality \"1\" .",
        );
        let patient = t(&mut world, "Patient");
        let mrn = t(&mut world, "mrn");
        let value = Value::String("AB-1".to_owned());

        let p = world
            .assert_instance(patient, [(mrn, value.clone())])
            .expect("accepted");
        let err = world
            .retract(p, mrn, &value)
            .expect_err("the instance would be left without its required mrn");
        assert_eq!(
            witness_of(&err).kind,
            ConstraintKind::CardinalityMin {
                bound: "1".to_owned()
            }
        );
        assert_eq!(
            world.values(p, mrn).count(),
            1,
            "the rejected retraction must leave the fact in place"
        );
    }

    // -- gate check 5: the horis:assignedBy provenance policy (U7) -----------

    #[test]
    fn the_witness_records_the_assertions_provenance() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let err = world
            .assert_instance_as(
                Provenance::Perception,
                None,
                patient,
                [(heart_rate, Value::Integer(9001))],
            )
            .expect_err("rejected");
        assert_eq!(witness_of(&err).provenance, Provenance::Perception);
    }

    #[test]
    fn default_assigned_by_accepts_every_provenance() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        assert_eq!(
            world
                .ontology()
                .schema()
                .property(heart_rate)
                .expect("heartRate")
                .assigned_by,
            AssignedBy::PerceptionAllowed
        );
        for provenance in [
            Provenance::Perception,
            Provenance::Rule,
            Provenance::Tool,
            Provenance::Api,
        ] {
            world
                .assert_instance_as(
                    provenance,
                    None,
                    patient,
                    [(heart_rate, Value::Integer(80))],
                )
                .unwrap_or_else(|e| panic!("{provenance:?} must be accepted: {e}"));
        }
    }

    #[test]
    fn a_rules_only_property_is_refused_from_perception_in_the_documented_wording() {
        // :acuity is horis:assignedBy horis:RulesOnly — "perception may NEVER
        // set this field" (triage.ttl), the property-level enforcement of
        // "neural proposes, symbolic decides".
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let acuity = tri(&mut world, "acuity");
        let immediate = tri(&mut world, "Immediate");

        let err = world
            .assert_instance_as(
                Provenance::Perception,
                None,
                patient,
                [(acuity, Value::Ref(Ref::new(immediate)))],
            )
            .expect_err("a transducer may never assign a horis:RulesOnly property");

        let w = witness_of(&err);
        assert_eq!(
            w.kind,
            ConstraintKind::ProvenanceRefused {
                policy: "horis:RulesOnly".to_owned()
            }
        );
        assert_eq!(w.provenance, Provenance::Perception);
        assert_eq!(w.operation, Operation::Assert);
        assert_eq!(
            w.property.as_deref(),
            Some("https://example.org/triage#acuity")
        );
        assert_eq!(
            err.domain_text(),
            "acuity may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from perception"
        );
    }

    #[test]
    fn a_rules_only_property_refuses_every_provenance_but_rule() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let acuity = tri(&mut world, "acuity");
        let immediate = Value::Ref(Ref::new(tri(&mut world, "Immediate")));

        for refused in [Provenance::Perception, Provenance::Tool, Provenance::Api] {
            let err = world
                .assert_instance_as(refused, None, patient, [(acuity, immediate.clone())])
                .expect_err("only a rule may assign a horis:RulesOnly property");
            assert_eq!(
                witness_of(&err).kind,
                ConstraintKind::ProvenanceRefused {
                    policy: "horis:RulesOnly".to_owned()
                },
                "{refused:?}"
            );
            assert_eq!(
                world.instance_count(),
                0,
                "a refused assertion stores nothing"
            );
        }
    }

    #[test]
    fn a_tools_only_property_admits_only_tool_provenance() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :labResult a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:assignedBy horis:ToolsOnly .",
        );
        let patient = t(&mut world, "Patient");
        let lab_result = t(&mut world, "labResult");
        let value = Value::String("K 5.9".to_owned());

        world
            .assert_instance_as(
                Provenance::Tool,
                None,
                patient,
                [(lab_result, value.clone())],
            )
            .expect("a tool effect may assign a horis:ToolsOnly property");

        for refused in [Provenance::Perception, Provenance::Rule, Provenance::Api] {
            let err = world
                .assert_instance_as(refused, None, patient, [(lab_result, value.clone())])
                .expect_err("only a tool may assign a horis:ToolsOnly property");
            assert_eq!(
                witness_of(&err).kind,
                ConstraintKind::ProvenanceRefused {
                    policy: "horis:ToolsOnly".to_owned()
                },
                "{refused:?}"
            );
        }

        let err = world
            .assert_instance_as(Provenance::Perception, None, patient, [(lab_result, value)])
            .expect_err("rejected");
        assert_eq!(
            err.domain_text(),
            "labResult may only be assigned by tools (horis:assignedBy horis:ToolsOnly); assertion came from perception"
        );
    }

    #[test]
    fn a_rules_only_property_is_accepted_from_a_rule() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let acuity = tri(&mut world, "acuity");
        let immediate = tri(&mut world, "Immediate");

        let p = world
            .assert_instance_as(
                Provenance::Rule,
                None,
                patient,
                [(acuity, Value::Ref(Ref::new(immediate)))],
            )
            .expect("a rule conclusion is exactly what horis:RulesOnly admits");
        assert_eq!(world.values(p, acuity).count(), 1);
    }

    #[test]
    fn retraction_passes_the_same_provenance_policy() {
        let mut world = world_of(
            ":Patient a rdfs:Class .\n\
             :AcuityLevel a rdfs:Class ; horis:oneOf ( :Immediate ) .\n\
             :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :AcuityLevel ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:assignedBy horis:RulesOnly .",
        );
        let patient = t(&mut world, "Patient");
        let acuity = t(&mut world, "acuity");
        let immediate = Value::Ref(Ref::new(t(&mut world, "Immediate")));

        let p = world
            .assert_instance_as(
                Provenance::Rule,
                None,
                patient,
                [(acuity, immediate.clone())],
            )
            .expect("accepted");

        let err = world
            .retract_as(Provenance::Perception, p, acuity, &immediate)
            .expect_err("perception may not retract a horis:RulesOnly fact either");
        let w = witness_of(&err);
        assert_eq!(
            w.kind,
            ConstraintKind::ProvenanceRefused {
                policy: "horis:RulesOnly".to_owned()
            }
        );
        assert_eq!(w.operation, Operation::Retract);
        assert_eq!(
            err.domain_text(),
            "acuity may only be assigned by rules (horis:assignedBy horis:RulesOnly); retraction came from perception"
        );
        assert_eq!(
            world.values(p, acuity).count(),
            1,
            "the refused retraction must leave the fact in place"
        );

        world
            .retract_as(Provenance::Rule, p, acuity, &immediate)
            .expect("a rule may retract what a rule asserted");
        assert_eq!(world.values(p, acuity).count(), 0);
    }

    // -- tier-1 pattern queries (U9) -----------------------------------------

    #[test]
    fn instances_is_subsumption_aware() {
        // triage.ttl: Symptom <- ChestPain <- CrushingChestPain. An instance
        // asserted as the leaf class must come back from a query for any
        // ancestor, doc 04's "is matches any subclass" applied to queries.
        let mut world = triage_world();
        let crushing = tri(&mut world, "CrushingChestPain");
        let chest_pain = tri(&mut world, "ChestPain");
        let symptom = tri(&mut world, "Symptom");
        let cp1 = tri(&mut world, "cp1");

        let instance = world
            .assert_named_instance(cp1, crushing, [])
            .expect("CrushingChestPain is a declared class (implicit via subClassOf)");

        assert_eq!(
            world.instances(crushing).collect::<Vec<_>>(),
            vec![instance]
        );
        assert_eq!(
            world.instances(chest_pain).collect::<Vec<_>>(),
            vec![instance],
            "a direct parent must match too"
        );
        assert_eq!(
            world.instances(symptom).collect::<Vec<_>>(),
            vec![instance],
            "a transitive ancestor must match"
        );
    }

    #[test]
    fn instances_excludes_a_sibling_class() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let crushing = tri(&mut world, "CrushingChestPain");
        world
            .assert_instance(patient, [])
            .expect("a Patient instance");

        assert_eq!(
            world.instances(crushing).collect::<Vec<_>>(),
            Vec::<Ref>::new(),
            "a Patient instance must not satisfy an unrelated class query"
        );
    }

    #[test]
    fn instances_returns_every_matching_instance_in_iri_token_order() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let p1 = tri(&mut world, "p1");
        let p2 = tri(&mut world, "p2");

        let first = world
            .assert_named_instance(p1, patient, [])
            .expect("accepted");
        let second = world
            .assert_named_instance(p2, patient, [])
            .expect("accepted");

        assert_eq!(
            world.instances(patient).collect::<Vec<_>>(),
            vec![first, second]
        );
    }

    #[test]
    fn instances_query_order_is_deterministic_across_two_identically_built_worlds() {
        let onto = Ontology::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl must load");

        let run = |world: &mut World| -> Vec<String> {
            let patient = tri(world, "Patient");
            let heart_rate = tri(world, "heartRate");
            world
                .assert_instance(patient, [(heart_rate, Value::Integer(70))])
                .expect("accepted");
            world
                .assert_instance(patient, [(heart_rate, Value::Integer(90))])
                .expect("accepted");
            world
                .instances(patient)
                .map(|r| world.resolve(r.iri()).to_owned())
                .collect()
        };

        let mut first = World::new(&onto);
        let mut second = World::new(&onto);
        assert_eq!(run(&mut first), run(&mut second));
    }

    #[test]
    fn value_returns_the_single_value_of_a_single_valued_property() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p = world
            .assert_instance(patient, [(heart_rate, Value::Integer(128))])
            .expect("accepted");
        assert_eq!(world.value(p, heart_rate), Some(&Value::Integer(128)));
    }

    #[test]
    fn value_is_none_for_an_unset_property() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let heart_rate = tri(&mut world, "heartRate");
        let p = world.assert_instance(patient, []).expect("accepted");
        assert_eq!(world.value(p, heart_rate), None);
    }

    #[test]
    fn value_is_none_for_an_unknown_subject() {
        let mut world = triage_world();
        let heart_rate = tri(&mut world, "heartRate");
        let ghost = tri(&mut world, "p404");
        assert_eq!(world.value(Ref::new(ghost), heart_rate), None);
    }

    #[test]
    fn value_is_the_first_asserted_value_of_a_multi_valued_property() {
        let mut world = triage_world();
        let patient = tri(&mut world, "Patient");
        let has_symptom = tri(&mut world, "hasSymptom");
        let crushing = Value::Ref(Ref::new(tri(&mut world, "CrushingChestPain")));
        let chest_pain = Value::Ref(Ref::new(tri(&mut world, "ChestPain")));
        let p = world
            .assert_instance(
                patient,
                [(has_symptom, crushing.clone()), (has_symptom, chest_pain)],
            )
            .expect("accepted");
        assert_eq!(world.value(p, has_symptom), Some(&crushing));
    }
}

#[cfg(test)]
mod retraction_ruling_tests {
    use super::*;
    use crate::reason::testing::example_world;
    use crate::world::gate::ConstraintKind;

    fn t(world: &mut World, local: &str) -> Iri {
        world.intern(&format!("https://example.org/t#{local}"))
    }

    fn witness_of(err: &HorismosError) -> &gate::Witness {
        err.witness().expect("a gate rejection carries a witness")
    }

    /// D-6: horis:immutable binds retraction — from every provenance — so
    /// the retract-then-assert bypass is closed.
    #[test]
    fn an_immutable_fact_refuses_retraction_in_the_documented_wording() {
        let onto = Ontology::load_from_str_allow_incomplete(
            "@prefix : <https://example.org/t#> .\n\
             @prefix horis: <https://w3id.org/horismos#> .\n\
             @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             :Patient a rdfs:Class .\n\
             :mrn a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:string ;\n\
             \x20   horis:cardinality \"0..1\" ; horis:immutable true .\n",
            |_| None,
        )
        .expect("loads");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let mrn = t(&mut world, "mrn");
        let value = Value::String("AB-1".to_owned());

        let p1 = t(&mut world, "p1");
        let p = world
            .assert_named_instance(p1, patient, [(mrn, value.clone())])
            .expect("first assignment");

        for provenance in [
            Provenance::Api,
            Provenance::Perception,
            Provenance::Rule,
            Provenance::Tool,
        ] {
            let err = world
                .retract_as(provenance, p, mrn, &value)
                .expect_err("immutable binds retraction, whatever the provenance");
            assert_eq!(witness_of(&err).kind, ConstraintKind::ImmutableRetraction);
        }

        let err = world.retract(p, mrn, &value).expect_err("refused");
        assert_eq!(
            err.domain_text(),
            "mrn is horis:immutable and holds \"AB-1\"; an immutable fact can be neither re-asserted differently nor retracted on :Patient"
        );

        // The two-step bypass is closed: the value cannot be replaced.
        let err = world
            .assert_named_instance(p.iri(), patient, [(mrn, Value::String("CD-2".to_owned()))])
            .expect_err("re-assertion path was already closed in PRP-001");
        assert!(matches!(
            witness_of(&err).kind,
            ConstraintKind::Immutable { .. }
        ));
        assert_eq!(
            world.values(p, mrn).cloned().collect::<Vec<_>>(),
            vec![value],
            "the original value stands"
        );
    }

    /// D-5: a rule-derived fact refuses retraction from the api, naming the
    /// deriving rule; the rule path itself remains admitted.
    #[test]
    fn a_derived_fact_refuses_api_retraction_but_admits_the_rule_path() {
        let mut world = example_world();
        let patient = t(&mut world, "Patient");
        let heart_rate = t(&mut world, "heartRate");
        let has_symptom = t(&mut world, "hasSymptom");
        let crushing = t(&mut world, "CrushingChestPain");
        let acuity = t(&mut world, "acuity");
        let immediate = t(&mut world, "Immediate");
        let monitor = t(&mut world, "needsCardiacMonitor");

        let p7 = world
            .assert_instance(
                patient,
                [
                    (heart_rate, Value::Integer(128)),
                    (has_symptom, Value::Ref(Ref::new(crushing))),
                ],
            )
            .expect("accepted; chaining derives acuity and the monitor flag");

        let derived = Value::Ref(Ref::new(immediate));
        let err = world
            .retract(p7, acuity, &derived)
            .expect_err("the api may not retract what a rule derived");
        let w = witness_of(&err);
        assert_eq!(
            w.kind,
            ConstraintKind::DerivedFactRetraction {
                rule: ":r1".to_owned()
            }
        );
        assert_eq!(
            err.domain_text(),
            "acuity :Immediate was derived by :r1; derived facts change only by re-running inference after base-fact changes; retraction came from the api"
        );
        assert_eq!(world.values(p7, acuity).count(), 1, "the fact stands");

        // Tool/perception are refused the same way; the rule path is not.
        for refused in [Provenance::Perception, Provenance::Tool] {
            let err = world
                .retract_as(refused, p7, acuity, &derived)
                .expect_err("only the rule path may unwind a derivation");
            assert!(matches!(
                witness_of(&err).kind,
                ConstraintKind::DerivedFactRetraction { .. }
            ));
        }
        world
            .retract_as(Provenance::Rule, p7, monitor, &Value::Boolean(true))
            .expect("the rule path retracts what rules derived (PRP-001 U7 pin)");
    }

    /// D-7 at the gate: an owl:sameAs-anchored enum member is accepted where
    /// its local twin is enumerated — the PRP-001 spurious reject is gone.
    #[test]
    fn one_of_accepts_a_same_as_anchored_member() {
        let onto = Ontology::load_from_str_allow_incomplete(
            "@prefix : <https://example.org/t#> .\n\
             @prefix horis: <https://w3id.org/horismos#> .\n\
             @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             @prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
             @prefix sct: <http://snomed.info/id/> .\n\
             @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             :Patient a rdfs:Class .\n\
             :AcuityLevel a rdfs:Class ; horis:oneOf ( :Immediate :Urgent ) .\n\
             :Immediate owl:sameAs sct:14816004 .\n\
             :acuity a rdf:Property ; rdfs:domain :Patient ; rdfs:range :AcuityLevel ;\n\
             \x20   horis:cardinality \"0..1\" .\n",
            |_| None,
        )
        .expect("loads");
        let mut world = World::new(&onto);
        let patient = t(&mut world, "Patient");
        let acuity = t(&mut world, "acuity");
        let anchored = world.intern("http://snomed.info/id/14816004");

        let p = world
            .assert_instance(patient, [(acuity, Value::Ref(Ref::new(anchored)))])
            .expect("the anchored twin is the same individual as :Immediate");
        assert_eq!(world.values(p, acuity).count(), 1);
    }
}
