//! Workflows and the Agent: the workflow step interpreter.
//!
//! Step types (Perception/Reasoning/Action/Narration/Proposal/Wait/Choice/
//! Subflow), transition edges and mandatory fallbacks, preconditions, run
//! outcomes, subflow composition, `horis:Tool` contracts. The Agent is a
//! generic interpreter — all domain behavior comes from the ontology.
//!
//! Spec: `docs/framework/05-workflows.md`, `06-llm-boundaries.md`,
//! `07-tools.md` · Owner: PRP-004

pub mod agent;
pub mod deferred;
pub mod model;
pub mod tool;

pub use agent::{Agent, AgentBuilder, RunInput, RunOutcome};
pub use model::{Step, StepBody, Workflow};
pub use tool::Tool;

use thiserror::Error;

/// Errors from the workflow step interpreter (U7). This is a U5
/// error-surface shell (`HorismosError::WorkflowError`,
/// `docs/framework/12-rust-api.md` § Errors), filled in as the interpreter
/// itself (U7/U8/U11/U12) landed.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum WorkflowError {
    /// The current step's kind parses (already-allowed profile vocabulary)
    /// but its execution is deferred past v1 (D3, D6, U12:
    /// `ChoiceStep`/`ProposalStep`/`WaitStep`/`SubflowStep`). Names the
    /// concrete step and its kind in domain terms — never a panic, never a
    /// silent no-op.
    #[error("step {step} ({kind}) is not yet executable — deferred past v1")]
    StepDeferred { step: String, kind: String },

    /// `AgentBuilder::workflow(name)` named something that isn't a declared
    /// `horis:Workflow`.
    #[error("\"{name}\" does not name a declared horis:Workflow: {reason}")]
    UnknownWorkflow { name: String, reason: String },

    /// `AgentBuilder::build()` was called with no `.workflow(...)` selected.
    #[error("AgentBuilder::build() called with no workflow selected")]
    NoWorkflowSelected,

    /// An `ActionStep`'s tool names a `rust:` binding with no
    /// `AgentBuilder::tool(name, ...)` registered for it — checked at
    /// `build()`, never mid-run.
    #[error("no Tool implementation registered for \"{tool}\"")]
    UnboundTool { tool: String },

    /// A step's `horis:model` names a prefix with no `AgentBuilder::bridge`
    /// registered for it (and more than one bridge is registered, so there
    /// is no unambiguous default) — checked at `build()`, never mid-run.
    #[error("no Bridge registered for the \"{prefix}\" model prefix")]
    UnboundBridge { prefix: String },

    /// A `ReasoningStep`/`ActionStep`/`NarrationStep` was reached before any
    /// `PerceptionStep` established a case subject (see `workflow::agent`'s
    /// module doc) — a malformed workflow verification (PRP-005) should
    /// catch statically; this is the runtime backstop.
    #[error(
        "no case subject: this workflow reaches a condition-bearing step before any PerceptionStep resolves one"
    )]
    NoCaseSubject,

    /// A `Bridge::perceive` call returned a schema-conforming-shaped result
    /// this runtime still could not interpret (a missing `"class"` field, an
    /// undeclared class name) — treated as a load-bearing bug in the
    /// Bridge/schema pairing, not a normal low-confidence case.
    #[error("perception result malformed: {reason}")]
    PerceptionMalformed { reason: String },

    /// A perceived instance was well-formed but the validation gate refused
    /// it after exhausting `horis:retries` — the step's `horis:fallback`
    /// edge is what a caller sees; this variant exists for the case a
    /// caller inspects the failure directly.
    #[error("perceived instance rejected: {reason}")]
    PerceptionRejected { reason: String },

    /// An `ActionStep`'s `horis:requires` never held and it declares no
    /// `horis:onUnsatisfied` edge (doc 05: "fail the run if absent —
    /// verification proves this can't happen for well-formed workflows").
    #[error("step {step}'s horis:requires is unsatisfiable and it declares no horis:onUnsatisfied")]
    RequiresUnsatisfiable { step: String },

    /// An explicitly `allow_incomplete` ontology reached a fallible step
    /// whose required failure edge verification had already reported.
    #[error("step {step} can fail but declares no {edge} edge")]
    MissingFailureEdge { step: String, edge: String },

    /// A `NarrationStep` declaring `horis:required true` failed (D7).
    #[error("required narration step {step} failed: {reason}")]
    NarrationFailed { step: String, reason: String },
}

impl WorkflowError {
    /// The failing condition, stated in domain vocabulary.
    pub fn domain_text(&self) -> String {
        self.to_string()
    }
}
