//! Domain-term host boundary failures.
//!
//! Spec: `docs/framework/03-world.md` and `07-tools.md` · Owner: PRP-008.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Machine-readable counterpart to every host-boundary rejection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostWitness {
    pub constraint: String,
    pub subject: String,
    pub state: Option<String>,
    pub detail: String,
}

/// A fail-closed adapter/SPI error with domain vocabulary and a witness.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{text}")]
pub struct HostError {
    text: String,
    witness: HostWitness,
}

impl HostError {
    pub(crate) fn new(
        text: impl Into<String>,
        constraint: impl Into<String>,
        subject: impl Into<String>,
        state: Option<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            text: text.into(),
            witness: HostWitness {
                constraint: constraint.into(),
                subject: subject.into(),
                state,
                detail: detail.into(),
            },
        }
    }

    /// The failure stated using the owning `horis:` boundary term.
    pub fn domain_text(&self) -> &str {
        &self.text
    }

    /// Structured evidence for policy handling and trace emission.
    pub fn witness(&self) -> Option<&HostWitness> {
        Some(&self.witness)
    }

    /// Admission never exposes its private candidate World on failure.
    pub fn rejected_world(&self) -> Option<&horismos_core::World> {
        None
    }
}
