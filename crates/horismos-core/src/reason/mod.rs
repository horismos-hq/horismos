//! The Reasoner: deterministic inference over the World.
//!
//! Subsumption (always on), forward chaining to fixed point (Rete-lite),
//! backward chaining (`prove`), proof trees with provenance down to perception
//! confidence (`explain`). No wall-clock, no randomness, no iteration-order
//! dependence — same inputs, same conclusions, always.
//!
//! Spec: `docs/framework/04-reasoning.md` · Owner: PRP-002
