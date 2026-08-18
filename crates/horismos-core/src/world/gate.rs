//! The validation gate: the one path every World mutation takes.
//!
//! "The validation gate has no back door" (CLAUDE.md) is implemented here.
//! [`check_assertion`] and [`check_retraction`] are *pure* functions of the
//! current store plus the proposed mutation: they either return the candidate
//! [`Instance`] the caller may swap in, or a [`Witness`] describing — in
//! domain terms and in machine-readable form — the first check that refused.
//! Nothing in this module mutates a [`World`], which is what makes rejection
//! atomic: [`crate::world::World::assert_instance_as`] only touches the store
//! once a candidate has come back.
//!
//! ## The five checks, in documented order
//!
//! `docs/framework/03-world.md` fixes both the checks and their order, and
//! the order is observable: a mutation that fails several checks is reported
//! against the *first* one. The checks run check-major (every property is
//! taken through check 2 before any property reaches check 3), not
//! property-major, so the reported failure is the earliest *check*, not the
//! earliest property.
//!
//! 1. **Class exists** — [`check_class`]
//! 2. **Property domains/ranges** — [`check_domain_and_range`]
//! 3. **Constraints** (`horis:min`/`max`/`pattern`/`oneOf`) —
//!    [`check_constraints`]
//! 4. **Cardinality** (`horis:cardinality`, `horis:immutable`) —
//!    [`check_cardinality`], which also builds the candidate instance
//! 5. **Provenance policy** (`horis:assignedBy`) — [`check_provenance`]
//!
//! ### On unit consistency
//!
//! Doc 03 lists "unit consistency" under check 3. In v1 an instance value
//! carries no unit annotation — `horis:unit` is *property* metadata, so a
//! value asserted for `:heartRate` is in bpm by construction and there is
//! nothing per-assertion to compare. The unit check that does exist is the
//! rule-side one (decision D6: a rule condition's unit must match the
//! property's), and it runs at rule load, not at the gate — PRP-002. The
//! gate's only use of `horis:unit` is naming it in the domain text of a
//! `horis:min`/`horis:max` rejection ("… exceeds horis:max 300 (bpm) …").
//!
//! Spec: `docs/framework/03-world.md`, `docs/framework/02-ontology.md`
//! · Owner: PRP-001

use imbl::{OrdMap, Vector};
use rust_decimal::Decimal;

use crate::ontology::schema::{AssignedBy, Cardinality, PropertyDecl, PropertyRange, Schema};
use crate::ontology::term::{Datatype, Iri, Ref, Value};
use crate::world::{Instance, Provenance, SKOLEM_NAMESPACE, World};

// -- The witness -----------------------------------------------------------

/// Where an assertion's subject IRI came from — which decides whether it may
/// live in the reserved skolem namespace.
///
/// The distinction is load-bearing, not cosmetic: the skolem counter advances
/// only on commit, so a caller-supplied IRI squatting on the *next* id would
/// wedge anonymous assertion permanently (every subsequent
/// `assert_instance` would mint the same taken IRI, be refused, and leave the
/// counter where it was). Check 1 refuses the squat instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Subject {
    /// The caller named the instance (`World::assert_named_instance`).
    Named(Iri),
    /// The world skolemized an anonymous instance (`World::assert_instance`).
    Skolem(Iri),
}

impl Subject {
    /// The IRI either way.
    pub fn iri(self) -> Iri {
        match self {
            Self::Named(iri) | Self::Skolem(iri) => iri,
        }
    }
}

/// Which kind of mutation the gate was asked to admit. Rejection texts
/// differ between the two (a min-cardinality bound is broken by *omission*
/// on an assertion and by *removal* on a retraction), and the perception
/// retry loop (doc 06) needs to tell them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// A `World::assert_*` call.
    Assert,
    /// A `World::retract*` call.
    Retract,
}

impl Operation {
    /// The noun domain-term errors use ("… assertion came from perception").
    fn noun(self) -> &'static str {
        match self {
            Self::Assert => "assertion",
            Self::Retract => "retraction",
        }
    }
}

