//! Format-neutral host integration for bounded Horismos projections.
//!
//! Spec: `docs/framework/03-world.md`, `07-tools.md`, `09-traces.md`, and
//! `14-architecture.md` · Owner: PRP-008.

pub mod artifact;
pub mod assumptions;
pub mod error;
pub mod spi;
pub mod trace;

pub use artifact::*;
pub use assumptions::AssumptionManifest;
pub use error::{HostError, HostWitness};
pub use spi::*;
pub use trace::*;
