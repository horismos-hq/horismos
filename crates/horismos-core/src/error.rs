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
}

impl HorismosError {
    /// The failing constraint or condition, stated in domain vocabulary.
    pub fn domain_text(&self) -> String {
        match self {
            Self::Ontology(e) => e.domain_text(),
            Self::ValidationRejected { witness } => witness.text.clone(),
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
