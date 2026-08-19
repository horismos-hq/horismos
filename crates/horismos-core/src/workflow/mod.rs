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
