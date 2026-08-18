//! The Schema (U4): classes, the `subClassOf` closure, property
//! domain/range/constraints, and class enumerations, built from a
//! profile-valid [`Document`] (U3) into a queryable structure. This is U6's
//! direct input to the validation gate.
//!
//! Rules and workflows are *not* modeled here — they parse and pass profile
//! validation (U3), but structured rule/workflow models are PRP-002/004's
//! job (`docs/framework/02-ontology.md`, `docs/framework/05-workflows.md`).
//!
//! Classes and properties are stored in `BTreeMap<Iri, _>`, keyed by the
//! `Iri` token itself. Since `Iri` tokens are assigned in first-seen
//! (interning) order (`term.rs`), iterating a `BTreeMap<Iri, _>` yields a
//! deterministic order — first-seen order — without a separate index, while
//! keeping point lookups (`class`/`property`) logarithmic ("O(1)-ish" for
//! the ontology sizes this profile targets).
//!
//! Spec: `docs/framework/02-ontology.md` · Owner: PRP-001

use std::collections::{BTreeMap, BTreeSet};

use rust_decimal::Decimal;

use crate::error::OntologyError;
use crate::ontology::profile::{HORIS_ASSIGNED_BY, HORIS_CARDINALITY};
use crate::ontology::term::{Datatype, Interner, Iri, Value};
use crate::ontology::turtle::{Document, Literal, Node, Object, Triple, xsd_datatype_from_iri};

// -- Vocabulary this module reads directly -------------------------------

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_CLASS: &str = "http://www.w3.org/2000/01/rdf-schema#Class";
const RDF_PROPERTY: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Property";
const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
const RDFS_DOMAIN: &str = "http://www.w3.org/2000/01/rdf-schema#domain";
const RDFS_RANGE: &str = "http://www.w3.org/2000/01/rdf-schema#range";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const RDFS_COMMENT: &str = "http://www.w3.org/2000/01/rdf-schema#comment";
const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";
const HORIS_MIN: &str = "https://w3id.org/horismos#min";
const HORIS_MAX: &str = "https://w3id.org/horismos#max";
const HORIS_UNIT: &str = "https://w3id.org/horismos#unit";
const HORIS_ONE_OF: &str = "https://w3id.org/horismos#oneOf";
const HORIS_PATTERN: &str = "https://w3id.org/horismos#pattern";
const HORIS_IMMUTABLE: &str = "https://w3id.org/horismos#immutable";
const HORIS_RULES_ONLY: &str = "https://w3id.org/horismos#RulesOnly";
const HORIS_PERCEPTION_ALLOWED: &str = "https://w3id.org/horismos#PerceptionAllowed";
const HORIS_TOOLS_ONLY: &str = "https://w3id.org/horismos#ToolsOnly";
/// The one OWL term the RDFS-plus profile admits (`docs/framework/02-ontology.md`):
/// anchoring a local class to an external vocabulary's IRI.
const OWL_SAME_AS: &str = "http://www.w3.org/2002/07/owl#sameAs";

// -- Public model ----------------------------------------------------------

/// `horis:cardinality`, parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    One,
    ZeroOrOne,
    OneOrMany,
    ZeroOrMany,
}

impl Cardinality {
    /// The declared lexical form, for domain-term errors ("… but
    /// horis:cardinality \"0..1\" allows at most one value …").
    pub fn as_str(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::ZeroOrOne => "0..1",
            Self::OneOrMany => "1..*",
            Self::ZeroOrMany => "0..*",
        }
    }

    /// True for `"1"` and `"0..1"` — the bounds that admit at most one value.
    pub fn is_single_valued(self) -> bool {
        matches!(self, Self::One | Self::ZeroOrOne)
    }

    /// True for `"1"` and `"1..*"` — the bounds that require a value.
    pub fn requires_a_value(self) -> bool {
        matches!(self, Self::One | Self::OneOrMany)
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "1" => Some(Self::One),
            "0..1" => Some(Self::ZeroOrOne),
            "1..*" => Some(Self::OneOrMany),
            "0..*" => Some(Self::ZeroOrMany),
            _ => None,
        }
    }
}

/// `horis:assignedBy`, parsed. Defaults to [`AssignedBy::PerceptionAllowed`]
/// when the property declares no `horis:assignedBy` triple at all
/// (`docs/framework/02-ontology.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssignedBy {
    RulesOnly,
    #[default]
    PerceptionAllowed,
    ToolsOnly,
}

impl AssignedBy {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            HORIS_RULES_ONLY => Some(Self::RulesOnly),
            HORIS_PERCEPTION_ALLOWED => Some(Self::PerceptionAllowed),
            HORIS_TOOLS_ONLY => Some(Self::ToolsOnly),
            _ => None,
        }
    }
}

/// A property's `rdfs:range`: either another declared class, or one of the
/// six XSD datatypes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyRange {
    Class(Iri),
    Datatype(Datatype),
}

/// A declared `rdfs:Class`, or a class that only appears as a
/// `rdfs:subClassOf` parent/child (still a class per the profile — see
/// module docs).
#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub iri: Iri,
    pub label: Option<String>,
    pub comment: Option<String>,
    /// `horis:oneOf`'s closed instance enumeration, in list order, if this
    /// class declares one.
    pub one_of: Option<Vec<Iri>>,
}

/// A declared `rdf:Property` and its full constraint set.
#[derive(Debug, Clone)]
pub struct PropertyDecl {
    pub iri: Iri,
    pub domain: Option<Iri>,
    pub range: Option<PropertyRange>,
    pub unit: Option<String>,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub cardinality: Option<Cardinality>,
    pub pattern: Option<String>,
    pub immutable: bool,
    pub assigned_by: AssignedBy,
    pub label: Option<String>,
    pub comment: Option<String>,
}