/// The specific check that refused, with the bound it was checking against.
/// This is the machine-readable half of a [`Witness`]: the perception retry
/// loop (doc 06) branches on it, and tests assert on it rather than on
/// rendered prose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintKind {
    // -- check 1 -----------------------------------------------------------
    /// The asserted class is not declared by the ontology or any import.
    ClassNotDeclared,
    /// The subject is already in the world under a different class; an
    /// instance's class is fixed at first assertion.
    ClassConflict {
        /// The class the subject was originally asserted as, prefixed.
        existing: String,
    },
    /// A caller named an instance inside the namespace the world reserves for
    /// skolemizing anonymous ones.
    ReservedSubject {
        /// The reserved namespace, as a full IRI prefix.
        namespace: String,
    },

    // -- check 2 -----------------------------------------------------------
    /// The property is not declared by the ontology or any import.
    PropertyNotDeclared,
    /// The property's `rdfs:domain` does not cover the subject's class.
    Domain {
        /// The declared `rdfs:domain`, prefixed.
        expected: String,
    },
    /// The value does not match the property's `rdfs:range`.
    Range {
        /// The declared `rdfs:range` — an XSD datatype name or a prefixed
        /// class IRI.
        expected: String,
    },
    /// An object-property value names something the world does not hold and
    /// the ontology does not declare.
    UnknownReference,

    // -- check 3 -----------------------------------------------------------
    /// The value is below the property's `horis:min`.
    Min {
        /// The declared bound, in its lexical form.
        bound: String,
        /// The property's `horis:unit`, if it declares one.
        unit: Option<String>,
    },
    /// The value is above the property's `horis:max`.
    Max {
        /// The declared bound, in its lexical form.
        bound: String,
        /// The property's `horis:unit`, if it declares one.
        unit: Option<String>,
    },
    /// The value does not match the property's `horis:pattern`.
    Pattern {
        /// The declared regular expression, as authored.
        pattern: String,
    },
    /// The value is not a member of the range class's `horis:oneOf`
    /// enumeration.
    OneOf {
        /// The enumerated members, prefixed, in declaration order.
        members: Vec<String>,
    },

    // -- check 4 -----------------------------------------------------------
    /// More values were asserted for a property than its
    /// `horis:cardinality` upper bound admits.
    CardinalityMax {
        /// The declared `horis:cardinality`, as authored.
        bound: String,
        /// How many distinct values the assertion carried.
        asserted: usize,
    },
    /// A `horis:cardinality` lower bound would be left unsatisfied — by an
    /// assertion that omits the property, or by a retraction that removes
    /// its last value.
    CardinalityMin {
        /// The declared `horis:cardinality`, as authored.
        bound: String,
    },
    /// A `horis:immutable` property already holds a different value.
    Immutable {
        /// The value(s) the property already holds, rendered.
        existing: String,
    },

    // -- check 5 -----------------------------------------------------------
    /// The property's `horis:assignedBy` policy refuses this mutation's
    /// provenance.
    ProvenanceRefused {
        /// The declared policy, as the `horis:` term it was authored as.
        policy: String,
    },

    // -- retraction-only ---------------------------------------------------
    /// The retraction named a subject the world does not hold.
    SubjectNotPresent,
    /// The retraction named a fact the subject does not hold.
    FactNotPresent,
    /// The fact belongs to a `horis:immutable` property — once set it can
    /// be neither re-asserted differently nor retracted (decided, PRP-002:
    /// the two-step retract-then-assert bypass is closed).
    ImmutableRetraction,
    /// The fact is rule-derived and the retraction did not come from the
    /// rule path itself (decided, PRP-002: derived facts change only by
    /// re-running inference after base-fact changes).
    DerivedFactRetraction {
        /// The deriving rule, in prefixed form.
        rule: String,
    },
}

/// Why the gate refused a mutation: the failing constraint stated in domain
/// vocabulary (`text`) alongside the machine-readable structure the
/// perception retry loop (doc 06) consumes.
///
/// IRIs in the structured fields are full and unabbreviated — they are for
/// programs. `text` is for people, and names classes in prefixed form
/// (`:Patient`) and properties by their local name (`heartRate`), matching
/// the template in `docs/framework/03-world.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    /// Assertion or retraction.
    pub operation: Operation,
    /// Where the refused mutation came from.
    pub provenance: Provenance,
    /// The instance the mutation was about (full IRI) — for an anonymous
    /// assertion, the skolem IRI it would have been stored under.
    pub subject: String,
    /// The subject's class (full IRI).
    pub class: String,
    /// The property at fault (full IRI), when one property is to blame.
    pub property: Option<String>,
    /// The value at fault, rendered as the domain text renders it.
    pub value: Option<String>,
    /// Which check refused, and against which bound.
    pub kind: ConstraintKind,
    /// The failing constraint in domain vocabulary — `HorismosError`'s
    /// `Display` and `domain_text()`.
    pub text: String,
}

// -- Entry points ----------------------------------------------------------

/// Runs the five checks over a proposed assertion of `class` under
/// `subject`, returning the [`Instance`] the caller should store on success.
///
/// If `subject` is already in the world, the returned candidate is the
/// *merge* of the stored instance with `properties` under check 4's rules
/// (single-valued properties replaced, multi-valued ones extended,
/// `horis:immutable` ones refused); otherwise it is a fresh instance. Either
/// way `world` is left untouched — the caller commits, or drops the witness
/// and changes nothing.
pub fn check_assertion(
    world: &World,
    provenance: Provenance,
    subject: Subject,
    class: Iri,
    properties: &[(Iri, Value)],
) -> Result<Instance, Box<Witness>> {
    let gate = Gate {
        world,
        provenance,
        operation: Operation::Assert,
        subject: subject.iri(),
        class,
        deferred: None,
    };

    gate.check_class(subject)?;
    gate.check_domain_and_range(properties)?;
    gate.check_constraints(properties)?;
    let candidate = gate.check_cardinality(properties)?;
    gate.check_assertion_provenance(properties)?;
    Ok(candidate)
}

