//! The agent loop from Python: bridge handles, and one `Agent::run` per call —
//! the whole run in, the whole outcome out.
//!
//! Marshaling only. Schema derivation (rules-only properties dropped before a
//! bridge sees the schema), the step's `horis:confidence` gate, the mandatory
//! `horis:fallback` edge, and the perception-provenance assertion through the
//! validation gate all happen inside `horismos_core::workflow::Agent::run`.
//! There is deliberately no binding that calls a bridge's `perceive` directly:
//! its result could reach the World only by a path that skips that gate.
//!
//! Spec: `docs/framework/11-python-api.md` § Agent, `08-bridge.md` · Owner:
//! PRP-004 (U-Python)

use std::sync::{Arc, Mutex, PoisonError};

use chrono::{DateTime, FixedOffset};
use pyo3::prelude::*;

use horismos_bridges::mock::{Invocation, MockBridge};
use horismos_core::bridge::{Bridge, NarrateResult, PerceiveResult};
use horismos_core::error::HorismosError as CoreError;
use horismos_core::workflow::{Agent, RunInput, RunOutcome, WorkflowError};

use crate::errors::{bridge_err, internal_err, to_py_err};
use crate::ontology::{PyNodeRef, PyOntology};
use crate::values::from_value;
use crate::world::{PyDiff, PyProof, PyWorld, marshal_diff, node, proof};

/// `MockBridge` (doc 08 § The mock backend): scripted, deterministic, every
/// call recorded.
#[pyclass(frozen, name = "MockBridge", module = "horismos._horismos")]
pub(crate) struct PyMockBridge {
    inner: Arc<MockBridge>,
}

#[pymethods]
impl PyMockBridge {
    /// The scripts arrive whole, as `(pattern, JSON)` pairs whose JSON is a
    /// complete `PerceiveResult` / `NarrateResult` the pure-Python layer
    /// serialised.
    #[new]
    fn new(
        py: Python<'_>,
        id: String,
        perceive: Vec<(String, String)>,
        narrate: Vec<(String, String)>,
    ) -> PyResult<Self> {
        let mut bridge = MockBridge::with_id(id);
        for (pattern, json) in perceive {
            let result: PerceiveResult = serde_json::from_str(&json).map_err(|e| {
                bridge_err(
                    py,
                    format!("MockBridge script for {pattern:?} is not a PerceiveResult: {e}"),
                )
            })?;
            bridge = bridge.script_perceive(pattern, result);
        }
        for (pattern, json) in narrate {
            let result: NarrateResult = serde_json::from_str(&json).map_err(|e| {
                bridge_err(
                    py,
                    format!("MockBridge narration for {pattern:?} is not a NarrateResult: {e}"),
                )
            })?;
            bridge = bridge.script_narrate(pattern, result);
        }
        Ok(PyMockBridge {
            inner: Arc::new(bridge),
        })
    }

    /// `Bridge::id()` — the trace's `model` field.
    #[getter]
    fn id(&self) -> &str {
        self.inner.id()
    }

    /// Every call served so far, in order: `(kind, request JSON)`.
    fn invocations(&self, py: Python<'_>) -> PyResult<Vec<(&'static str, String)>> {
        self.inner
            .invocations()
            .into_iter()
            .map(|call| {
                let (kind, json) = match &call {
                    Invocation::Perceive(req) => ("perceive", serde_json::to_string(req)),
                    Invocation::Narrate(req) => ("narrate", serde_json::to_string(req)),
                    Invocation::Rank(req) => ("rank", serde_json::to_string(req)),
                };
                json.map(|json| (kind, json))
                    .map_err(|e| internal_err(py, e.to_string()))
            })
            .collect()
    }
}

/// `JevBridge` (`bridges-jev`): TypeSafe's text → decisions model.
#[cfg(feature = "bridges-jev")]
#[pyclass(frozen, name = "JevBridge", module = "horismos._horismos")]
pub(crate) struct PyJevBridge {
    inner: Arc<horismos_bridges::jev::JevBridge>,
}