/// The queryable schema: classes, properties, and the `subClassOf` closure.
/// Built once by [`Schema::build`] (or, over a named-graph set, U5's
/// [`Schema::build_all`]) from profile-valid [`Document`]s — U6's gate
/// consults it on every World mutation.
#[derive(Debug, Clone, Default)]
pub struct Schema {
    classes: BTreeMap<Iri, ClassDecl>,
    properties: BTreeMap<Iri, PropertyDecl>,
    /// Reflexive-transitive closure, keyed by *canonical* IRI (see
    /// `same_as` below): `superclasses[canonical(c)]` contains
    /// `canonical(c)` itself plus every ancestor's canonical IRI.
    superclasses: BTreeMap<Iri, BTreeSet<Iri>>,
    /// The reverse of `superclasses`, canonical-keyed the same way.
    subclasses: BTreeMap<Iri, BTreeSet<Iri>>,
    /// `owl:sameAs` anchoring (U5, `docs/framework/02-ontology.md` §Imports
    /// and anchoring): every IRI that appeared on either side of a
    /// `sameAs` triple, in any graph, maps to its equivalence class's
    /// canonical representative. `sameAs` is symmetric and transitive
    /// (union-find over the raw pairs); the canonical representative is the
    /// *lowest* interned [`Iri`] token in the class — since `Interner`
    /// assigns tokens in first-seen order (`term.rs`), that's the
    /// first-declared member, and the choice is deterministic regardless of
    /// how many `sameAs` triples exist or what order they're read in. An
    /// IRI absent from this map was never named in a `sameAs` triple and is
    /// its own canonical representative.
    same_as: BTreeMap<Iri, Iri>,
}

impl Schema {
    /// Looks up a class declaration by IRI.
    pub fn class(&self, iri: Iri) -> Option<&ClassDecl> {
        self.classes.get(&iri)
    }

    /// Looks up a property declaration by IRI.
    pub fn property(&self, iri: Iri) -> Option<&PropertyDecl> {
        self.properties.get(&iri)
    }

    /// Every declared class, in `Iri`-token (first-seen) order.
    pub fn classes(&self) -> impl Iterator<Item = &ClassDecl> {
        self.classes.values()
    }

    /// Every declared property, in `Iri`-token (first-seen) order.
    pub fn properties(&self) -> impl Iterator<Item = &PropertyDecl> {
        self.properties.values()
    }

    /// True iff `a` is `b`, or a transitive `rdfs:subClassOf` descendant of
    /// `b` — `owl:sameAs`-bridged: if `a` (or an IRI `a` is `sameAs`) has an
    /// imported-graph ancestor that `b` (or an IRI `b` is `sameAs`) is
    /// anchored to, this holds too (U5). Reflexive: `is_subclass_of(a, a)`
    /// is always true for a declared class `a`.
    pub fn is_subclass_of(&self, a: Iri, b: Iri) -> bool {
        self.superclasses
            .get(&self.canonical(a))
            .is_some_and(|s| s.contains(&self.canonical(b)))
    }