/// An object reference check 2 could not settle during snapshot restoration
/// because the referent had not been restored yet ([`Snapshot::from_bytes`]
/// restores instances in wire order, and a legitimate world may hold
/// reference cycles, so no restoration order settles every reference
/// up front). [`check_restored_reference`] settles it once the store is
/// complete.
///
/// [`Snapshot::from_bytes`]: crate::world::snapshot::Snapshot::from_bytes
#[derive(Debug)]
pub(crate) struct DeferredRef {
    pub(crate) property: Iri,
    pub(crate) value: Value,
    pub(crate) range: Iri,
}

/// [`check_assertion`], in restoration mode: identical checks, except that an
/// object reference whose referent is not (yet) in the store is *deferred*
/// rather than refused, and check 5 runs **per fact**, against each restored
/// fact's own recorded provenance (PRP-002: provenance travels with the wire
/// form, so a `horis:RulesOnly` fact a rule legitimately derived restores as
/// `Provenance::Rule`, exactly as it was admitted live). The caller must
/// settle every returned [`DeferredRef`] with [`check_restored_reference`]
/// once all instances are restored — until then the gate's guarantee is not
/// yet re-established.
pub(crate) fn check_assertion_restoring(
    world: &World,
    subject: Subject,
    class: Iri,
    facts: &[(Iri, Value, Provenance)],
) -> Result<(Instance, Vec<DeferredRef>), Box<Witness>> {
    let properties: Vec<(Iri, Value)> = facts
        .iter()
        .map(|(property, value, _)| (*property, value.clone()))
        .collect();
    let deferred = std::cell::RefCell::new(Vec::new());
    let gate = Gate {
        world,
        provenance: Provenance::Api,
        operation: Operation::Assert,
        subject: subject.iri(),
        class,
        deferred: Some(&deferred),
    };

    gate.check_class(subject)?;
    gate.check_domain_and_range(&properties)?;
    gate.check_constraints(&properties)?;
    let candidate = gate.check_cardinality(&properties)?;
    // Check 5, per fact: the recorded provenance is the one the policy
    // judges — and the one a refusal's witness names.
    for (property, value, provenance) in facts {
        let decl = gate
            .schema()
            .property(*property)
            .expect("check 2 established that every asserted property is declared");
        let fact_gate = Gate {
            world,
            provenance: *provenance,
            operation: Operation::Assert,
            subject: subject.iri(),
            class,
            deferred: None,
        };
        check_provenance(decl, *provenance)
            .map_err(|kind| fact_gate.reject(Some(*property), Some(value), kind))?;
    }
    Ok((candidate, deferred.into_inner()))
}

/// Settles one [`DeferredRef`] against the now-complete restored store: the
/// same referent-existence and range-subsumption rules as check 2's
/// [`Gate::check_object_range`], producing the same witnesses. Runs with
/// `deferred: None`, so an unresolved referent is refused here, not deferred
/// again.
pub(crate) fn check_restored_reference(
    world: &World,
    provenance: Provenance,
    subject: Iri,
    class: Iri,
    deferred: &DeferredRef,
) -> Result<(), Box<Witness>> {
    let gate = Gate {
        world,
        provenance,
        operation: Operation::Assert,
        subject,
        class,
        deferred: None,
    };
    gate.check_object_range(deferred.property, &deferred.value, deferred.range)
}

/// Runs the retraction checks over the removal of one fact, returning the
/// [`Instance`] the caller should store in the subject's place.
///
/// The checks, in order: the subject is in the world; the property is
/// declared; the fact is actually held; the property is not
/// `horis:immutable`; the fact is not rule-derived (unless the retraction
/// *is* the rule path); removing it would not break the property's
/// `horis:cardinality` minimum; and the `horis:assignedBy` policy admits
/// this provenance (check 5, shared with assertion).
///
/// The assertion-only checks do *not* re-run, and cannot: `rdfs:range`,
/// `horis:min`/`max`/`pattern`/`oneOf`, and cardinality *maxima* all constrain
/// what may enter the world, and removal cannot violate them.
///
/// Two rulings decided by PRP-002 (doc 02/03, markers struck there):
///
/// - **`horis:immutable` binds retraction.** "Once set, cannot be
///   re-asserted with a different value" would be vacuous if
///   retract-then-assert replaced the value in two steps; an immutable
///   fact is refused retraction outright, from every provenance.
/// - **Derived facts are not retracted, they are re-derived.** A fact the
///   reasoner concluded (it has a recorded derivation) refuses retraction
///   from perception/tool/api — the witness names the deriving rule and
///   the remedy (change the base facts, re-run inference).
///   [`Provenance::Rule`] remains admitted: it is the inference side's own
///   mechanism, and PRP-001 U7 pinned "a rule may retract what a rule
///   asserted".
pub fn check_retraction(
    world: &World,
    provenance: Provenance,
    subject: Ref,
    property: Iri,
    value: &Value,
) -> Result<Instance, Box<Witness>> {
    let Some(instance) = world.instances.get(&subject.iri()) else {
        // No instance means no class to name; the subject's own IRI stands
        // in so the witness is still fully addressed.
        let gate = Gate {
            world,
            provenance,
            operation: Operation::Retract,
            subject: subject.iri(),
            class: subject.iri(),
            deferred: None,
        };
        return Err(gate.reject(
            Some(property),
            Some(value),
            ConstraintKind::SubjectNotPresent,
        ));
    };

    let gate = Gate {
        world,
        provenance,
        operation: Operation::Retract,
        subject: subject.iri(),
        class: instance.class,
        deferred: None,
    };

    let Some(decl) = gate.schema().property(property) else {
        return Err(gate.reject(
            Some(property),
            Some(value),
            ConstraintKind::PropertyNotDeclared,
        ));
    };

    let held = instance
        .properties
        .get(&property)
        .cloned()
        .unwrap_or_default();
    if !held.contains(value) {
        return Err(gate.reject(Some(property), Some(value), ConstraintKind::FactNotPresent));
    }

    // PRP-002's retraction rulings (see the function doc): immutability
    // binds retraction, and a rule-derived fact only the rule path may
    // touch.
    if decl.immutable {
        return Err(gate.reject(
            Some(property),
            Some(value),
            ConstraintKind::ImmutableRetraction,
        ));
    }
    if provenance != Provenance::Rule
        && let Some(derivation) = world.derivations.get(&(subject.iri(), property))
        && &derivation.value == value
    {
        return Err(gate.reject(
            Some(property),
            Some(value),
            ConstraintKind::DerivedFactRetraction {
                rule: gate.iri_text(derivation.rule),
            },
        ));
    }

    let remaining: Vector<Value> = held.into_iter().filter(|v| v != value).collect();
    if remaining.is_empty()
        && let Some(card) = decl.cardinality
        && card.requires_a_value()
    {
        return Err(gate.reject(
            Some(property),
            Some(value),
            ConstraintKind::CardinalityMin {
                bound: card.as_str().to_owned(),
            },
        ));
    }

    check_provenance(decl, provenance)
        .map_err(|kind| gate.reject(Some(property), Some(value), kind))?;

    let mut candidate = instance.clone();
    if remaining.is_empty() {
        candidate.properties.remove(&property);
    } else {
        candidate.properties.insert(property, remaining);
    }
    Ok(candidate)
}