/// One `PropertyHint`, flat: `(property, instructions, when_true, when_false,
/// ordinal)`.
#[cfg(feature = "bridges-jev")]
type HintRow = (String, Option<String>, Option<String>, Option<String>, bool);

#[cfg(feature = "bridges-jev")]
fn with_hints(
    mut bridge: horismos_bridges::jev::JevBridge,
    hints: Vec<HintRow>,
) -> horismos_bridges::jev::JevBridge {
    for (property, instructions, when_true, when_false, ordinal) in hints {
        bridge = bridge.with_hint(
            property,
            horismos_bridges::jev::PropertyHint {
                instructions,
                when_true,
                when_false,
                ordinal,
            },
        );
    }
    bridge
}

#[cfg(feature = "bridges-jev")]
#[pymethods]
impl PyJevBridge {
    /// `transport` is `"openrouter"` or `"typesafe"`; `model` defaults to the
    /// transport's own alias. The key, when given, is held by the Rust client
    /// only — never echoed into a repr, a trace, or an error.
    #[new]
    fn new(
        py: Python<'_>,
        transport: &str,
        model: Option<String>,
        api_key: Option<String>,
        hints: Vec<HintRow>,
    ) -> PyResult<Self> {
        use horismos_bridges::jev::{JevBridge, JevTransport};
        let transport = match transport {
            "openrouter" => JevTransport::OpenRouter,
            "typesafe" => JevTransport::TypeSafe,
            other => {
                return Err(bridge_err(
                    py,
                    format!(
                        "unknown Jev transport {other:?}: expected \"openrouter\" or \"typesafe\""
                    ),
                ));
            }
        };
        let mut bridge = match model {
            Some(model) => JevBridge::with_model(transport, model),
            None => JevBridge::new(transport),
        };
        if let Some(key) = api_key {
            bridge = bridge.with_api_key(key);
        }
        Ok(PyJevBridge {
            inner: Arc::new(with_hints(bridge, hints)),
        })
    }

    /// `JevBridge::from_env()`: OpenRouter when its key is configured, else
    /// TypeSafe; `JEV_MODEL` overrides the model alias.
    #[staticmethod]
    fn from_env(hints: Vec<HintRow>) -> Self {
        PyJevBridge {
            inner: Arc::new(with_hints(
                horismos_bridges::jev::JevBridge::from_env(),
                hints,
            )),
        }
    }

    #[getter]
    fn id(&self) -> &str {
        self.inner.id()
    }
}

/// `OpenRouterBridge` (`bridges-openrouter`): OpenAI-compatible chat
/// completions through OpenRouter, forced tool call as the typed channel.
#[cfg(feature = "bridges-openrouter")]
#[pyclass(frozen, name = "OpenRouterBridge", module = "horismos._horismos")]
pub(crate) struct PyOpenRouterBridge {
    inner: Arc<horismos_bridges::openrouter::OpenRouterBridge>,
}

#[cfg(feature = "bridges-openrouter")]
#[pymethods]
impl PyOpenRouterBridge {
    /// A bridge for one OpenRouter-routed model id; the key is read from
    /// `OPENROUTER_API_KEY` / `OPENROUTER_API_KEY_FILE` at call time.
    #[new]
    fn new(model: String) -> Self {
        PyOpenRouterBridge {
            inner: Arc::new(horismos_bridges::openrouter::OpenRouterBridge::new(model)),
        }
    }

    /// `OpenRouterBridge::from_env()`: `OPENROUTER_MODEL`, else the default.
    #[staticmethod]
    fn from_env() -> Self {
        PyOpenRouterBridge {
            inner: Arc::new(horismos_bridges::openrouter::OpenRouterBridge::from_env()),
        }
    }

    #[getter]
    fn id(&self) -> &str {
        self.inner.id()
    }
}