    /// `c` and every transitive `rdfs:subClassOf` ancestor of `c` —
    /// `sameAs`-bridged (U5) — in `Iri`-token order. Ancestors are returned
    /// as their *canonical* IRI (see `Schema`'s `same_as` field); empty if
    /// `c` is not a declared class.
    pub fn superclasses(&self, c: Iri) -> impl Iterator<Item = Iri> + '_ {
        self.superclasses
            .get(&self.canonical(c))
            .into_iter()
            .flatten()
            .copied()
    }

    /// `c` and every transitive `rdfs:subClassOf` descendant of `c` —
    /// `sameAs`-bridged (U5) — in `Iri`-token order. Descendants are
    /// returned as their *canonical* IRI; empty if `c` is not a declared
    /// class.
    pub fn subclasses(&self, c: Iri) -> impl Iterator<Item = Iri> + '_ {
        self.subclasses
            .get(&self.canonical(c))
            .into_iter()
            .flatten()
            .copied()
    }

    /// `iri`'s `owl:sameAs` canonical representative, or `iri` itself if it
    /// never appeared in a `sameAs` triple.
    fn canonical(&self, iri: Iri) -> Iri {
        self.same_as.get(&iri).copied().unwrap_or(iri)
    }

    /// `iri`'s `owl:sameAs` canonical representative — public face of the
    /// equivalence map (PRP-002 D-7), for *individual* identity questions
    /// ("is `:Immediate` the same individual as its anchored twin?") where
    /// no class declaration is involved. Returns `iri` itself when it never
    /// appeared in a `sameAs` triple.
    pub fn canonical_iri(&self, iri: Iri) -> Iri {
        self.canonical(iri)
    }

    /// The *declared* class `iri` names, looked up `owl:sameAs`-canonically
    /// (PRP-002 D-7): an exact declaration wins; otherwise any member of
    /// `iri`'s `sameAs` equivalence class that *is* declared answers for it,
    /// lowest-token member first (deterministic — `BTreeMap` order). `None`
    /// when neither `iri` nor any of its twins is a declared class.
    ///
    /// This is how an anchored external IRI (`sct:29857009`) becomes
    /// first-class wherever its local twin (`:ChestPain`) is: rule
    /// conditions, gate range/oneOf checks, and reasoner class positions all
    /// resolve through here rather than through exact-IRI [`Schema::class`].
    pub fn class_declared(&self, iri: Iri) -> Option<Iri> {
        if self.classes.contains_key(&iri) {
            return Some(iri);
        }
        let canonical = self.canonical(iri);
        if self.classes.contains_key(&canonical) {
            return Some(canonical);
        }
        self.same_as
            .iter()
            .filter(|&(_, &c)| c == canonical)
            .map(|(&member, _)| member)
            .find(|member| self.classes.contains_key(member))
    }

    /// Builds a [`Schema`] from `document`. Callers are expected to have
    /// already run [`crate::ontology::profile::validate`] — an
    /// out-of-profile document (a bad datatype in `rdfs:range`, say) simply
    /// surfaces as one of this function's own errors ("undeclared class")
    /// rather than a distinct check, since U3 already owns rejecting
    /// out-of-profile vocabulary.
    pub fn build(document: &Document, interner: &Interner) -> Result<Schema, OntologyError> {
        Self::build_all([document], interner)
    }

    /// Builds a [`Schema`] from the root document plus every imported graph
    /// (U5, `ontology::imports::OntologySet::documents`) as one merged
    /// schema: classes/properties from every graph become queryable
    /// alongside the root's own, and `owl:sameAs` anchors between them
    /// (`docs/framework/02-ontology.md` §Imports and anchoring) bridge the
    /// `subClassOf` closure across graphs — see `Schema`'s `same_as` field.
    /// `Schema::build` is the single-document case of this.
    pub fn build_all<'a>(
        documents: impl IntoIterator<Item = &'a Document>,
        interner: &Interner,
    ) -> Result<Schema, OntologyError> {
        let raw = RawTriples::collect_all(documents, interner);
        let same_as = canonicalize_same_as(&raw.same_as_pairs);
        let canonical = |iri: Iri| same_as.get(&iri).copied().unwrap_or(iri);

        let canon_class_ids: BTreeSet<Iri> = raw.class_ids.iter().map(|&c| canonical(c)).collect();
        let mut canon_parents: BTreeMap<Iri, Vec<Iri>> = BTreeMap::new();
        for (&child, parents) in &raw.subclass_parents {
            canon_parents
                .entry(canonical(child))
                .or_default()
                .extend(parents.iter().map(|&p| canonical(p)));
        }

        let (superclasses, subclasses) =
            close_subclasses(&canon_class_ids, &canon_parents, interner)?;

        let mut classes = BTreeMap::new();
        for &iri in &raw.class_ids {
            classes.insert(
                iri,
                ClassDecl {
                    iri,
                    label: raw.labels.get(&iri).cloned(),
                    comment: raw.comments.get(&iri).cloned(),
                    one_of: None,
                },
            );
        }

        for (&subject, &head) in &raw.one_of_heads {
            if !raw.class_ids.contains(&subject) {
                return Err(OntologyError::OneOfOnNonClass {
                    subject: interner.resolve(subject).to_owned(),
                });
            }
            let members = walk_rdf_list(head, &raw.list_first, &raw.list_rest, interner);
            classes
                .get_mut(&subject)
                .expect("just inserted above")
                .one_of = Some(members);
        }

        let mut properties = BTreeMap::new();
        for &iri in &raw.property_ids {
            let property_name = interner.resolve(iri).to_owned();

            let domain = match raw.domains.get(&iri) {
                Some(&d) if raw.class_ids.contains(&d) => Some(d),
                Some(&d) => {
                    return Err(OntologyError::UndeclaredClass {
                        property: property_name,
                        position: "domain".to_owned(),
                        class: interner.resolve(d).to_owned(),
                    });
                }
                None => None,
            };

            let range = match raw.ranges.get(&iri) {
                Some(&r) => {
                    let r_str = interner.resolve(r);
                    if let Some(dt) = xsd_datatype_from_iri(r_str) {
                        Some(PropertyRange::Datatype(dt))
                    } else if raw.class_ids.contains(&r) {
                        Some(PropertyRange::Class(r))
                    } else {
                        return Err(OntologyError::UndeclaredClass {
                            property: property_name,
                            position: "range".to_owned(),
                            class: r_str.to_owned(),
                        });
                    }
                }
                // No `rdfs:range` triple at all: left unset, mirroring
                // `domain`'s treatment, rather than guessing a datatype.
                None => None,
            };

            let cardinality = match raw.cardinalities.get(&iri) {
                Some(raw_value) => match Cardinality::parse(raw_value) {
                    Some(c) => Some(c),
                    None => {
                        return Err(OntologyError::InvalidCardinality {
                            value: raw_value.clone(),
                        });
                    }
                },
                None => None,
            };

            let assigned_by = match raw.assigned_bys.get(&iri) {
                Some(raw_value) => match AssignedBy::parse(raw_value) {
                    Some(a) => a,
                    None => {
                        return Err(OntologyError::InvalidAssignedBy {
                            value: raw_value.clone(),
                        });
                    }
                },
                None => AssignedBy::default(),
            };

            let min = raw.mins.get(&iri).cloned();
            let max = raw.maxs.get(&iri).cloned();
            if let (Some(min_v), Some(max_v)) = (&min, &max)
                && let (Some(min_d), Some(max_d)) =
                    (numeric_as_decimal(min_v), numeric_as_decimal(max_v))
                && min_d > max_d
            {
                return Err(OntologyError::MinExceedsMax {
                    property: property_name,
                    min: min_v.lexical_form().unwrap_or_default(),
                    max: max_v.lexical_form().unwrap_or_default(),
                });
            }

            properties.insert(
                iri,
                PropertyDecl {
                    iri,
                    domain,
                    range,
                    unit: raw.units.get(&iri).cloned(),
                    min,
                    max,
                    cardinality,
                    pattern: raw.patterns.get(&iri).cloned(),
                    immutable: raw.immutables.get(&iri).copied().unwrap_or(false),
                    assigned_by,
                    label: raw.labels.get(&iri).cloned(),
                    comment: raw.comments.get(&iri).cloned(),
                },
            );
        }

        Ok(Schema {
            classes,
            properties,
            superclasses,
            subclasses,
            same_as,
        })
    }
}

