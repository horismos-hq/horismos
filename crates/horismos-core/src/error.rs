//! The `HorismosError` hierarchy, mirrored by the Python `HorismosError` classes.
//!
//! Every error carries `domain_text()` — the failing constraint stated in
//! domain terms ("heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"),
//! plus a machine-readable structure used by the perception retry loop.
//!
//! U1/U2 ship only term/value/Turtle-syntax variants. U3 adds RDFS-plus
//! profile violations; U4 adds schema-build errors (subclass cycles,
//! min>max, oneOf/domain/range misuse); U5 adds named-graph import errors
//! (missing/cyclic imports, duplicate graph names, prefix conflicts) and
//! attributes a profile violation to the graph it came from. The remaining
//! variants (`VerificationFailed`, `WorkflowError`, `BridgeError`,
//! `ToolError`) are added by their owning units/PRPs as those pieces land —
//! see `.claude/skills/pyo3-boundary`. U6 adds `ValidationRejected`, whose
//! witness type lives with the gate that produces it
//! (`crate::world::gate`).
//!
//! Spec: `docs/framework/12-rust-api.md` (§ Errors) · Owner: PRP-001

use thiserror::Error;

use crate::world::gate::Witness;

/// The top-level Horismos error type; mirrors the Python `HorismosError`
/// exception hierarchy 1:1.
#[derive(Debug, Error)]
pub enum HorismosError {
    /// An error raised while parsing or interpreting ontology-level terms.
    #[error(transparent)]
    Ontology(#[from] OntologyError),

    /// The validation gate rejected a World mutation
    /// (`docs/framework/03-world.md`). `witness` carries both the rendered
    /// domain-term sentence and the machine-readable structure the
    /// perception retry loop (doc 06) consumes. Boxed to keep
    /// `HorismosError` small.
    #[error("{}", .witness.text)]
    ValidationRejected { witness: Box<Witness> },

    /// A [`crate::world::Snapshot`]'s binary form (U8,
    /// `docs/framework/03-world.md` § Snapshots) failed to decode: an
    /// unrecognized magic header, an unsupported format version, or a
    /// corrupt postcard payload. Distinct from `ValidationRejected` — this is
    /// a *format* failure, before any fact was well-typed enough to reach the
    /// gate at all; a value that decodes cleanly but fails a gate check
    /// (`horis:min`/`max`/…) surfaces as `ValidationRejected` instead, exactly
    /// as it would from a live assertion.
    #[error("{reason}")]
    SnapshotFormat { reason: String },

    /// A well-formed [`crate::world::Snapshot`] was offered to
    /// [`crate::World::from_snapshot`] with an [`crate::Ontology`] other
    /// than the one its world was validated under. Installing it would put
    /// never-gated state behind this ontology's gate and silently re-map
    /// every interned token — so it is refused outright.
    #[error("{reason}")]
    SnapshotMismatch { reason: String },

    /// Forward chaining hit its iteration cap with candidates still fireable
    /// (PRP-002 U4, `docs/framework/04-reasoning.md`): the tripwire for a
    /// rule set that may not terminate — termination is *verified* in
    /// PRP-005 (doc 10). The whole transaction rolled back; the world is
    /// exactly as before the triggering assertion.
    #[error(
        "forward chaining exceeded {cap} iterations without reaching a fixed point (last fired rule {last_rule}); the transaction was rolled back"
    )]
    IterationCapExceeded { cap: usize, last_rule: String },
}

impl HorismosError {
    /// The failing constraint or condition, stated in domain vocabulary.
    pub fn domain_text(&self) -> String {
        match self {
            Self::Ontology(e) => e.domain_text(),
            Self::ValidationRejected { witness } => witness.text.clone(),
            Self::SnapshotFormat { reason } => reason.clone(),
            Self::SnapshotMismatch { reason } => reason.clone(),
            Self::IterationCapExceeded { .. } => self.to_string(),
        }
    }

    /// The rejection witness, if this error is a gate rejection.
    pub fn witness(&self) -> Option<&Witness> {
        match self {
            Self::ValidationRejected { witness } => Some(witness),
            _ => None,
        }
    }
}

