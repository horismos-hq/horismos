//! Workflows and the Agent: the workflow step interpreter.
//!
//! Step types (Perception/Reasoning/Action/Narration/Proposal/Wait/Choice/
//! Subflow), transition edges and mandatory fallbacks, preconditions, run
//! outcomes, subflow composition, `horis:Tool` contracts. The Agent is a
//! generic interpreter — all domain behavior comes from the ontology.
//!
//! Spec: `docs/framework/05-workflows.md`, `06-llm-boundaries.md`,
//! `07-tools.md` · Owner: PRP-004

pub mod model;
pub mod tool;
// `agent` (U7) lands once the Bridge trait (U4, in flight on a parallel
// branch — see PRP-004-agent-loop/02-tasks-and-gaps.md) is integrated.

pub use model::{Step, StepBody, Workflow};
pub use tool::Tool;

use thiserror::Error;

/// Errors from the workflow step interpreter. This is a U5 error-surface
/// shell (`HorismosError::WorkflowError`, `docs/framework/12-rust-api.md`
/// § Errors) — the units that build the interpreter itself (U2/U7/U8/U11/
/// U12) add variants here as their behavior lands; this is not an
/// exhaustive list today.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkflowError {
    /// The current step's kind parses (already-allowed profile vocabulary)
    /// but its execution is deferred past v1 (D3, D6, U12:
    /// `ChoiceStep`/`ProposalStep`/`WaitStep`/`SubflowStep`). Names the
    /// concrete step and its kind in domain terms — never a panic, never a
    /// silent no-op.
    #[error("step {step} ({kind}) is not yet executable — deferred past v1")]
    StepDeferred { step: String, kind: String },
}

impl WorkflowError {
    /// The failing condition, stated in domain vocabulary.
    pub fn domain_text(&self) -> String {
        self.to_string()
    }
}