/// Builds the `owl:sameAs` equivalence map from every raw `sameAs` pair
/// collected across every graph: symmetric + transitive closure via
/// union-find, canonical representative = the *lowest* interned [`Iri`]
/// token in each equivalence class (see `Schema`'s `same_as` field doc for
/// why that's deterministic).
fn canonicalize_same_as(pairs: &[(Iri, Iri)]) -> BTreeMap<Iri, Iri> {
    let mut parent: BTreeMap<Iri, Iri> = BTreeMap::new();
    for &(a, b) in pairs {
        parent.entry(a).or_insert(a);
        parent.entry(b).or_insert(b);
        let ra = find_root(&mut parent, a);
        let rb = find_root(&mut parent, b);
        if ra != rb {
            let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
            parent.insert(hi, lo);
        }
    }
    // A second pass so every key maps directly to its final root — `find_root`
    // path-compresses as it goes, but only along the path taken during
    // union(); this guarantees a flat map regardless of union order.
    let keys: Vec<Iri> = parent.keys().copied().collect();
    let mut canon = BTreeMap::new();
    for k in keys {
        canon.insert(k, find_root(&mut parent, k));
    }
    canon
}

/// Union-find `find`, with path compression, over a plain `BTreeMap` parent
/// pointer table (no separate rank — [`canonicalize_same_as`] always
/// attaches the larger token under the smaller one, so the tree never grows
/// pathologically deep for the ontology sizes this profile targets).
fn find_root(parent: &mut BTreeMap<Iri, Iri>, x: Iri) -> Iri {
    let mut root = x;
    while let Some(&p) = parent.get(&root) {
        if p == root {
            break;
        }
        root = p;
    }
    let mut cur = x;
    while cur != root {
        let next = parent[&cur];
        parent.insert(cur, root);
        cur = next;
    }
    root
}

fn numeric_as_decimal(value: &Value) -> Option<Decimal> {
    match value {
        Value::Decimal(d) => Some(*d),
        Value::Integer(i) => Some(Decimal::from(*i)),
        _ => None,
    }
}

fn node_iri(node: Node) -> Iri {
    match node {
        Node::Named(iri) | Node::Blank(iri) => iri,
    }
}

fn object_node_iri(object: &Object) -> Option<Iri> {
    match object {
        Object::Node(n) => Some(node_iri(*n)),
        Object::Literal(_) => None,
    }
}

fn object_as_string(object: &Object) -> Option<String> {
    match object {
        Object::Literal(Literal::Typed(Value::String(s))) => Some(s.clone()),
        _ => None,
    }
}

fn object_as_bool(object: &Object) -> Option<bool> {
    match object {
        Object::Literal(Literal::Typed(Value::Boolean(b))) => Some(*b),
        _ => None,
    }
}

fn object_as_value(object: &Object) -> Option<Value> {
    match object {
        Object::Literal(Literal::Typed(v)) => Some(v.clone()),
        _ => None,
    }
}

/// The raw per-subject facts gathered by one pass over `document.triples`,
/// before any cross-checking (cycles, undeclared classes, min>max) happens.
/// Later triples win when a predicate is repeated for the same subject —
/// documents aren't expected to redeclare a property's constraints, and
/// nothing in the brief requires flagging it as an error.
#[derive(Default)]
struct RawTriples {
    class_ids: BTreeSet<Iri>,
    subclass_parents: BTreeMap<Iri, Vec<Iri>>,
    property_ids: BTreeSet<Iri>,
    labels: BTreeMap<Iri, String>,
    comments: BTreeMap<Iri, String>,
    domains: BTreeMap<Iri, Iri>,
    ranges: BTreeMap<Iri, Iri>,
    units: BTreeMap<Iri, String>,
    mins: BTreeMap<Iri, Value>,
    maxs: BTreeMap<Iri, Value>,
    /// Cardinality's raw lexical value, kept as text (not yet parsed) so a
    /// bad value can still be named in an error.
    cardinalities: BTreeMap<Iri, String>,
    patterns: BTreeMap<Iri, String>,
    immutables: BTreeMap<Iri, bool>,
    /// `assignedBy`'s raw object, rendered as text (the resolved IRI when
    /// it's a node, else a fallback) so a bad value can still be named.
    assigned_bys: BTreeMap<Iri, String>,
    one_of_heads: BTreeMap<Iri, Node>,
    list_first: BTreeMap<Iri, Object>,
    list_rest: BTreeMap<Iri, Node>,
    /// Every `owl:sameAs` triple seen, as `(subject, object)` — U5 anchoring
    /// (`docs/framework/02-ontology.md` §Imports and anchoring). Raw pairs
    /// only; [`canonicalize_same_as`] turns them into an equivalence map.
    same_as_pairs: Vec<(Iri, Iri)>,
}

