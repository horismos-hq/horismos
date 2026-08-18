//! The `World` handle: whole transactions in, whole outcomes out. Every
//! call locks the world, marshals its inputs while holding the GIL, then
//! detaches from the interpreter (D3) for the core call — chaining, proof
//! search, and the snapshot codecs never hold the GIL, and never touch a
//! Python object while detached.
//!
//! Spec: `docs/framework/11-python-api.md` § World · Owner: PRP-003 (U3–U5)

use std::sync::Mutex;

use pyo3::prelude::*;

use horismos_core::world::snapshot::{Diff, Fact};
use horismos_core::{Iri, Ref, Snapshot, Value};

use crate::errors::{naming_err, to_py_err};
use crate::ontology::{PyNodeRef, PyOntology};
use crate::values::{from_value, to_value};

/// A proof tree node, marshaled eagerly: the doc-04 rendering is computed
/// Rust-side in the same call that produced the proof, so `render()` is a
/// stored-string read with no world back-reference.
#[pyclass(frozen, name = "ProofNode", module = "horismos._horismos")]
pub(crate) struct PyProof {
    rendered: String,
    /// The concluding rule in prefixed form, when this node is a rule
    /// conclusion.
    #[pyo3(get)]
    rule: Option<String>,
}

#[pymethods]
impl PyProof {
    fn render(&self) -> &str {
        &self.rendered
    }

    fn __repr__(&self) -> String {
        format!(
            "ProofNode({})",
            self.rendered.lines().next().unwrap_or_default()
        )
    }
}

/// One added/removed fact in a [`PyDiff`].
#[pyclass(frozen, name = "Fact", module = "horismos._horismos")]
pub(crate) struct PyFact {
    /// `"class"` or `"property"`.
    #[pyo3(get)]
    kind: &'static str,
    #[pyo3(get)]
    subject: String,
    /// The property IRI (`None` for a class fact).
    #[pyo3(get)]
    property: Option<String>,
    /// The value as a Python object (for a class fact: the class IRI text).
    #[pyo3(get)]
    value: Py<PyAny>,
}

/// The added/removed facts between a world and an earlier snapshot.
#[pyclass(frozen, name = "Diff", module = "horismos._horismos")]
pub(crate) struct PyDiff {
    #[pyo3(get)]
    added: Py<PyAny>,
    #[pyo3(get)]
    removed: Py<PyAny>,
}

/// An immutable snapshot handle (core `Snapshot`: persistent structure
/// sharing, cheap to clone).
#[pyclass(frozen, name = "Snapshot", module = "horismos._horismos")]
pub(crate) struct PySnapshot {
    pub(crate) inner: Snapshot,
}

#[pymethods]
impl PySnapshot {
    /// The versioned postcard wire form (doc 03 § Snapshots), one crossing.
    fn to_bytes<'py>(&self, py: Python<'py>) -> Bound<'py, pyo3::types::PyBytes> {
        let bytes = py.detach(|| self.inner.to_bytes());
        pyo3::types::PyBytes::new(py, &bytes)
    }

    /// Decodes a wire form under `onto` — every fact re-enters through the
    /// gate in core; a corrupt payload or a foreign ontology is refused in
    /// domain terms.
    #[staticmethod]
    fn from_bytes(py: Python<'_>, onto: &PyOntology, data: &[u8]) -> PyResult<PySnapshot> {
        let inner = py
            .detach(|| Snapshot::from_bytes(&onto.inner, data))
            .map_err(|e| to_py_err(py, e))?;
        Ok(PySnapshot { inner })
    }
}

/// The mutable, gate-guarded instance store (doc 03). The `Mutex` makes the
/// handle `Sync` for Python's free-threaded future without adding any
/// semantics: one transaction at a time, exactly as core requires `&mut`.
#[pyclass(name = "World", module = "horismos._horismos")]
pub(crate) struct PyWorld {
    inner: Mutex<horismos_core::World>,
}

impl PyWorld {
    fn lock(&self, py: Python<'_>) -> PyResult<std::sync::MutexGuard<'_, horismos_core::World>> {
        self.inner
            .lock()
            .map_err(|_| naming_err(py, "the world lock was poisoned by an earlier panic".into()))
    }
}

#[pymethods]
impl PyWorld {
    #[new]
    fn new(onto: &PyOntology) -> PyWorld {
        PyWorld {
            inner: Mutex::new(horismos_core::World::new(&onto.inner)),
        }
    }