/// **Check 5.** `horis:assignedBy` versus the mutation's provenance
/// (`docs/framework/02-ontology.md`): `horis:RulesOnly` admits only
/// [`Provenance::Rule`], `horis:ToolsOnly` only [`Provenance::Tool`], and
/// `horis:PerceptionAllowed` — the default when a property declares no
/// policy — admits all four.
///
/// This is the property-level enforcement of "neural proposes, symbolic
/// decides" (CLAUDE.md): a transducer's output can never reach a
/// `horis:RulesOnly` property, however confident the model was. The policy
/// is about the *property*, not about the direction of the mutation, so
/// retraction is held to it identically — a provenance that may not assign
/// `:acuity` may not take it away either.
pub fn check_provenance(
    property: &PropertyDecl,
    provenance: Provenance,
) -> Result<(), ConstraintKind> {
    let permitted = match property.assigned_by {
        AssignedBy::RulesOnly => provenance == Provenance::Rule,
        AssignedBy::ToolsOnly => provenance == Provenance::Tool,
        AssignedBy::PerceptionAllowed => true,
    };
    if permitted {
        return Ok(());
    }
    Err(ConstraintKind::ProvenanceRefused {
        policy: assigned_by_term(property.assigned_by).to_owned(),
    })
}

/// The `horis:` term a policy was authored as, for the witness.
fn assigned_by_term(policy: AssignedBy) -> &'static str {
    match policy {
        AssignedBy::RulesOnly => "horis:RulesOnly",
        AssignedBy::ToolsOnly => "horis:ToolsOnly",
        AssignedBy::PerceptionAllowed => "horis:PerceptionAllowed",
    }
}

/// Who a policy admits, for the domain text ("… may only be assigned by
/// rules …"). `horis:PerceptionAllowed` never reaches this — it refuses
/// nothing, so it never produces a witness.
fn assigned_by_audience(policy: AssignedBy) -> &'static str {
    match policy {
        AssignedBy::RulesOnly => "rules",
        AssignedBy::ToolsOnly => "tools",
        AssignedBy::PerceptionAllowed => "anything",
    }
}

// -- The gate --------------------------------------------------------------

/// One mutation under examination: the store it is proposed against plus the
/// subject/class/provenance every check and every witness needs.
struct Gate<'a> {
    world: &'a World,
    provenance: Provenance,
    operation: Operation,
    subject: Iri,
    class: Iri,
    /// `Some` only in restoration mode ([`check_assertion_restoring`]):
    /// check 2 pushes an unresolvable object reference here instead of
    /// refusing it. `None` on every live path.
    deferred: Option<&'a std::cell::RefCell<Vec<DeferredRef>>>,
}