impl RawTriples {
    /// Gathers `RawTriples` across several [`Document`]s (U5: one per named
    /// graph) as if they were one — the merged view `Schema::build_all`
    /// needs. Cross-graph structural checks (subclass cycles, undeclared
    /// domain/range classes, …) run downstream in `Schema::build_all`
    /// itself, over this merged data, exactly as they do for a single
    /// document.
    fn collect_all<'a>(
        documents: impl IntoIterator<Item = &'a Document>,
        interner: &Interner,
    ) -> Self {
        let mut raw = RawTriples::default();
        for document in documents {
            for triple in &document.triples {
                raw.ingest(triple, interner);
            }
        }
        raw
    }

    fn ingest(&mut self, triple: &Triple, interner: &Interner) {
        let raw = self;
        let subject = node_iri(triple.subject);
        let predicate = interner.resolve(triple.predicate);

        match predicate {
            RDF_TYPE => match object_node_iri(&triple.object).map(|i| interner.resolve(i)) {
                Some(RDFS_CLASS) => {
                    raw.class_ids.insert(subject);
                }
                Some(RDF_PROPERTY) => {
                    raw.property_ids.insert(subject);
                }
                _ => {}
            },
            RDFS_SUBCLASS_OF => {
                if let Some(parent) = object_node_iri(&triple.object) {
                    raw.class_ids.insert(subject);
                    raw.class_ids.insert(parent);
                    raw.subclass_parents
                        .entry(subject)
                        .or_default()
                        .push(parent);
                }
            }
            RDFS_DOMAIN => {
                if let Some(d) = object_node_iri(&triple.object) {
                    raw.domains.insert(subject, d);
                }
            }
            RDFS_RANGE => {
                if let Some(r) = object_node_iri(&triple.object) {
                    raw.ranges.insert(subject, r);
                }
            }
            RDFS_LABEL => {
                if let Some(s) = object_as_string(&triple.object) {
                    raw.labels.insert(subject, s);
                }
            }
            RDFS_COMMENT => {
                if let Some(s) = object_as_string(&triple.object) {
                    raw.comments.insert(subject, s);
                }
            }
            HORIS_UNIT => {
                if let Some(s) = object_as_string(&triple.object) {
                    raw.units.insert(subject, s);
                }
            }
            HORIS_MIN => {
                if let Some(v) = object_as_value(&triple.object) {
                    raw.mins.insert(subject, v);
                }
            }
            HORIS_MAX => {
                if let Some(v) = object_as_value(&triple.object) {
                    raw.maxs.insert(subject, v);
                }
            }
            HORIS_CARDINALITY => {
                let text = object_as_string(&triple.object)
                    .unwrap_or_else(|| object_display(interner, &triple.object));
                raw.cardinalities.insert(subject, text);
            }
            HORIS_PATTERN => {
                if let Some(s) = object_as_string(&triple.object) {
                    raw.patterns.insert(subject, s);
                }
            }
            HORIS_IMMUTABLE => {
                if let Some(b) = object_as_bool(&triple.object) {
                    raw.immutables.insert(subject, b);
                }
            }
            HORIS_ASSIGNED_BY => {
                let text = object_display(interner, &triple.object);
                raw.assigned_bys.insert(subject, text);
            }
            HORIS_ONE_OF => {
                if let Object::Node(node) = &triple.object {
                    raw.one_of_heads.insert(subject, *node);
                }
            }
            RDF_FIRST => {
                raw.list_first.insert(subject, triple.object.clone());
            }
            RDF_REST => {
                if let Object::Node(node) = &triple.object {
                    raw.list_rest.insert(subject, *node);
                }
            }
            OWL_SAME_AS => {
                if let Some(o) = object_node_iri(&triple.object) {
                    raw.same_as_pairs.push((subject, o));
                }
            }
            _ => {}
        }
    }
}

fn object_display(interner: &Interner, object: &Object) -> String {
    match object {
        Object::Node(node) => interner.resolve(node_iri(*node)).to_owned(),
        Object::Literal(Literal::Typed(v)) => v.lexical_form().unwrap_or_default(),
        Object::Literal(Literal::Other { lexical, .. }) => lexical.clone(),
        Object::Literal(Literal::LangString { lexical, .. }) => lexical.clone(),
    }
}

/// Walks the `rdf:first`/`rdf:rest` chain that `( ... )` Turtle syntax
/// desugars into, starting at `head`, collecting each `rdf:first` that is a
/// named node. Stops at `rdf:nil`, or if the chain runs out (a malformed
/// list from a document that skipped U3 profile validation) — best-effort,
/// since well-formedness of the list machinery itself is U3's job.
fn walk_rdf_list(
    head: Node,
    first: &BTreeMap<Iri, Object>,
    rest: &BTreeMap<Iri, Node>,
    interner: &Interner,
) -> Vec<Iri> {
    let mut out = Vec::new();
    let mut visited = BTreeSet::new();
    let mut cursor = head;
    loop {
        let cell = node_iri(cursor);
        // The visited check keeps a hand-authored cyclic list
        // (`_:l rdf:rest _:l`) from hanging the load (PRP-002 review).
        if interner.resolve(cell) == RDF_NIL || !visited.insert(cell) {
            break;
        }
        match first.get(&cell) {
            Some(Object::Node(Node::Named(item))) => out.push(*item),
            _ => break,
        }
        match rest.get(&cell) {
            Some(&next) => cursor = next,
            None => break,
        }
    }
    out
}

// -- subClassOf closure, with cycle detection ----------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Color {
    Gray,
    Black,
}

/// Computes the reflexive-transitive `subClassOf` closure in both
/// directions (ancestors and descendants), rejecting any cycle in the raw
/// `subClassOf` edges with a domain-term error naming the cycle.
type ClosureMap = BTreeMap<Iri, BTreeSet<Iri>>;

fn close_subclasses(
    class_ids: &BTreeSet<Iri>,
    parents: &BTreeMap<Iri, Vec<Iri>>,
    interner: &Interner,
) -> Result<(ClosureMap, ClosureMap), OntologyError> {
    let mut color: BTreeMap<Iri, Color> = BTreeMap::new();
    let mut memo: BTreeMap<Iri, BTreeSet<Iri>> = BTreeMap::new();
    let mut path: Vec<Iri> = Vec::new();

    let mut superclasses = BTreeMap::new();
    for &c in class_ids {
        let ancestors = visit_ancestors(c, parents, &mut color, &mut memo, &mut path, interner)?;
        superclasses.insert(c, ancestors);
    }

    let mut subclasses: BTreeMap<Iri, BTreeSet<Iri>> =
        class_ids.iter().map(|&c| (c, BTreeSet::new())).collect();
    for (&descendant, ancestors) in &superclasses {
        for &ancestor in ancestors {
            subclasses.entry(ancestor).or_default().insert(descendant);
        }
    }

    Ok((superclasses, subclasses))
}