/// The shared handle behind any bridge object this module defines.
fn bridge_of(ob: &Bound<'_, PyAny>) -> PyResult<Arc<dyn Bridge>> {
    if let Ok(mock) = ob.cast::<PyMockBridge>() {
        return Ok(mock.get().inner.clone());
    }
    #[cfg(feature = "bridges-jev")]
    if let Ok(jev) = ob.cast::<PyJevBridge>() {
        return Ok(jev.get().inner.clone());
    }
    #[cfg(feature = "bridges-openrouter")]
    if let Ok(openrouter) = ob.cast::<PyOpenRouterBridge>() {
        return Ok(openrouter.get().inner.clone());
    }
    let type_name = ob
        .get_type()
        .name()
        .map_or_else(|_| "?".to_owned(), |n| n.to_string());
    Err(bridge_err(
        ob.py(),
        format!("a {type_name} is not a Horismos bridge"),
    ))
}

fn bridges_of(bridges: &[(String, Bound<'_, PyAny>)]) -> PyResult<Vec<(String, Arc<dyn Bridge>)>> {
    bridges
        .iter()
        .map(|(prefix, ob)| Ok((prefix.clone(), bridge_of(ob)?)))
        .collect()
}

/// `AgentBuilder`: workflow selected, every bridge bound under its prefix,
/// binding completeness checked (`build()` fails on an unbound prefix or tool).
fn build_agent<'a>(
    onto: &'a horismos_core::Ontology,
    workflow: &str,
    bridges: Vec<(String, Arc<dyn Bridge>)>,
) -> Result<Agent<'a>, WorkflowError> {
    let mut builder = Agent::builder(onto).workflow(workflow)?;
    for (prefix, bridge) in bridges {
        builder = builder.bridge(prefix, bridge);
    }
    builder.build()
}

fn workflow_err(py: Python<'_>, err: WorkflowError) -> PyErr {
    to_py_err(py, CoreError::from(err))
}

