//! The `HorismosError` hierarchy, mirrored by the Python `HorismosError` classes.
//!
//! Every error carries `domain_text()` — the failing constraint stated in
//! domain terms ("heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"),
//! plus a machine-readable structure used by the perception retry loop.
//!
//! U1 ships only the `Ontology` variant (term/value parse failures). The
//! remaining variants (`VerificationFailed`, `ValidationRejected`,
//! `WorkflowError`, `BridgeError`, `ToolError`) are added by their owning
//! units/PRPs as those pieces land — see `.claude/skills/pyo3-boundary`.
//!
//! Spec: `docs/framework/12-rust-api.md` (§ Errors) · Owner: PRP-001

use thiserror::Error;

/// The top-level Horismos error type; mirrors the Python `HorismosError`
/// exception hierarchy 1:1.
#[derive(Debug, Error)]
pub enum HorismosError {
    /// An error raised while parsing or interpreting ontology-level terms.
    #[error(transparent)]
    Ontology(#[from] OntologyError),
}

impl HorismosError {
    /// The failing constraint or condition, stated in domain vocabulary.
    pub fn domain_text(&self) -> String {
        match self {
            Self::Ontology(e) => e.domain_text(),
        }
    }
}

/// Errors from `horismos_core::ontology` — parsing terms and typed values.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OntologyError {
    /// A lexical form did not match the grammar of its declared datatype.
    #[error("{lexical:?} is not a valid {datatype} lexical form")]
    InvalidLexicalForm { datatype: String, lexical: String },
}

impl OntologyError {
    /// Builds an [`OntologyError::InvalidLexicalForm`].
    pub fn invalid_lexical_form(datatype: impl Into<String>, lexical: impl Into<String>) -> Self {
        Self::InvalidLexicalForm {
            datatype: datatype.into(),
            lexical: lexical.into(),
        }
    }

    /// The failing constraint, stated in domain vocabulary.
    pub fn domain_text(&self) -> String {
        self.to_string()
    }
}