fn visit_ancestors(
    node: Iri,
    parents: &BTreeMap<Iri, Vec<Iri>>,
    color: &mut BTreeMap<Iri, Color>,
    memo: &mut BTreeMap<Iri, BTreeSet<Iri>>,
    path: &mut Vec<Iri>,
    interner: &Interner,
) -> Result<BTreeSet<Iri>, OntologyError> {
    match color.get(&node) {
        Some(Color::Black) => return Ok(memo[&node].clone()),
        Some(Color::Gray) => {
            let start = path
                .iter()
                .position(|&n| n == node)
                .expect("a Gray node is always on the current path");
            let mut cycle_text: Vec<&str> =
                path[start..].iter().map(|&i| interner.resolve(i)).collect();
            cycle_text.push(interner.resolve(node));
            return Err(OntologyError::SubclassCycle {
                cycle: cycle_text.join(" -> "),
            });
        }
        None => {}
    }

    color.insert(node, Color::Gray);
    path.push(node);

    let mut ancestors = BTreeSet::new();
    ancestors.insert(node);
    if let Some(direct_parents) = parents.get(&node) {
        for &parent in direct_parents {
            let parent_ancestors = visit_ancestors(parent, parents, color, memo, path, interner)?;
            ancestors.extend(parent_ancestors);
        }
    }

    path.pop();
    color.insert(node, Color::Black);
    memo.insert(node, ancestors.clone());
    Ok(ancestors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ontology::turtle::parse;

    fn iri(interner: &mut Interner, local: &str) -> Iri {
        interner.intern(&format!("https://example.org/t#{local}"))
    }

    fn build(source: &str) -> (Schema, Interner) {
        let full = format!(
            "@prefix : <https://example.org/t#> .\n\
             @prefix horis: <https://w3id.org/horismos#> .\n\
             @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             {source}\n"
        );
        let mut interner = Interner::new();
        let doc = parse(&full, &mut interner).expect("valid Turtle snippet must parse");
        let schema = Schema::build(&doc, &interner).expect("schema build must succeed");
        (schema, interner)
    }

    fn try_build(source: &str) -> Result<Schema, OntologyError> {
        let full = format!(
            "@prefix : <https://example.org/t#> .\n\
             @prefix horis: <https://w3id.org/horismos#> .\n\
             @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
             @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
             {source}\n"
        );
        let mut interner = Interner::new();
        let doc = parse(&full, &mut interner).expect("valid Turtle snippet must parse");
        Schema::build(&doc, &interner)
    }

    #[test]
    fn collects_explicit_class() {
        let (schema, mut interner) = build(":Patient a rdfs:Class ; rdfs:label \"Patient\" .");
        let patient = iri(&mut interner, "Patient");
        let decl = schema.class(patient).expect("Patient must be a class");
        assert_eq!(decl.label.as_deref(), Some("Patient"));
    }

    #[test]
    fn collects_implicit_class_from_subclass_of() {
        let (schema, mut interner) = build(
            ":Symptom a rdfs:Class .\n\
             :ChestPain rdfs:subClassOf :Symptom .",
        );
        let chest_pain = iri(&mut interner, "ChestPain");
        assert!(
            schema.class(chest_pain).is_some(),
            "ChestPain must be a class even without an explicit `a rdfs:Class`"
        );
    }

    #[test]
    fn subclass_closure_is_reflexive_and_transitive() {
        let (schema, mut interner) = build(
            ":Symptom a rdfs:Class .\n\
             :ChestPain rdfs:subClassOf :Symptom .\n\
             :CrushingChestPain rdfs:subClassOf :ChestPain .",
        );
        let symptom = iri(&mut interner, "Symptom");
        let chest_pain = iri(&mut interner, "ChestPain");
        let crushing = iri(&mut interner, "CrushingChestPain");

        assert!(schema.is_subclass_of(crushing, crushing), "reflexive");
        assert!(schema.is_subclass_of(crushing, chest_pain), "direct parent");
        assert!(
            schema.is_subclass_of(crushing, symptom),
            "transitive grandparent"
        );
        assert!(
            !schema.is_subclass_of(symptom, crushing),
            "must not hold in reverse"
        );

        let supers: BTreeSet<Iri> = schema.superclasses(crushing).collect();
        assert_eq!(supers, BTreeSet::from([crushing, chest_pain, symptom]));

        let subs: BTreeSet<Iri> = schema.subclasses(symptom).collect();
        assert_eq!(subs, BTreeSet::from([symptom, chest_pain, crushing]));
    }

    #[test]
    fn rejects_subclass_cycle() {
        let err = try_build(":A rdfs:subClassOf :B .\n:B rdfs:subClassOf :A .")
            .expect_err("a subClassOf cycle must be rejected");
        match err {
            OntologyError::SubclassCycle { cycle } => {
                assert!(cycle.contains("t#A"), "{cycle}");
                assert!(cycle.contains("t#B"), "{cycle}");
            }
            other => panic!("expected SubclassCycle, got {other:?}"),
        }
    }

    #[test]
    fn property_domain_range_and_constraints() {
        let (schema, mut interner) = build(
            ":Patient a rdfs:Class .\n\
             :heartRate a rdf:Property ;\n\
             \x20   rdfs:domain :Patient ;\n\
             \x20   rdfs:range  xsd:decimal ;\n\
             \x20   horis:unit  \"bpm\" ;\n\
             \x20   horis:min   0 ;\n\
             \x20   horis:max   300 ;\n\
             \x20   horis:cardinality \"0..1\" .",
        );
        let patient = iri(&mut interner, "Patient");
        let heart_rate = iri(&mut interner, "heartRate");
        let decl = schema
            .property(heart_rate)
            .expect("heartRate must be a property");
        assert_eq!(decl.domain, Some(patient));
        assert_eq!(decl.range, Some(PropertyRange::Datatype(Datatype::Decimal)));
        assert_eq!(decl.unit.as_deref(), Some("bpm"));
        assert_eq!(decl.min, Some(Value::Integer(0)));
        assert_eq!(decl.max, Some(Value::Integer(300)));
        assert_eq!(decl.cardinality, Some(Cardinality::ZeroOrOne));
        assert_eq!(decl.assigned_by, AssignedBy::PerceptionAllowed, "default");
    }

    #[test]
    fn property_range_can_be_a_class() {
        let (schema, mut interner) = build(
            ":Patient a rdfs:Class .\n\
             :Symptom a rdfs:Class .\n\
             :hasSymptom a rdf:Property ;\n\
             \x20   rdfs:domain :Patient ;\n\
             \x20   rdfs:range  :Symptom .",
        );
        let symptom = iri(&mut interner, "Symptom");
        let has_symptom = iri(&mut interner, "hasSymptom");
        let decl = schema
            .property(has_symptom)
            .expect("hasSymptom must be a property");
        assert_eq!(decl.range, Some(PropertyRange::Class(symptom)));
    }

    #[test]
    fn property_domain_and_range_are_none_when_undeclared() {
        let (schema, mut interner) = build(":hasFoo a rdf:Property .");
        let has_foo = iri(&mut interner, "hasFoo");
        let decl = schema.property(has_foo).expect("hasFoo must be a property");
        assert_eq!(decl.domain, None);
        assert_eq!(decl.range, None);
    }

    #[test]
    fn assigned_by_rules_only_is_parsed() {
        let (schema, mut interner) = build(
            ":Patient a rdfs:Class .\n\
             :AcuityLevel a rdfs:Class .\n\
             :acuity a rdf:Property ;\n\
             \x20   rdfs:domain :Patient ; rdfs:range :AcuityLevel ;\n\
             \x20   horis:assignedBy horis:RulesOnly .",
        );
        let acuity = iri(&mut interner, "acuity");
        let decl = schema.property(acuity).expect("acuity must be a property");
        assert_eq!(decl.assigned_by, AssignedBy::RulesOnly);
    }

    #[test]
    fn class_one_of_enumeration() {
        let (schema, mut interner) = build(
            ":AcuityLevel a rdfs:Class ;\n\
             \x20   horis:oneOf ( :Immediate :Urgent :Standard ) .",
        );
        let acuity_level = iri(&mut interner, "AcuityLevel");
        let immediate = iri(&mut interner, "Immediate");
        let urgent = iri(&mut interner, "Urgent");
        let standard = iri(&mut interner, "Standard");
        let decl = schema
            .class(acuity_level)
            .expect("AcuityLevel must be a class");
        assert_eq!(
            decl.one_of.as_deref(),
            Some([immediate, urgent, standard].as_slice())
        );
    }

    #[test]
    fn rejects_one_of_on_non_class() {
        let err = try_build(
            ":acuity a rdf:Property ;\n\
             \x20   horis:oneOf ( :Immediate ) .",
        )
        .expect_err("horis:oneOf on a property, not a class, must be rejected");
        match err {
            OntologyError::OneOfOnNonClass { subject } => {
                assert!(subject.contains("t#acuity"), "{subject}");
            }
            other => panic!("expected OneOfOnNonClass, got {other:?}"),
        }
    }

    #[test]
    fn rejects_min_greater_than_max() {
        let err = try_build(
            ":heartRate a rdf:Property ;\n\
             \x20   horis:min 300 ;\n\
             \x20   horis:max 0 .",
        )
        .expect_err("min > max must be rejected");
        match err {
            OntologyError::MinExceedsMax { property, min, max } => {
                assert!(property.contains("t#heartRate"), "{property}");
                assert_eq!(min, "300");
                assert_eq!(max, "0");
            }
            other => panic!("expected MinExceedsMax, got {other:?}"),
        }
    }

    #[test]
    fn rejects_undeclared_range_class() {
        let err = try_build(
            ":hasSymptom a rdf:Property ;\n\
             \x20   rdfs:range :NeverDeclared .",
        )
        .expect_err("an undeclared range class must be rejected");
        match err {
            OntologyError::UndeclaredClass {
                property,
                position,
                class,
            } => {
                assert!(property.contains("t#hasSymptom"), "{property}");
                assert_eq!(position, "range");
                assert!(class.contains("t#NeverDeclared"), "{class}");
            }
            other => panic!("expected UndeclaredClass, got {other:?}"),
        }
    }

    #[test]
    fn rejects_undeclared_domain_class() {
        let err = try_build(
            ":hasSymptom a rdf:Property ;\n\
             \x20   rdfs:domain :NeverDeclared .",
        )
        .expect_err("an undeclared domain class must be rejected");
        match err {
            OntologyError::UndeclaredClass { position, .. } => assert_eq!(position, "domain"),
            other => panic!("expected UndeclaredClass, got {other:?}"),
        }
    }

    #[test]
    fn triage_fixture_produces_the_expected_schema() {
        let source = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/triage/triage.ttl"
        ))
        .expect("triage.ttl fixture must exist");
        let mut interner = Interner::new();
        let doc = parse(&source, &mut interner).expect("triage.ttl must parse");
        let schema = Schema::build(&doc, &interner).expect("triage.ttl must build a schema");

        let heart_rate = interner.intern("https://example.org/triage#heartRate");
        let hr = schema
            .property(heart_rate)
            .expect("heartRate must be a property");
        assert_eq!(hr.min, Some(Value::Integer(0)));
        assert_eq!(hr.max, Some(Value::Integer(300)));
        assert_eq!(hr.unit.as_deref(), Some("bpm"));
        assert_eq!(hr.cardinality, Some(Cardinality::ZeroOrOne));

        let acuity = interner.intern("https://example.org/triage#acuity");
        let acuity_decl = schema.property(acuity).expect("acuity must be a property");
        assert_eq!(acuity_decl.assigned_by, AssignedBy::RulesOnly);

        let symptom = interner.intern("https://example.org/triage#Symptom");
        let chest_pain = interner.intern("https://example.org/triage#ChestPain");
        let crushing = interner.intern("https://example.org/triage#CrushingChestPain");
        assert!(schema.is_subclass_of(crushing, chest_pain));
        assert!(schema.is_subclass_of(crushing, symptom));
        assert!(schema.is_subclass_of(chest_pain, symptom));

        let acuity_level = interner.intern("https://example.org/triage#AcuityLevel");
        let immediate = interner.intern("https://example.org/triage#Immediate");
        let urgent = interner.intern("https://example.org/triage#Urgent");
        let standard = interner.intern("https://example.org/triage#Standard");
        let acuity_level_decl = schema
            .class(acuity_level)
            .expect("AcuityLevel must be a class");
        assert_eq!(
            acuity_level_decl.one_of.as_deref(),
            Some([immediate, urgent, standard].as_slice())
        );
    }

    // -- U5: build_all + owl:sameAs anchoring --------------------------------

    /// Parses `source` as a standalone graph sharing `interner` with whatever
    /// else the caller already parsed — the shape `Schema::build_all`'s real
    /// callers (U5's `OntologySet`) use: one `Interner` threaded across every
    /// graph so IRIs mean the same thing everywhere.
    fn parse_graph(interner: &mut Interner, prelude: &str, source: &str) -> Document {
        let full = format!("{prelude}\n{source}\n");
        parse(&full, interner).expect("valid Turtle snippet must parse")
    }

    const DOMAIN_PRELUDE: &str = "@prefix : <https://example.org/t#> .\n\
         @prefix horis: <https://w3id.org/horismos#> .\n\
         @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\
         @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .\n\
         @prefix owl: <http://www.w3.org/2002/07/owl#> .\n\
         @prefix sct: <http://snomed.info/id/> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .";

    #[test]
    fn build_all_over_one_document_matches_build() {
        // Regression guard for the U5 refactor: `Schema::build` now delegates
        // to `Schema::build_all([document], ..)` — this pins that the
        // single-document behavior every U1-U4 test already covers didn't
        // change.
        let mut interner = Interner::new();
        let doc = parse_graph(
            &mut interner,
            DOMAIN_PRELUDE,
            ":Symptom a rdfs:Class .\n:ChestPain rdfs:subClassOf :Symptom .",
        );
        let via_build = Schema::build(&doc, &interner).expect("build must succeed");
        let via_build_all = Schema::build_all([&doc], &interner).expect("build_all must succeed");
        let symptom = interner.intern("https://example.org/t#Symptom");
        let chest_pain = interner.intern("https://example.org/t#ChestPain");
        assert_eq!(
            via_build.is_subclass_of(chest_pain, symptom),
            via_build_all.is_subclass_of(chest_pain, symptom)
        );
    }

    #[test]
    fn same_as_bridges_subsumption_across_an_imported_graph() {
        // The exact example from docs/framework/02-ontology.md and the U5
        // brief: :A sameAs sct:X, :B sameAs sct:Y, and the imported graph
        // says sct:X subClassOf sct:Y — is_subclass_of(:A, :B) must hold
        // even though :A and :B share no direct rdfs:subClassOf triple.
        let mut interner = Interner::new();
        let root = parse_graph(
            &mut interner,
            DOMAIN_PRELUDE,
            ":A a rdfs:Class ; owl:sameAs sct:X .\n\
             :B a rdfs:Class ; owl:sameAs sct:Y .",
        );
        let imported = parse_graph(
            &mut interner,
            "@prefix sct: <http://snomed.info/id/> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .",
            "sct:X a rdfs:Class ; rdfs:subClassOf sct:Y .\nsct:Y a rdfs:Class .",
        );

        let schema =
            Schema::build_all([&root, &imported], &interner).expect("build_all must succeed");

        let a = interner.intern("https://example.org/t#A");
        let b = interner.intern("https://example.org/t#B");
        assert!(
            schema.is_subclass_of(a, b),
            "sameAs anchoring must bridge subsumption through the imported hierarchy"
        );
        assert!(!schema.is_subclass_of(b, a), "must not hold in reverse");
    }

    #[test]
    fn same_as_is_symmetric_and_transitive() {
        let mut interner = Interner::new();
        let root = parse_graph(
            &mut interner,
            DOMAIN_PRELUDE,
            ":A owl:sameAs sct:X .\n\
             sct:X owl:sameAs sct:Y .\n\
             :C a rdfs:Class .\n\
             sct:Y rdfs:subClassOf :C .",
        );
        let schema = Schema::build_all([&root], &interner).expect("build_all must succeed");

        let a = interner.intern("https://example.org/t#A");
        let c = interner.intern("https://example.org/t#C");
        // Transitive: A sameAs X sameAs Y, Y subClassOf C => A subclass of C.
        assert!(schema.is_subclass_of(a, c));
    }

    #[test]
    fn without_same_as_triples_no_spurious_subsumption_is_introduced() {
        let mut interner = Interner::new();
        let root = parse_graph(
            &mut interner,
            DOMAIN_PRELUDE,
            ":A a rdfs:Class .\n:B a rdfs:Class .",
        );
        let imported = parse_graph(
            &mut interner,
            "@prefix sct: <http://snomed.info/id/> .\n\
             @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .",
            "sct:X a rdfs:Class ; rdfs:subClassOf sct:Y .\nsct:Y a rdfs:Class .",
        );
        let schema =
            Schema::build_all([&root, &imported], &interner).expect("build_all must succeed");
        let a = interner.intern("https://example.org/t#A");
        let b = interner.intern("https://example.org/t#B");
        assert!(!schema.is_subclass_of(a, b));
    }
}
