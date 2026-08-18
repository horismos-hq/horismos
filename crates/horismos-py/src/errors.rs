//! The Python `HorismosError` hierarchy (doc 11 § Errors) and the one
//! mapper from `horismos_core` errors to raised exceptions. Marshaling
//! only: every message is the core error's `domain_text()`, and the
//! machine-readable witness fields ride along as exception attributes —
//! no message is composed here.
//!
//! Spec: `docs/framework/11-python-api.md` § Errors · Owner: PRP-003 (U2)

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use horismos_core::error::HorismosError as CoreError;
use horismos_core::error::OntologyError;
use horismos_core::world::gate::{ConstraintKind, Witness};

create_exception!(
    horismos._horismos,
    HorismosError,
    PyException,
    "Base of every error Horismos raises. `str(exc)` is the domain-term \
     sentence (`domain_text`); machine-readable structure rides as \
     attributes."
);
create_exception!(
    horismos._horismos,
    HorismosOntologyError,
    HorismosError,
    "Ontology parse / profile / import / naming problems."
);
create_exception!(
    horismos._horismos,
    HorismosVerificationError,
    HorismosError,
    "Verification failed (BDD checks — PRP-005)."
);
create_exception!(
    horismos._horismos,
    HorismosValidationError,
    HorismosError,
    "The validation gate rejected a World mutation; the witness rides as \
     `constraint`, `subject`, `class_`, `property`, `value`, `provenance`, \
     `operation`, and `data`."
);
create_exception!(
    horismos._horismos,
    HorismosWorkflowError,
    HorismosError,
    "Workflow definition/execution problems (PRP-004)."
);
create_exception!(
    horismos._horismos,
    HorismosBridgeError,
    HorismosError,
    "Bridge/provider failures (PRP-004)."
);
create_exception!(
    horismos._horismos,
    HorismosToolError,
    HorismosError,
    "Tool binding invocation/effect problems (PRP-004)."
);

/// Registers the hierarchy on the module.
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("HorismosError", py.get_type::<HorismosError>())?;
    m.add(
        "HorismosOntologyError",
        py.get_type::<HorismosOntologyError>(),
    )?;
    m.add(
        "HorismosVerificationError",
        py.get_type::<HorismosVerificationError>(),
    )?;
    m.add(
        "HorismosValidationError",
        py.get_type::<HorismosValidationError>(),
    )?;
    m.add(
        "HorismosWorkflowError",
        py.get_type::<HorismosWorkflowError>(),
    )?;
    m.add("HorismosBridgeError", py.get_type::<HorismosBridgeError>())?;
    m.add("HorismosToolError", py.get_type::<HorismosToolError>())?;
    Ok(())
}

/// One core error → one raised Python exception carrying the whole outcome.
// Consumed by the World bindings from U3 on.
#[allow(dead_code)]
pub(crate) fn to_py_err(py: Python<'_>, err: CoreError) -> PyErr {
    let text = err.domain_text();
    match err {
        CoreError::Ontology(inner) => ontology_err_with(py, text, &inner),
        CoreError::ValidationRejected { witness } => validation_err(py, text, &witness),
        // Snapshot format/mismatch and the chaining tripwire are still
        // validation-shaped refusals of a World mutation.
        CoreError::SnapshotFormat { .. }
        | CoreError::SnapshotMismatch { .. }
        | CoreError::IterationCapExceeded { .. } => {
            plain(py, py.get_type::<HorismosValidationError>(), text)
        }
    }
}

/// An `OntologyError` reached outside a `HorismosError` wrapper.
pub(crate) fn ontology_err(py: Python<'_>, err: OntologyError) -> PyErr {
    let text = err.domain_text();
    ontology_err_with(py, text, &err)
}

fn ontology_err_with(py: Python<'_>, text: String, _err: &OntologyError) -> PyErr {
    plain(py, py.get_type::<HorismosOntologyError>(), text)
}

/// Builds an exception instance with the shared attributes every Horismos
/// error carries (`domain_text`, `suggestion`), raises-ready.
fn plain(py: Python<'_>, class: Bound<'_, pyo3::types::PyType>, text: String) -> PyErr {
    match build(py, &class, &text, |_| Ok(())) {
        Ok(err) => err,
        Err(err) => err,
    }
}