    /// One transaction: the whole instance in, one outcome out. `properties`
    /// arrives flat — `(property IRI or CURIE, value)` pairs — the
    /// pure-Python layer already unrolled lists and kwargs naming.
    #[pyo3(signature = (class_term, properties, subject = None))]
    fn assert_(
        &self,
        py: Python<'_>,
        class_term: &str,
        properties: Vec<(String, Bound<'_, PyAny>)>,
        subject: Option<String>,
    ) -> PyResult<PyNodeRef> {
        let mut world = self.lock(py)?;
        let class = expand_intern(py, &mut world, class_term)?;
        let subject_token = subject
            .map(|s| expand_intern(py, &mut world, &s))
            .transpose()?;

        let mut typed: Vec<(Iri, Value)> = Vec::with_capacity(properties.len());
        for (term, ob) in &properties {
            let property = expand_intern(py, &mut world, term)?;
            let range = world
                .ontology()
                .schema()
                .property(property)
                .and_then(|decl| decl.range);
            let value = to_value(py, ob, range.as_ref(), &mut world)?;
            typed.push((property, value));
        }

        let core: &mut horismos_core::World = &mut world;
        let result = py.detach(move || match subject_token {
            Some(iri) => core.assert_named_instance(iri, class, typed),
            None => core.assert_instance(class, typed),
        });
        let subject_ref = result.map_err(|e| to_py_err(py, e))?;
        Ok(node(&world, subject_ref))
    }

    /// Retracts one fact (doc 03's retraction gate runs in core).
    fn retract(
        &self,
        py: Python<'_>,
        subject: PyNodeRef,
        property_term: &str,
        value: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let mut world = self.lock(py)?;
        let property = expand_intern(py, &mut world, property_term)?;
        let range = world
            .ontology()
            .schema()
            .property(property)
            .and_then(|decl| decl.range);
        let value = to_value(py, &value, range.as_ref(), &mut world)?;
        let subject = Ref::new(world.intern(&subject.iri));
        let core: &mut horismos_core::World = &mut world;
        py.detach(move || core.retract(subject, property, &value))
            .map_err(|e| to_py_err(py, e))
    }

    /// Every instance of `class_term`, subsumption-aware, in the store's
    /// deterministic order.
    fn instances(&self, py: Python<'_>, class_term: &str) -> PyResult<Vec<PyNodeRef>> {
        let mut world = self.lock(py)?;
        let class = expand_intern(py, &mut world, class_term)?;
        let refs: Vec<Ref> = world.instances(class).collect();
        Ok(refs.into_iter().map(|r| node(&world, r)).collect())
    }

    /// The first value of `(subject, property)`, or None.
    fn value(
        &self,
        py: Python<'_>,
        subject: PyNodeRef,
        property_term: &str,
    ) -> PyResult<Option<Py<PyAny>>> {
        Ok(self.values(py, subject, property_term)?.into_iter().next())
    }

    /// Every value of `(subject, property)`, in assertion order.
    fn values(
        &self,
        py: Python<'_>,
        subject: PyNodeRef,
        property_term: &str,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let mut world = self.lock(py)?;
        let property = expand_intern(py, &mut world, property_term)?;
        let subject = Ref::new(world.intern(&subject.iri));
        let values: Vec<Value> = world.values(subject, property).cloned().collect();
        values.iter().map(|v| from_value(py, v, &world)).collect()
    }

    /// The tier-2 pattern query (core `World::query_pattern`, D2): the
    /// whole conjunction in, the whole row set out. Undeclared terms are
    /// core-worded errors, never silently empty results.
    fn query(
        &self,
        py: Python<'_>,
        class_term: &str,
        patterns: Vec<(String, Bound<'_, PyAny>)>,
    ) -> PyResult<Vec<PyNodeRef>> {
        let mut world = self.lock(py)?;
        let class = expand_intern(py, &mut world, class_term)?;
        let mut typed: Vec<(Iri, Value)> = Vec::with_capacity(patterns.len());
        for (term, ob) in &patterns {
            let property = expand_intern(py, &mut world, term)?;
            let range = world
                .ontology()
                .schema()
                .property(property)
                .and_then(|decl| decl.range);
            let value = to_value(py, ob, range.as_ref(), &mut world)?;
            typed.push((property, value));
        }
        let rows = world
            .query_pattern(class, &typed)
            .map_err(|e| to_py_err(py, e))?;
        Ok(rows.into_iter().map(|r| node(&world, r)).collect())
    }

    /// The recorded derivation of an existing rule-derived fact, rendered
    /// in the same call (doc 04's "why *is* this established?").
    fn explain(
        &self,
        py: Python<'_>,
        subject: PyNodeRef,
        property_term: &str,
    ) -> PyResult<Option<PyProof>> {
        let mut world = self.lock(py)?;
        let property = expand_intern(py, &mut world, property_term)?;
        let subject = Ref::new(world.intern(&subject.iri));
        Ok(world.explain(subject, property).map(|p| proof(&world, p)))
    }

    /// Goal-directed proving (doc 04's "could it be established?").
    fn prove(
        &self,
        py: Python<'_>,
        subject: PyNodeRef,
        property_term: &str,
        value: Bound<'_, PyAny>,
    ) -> PyResult<Option<PyProof>> {
        let mut world = self.lock(py)?;
        let property = expand_intern(py, &mut world, property_term)?;
        let range = world
            .ontology()
            .schema()
            .property(property)
            .and_then(|decl| decl.range);
        let value = to_value(py, &value, range.as_ref(), &mut world)?;
        let subject = Ref::new(world.intern(&subject.iri));
        let found = py.detach(|| world.prove(subject, property, &value));
        Ok(found.map(|p| proof(&world, &p)))
    }

    /// An O(1) snapshot (persistent structure sharing).
    fn snapshot(&self, py: Python<'_>) -> PyResult<PySnapshot> {
        Ok(PySnapshot {
            inner: self.lock(py)?.snapshot(),
        })
    }

    /// A live world restored from `snapshot` — refused when the snapshot
    /// was taken over a different ontology (core's pairing check).
    #[staticmethod]
    fn from_snapshot(
        py: Python<'_>,
        onto: &PyOntology,
        snapshot: &PySnapshot,
    ) -> PyResult<PyWorld> {
        let world = horismos_core::World::from_snapshot(&onto.inner, snapshot.inner.clone())
            .map_err(|e| to_py_err(py, e))?;
        Ok(PyWorld {
            inner: Mutex::new(world),
        })
    }

    /// Added/removed facts since `snapshot`, in the store's deterministic
    /// order, marshaled whole.
    fn diff(&self, py: Python<'_>, snapshot: &PySnapshot) -> PyResult<PyDiff> {
        let world = self.lock(py)?;
        let diff: Diff = world.diff(&snapshot.inner);
        let marshal = |facts: &[Fact]| -> PyResult<Py<PyAny>> {
            let out: PyResult<Vec<PyFact>> = facts.iter().map(|f| fact(py, &world, f)).collect();
            Ok(out?.into_pyobject(py)?.into_any().unbind())
        };
        Ok(PyDiff {
            added: marshal(diff.added())?,
            removed: marshal(diff.removed())?,
        })
    }
}

/// Expands a CURIE/IRI term against the world's ontology and interns it
/// into the world (an IRI the ontology never mentions is still nameable
/// against a world — that is where instances live).
fn expand_intern(py: Python<'_>, world: &mut horismos_core::World, term: &str) -> PyResult<Iri> {
    let iri = world
        .ontology()
        .expand(term)
        .map_err(|e| crate::errors::ontology_err(py, e))?;
    Ok(world.intern(&iri))
}

fn node(world: &horismos_core::World, subject: Ref) -> PyNodeRef {
    let iri = world.resolve(subject.iri()).to_owned();
    let compact = world.ontology().compact(&iri);
    PyNodeRef { iri, compact }
}

fn proof(world: &horismos_core::World, p: &horismos_core::reason::proof::ProofNode) -> PyProof {
    use horismos_core::reason::proof::ProofKind;
    let rule = match &p.kind {
        ProofKind::Conclusion { rule, .. } => {
            let iri = world.resolve(*rule).to_owned();
            Some(world.ontology().compact(&iri))
        }
        _ => None,
    };
    PyProof {
        rendered: p.render(world),
        rule,
    }
}

fn fact(py: Python<'_>, world: &horismos_core::World, f: &Fact) -> PyResult<PyFact> {
    Ok(match f {
        Fact::Class { subject, class } => PyFact {
            kind: "class",
            subject: world.resolve(*subject).to_owned(),
            property: None,
            value: world
                .resolve(*class)
                .to_owned()
                .into_pyobject(py)?
                .into_any()
                .unbind(),
        },
        Fact::Property {
            subject,
            property,
            value,
        } => PyFact {
            kind: "property",
            subject: world.resolve(*subject).to_owned(),
            property: Some(world.resolve(*property).to_owned()),
            value: from_value(py, value, world)?,
        },
    })
}

/// Registers the classes on the module.
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyWorld>()?;
    m.add_class::<PySnapshot>()?;
    m.add_class::<PyDiff>()?;
    m.add_class::<PyFact>()?;
    m.add_class::<PyProof>()?;
    Ok(())
}
