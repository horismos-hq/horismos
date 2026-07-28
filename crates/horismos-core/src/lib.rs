//! # horismos-core
//!
//! The symbolic engine of the Horismos framework: ontology, world, reasoner,
//! workflows, traces, verification. **Zero LLM dependencies** — this crate
//! must always compile and pass its full test suite with no network, no keys,
//! and `--no-default-features --features verify` (the anti-wrapper gate,
//! `docs/framework/14-architecture.md`).
//!
//! The framework definition lives in `docs/framework/`; each module below
//! implements one part of it and is filled in by its owning PRP.

pub mod bridge;
pub mod error;
pub mod ontology;
pub mod reason;
pub mod trace;
#[cfg(feature = "verify")]
pub mod verify;
pub mod workflow;
pub mod world;

#[cfg(feature = "wasm")]
pub mod wasm;

/// The RDFS-plus profile version this engine targets
/// (`horis:profileVersion`, docs/framework/02-ontology.md).
pub const PROFILE_VERSION: &str = "0.1";

/// Engine version, mirrored into `horismos.__version__` on the Python side.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