/// Errors from `horismos_core::ontology` — parsing terms and typed values.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OntologyError {
    /// A lexical form did not match the grammar of its declared datatype.
    #[error("{lexical:?} is not a valid {datatype} lexical form")]
    InvalidLexicalForm { datatype: String, lexical: String },

    /// Turtle source did not conform to Turtle grammar (U2 parsing).
    #[error("Turtle syntax error at line {line}, column {column}: {message}")]
    TurtleSyntax {
        line: u64,
        column: u64,
        message: String,
    },

    // -- U3: RDFS-plus profile validation --------------------------------
    /// An `rdf:`/`rdfs:`/`owl:`/`xsd:` term outside the RDFS-plus profile
    /// (e.g. `owl:Restriction`, `xsd:anyURI`, `rdf:Bag`).
    #[error("{term} is outside the RDFS-plus profile")]
    OutOfProfileTerm { term: String },

    /// A `horis:` term that is not part of the documented vocabulary
    /// (`docs/framework/02-ontology.md`).
    #[error("no such term in the horis: vocabulary: {term}")]
    UnknownHorisTerm { term: String },

    /// A prefixed name (`nope:Immediate`) whose prefix has no binding in the
    /// loaded graph set's unified prefix map — from
    /// [`crate::Ontology::expand`] (PRP-003).
    #[error("cannot expand {term}: prefix \"{prefix}\" is not declared by the ontology")]
    UnknownPrefix { term: String, prefix: String },

    /// A term that is neither a CURIE (`prefix:localName`) nor a full IRI —
    /// from [`crate::Ontology::expand`] (PRP-003).
    #[error("cannot expand {term}: expected a CURIE (prefix:localName) or a full IRI")]
    NotACurie { term: String },

    /// A well-formed term that none of the loaded graphs mention — from
    /// [`crate::Ontology::node_ref`] (PRP-003). `term` is already in
    /// compacted domain form.
    #[error("{term} names no term declared by the ontology")]
    UndeclaredTerm { term: String },

    /// A literal whose datatype is not one of the six XSD types the profile
    /// supports, or a language-tagged string (the profile has no vocabulary
    /// for `rdf:langString`).
    #[error(
        "{datatype} is not part of the RDFS-plus XSD datatype subset (xsd:decimal, xsd:integer, xsd:string, xsd:boolean, xsd:dateTime, xsd:duration)"
    )]
    DatatypeNotInProfile { datatype: String },

    /// A `horis:cardinality` object other than `"1"`, `"0..1"`, `"1..*"`, or
    /// `"0..*"`.
    #[error(
        "{value:?} is not a valid horis:cardinality value (expected \"1\", \"0..1\", \"1..*\", or \"0..*\")"
    )]
    InvalidCardinality { value: String },

    /// A `horis:assignedBy` object other than `horis:RulesOnly`,
    /// `horis:PerceptionAllowed`, or `horis:ToolsOnly`.
    #[error(
        "{value} is not a valid horis:assignedBy object (expected horis:RulesOnly, horis:PerceptionAllowed, or horis:ToolsOnly)"
    )]
    InvalidAssignedBy { value: String },

    /// One or more profile violations found in a single validation pass
    /// (`ontology::profile::validate` reports all of them, not just the
    /// first). `domain_text()` joins each violation's own `domain_text()`.
    #[error("ontology has {} RDFS-plus profile violation(s)", .0.len())]
    ProfileViolations(Vec<OntologyError>),

    // -- U4: schema build -------------------------------------------------
    /// `rdfs:subClassOf` edges form a cycle.
    #[error("subClassOf cycle: {cycle}")]
    SubclassCycle { cycle: String },

    /// A property's `horis:min` is greater than its `horis:max`.
    #[error("{property} horis:min {min} exceeds horis:max {max}")]
    MinExceedsMax {
        property: String,
        min: String,
        max: String,
    },

    /// `horis:oneOf` was declared on a subject that is not a declared
    /// `rdfs:Class`.
    #[error("{subject} horis:oneOf is declared but {subject} is not a declared rdfs:Class")]
    OneOfOnNonClass { subject: String },

    /// A property's `rdfs:domain`/`rdfs:range` names a class that is not
    /// declared anywhere in the schema.
    #[error("{property} rdfs:{position} names {class}, which is not a declared rdfs:Class")]
    UndeclaredClass {
        property: String,
        position: String,
        class: String,
    },

    // -- U5: imports across named graphs -----------------------------------
    /// A U3 profile violation attributed to a specific named graph in a
    /// multi-graph load (U5) — the root document or one of its imports.
    /// `ontology::imports::OntologySet::validate` wraps every violation it
    /// finds this way before collecting them into `ProfileViolations`, so
    /// `domain_text()` on the aggregate names the graph each violation came
    /// from.
    #[error("{graph}: {inner}")]
    ProfileViolationInGraph {
        graph: String,
        inner: Box<OntologyError>,
    },

    /// A declared `horis:imports` target — or an explicit `with_import` —
    /// could not be resolved: the file doesn't exist (fs convenience), or
    /// the pure API's import-source map/closure had nothing for this IRI.
    #[error("horis:imports {import}: {reason}")]
    ImportNotFound { import: String, reason: String },

    /// The import graph reachable from the root contains a cycle (A imports
    /// B, B imports A, …).
    #[error("import cycle: {cycle}")]
    ImportCycle { cycle: String },

    /// An explicit `with_import` named a graph that is already loaded —
    /// either the root itself or an already-resolved import.
    #[error("duplicate import graph: {name}")]
    DuplicateGraphName { name: String },

    /// Two imports at equal standing (neither is the root) bound the same
    /// Turtle prefix to different IRIs.
    #[error("prefix {prefix}: conflicting bindings ({a} vs {b})")]
    PrefixConflict {
        prefix: String,
        a: String,
        b: String,
    },

    // -- PRP-002 U1: rule parse/validation at load -------------------------
    /// A `horis:Rule` is structurally incomplete: no `horis:when`, no
    /// `horis:then`, no `horis:subject` on its conditions, or a malformed
    /// body where a condition node was expected.
    #[error("rule {rule} is missing {part}")]
    RuleMissingPart { rule: String, part: String },

    /// A rule condition or conclusion names a property the schema does not
    /// declare.
    #[error("rule {rule} names {property}, which is not a declared property in the ontology")]
    RuleUndeclaredProperty { rule: String, property: String },

    /// A condition's `horis:subject` names a class the schema does not
    /// declare (checked `owl:sameAs`-canonically, so an anchored external
    /// IRI is as good as its local twin).
    #[error(
        "rule {rule} horis:subject names {class}, which is not a declared class in the ontology"
    )]
    RuleUndeclaredSubject { rule: String, class: String },

    /// Conditions of one rule name two different subject classes. A v1 rule
    /// binds exactly one subject instance (doc 04's `{?p → :p7}` bindings);
    /// cross-subject joins are not in the profile.
    #[error(
        "rule {rule} conditions name both {a} and {b} as horis:subject; a rule binds one subject class"
    )]
    RuleMixedSubjects { rule: String, a: String, b: String },

    /// A condition states no recognized operator (one of the 11,
    /// `docs/framework/02-ontology.md`).
    #[error(
        "rule {rule} condition on {property} states no operator (expected one of the 11 horis: condition operators)"
    )]
    RuleMissingOperator { rule: String, property: String },

    /// A condition states more than one operator; each condition tests one.
    #[error(
        "rule {rule} condition on {property} states both horis:{a} and horis:{b}; a condition tests one operator"
    )]
    RuleMultipleOperators {
        rule: String,
        property: String,
        a: String,
        b: String,
    },

    /// `horis:priority`'s object is not an `xsd:integer`.
    #[error("rule {rule} horis:priority {value} is not an xsd:integer")]
    RuleInvalidPriority { rule: String, value: String },

    /// A `horis:Rule` was declared on an anonymous (blank) subject. Blank
    /// ids are minted per parse, so an anonymous rule would have a
    /// different identity on every load — nondeterministic tie-breaks,
    /// error text, and traces. Rules must be named.
    #[error(
        "a horis:Rule is declared on an anonymous subject; rules must be named (a blank node's identity differs on every load)"
    )]
    RuleAnonymous,

    /// A condition node states more than one combinator
    /// (`horis:all`/`any`/`none`); each node is one combinator or one leaf.
    #[error(
        "rule {rule} a condition node states both horis:{a} and horis:{b}; a node is one combinator or one leaf condition"
    )]
    RuleMultipleCombinators { rule: String, a: String, b: String },

    /// An operator was given an operand of the wrong shape (a node where a
    /// literal is required, a scalar where a list is required, …).
    #[error("rule {rule} {operator} {value} is not usable here (expected {expected})")]
    RuleInvalidOperand {
        rule: String,
        operator: String,
        value: String,
        expected: String,
    },

    /// `horis:between` did not receive a list of exactly two values.
    #[error("rule {rule} horis:between expects a list of exactly two values, found {found}")]
    RuleBetweenArity { rule: String, found: usize },

    /// D6 (strict units), rule-side: a condition's stated `horis:unit`
    /// contradicts the property's declared unit.
    #[error(
        "rule {rule} {property} condition states horis:unit \"{stated}\" but {property} is declared in \"{declared}\""
    )]
    RuleUnitMismatch {
        rule: String,
        property: String,
        stated: String,
        declared: String,
    },

    /// D6 (strict units), rule-side: a condition states a `horis:unit` on a
    /// property that declares none.
    #[error(
        "rule {rule} {property} condition states horis:unit \"{stated}\" but {property} declares no unit"
    )]
    RuleUnitWithoutDeclaration {
        rule: String,
        property: String,
        stated: String,
    },

    // -- U6: the Ontology facade ------------------------------------------
    /// A property's `horis:pattern` is not a valid regular expression.
    /// Patterns are compiled once when the [`crate::Ontology`] is assembled,
    /// so this is a *load* error — the validation gate never has to fail on
    /// a malformed pattern.
    #[error("{property} horis:pattern {pattern:?} is not a valid regular expression: {reason}")]
    InvalidPattern {
        property: String,
        pattern: String,
        reason: String,
    },

    // -- PRP-004 U2/U3: workflow/step/tool parse+validation at load --------
    /// A `horis:Workflow` or `horis:*Step` is missing a required part
    /// (`horis:startsAt`, `horis:fallback` on a `PerceptionStep`,
    /// `horis:onError` on an `ActionStep`, …) — the load-time posture doc 05
    /// documents ("Required" column) made mechanical, the same shape as
    /// [`OntologyError::RuleMissingPart`].
    #[error("{subject} is missing {part}")]
    WorkflowMissingPart { subject: String, part: String },

    /// `horis:startsAt`/`next`/`fallback`/`onError`/`onUnreachable`/
    /// `onUnsatisfied`/`otherwise`/`workflow` (subflow) named a step or
    /// workflow IRI nothing in the loaded graphs declares.
    #[error("{subject} {edge} names {target}, which is not a declared step in this workflow")]
    WorkflowUndeclaredStep {
        subject: String,
        edge: String,
        target: String,
    },

    /// A `horis:Workflow` or a `horis:*Step` was declared on an anonymous
    /// (blank) subject — refused for the same reason as
    /// [`OntologyError::RuleAnonymous`]: blank ids are minted per parse, so
    /// identity (and therefore edges naming it) would differ on every load.
    #[error(
        "a {kind} is declared on an anonymous subject; workflows and steps must be named (a blank node's identity differs on every load)"
    )]
    WorkflowAnonymous { kind: String },

    /// One subject was typed as more than one step kind
    /// (`a horis:PerceptionStep, horis:ActionStep`) — a step is exactly one
    /// kind.
    #[error("{step} is declared as both {a} and {b}; a step is exactly one kind")]
    StepAmbiguousKind { step: String, a: String, b: String },

    /// A `horis:goal`/`horis:requires`/`horis:when`(-shaped) condition on a
    /// step names a property the schema does not declare.
    #[error("{subject} names {property}, which is not a declared property in the ontology")]
    StepUndeclaredProperty { subject: String, property: String },

    /// A `horis:PerceptionStep`'s `horis:extracts` names a class the schema
    /// does not declare.
    #[error("{step} horis:extracts names {class}, which is not a declared class in the ontology")]
    StepUndeclaredClass { step: String, class: String },

    /// A step condition (goal/requires/branch `horis:when`) states no
    /// recognized operator — the step-scoped sibling of
    /// [`OntologyError::RuleMissingOperator`].
    #[error(
        "{subject} condition on {property} states no operator (expected one of the 11 horis: condition operators)"
    )]
    StepMissingOperator { subject: String, property: String },

    /// A step condition states more than one operator.
    #[error(
        "{subject} condition on {property} states both horis:{a} and horis:{b}; a condition tests one operator"
    )]
    StepMultipleOperators {
        subject: String,
        property: String,
        a: String,
        b: String,
    },

    /// A step condition node states more than one combinator.
    #[error(
        "{subject} a condition node states both horis:{a} and horis:{b}; a node is one combinator or one leaf condition"
    )]
    StepMultipleCombinators {
        subject: String,
        a: String,
        b: String,
    },

    /// A step condition operator was given an operand of the wrong shape.
    #[error("{subject} {operator} {value} is not usable here (expected {expected})")]
    StepInvalidOperand {
        subject: String,
        operator: String,
        value: String,
        expected: String,
    },

    /// `horis:between` on a step condition did not receive a list of
    /// exactly two values.
    #[error("{subject} horis:between expects a list of exactly two values, found {found}")]
    StepBetweenArity { subject: String, found: usize },

    /// `horis:confidence` on a `PerceptionStep`/`ProposalStep` is not an
    /// `xsd:decimal` in `[0, 1]`.
    #[error("{step} horis:confidence {value} is not a decimal in [0, 1]")]
    StepInvalidConfidence { step: String, value: String },

    /// A `horis:SubflowStep`'s target workflow itself declares a
    /// `horis:SubflowStep` — D6 (`PRP-004-agent-loop/02-tasks-and-gaps.md`):
    /// one level of nesting only, checked at load.
    #[error(
        "{step} horis:workflow {target} itself declares a horis:SubflowStep; subflow nesting is capped at one level"
    )]
    SubflowNestingTooDeep { step: String, target: String },

    /// A `horis:Tool`'s `horis:takes`/`horis:effect`/`horis:binding` is
    /// structurally incomplete or malformed — the same load-time posture as
    /// rule/step parsing.
    #[error("{tool} is missing {part}")]
    ToolMissingPart { tool: String, part: String },

    /// A `horis:Tool` argument or effect names a property/class the schema
    /// does not declare.
    #[error("{tool} names {name}, which is not declared in the ontology")]
    ToolUndeclaredReference { tool: String, name: String },
}

