//! The `Ontology` handle and the `Ref` node handle across the boundary.
//! Marshaling only: loading, naming (expand/compact), and lookups all
//! happen in `horismos_core`; this module converts whole inputs in and
//! whole outcomes out (one crossing per call).
//!
//! Spec: `docs/framework/11-python-api.md` · Owner: PRP-003 (U1)

use std::path::PathBuf;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::errors::ontology_err;

/// A node handle: one term's full IRI plus its compacted domain form,
/// resolved eagerly at creation so the handle is self-contained (no
/// back-references across the boundary). Equality and hashing are on the
/// full IRI only.
#[pyclass(
    frozen,
    eq,
    hash,
    from_py_object,
    name = "Ref",
    module = "horismos._horismos"
)]
#[derive(Clone)]
pub(crate) struct PyNodeRef {
    #[pyo3(get)]
    pub(crate) iri: String,
    /// The Turtle prefixed form (`:Immediate`), for repr and rendering.
    pub(crate) compact: String,
}

impl PartialEq for PyNodeRef {
    fn eq(&self, other: &Self) -> bool {
        self.iri == other.iri
    }
}

impl Eq for PyNodeRef {}

impl std::hash::Hash for PyNodeRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.iri.hash(state);
    }
}

#[pymethods]
impl PyNodeRef {
    #[getter]
    fn curie(&self) -> &str {
        &self.compact
    }

    fn __repr__(&self) -> String {
        format!("Ref({})", self.compact)
    }
}

/// One parsed `horis:Rule`, dumped whole at introspection time.
#[pyclass(frozen, name = "Rule", module = "horismos._horismos")]
pub(crate) struct PyRule {
    #[pyo3(get)]
    iri: String,
    #[pyo3(get)]
    curie: String,
    #[pyo3(get)]
    priority: i64,
    #[pyo3(get)]
    explain: Option<String>,
}

/// The loaded, profile-valid ontology (`horismos_core::Ontology` is
/// immutable, `Send + Sync`, and cheap to clone — doc 12 § Core types).
#[pyclass(frozen, name = "Ontology", module = "horismos._horismos")]
pub(crate) struct PyOntology {
    pub(crate) inner: horismos_core::Ontology,
}

#[pymethods]
impl PyOntology {
    /// `Ontology.load(path, imports=[...])` — core `load` + one
    /// `with_import` per entry, all inside one boundary crossing, GIL
    /// released while the parser and schema builder run (D3).
    #[staticmethod]
    #[pyo3(signature = (path, imports = Vec::new(), allow_incomplete = false))]
    fn load(
        py: Python<'_>,
        path: PathBuf,
        imports: Vec<PathBuf>,
        allow_incomplete: bool,
    ) -> PyResult<PyOntology> {
        let loaded = py
            .detach(|| {
                horismos_core::Ontology::load_with_imports_and_options(
                    &path,
                    &imports,
                    horismos_core::LoadOptions { allow_incomplete },
                )
            })
            .map_err(|e| ontology_err(py, e))?;
        Ok(PyOntology { inner: loaded })
    }

    /// One-crossing verification payload: canonical JSON plus the core's
    /// CI process status.
    fn verification_payload(&self, py: Python<'_>) -> PyResult<(String, i32)> {
        let report = py.detach(|| self.inner.verify());
        let exit_code = report.exit_code();
        let json = report
            .to_json()
            .map_err(|error| crate::errors::internal_err(py, error.to_string()))?;
        Ok((json, exit_code))
    }

    /// The unified prefix map, as `{prefix: namespace}`.
    fn namespaces<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (prefix, namespace) in self.inner.prefixes() {
            dict.set_item(prefix, namespace)?;
        }
        Ok(dict)
    }

    /// Expands a CURIE / passes a full IRI through (core
    /// `Ontology::expand`).
    fn expand(&self, py: Python<'_>, term: &str) -> PyResult<String> {
        self.inner.expand(term).map_err(|e| ontology_err(py, e))
    }

    /// The Turtle prefixed form of a full IRI (core `Ontology::compact`).
    fn compact(&self, iri: &str) -> String {
        self.inner.compact(iri)
    }

    /// A node handle for a term the ontology declares (core
    /// `Ontology::node_ref`).
    #[pyo3(name = "ref")]
    fn ref_(&self, py: Python<'_>, term: &str) -> PyResult<PyNodeRef> {
        let token = self.inner.node_ref(term).map_err(|e| ontology_err(py, e))?;
        let iri = self.inner.resolve(token).to_owned();
        let compact = self.inner.compact(&iri);
        Ok(PyNodeRef { iri, compact })
    }

    /// One-crossing bulk dump of every declared property — what the
    /// pure-Python layer builds its keyword/name maps from (ledger P9).
    /// Rows: `(iri, local name, is_object, domain iri or None)`.
    fn property_table(&self) -> Vec<(String, String, bool, Option<String>)> {
        use horismos_core::ontology::schema::PropertyRange;
        self.inner
            .schema()
            .properties()
            .map(|decl| {
                let iri = self.inner.resolve(decl.iri).to_owned();
                let local = self.inner.local_name(&iri);
                let is_object = matches!(decl.range, Some(PropertyRange::Class(_)));
                let domain = decl.domain.map(|d| self.inner.resolve(d).to_owned());
                (iri, local, is_object, domain)
            })
            .collect()
    }

    /// One-crossing bulk dump of every declared class:
    /// `(iri, local name, oneOf member IRIs or None)`.
    fn class_table(&self) -> Vec<(String, String, Option<Vec<String>>)> {
        self.inner
            .schema()
            .classes()
            .map(|decl| {
                let iri = self.inner.resolve(decl.iri).to_owned();
                let local = self.inner.local_name(&iri);
                let one_of = decl.one_of.as_ref().map(|members| {
                    members
                        .iter()
                        .map(|m| self.inner.resolve(*m).to_owned())
                        .collect()
                });
                (iri, local, one_of)
            })
            .collect()
    }

    /// Every parsed rule, in the engine's firing order (priority desc,
    /// rule IRI lexicographic).
    fn rules(&self) -> Vec<PyRule> {
        self.inner
            .rules()
            .iter()
            .map(|rule| {
                let iri = self.inner.resolve(rule.iri).to_owned();
                PyRule {
                    curie: self.inner.compact(&iri),
                    iri,
                    priority: rule.priority,
                    explain: rule.explain.clone(),
                }
            })
            .collect()
    }
}

/// Registers the classes on the module.
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyOntology>()?;
    m.add_class::<PyNodeRef>()?;
    m.add_class::<PyRule>()?;
    Ok(())
}
