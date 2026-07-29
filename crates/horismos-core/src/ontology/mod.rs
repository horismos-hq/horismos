//! The Ontology: the single authored artifact of a Horismos deployment.
//!
//! Turtle parse/serialize, the RDFS-plus profile, imports and external-vocabulary
//! anchoring, IRI interning.
//!
//! Spec: `docs/framework/02-ontology.md` · Owner: PRP-001

pub mod facade;
pub mod imports;
pub mod profile;
pub mod schema;
pub mod term;
pub mod turtle;

pub use facade::Ontology;