impl OntologyError {
    /// Builds an [`OntologyError::InvalidLexicalForm`].
    pub fn invalid_lexical_form(datatype: impl Into<String>, lexical: impl Into<String>) -> Self {
        Self::InvalidLexicalForm {
            datatype: datatype.into(),
            lexical: lexical.into(),
        }
    }

    /// Builds an [`OntologyError::TurtleSyntax`]. `line`/`column` are
    /// 1-based, matching how editors and `oxttl`'s own error display report
    /// position.
    pub fn turtle_syntax(line: u64, column: u64, message: impl Into<String>) -> Self {
        Self::TurtleSyntax {
            line,
            column,
            message: message.into(),
        }
    }

    /// The failing constraint, stated in domain vocabulary. For
    /// [`OntologyError::ProfileViolations`] this joins every violation's own
    /// `domain_text()` rather than the aggregate `Display` summary, so a
    /// caller sees the full list without unwrapping the variant itself.
    pub fn domain_text(&self) -> String {
        match self {
            Self::ProfileViolations(violations) => violations
                .iter()
                .map(OntologyError::domain_text)
                .collect::<Vec<_>>()
                .join("; "),
            Self::ProfileViolationInGraph { graph, inner } => {
                format!("{graph}: {}", inner.domain_text())
            }
            other => other.to_string(),
        }
    }
}
