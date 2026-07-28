//! The World: the typed instance store — the agent's memory of record.
//!
//! Instance graph, the validation gate (every mutation passes it; no back door),
//! snapshots with persistent structure sharing, transactions, pattern queries,
//! the SPARQL subset (behind the `sparql` feature), subscriptions.
//!
//! Spec: `docs/framework/03-world.md` · Owner: PRP-001
