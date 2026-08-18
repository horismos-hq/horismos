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
pub(crate) fn to_py_err(py: Python<'_>, err: CoreError) -> PyErr {
    let text = err.domain_text();
    match err {
        CoreError::Ontology(inner) => ontology_err_with(py, text, &inner),
        CoreError::ValidationRejected { witness } => validation_err(py, text, &witness),
        // Refusals of a World mutation that carry no gate witness: still
        // HorismosValidationError, with a constraint tag and every witness
        // attribute present (None where a witness would have carried it) —
        // the stubs promise the full attribute set (B3).
        CoreError::SnapshotFormat { .. } => witnessless_err(py, "snapshot_format", text),
        CoreError::SnapshotMismatch { .. } => witnessless_err(py, "snapshot_mismatch", text),
        CoreError::IterationCapExceeded { .. } => {
            witnessless_err(py, "iteration_cap_exceeded", text)
        }
    }
}

/// An `OntologyError` reached outside a `HorismosError` wrapper.
pub(crate) fn ontology_err(py: Python<'_>, err: OntologyError) -> PyErr {
    let text = err.domain_text();
    ontology_err_with(py, text, &err)
}

fn ontology_err_with(py: Python<'_>, text: String, err: &OntologyError) -> PyErr {
    let class = py.get_type::<HorismosOntologyError>();
    match build(py, &class, &text, |inst| {
        // The machine-readable structure (B3) for the variants a running
        // program can hit through the boundary's own naming paths; the
        // remaining load-time authoring errors carry an empty dict (their
        // structured surfacing is a recorded gap, 02-tasks-and-gaps.md).
        let data = PyDict::new(py);
        match err {
            OntologyError::UnknownPrefix { term, prefix } => {
                data.set_item("term", term)?;
                data.set_item("prefix", prefix)?;
            }
            OntologyError::NotACurie { term } => {
                data.set_item("term", term)?;
            }
            OntologyError::UndeclaredTerm { term } => {
                data.set_item("term", term)?;
            }
            _ => {}
        }
        inst.setattr("data", data)?;
        Ok(())
    }) {
        Ok(err) => err,
        Err(err) => err,
    }
}

/// A boundary-side naming/typing problem (an argument that cannot be
/// marshaled at all) — the one error family the binding words itself, still
/// carrying the shared `domain_text`/`suggestion`/`data` attributes.
pub(crate) fn naming_err(py: Python<'_>, text: String) -> PyErr {
    plain(py, py.get_type::<HorismosOntologyError>(), text)
}

/// An internal invariant failure (a poisoned lock) — the base class, not a
/// domain family, and never a bare builtin exception.
pub(crate) fn internal_err(py: Python<'_>, text: String) -> PyErr {
    plain(py, py.get_type::<HorismosError>(), text)
}

/// A `HorismosValidationError` for a mutation refusal that has no gate
/// witness: the full B3 attribute set, `None` where a witness would have
/// carried the value.
fn witnessless_err(py: Python<'_>, constraint: &'static str, text: String) -> PyErr {
    let class = py.get_type::<HorismosValidationError>();
    match build(py, &class, &text, |inst| {
        inst.setattr("constraint", constraint)?;
        inst.setattr("data", PyDict::new(py))?;
        inst.setattr("subject", py.None())?;
        inst.setattr("class_", py.None())?;
        inst.setattr("property", py.None())?;
        inst.setattr("value", py.None())?;
        inst.setattr("provenance", py.None())?;
        inst.setattr("operation", py.None())?;
        Ok(())
    }) {
        Ok(err) => err,
        Err(err) => err,
    }
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
        // The machine token (matching the trace schema's serde form), not
        // the domain-prose word `Provenance::as_str` renders into sentences.
        inst.setattr(
            "provenance",
            match witness.provenance {
                horismos_core::Provenance::Perception => "perception",
                horismos_core::Provenance::Rule => "rule",
                horismos_core::Provenance::Tool => "tool",
                horismos_core::Provenance::Api => "api",
            },
        )?;
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
    // Every Horismos error carries `data` (doc 11); `extra` may overwrite
    // this default with a populated dict.
    inst.setattr("data", PyDict::new(py))?;
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
