//! Value marshaling: Python objects ↔ `horismos_core::Value`, typed by the
//! property's declared range (the same posture as core's
//! `Value::parse_typed` — the ontology decides what a `str` means, never
//! this crate). Whole values, one conversion each way.
//!
//! Spec: `docs/framework/11-python-api.md` · Owner: PRP-003 (U3)

use chrono::{DateTime, FixedOffset};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyString};
use rust_decimal::Decimal;

use horismos_core::ontology::schema::PropertyRange;
use horismos_core::{Datatype, Ref, Value, World};

use crate::errors::ontology_err;
use crate::ontology::PyNodeRef;

/// One Python value → one typed core value. `range` is the property's
/// declared `rdfs:range` (None when the property is undeclared — the gate
/// will name that refusal itself; the value just needs *a* form to carry).
pub(crate) fn to_value(
    py: Python<'_>,
    ob: &Bound<'_, PyAny>,
    range: Option<&PropertyRange>,
    world: &mut World,
) -> PyResult<Value> {
    if let Ok(node) = ob.extract::<PyNodeRef>() {
        let token = world.intern(&node.iri);
        return Ok(Value::Ref(Ref::new(token)));
    }
    // bool before int: Python's bool subclasses int (ledger pin P7).
    if ob.is_instance_of::<PyBool>() {
        return Ok(Value::Boolean(ob.extract::<bool>()?));
    }
    if ob.is_instance_of::<PyInt>() {
        return Ok(Value::Integer(ob.extract::<i64>()?));
    }
    if ob.is_instance_of::<PyFloat>() {
        // Through core's lexical parser so an unrepresentable float (nan,
        // inf) is refused in core's own domain terms.
        let lexical = ob.extract::<f64>()?.to_string();
        return Value::parse_typed(&lexical, Datatype::Decimal).map_err(|e| ontology_err(py, e));
    }
    if ob.is_instance_of::<PyString>() {
        let text = ob.extract::<String>()?;
        return match range {
            // A string where the ontology expects a node names that node.
            Some(PropertyRange::Class(_)) => {
                let iri = world
                    .ontology()
                    .expand(&text)
                    .map_err(|e| ontology_err(py, e))?;
                let token = world.intern(&iri);
                Ok(Value::Ref(Ref::new(token)))
            }
            // A typed literal parses through core; a plain string range (or
            // an undeclared property) carries the text as-is.
            Some(PropertyRange::Datatype(dt)) if *dt != Datatype::String => {
                Value::parse_typed(&text, *dt).map_err(|e| ontology_err(py, e))
            }
            _ => Ok(Value::String(text)),
        };
    }
    if let Ok(dt) = ob.extract::<DateTime<FixedOffset>>() {
        return Ok(Value::DateTime(dt));
    }
    if let Ok(d) = ob.extract::<Decimal>() {
        return Ok(Value::Decimal(d));
    }
    Err(crate::errors::naming_err(
        py,
        format!(
            "cannot marshal a {} as an ontology value (accepted: bool, int, \
             float, str, decimal.Decimal, datetime.datetime with tzinfo, Ref)",
            ob.get_type()
                .name()
                .map(|n| n.to_string())
                .unwrap_or_default()
        ),
    ))
}

/// One typed core value → one Python object.
pub(crate) fn from_value(py: Python<'_>, value: &Value, world: &World) -> PyResult<Py<PyAny>> {
    let ob = match value {
        Value::Integer(i) => i.into_pyobject(py)?.into_any().unbind(),
        Value::Decimal(d) => d.into_pyobject(py)?.into_any().unbind(),
        Value::String(s) => s.into_pyobject(py)?.into_any().unbind(),
        Value::Boolean(b) => PyBool::new(py, *b).to_owned().into_any().unbind(),
        Value::DateTime(dt) => dt.into_pyobject(py)?.into_any().unbind(),
        // xsd:duration crosses as its lexical form until a dedicated
        // handle is needed (dated gap note, 02-tasks-and-gaps.md).
        Value::Duration(d) => d.to_lexical().into_pyobject(py)?.into_any().unbind(),
        Value::Ref(r) => {
            let iri = world.resolve(r.iri()).to_owned();
            let compact = world.ontology().compact(&iri);
            Py::new(py, PyNodeRef { iri, compact })?.into_any()
        }
    };
    Ok(ob)
}
