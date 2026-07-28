//! Workflows and the Agent: the workflow step interpreter.
//!
//! Step types (Perception/Reasoning/Action/Narration/Proposal/Wait/Choice),
//! transition edges and mandatory fallbacks, preconditions, run outcomes,
//! subflow composition. The Agent is a generic interpreter — all domain
//! behavior comes from the ontology.
//!
//! Spec: `docs/framework/05-workflows.md`, `06-llm-boundaries.md`,
//! `07-tools.md` · Owner: PRP-004