fn validation_err(py: Python<'_>, text: String, witness: &Witness) -> PyErr {
    let class = py.get_type::<HorismosValidationError>();
    match build(py, &class, &text, |inst| {
        let (constraint, data) = constraint_parts(py, &witness.kind)?;
        inst.setattr("constraint", constraint)?;
        inst.setattr("data", data)?;
        inst.setattr("subject", witness.subject.clone())?;
        inst.setattr("class_", witness.class.clone())?;
        inst.setattr("property", witness.property.clone())?;
        inst.setattr("value", witness.value.clone())?;
        inst.setattr("provenance", witness.provenance.as_str())?;
        inst.setattr(
            "operation",
            match witness.operation {
                horismos_core::world::gate::Operation::Assert => "assert",
                horismos_core::world::gate::Operation::Retract => "retract",
            },
        )?;
        Ok(())
    }) {
        Ok(err) => err,
        Err(err) => err,
    }
}

/// Constructs the instance, sets the shared attributes, applies `extra`,
/// and wraps it as a `PyErr`.
fn build(
    py: Python<'_>,
    class: &Bound<'_, pyo3::types::PyType>,
    text: &str,
    extra: impl FnOnce(&Bound<'_, PyAny>) -> PyResult<()>,
) -> Result<PyErr, PyErr> {
    let inst = class.call1((text,))?;
    inst.setattr("domain_text", text)?;
    inst.setattr("suggestion", py.None())?;
    extra(&inst)?;
    Ok(PyErr::from_value(inst))
}

/// The witness kind as a short snake-case tag plus its structured payload.
fn constraint_parts<'py>(
    py: Python<'py>,
    kind: &ConstraintKind,
) -> PyResult<(&'static str, Bound<'py, PyDict>)> {
    let data = PyDict::new(py);
    let tag = match kind {
        ConstraintKind::ClassNotDeclared => "class_not_declared",
        ConstraintKind::ClassConflict { existing } => {
            data.set_item("existing", existing)?;
            "class_conflict"
        }
        ConstraintKind::ReservedSubject { namespace } => {
            data.set_item("namespace", namespace)?;
            "reserved_subject"
        }
        ConstraintKind::PropertyNotDeclared => "property_not_declared",
        ConstraintKind::Domain { expected } => {
            data.set_item("expected", expected)?;
            "domain"
        }
        ConstraintKind::Range { expected } => {
            data.set_item("expected", expected)?;
            "range"
        }
        ConstraintKind::UnknownReference => "unknown_reference",
        ConstraintKind::Min { bound, unit } => {
            data.set_item("bound", bound)?;
            data.set_item("unit", unit.clone())?;
            "min"
        }
        ConstraintKind::Max { bound, unit } => {
            data.set_item("bound", bound)?;
            data.set_item("unit", unit.clone())?;
            "max"
        }
        ConstraintKind::Pattern { pattern } => {
            data.set_item("pattern", pattern)?;
            "pattern"
        }
        ConstraintKind::OneOf { members } => {
            data.set_item("members", members.clone())?;
            "one_of"
        }
        ConstraintKind::CardinalityMax { bound, asserted } => {
            data.set_item("bound", bound)?;
            data.set_item("asserted", asserted)?;
            "cardinality_max"
        }
        ConstraintKind::CardinalityMin { bound } => {
            data.set_item("bound", bound)?;
            "cardinality_min"
        }
        ConstraintKind::Immutable { existing } => {
            data.set_item("existing", existing)?;
            "immutable"
        }
        ConstraintKind::ProvenanceRefused { policy } => {
            data.set_item("policy", policy)?;
            "provenance_refused"
        }
        ConstraintKind::SubjectNotPresent => "subject_not_present",
        ConstraintKind::FactNotPresent => "fact_not_present",
        ConstraintKind::ImmutableRetraction => "immutable_retraction",
        ConstraintKind::DerivedFactRetraction { rule } => {
            data.set_item("rule", rule)?;
            "derived_fact_retraction"
        }
    };
    Ok((tag, data))
}