/// The process's one executor. Shared rather than per run because an HTTP
/// bridge's connection pool belongs to the runtime that opened it; a bridge
/// reused across runs must keep meeting the same one.
///
/// Keyed by process id: a `fork()`ed child inherits the parent's runtime but
/// none of its threads, so a DNS lookup handed to the parent's idle blocking
/// thread would wait forever. The child builds its own and leaks the
/// inherited one — dropping it would join threads that do not exist there.
fn runtime(py: Python<'_>) -> PyResult<&'static tokio::runtime::Runtime> {
    static RUNTIME: Mutex<Option<(u32, &'static tokio::runtime::Runtime)>> = Mutex::new(None);
    let pid = std::process::id();
    // Held only to read or install a reference, never across a call that
    // needs the GIL, so taking it with the GIL held cannot deadlock.
    let mut slot = RUNTIME.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some((owner, runtime)) = *slot
        && owner == pid
    {
        return Ok(runtime);
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| internal_err(py, format!("the agent executor could not start: {e}")))?;
    let runtime: &'static tokio::runtime::Runtime = Box::leak(Box::new(runtime));
    *slot = Some((pid, runtime));
    Ok(runtime)
}

/// Checks, without running anything, that `workflow` names a declared
/// `horis:Workflow` whose every model prefix and tool binding is bound — so
/// `Agent(...)` fails at construction, never mid-run (doc 11).
#[pyfunction]
fn check_agent(
    py: Python<'_>,
    onto: &PyOntology,
    workflow: &str,
    bridges: Vec<(String, Bound<'_, PyAny>)>,
) -> PyResult<()> {
    let bridges = bridges_of(&bridges)?;
    build_agent(&onto.inner, workflow, bridges)
        .map(drop)
        .map_err(|e| workflow_err(py, e))
}

/// One `Agent::run` against `world`, marshaled whole.
#[pyclass(frozen, name = "RunOutcome", module = "horismos._horismos")]
pub(crate) struct PyRunOutcome {
    /// The terminal step reached, in prefixed form.
    #[pyo3(get)]
    terminal: String,
    /// `(property, value)` for the workflow's `horis:decides`, when the run
    /// gave it a value on the case subject.
    #[pyo3(get)]
    decision: Option<(String, Py<PyAny>)>,
    #[pyo3(get)]
    fallback_taken: bool,
    /// The case subject perception resolved, if perception ran.
    #[pyo3(get)]
    case: Option<PyNodeRef>,
    /// The derivation of `decision`, read under the run's own lock so it can
    /// never describe a different world state than `decision` does.
    #[pyo3(get)]
    proof: Option<Py<PyProof>>,
    /// The `horismos.trace/2` trajectory, serialised by core.
    #[pyo3(get)]
    trace_json: String,
    /// The committed diff against the pre-run world, when asked for.
    #[pyo3(get)]
    diff: Option<Py<PyDiff>>,
}

/// The diff since `before`, when one was asked for.
fn diff_since(
    py: Python<'_>,
    world: &horismos_core::World,
    before: Option<&horismos_core::Snapshot>,
) -> PyResult<Option<Py<PyDiff>>> {
    before
        .map(|before| Py::new(py, marshal_diff(py, world, &world.diff(before))?))
        .transpose()
}

/// Runs `workflow` once against `world` — every step's transaction, gate and
/// chaining inside core, with the interpreter detached for the whole run
/// (bridge calls included). `run_id`/`now` are the caller's: core mints
/// neither randomness nor a clock reading.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn run_agent(
    py: Python<'_>,
    world: &PyWorld,
    workflow: &str,
    bridges: Vec<(String, Bound<'_, PyAny>)>,
    text: String,
    run_id: String,
    now: DateTime<FixedOffset>,
    with_diff: bool,
) -> PyResult<PyRunOutcome> {
    let bridges = bridges_of(&bridges)?;
    let runtime = runtime(py)?;
    let mut guard = world.lock(py)?;
    // The agent runs over exactly this world's ontology (a cheap `Arc`
    // clone), so a workflow can never be paired with a foreign world.
    let onto = guard.ontology().clone();
    let before = with_diff.then(|| guard.snapshot());
    let core: &mut horismos_core::World = &mut guard;
    let result: Result<RunOutcome, WorkflowError> = py.detach(|| {
        let agent = build_agent(&onto, workflow, bridges)?;
        runtime.block_on(agent.run(core, RunInput::new(run_id, text, now)))
    });
    let world: &horismos_core::World = &guard;
    let outcome = match result {
        Ok(outcome) => outcome,
        Err(e) => {
            // A step can fail after earlier steps committed (perception
            // facts, then a deferred step kind): those facts stand, so the
            // subscribers still get them — the Python layer dispatches `_diff`
            // before re-raising.
            let err = workflow_err(py, e);
            if let Some(diff) = diff_since(py, world, before.as_ref())? {
                err.value(py).setattr("_diff", diff)?;
            }
            return Err(err);
        }
    };

    let compact = |iri| world.ontology().compact(world.resolve(iri));
    let decision = match &outcome.decision {
        Some((property, value)) => Some((compact(*property), from_value(py, value, world)?)),
        None => None,
    };
    let derivation = match (&outcome.decision, outcome.case) {
        (Some((property, _)), Some(case)) => world
            .explain(case, *property)
            .map(|p| Py::new(py, proof(world, p)))
            .transpose()?,
        _ => None,
    };
    Ok(PyRunOutcome {
        terminal: compact(outcome.terminal),
        decision,
        fallback_taken: outcome.fallback_taken,
        case: outcome.case.map(|case| node(world, case)),
        proof: derivation,
        trace_json: outcome.trace.to_json(),
        diff: diff_since(py, world, before.as_ref())?,
    })
}

/// The bridges compiled into this build, by prefix-style name.
fn compiled_bridges() -> Vec<&'static str> {
    let mut names = vec!["mock"];
    if cfg!(feature = "bridges-jev") {
        names.push("jev");
    }
    if cfg!(feature = "bridges-openrouter") {
        names.push("openrouter");
    }
    names
}

/// Registers the classes and functions on the module.
pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("BRIDGES", compiled_bridges())?;
    m.add_class::<PyMockBridge>()?;
    #[cfg(feature = "bridges-jev")]
    m.add_class::<PyJevBridge>()?;
    #[cfg(feature = "bridges-openrouter")]
    m.add_class::<PyOpenRouterBridge>()?;
    m.add_class::<PyRunOutcome>()?;
    m.add_function(wrap_pyfunction!(check_agent, m)?)?;
    m.add_function(wrap_pyfunction!(run_agent, m)?)?;
    Ok(())
}
