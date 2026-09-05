//! The isolated coupling to PRP-005's provisional five-field assumptions shape.
//!
//! Once PRP-005 lands, only conversion code in this module may depend on its
//! concrete `report.assumptions` representation.
//!
//! Spec: `docs/framework/10-verification.md` · Owner: PRP-008 coordinating
//! with PRP-005.

use serde::{Deserialize, Serialize};

use crate::{ClosureMember, SourceVersionVector};

/// The five fixed assumption fields shared with the verification report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssumptionManifest {
    pub input_closure: Vec<ClosureMember>,
    pub external_versions: SourceVersionVector,
    pub mapping_version: String,
    pub supported_subset: Vec<String>,
    pub unverified_effects: Vec<String>,
}
