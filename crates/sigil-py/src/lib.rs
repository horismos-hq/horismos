//! # sigil-py
//!
//! The PyO3 extension module (`sigil._sigil`) behind the `sigil` Python
//! package. **No logic lives here** — only marshaling between Python and
//! `sigil-core`. Whole inputs in, whole outcomes out: no cross-language
//! chatter inside a transaction (`docs/framework/14-architecture.md`).
//!
//! Spec: `docs/framework/11-python-api.md` · Owner: PRP-003

use pyo3::prelude::*;

/// Native module registered as `sigil._sigil`; the pure-Python package in
/// `python/sigil/` is the public surface and re-exports from here.
#[pymodule]
fn _sigil(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", sigil_core::VERSION)?;
    m.add("PROFILE_VERSION", sigil_core::PROFILE_VERSION)?;
    Ok(())
}