impl Gate<'_> {
    fn schema(&self) -> &Schema {
        self.world.ontology().schema()
    }

    // -- check 1 -----------------------------------------------------------

    /// **Check 1.** The subject may be named by whoever named it, the
    /// asserted class is declared — by the root graph or by an import (U5
    /// merges both into one [`Schema`]) — and the subject, if the world
    /// already holds it, was asserted as that same class.
    ///
    /// A caller may not name an instance inside [`SKOLEM_NAMESPACE`]: that
    /// space belongs to the world's own counter, and a squatter there is not
    /// merely untidy but *wedging*. Because the counter advances only on
    /// commit, an occupied next-id makes every subsequent anonymous assertion
    /// re-mint the same taken IRI and fail identically, forever; and where the
    /// squatter happens to share the asserted class, the anonymous assertion
    /// would silently merge into the caller's instance instead, replacing its
    /// single-valued properties. Refusing the name closes both.
    ///
    /// The class must be declared under the exact IRI asserted: an
    /// `owl:sameAs` anchor makes two IRIs interchangeable for *subsumption*
    /// (check 2), not for declaration.
    fn check_class(&self, subject: Subject) -> Result<(), Box<Witness>> {
        if let Subject::Named(iri) = subject
            && self.world.resolve(iri).starts_with(SKOLEM_NAMESPACE)
        {
            return Err(self.reject(
                None,
                None,
                ConstraintKind::ReservedSubject {
                    namespace: SKOLEM_NAMESPACE.to_owned(),
                },
            ));
        }
        if self.schema().class(self.class).is_none() {
            return Err(self.reject(None, None, ConstraintKind::ClassNotDeclared));
        }
        if let Some(existing) = self.world.instances.get(&self.subject)
            && existing.class != self.class
        {
            return Err(self.reject(
                None,
                None,
                ConstraintKind::ClassConflict {
                    existing: self.iri_text(existing.class),
                },
            ));
        }
        Ok(())
    }

    // -- check 2 -----------------------------------------------------------

    /// **Check 2.** Every asserted property is declared, its `rdfs:domain`
    /// covers the subject's class (by the subsumption closure, so a subclass
    /// of the domain is accepted), and every value matches its
    /// `rdfs:range`.
    fn check_domain_and_range(&self, properties: &[(Iri, Value)]) -> Result<(), Box<Witness>> {
        for (property, value) in properties {
            let Some(decl) = self.schema().property(*property) else {
                return Err(self.reject(
                    Some(*property),
                    Some(value),
                    ConstraintKind::PropertyNotDeclared,
                ));
            };

            // A property with no `rdfs:domain` is unrestricted; one with a
            // domain admits that class and every subclass of it.
            if let Some(domain) = decl.domain
                && !self.schema().is_subclass_of(self.class, domain)
            {
                return Err(self.reject(
                    Some(*property),
                    Some(value),
                    ConstraintKind::Domain {
                        expected: self.iri_text(domain),
                    },
                ));
            }

            match decl.range {
                None => {}
                Some(PropertyRange::Datatype(datatype)) => {
                    if !literal_matches(value, datatype) {
                        return Err(self.reject(
                            Some(*property),
                            Some(value),
                            ConstraintKind::Range {
                                expected: datatype.to_string(),
                            },
                        ));
                    }
                }
                Some(PropertyRange::Class(range)) => {
                    self.check_object_range(*property, value, range)?
                }
            }
        }
        Ok(())
    }

    /// The object-property half of check 2: the value must be a node
    /// reference, and what it names must be subsumed by the range class.
    ///
    /// A reference may name an *instance* the world holds, a *class* the
    /// ontology declares (taxonomy nodes are legitimate object values — doc
    /// 03 asserts `:CrushingChestPain` for `:hasSymptom`), or the subject of
    /// this very assertion. When the range class declares a `horis:oneOf`
    /// enumeration, existence is not the question at all — membership is,
    /// and that is check 3's — so this check stands aside.
    fn check_object_range(
        &self,
        property: Iri,
        value: &Value,
        range: Iri,
    ) -> Result<(), Box<Witness>> {
        let range_mismatch = || {
            self.reject(
                Some(property),
                Some(value),
                ConstraintKind::Range {
                    expected: self.iri_text(range),
                },
            )
        };

        let Value::Ref(referent) = value else {
            return Err(range_mismatch());
        };

        if self
            .schema()
            .class(range)
            .is_some_and(|c| c.one_of.is_some())
        {
            return Ok(());
        }

        let referent_class = if referent.iri() == self.subject {
            // Self-reference: the class this very assertion is establishing.
            Some(self.class)
        } else if let Some(instance) = self.world.instances.get(&referent.iri()) {
            Some(instance.class)
        } else {
            // `owl:sameAs`-canonical (PRP-002 D-7): an anchored external
            // IRI is as good a class-value as its local twin.
            self.schema().class_declared(referent.iri())
        };

        match referent_class {
            // Restoration mode: the referent may simply not have been
            // restored yet — defer the question to the post-restore pass
            // instead of refusing ([`check_assertion_restoring`]).
            None if self.deferred.is_some() => {
                self.deferred
                    .expect("guarded by the match arm")
                    .borrow_mut()
                    .push(DeferredRef {
                        property,
                        value: value.clone(),
                        range,
                    });
                Ok(())
            }
            None => Err(self.reject(
                Some(property),
                Some(value),
                ConstraintKind::UnknownReference,
            )),
            Some(c) if self.schema().is_subclass_of(c, range) => Ok(()),
            Some(_) => Err(range_mismatch()),
        }
    }

    // -- check 3 -----------------------------------------------------------

    /// **Check 3.** `horis:min`, `horis:max`, `horis:pattern`, and the range
    /// class's `horis:oneOf` enumeration, in that order per property.
    fn check_constraints(&self, properties: &[(Iri, Value)]) -> Result<(), Box<Witness>> {
        for (property, value) in properties {
            let decl = self
                .schema()
                .property(*property)
                .expect("check 2 established that every asserted property is declared");

            if let (Some(bound), Some(actual)) =
                (decl.min.as_ref().and_then(numeric), numeric(value))
                && actual < bound
            {
                return Err(self.reject(
                    Some(*property),
                    Some(value),
                    ConstraintKind::Min {
                        bound: render_bound(decl.min.as_ref()),
                        unit: decl.unit.clone(),
                    },
                ));
            }

            if let (Some(bound), Some(actual)) =
                (decl.max.as_ref().and_then(numeric), numeric(value))
                && actual > bound
            {
                return Err(self.reject(
                    Some(*property),
                    Some(value),
                    ConstraintKind::Max {
                        bound: render_bound(decl.max.as_ref()),
                        unit: decl.unit.clone(),
                    },
                ));
            }

            if let Value::String(text) = value
                && let Some(regex) = self.world.ontology().pattern(*property)
                && !regex.is_match(text)
            {
                return Err(self.reject(
                    Some(*property),
                    Some(value),
                    ConstraintKind::Pattern {
                        pattern: decl
                            .pattern
                            .clone()
                            .expect("a compiled pattern implies a declared one"),
                    },
                ));
            }

            self.check_one_of(decl, value)?;
        }
        Ok(())
    }

    /// The `horis:oneOf` half of check 3: when a property's range class
    /// declares a closed enumeration, the value must name one of its
    /// members.
    ///
    /// Members are *instances* (doc 02's `:AcuityLevel` enumerates
    /// `:Immediate :Urgent :Standard`), so the usual test is IRI identity —
    /// modulo `owl:sameAs`, since an anchored member and its anchor are the
    /// same thing. Where an ontology instead enumerates classes, a subclass
    /// of a member counts as that member, which is the same subsumption
    /// courtesy check 2 extends to ranges.
    fn check_one_of(&self, decl: &PropertyDecl, value: &Value) -> Result<(), Box<Witness>> {
        let Some(PropertyRange::Class(range)) = decl.range else {
            return Ok(());
        };
        let Some(members) = self.schema().class(range).and_then(|c| c.one_of.as_ref()) else {
            return Ok(());
        };
        let Value::Ref(referent) = value else {
            // Check 2 already refuses a literal where a class is expected.
            return Ok(());
        };

        let member = members.iter().any(|&m| {
            // Identity is sameAs-canonical (PRP-002 D-7): an anchored
            // member and its anchor are the same individual.
            self.schema().canonical_iri(m) == self.schema().canonical_iri(referent.iri())
                || (self.schema().class_declared(referent.iri()).is_some()
                    && self.schema().is_subclass_of(referent.iri(), m))
        });
        if member {
            return Ok(());
        }
        Err(self.reject(
            Some(decl.iri),
            Some(value),
            ConstraintKind::OneOf {
                members: members.iter().map(|&m| self.iri_text(m)).collect(),
            },
        ))
    }

    // -- check 4 -----------------------------------------------------------

    /// **Check 4.** `horis:cardinality` and `horis:immutable` — and, since
    /// deciding what the instance would look like *is* the cardinality
    /// question, this is also where the candidate instance is built.
    ///
    /// Upper bounds: `"1"` and `"0..1"` admit at most one value, so a
    /// re-assertion **replaces** rather than accumulates, and an assertion
    /// carrying two distinct values for such a property is refused outright.
    /// `horis:immutable` overrides replacement: re-asserting the value the
    /// property already holds is a no-op, any other value is refused.
    ///
    /// Lower bounds (`"1"`, `"1..*"`) are checked against the *candidate*,
    /// not against the assertion's property list — the invariant worth
    /// holding is that every instance in the world satisfies its minimums,
    /// and a re-assertion that omits a required property the instance
    /// already carries does not endanger that.
    fn check_cardinality(&self, properties: &[(Iri, Value)]) -> Result<Instance, Box<Witness>> {
        let existing = self.world.instances.get(&self.subject);
        let mut values: OrdMap<Iri, Vector<Value>> =
            existing.map(|i| i.properties.clone()).unwrap_or_default();

        for (property, asserted) in group_by_property(properties) {
            let decl = self
                .schema()
                .property(property)
                .expect("check 2 established that every asserted property is declared");
            let held = values.get(&property).cloned().unwrap_or_default();

            if decl.cardinality.is_some_and(Cardinality::is_single_valued) && asserted.len() > 1 {
                return Err(self.reject(
                    Some(property),
                    Some(&asserted[0]),
                    ConstraintKind::CardinalityMax {
                        bound: decl
                            .cardinality
                            .expect("just matched a declared cardinality")
                            .as_str()
                            .to_owned(),
                        asserted: asserted.len(),
                    },
                ));
            }

            if decl.immutable && !held.is_empty() {
                if let Some(different) = asserted.iter().find(|v| !held.contains(v)) {
                    return Err(self.reject(
                        Some(property),
                        Some(different),
                        ConstraintKind::Immutable {
                            existing: self.render_values(&held),
                        },
                    ));
                }
                // Every asserted value is already held: a no-op.
                continue;
            }

            let updated = if decl.cardinality.is_some_and(Cardinality::is_single_valued) {
                asserted.into_iter().collect()
            } else {
                let mut merged = held;
                for value in asserted {
                    if !merged.contains(&value) {
                        merged.push_back(value);
                    }
                }
                merged
            };
            values.insert(property, updated);
        }

        // Lower bounds. Iterating the schema's properties (a `BTreeMap`
        // keyed by `Iri` token) makes which missing property gets reported
        // deterministic — first-declared wins (CLAUDE.md).
        for decl in self.schema().properties() {
            let Some(card) = decl.cardinality else {
                continue;
            };
            if !card.requires_a_value() || !self.applies_to_subject(decl) {
                continue;
            }
            if values.get(&decl.iri).is_none_or(Vector::is_empty) {
                return Err(self.reject(
                    Some(decl.iri),
                    None,
                    ConstraintKind::CardinalityMin {
                        bound: card.as_str().to_owned(),
                    },
                ));
            }
        }

        Ok(Instance {
            class: self.class,
            properties: values,
        })
    }

    /// Whether `decl` is a property of the subject's class: its
    /// `rdfs:domain` covers that class, or it declares no domain at all and
    /// so applies everywhere. The same reading check 2 uses, so a property
    /// that may be asserted here is exactly one that may be *required*
    /// here.
    fn applies_to_subject(&self, decl: &PropertyDecl) -> bool {
        decl.domain
            .is_none_or(|domain| self.schema().is_subclass_of(self.class, domain))
    }

    // -- check 5 -----------------------------------------------------------

    /// **Check 5.** Every asserted property's `horis:assignedBy` policy
    /// against this assertion's provenance — see [`check_provenance`].
    fn check_assertion_provenance(&self, properties: &[(Iri, Value)]) -> Result<(), Box<Witness>> {
        for (property, value) in properties {
            let decl = self
                .schema()
                .property(*property)
                .expect("check 2 established that every asserted property is declared");
            check_provenance(decl, self.provenance)
                .map_err(|kind| self.reject(Some(*property), Some(value), kind))?;
        }
        Ok(())
    }

    // -- rendering ---------------------------------------------------------

    /// Builds the witness for a refused mutation, rendering its domain text.
    ///
    /// `property` and `value` are supplied by every kind whose text names
    /// them; a kind that names neither (a missing class, say) passes `None`
    /// and its template never reads them.
    fn reject(
        &self,
        property: Option<Iri>,
        value: Option<&Value>,
        kind: ConstraintKind,
    ) -> Box<Witness> {
        let class = self.iri_text(self.class);
        let subject = self.iri_text(self.subject);
        let name = property.map(|p| self.local_name(p)).unwrap_or_default();
        let rendered = value.map(|v| self.render_value(v));
        let shown = rendered.clone().unwrap_or_default();

        let text = match &kind {
            ConstraintKind::ClassNotDeclared => {
                format!("{class} is not a declared class in the ontology")
            }
            ConstraintKind::ClassConflict { existing } => format!(
                "{subject} is already an instance of {existing}; it cannot be re-asserted as {class}"
            ),
            ConstraintKind::ReservedSubject { namespace } => format!(
                "{subject} is in {namespace}, the namespace reserved for anonymous instances; only the world may name instances there"
            ),
            ConstraintKind::PropertyNotDeclared => {
                format!("{name} is not a declared property in the ontology")
            }
            ConstraintKind::Domain { expected } => {
                format!("{name} does not apply to {class} (rdfs:domain {expected})")
            }
            ConstraintKind::Range { expected } => {
                format!("{name} {shown} does not match rdfs:range {expected} on {class}")
            }
            ConstraintKind::UnknownReference => {
                format!("{name} {shown} names an instance that is not in the world on {class}")
            }
            ConstraintKind::Min { bound, unit } => format!(
                "{name} {shown} is below horis:min {bound}{} on {class}",
                unit_suffix(unit)
            ),
            ConstraintKind::Max { bound, unit } => format!(
                "{name} {shown} exceeds horis:max {bound}{} on {class}",
                unit_suffix(unit)
            ),
            ConstraintKind::Pattern { pattern } => {
                format!("{name} {shown} does not match horis:pattern {pattern:?} on {class}")
            }
            ConstraintKind::OneOf { members } => format!(
                "{name} {shown} is not one of horis:oneOf ({}) on {class}",
                members.join(" ")
            ),
            ConstraintKind::CardinalityMax { bound, asserted } => format!(
                "{name} was asserted with {asserted} values but horis:cardinality {bound:?} admits at most one on {class}"
            ),
            ConstraintKind::CardinalityMin { bound } => match self.operation {
                Operation::Assert => format!(
                    "{name} is required by horis:cardinality {bound:?} but no value is asserted on {class}"
                ),
                Operation::Retract => format!(
                    "{name} is required by horis:cardinality {bound:?}; retracting {shown} would leave {subject} without one"
                ),
            },
            ConstraintKind::Immutable { existing } => format!(
                "{name} is horis:immutable and already holds {existing}; it cannot be re-asserted as {shown} on {class}"
            ),
            ConstraintKind::ProvenanceRefused { policy } => format!(
                "{name} may only be assigned by {} (horis:assignedBy {policy}); {} came from {}",
                assigned_by_audience(
                    self.schema()
                        .property(property.expect("a policy rejection always names its property"))
                        .expect("check 2 established that the property is declared")
                        .assigned_by
                ),
                self.operation.noun(),
                self.provenance.as_str(),
            ),
            ConstraintKind::SubjectNotPresent => {
                format!("{subject} is not an instance in the world")
            }
            ConstraintKind::FactNotPresent => {
                format!("{subject} does not hold {name} {shown}")
            }
            ConstraintKind::ImmutableRetraction => format!(
                "{name} is horis:immutable and holds {shown}; an immutable fact can be neither re-asserted differently nor retracted on {class}"
            ),
            ConstraintKind::DerivedFactRetraction { rule } => format!(
                "{name} {shown} was derived by {rule}; derived facts change only by re-running inference after base-fact changes; retraction came from {}",
                self.provenance.as_str()
            ),
        };

        Box::new(Witness {
            operation: self.operation,
            provenance: self.provenance,
            subject: self.world.resolve(self.subject).to_owned(),
            class: self.world.resolve(self.class).to_owned(),
            property: property.map(|p| self.world.resolve(p).to_owned()),
            value: rendered,
            kind,
            text,
        })
    }

    /// `iri` in prefixed form (`:Patient`) — how domain text names classes
    /// and node values.
    fn iri_text(&self, iri: Iri) -> String {
        self.world.ontology().compact(self.world.resolve(iri))
    }

    /// `iri`'s local name (`heartRate`) — how domain text names properties.
    fn local_name(&self, iri: Iri) -> String {
        self.world.ontology().local_name(self.world.resolve(iri))
    }

    /// One value as domain text shows it: node references prefixed, strings
    /// quoted (and escaped), other literals in their XSD lexical form.
    fn render_value(&self, value: &Value) -> String {
        match value {
            Value::Ref(referent) => self.iri_text(referent.iri()),
            Value::String(text) => format!("{text:?}"),
            other => other
                .lexical_form()
                .expect("every non-Ref value has a lexical form"),
        }
    }

    /// Several values, space-separated — for the `horis:immutable` witness,
    /// which reports everything the property already holds.
    fn render_values(&self, values: &Vector<Value>) -> String {
        values
            .iter()
            .map(|v| self.render_value(v))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// -- Free helpers ----------------------------------------------------------

/// Whether a literal satisfies a declared XSD range. `xsd:integer` is
/// derived from `xsd:decimal` in XSD's type hierarchy, so an integer
/// satisfies a decimal range; the converse does not hold.
fn literal_matches(value: &Value, datatype: Datatype) -> bool {
    matches!(
        (value, datatype),
        (Value::Decimal(_), Datatype::Decimal)
            | (Value::Integer(_), Datatype::Decimal | Datatype::Integer)
            | (Value::String(_), Datatype::String)
            | (Value::Boolean(_), Datatype::Boolean)
            | (Value::DateTime(_), Datatype::DateTime)
            | (Value::Duration(_), Datatype::Duration)
    )
}

/// A value as an exact decimal, if it is numeric — `horis:min`/`horis:max`
/// are numeric bounds, and comparing through [`Decimal`] keeps the
/// comparison exact for both `xsd:integer` and `xsd:decimal` (no float, so
/// no platform-dependent rounding; CLAUDE.md determinism).
fn numeric(value: &Value) -> Option<Decimal> {
    match value {
        Value::Decimal(d) => Some(*d),
        Value::Integer(i) => Some(Decimal::from(*i)),
        _ => None,
    }
}

/// A declared bound in its lexical form, for the witness and its text.
fn render_bound(bound: Option<&Value>) -> String {
    bound
        .and_then(Value::lexical_form)
        .expect("a numeric bound always has a lexical form")
}

/// `" (bpm)"` for a property that declares `horis:unit`, nothing otherwise.
fn unit_suffix(unit: &Option<String>) -> String {
    unit.as_ref().map(|u| format!(" ({u})")).unwrap_or_default()
}

/// Collapses an assertion's `(property, value)` list into one entry per
/// property, in first-mention order, dropping values a property was given
/// twice.
///
/// Deduplicating *before* check 4 counts is deliberate: asserting the same
/// symptom twice is a redundant statement, not a cardinality violation.
fn group_by_property(properties: &[(Iri, Value)]) -> Vec<(Iri, Vec<Value>)> {
    let mut grouped: Vec<(Iri, Vec<Value>)> = Vec::new();
    for (property, value) in properties {
        match grouped.iter_mut().find(|(p, _)| p == property) {
            Some((_, values)) => {
                if !values.contains(value) {
                    values.push(value.clone());
                }
            }
            None => grouped.push((*property, vec![value.clone()])),
        }
    }
    grouped
}
