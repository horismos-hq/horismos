//! # horismos-py
//!
//! The PyO3 extension module (`horismos._horismos`) behind the `horismos` Python
//! package. **No logic lives here** — only marshaling between Python and
//! `horismos-core`. Whole inputs in, whole outcomes out: no cross-language
//! chatter inside a transaction (`docs/framework/14-architecture.md`).
//!
//! Spec: `docs/framework/11-python-api.md` · Owner: PRP-003

use pyo3::prelude::*;

mod errors;
mod ontology;

/// Native module registered as `horismos._horismos`; the pure-Python package in
/// `python/horismos/` is the public surface and re-exports from here.
#[pymodule]
fn _horismos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", horismos_core::VERSION)?;
    m.add("PROFILE_VERSION", horismos_core::PROFILE_VERSION)?;
    errors::register(m)?;
    ontology::register(m)?;
    Ok(())
}
