# PRP-003 — Python Bindings & API

**Status**: Done (2026-08-18)
**Depends on**: PRP-001, PRP-002

## Intent

The developer front door: `pip install horismos`, a Pythonic API over the Rust core via
PyO3/maturin — the packaging pattern already proven in sigil-lang. After this PRP, a
Python developer can load the triage ontology, assert typed patients, watch the rules
derive `acuity`, ask *why*, and get a proof tree — without reading a line of Rust, and
without an LLM anywhere in the process.

## Pivot note (2026-08-24) — bounded projection kernel

Captain-adopted pivot (`horismos-thin-pivot-s1/report.md`, PRP-003 row; PRP-000 §"Snapshot-
scoped guarantees"): this PRP's historical DoD and shipped tests are unchanged; the API
surface below remains the shipped Python front door. Future whole-bundle host-adapter
entry/exit surfaces — hydrate an external snapshot in one call, return a decision plus
proposed effect set/commit receipt — are explicitly **owned by PRP-008**, not this PRP: FHIR
or repository parsing/mapping stays outside `horismos-py` (no logic in bindings, unchanged
hard rule), and no per-resource chatter enters a transaction. Nothing in this PRP's shipped
surface is deprecated or reduced by the pivot.

## What PRP-001 and PRP-002 already provide

Everything this PRP exposes already exists and is tested on the Rust side
(~235 tests, `make verify` green). The bindings *marshal*; they never decide.

- `horismos_core::Ontology` — `load(path)` / `with_import(path)` /
  `load_from_str`, prefix table, interner, parsed rules, profile validation
  (doc 02). Every load failure is an `OntologyError` in domain terms.
- `horismos_core::World` — `new(&ontology)`, `assert_instance` /
  `assert_named_instance` (one call = one transaction: five-check gate →
  base facts → forward chaining to fixed point → commit or full rollback),
  `retract`, `instances(class)` (subsumption-aware), `value` / `values`,
  `prove` (backward chaining), `explain` (recorded derivations),
  `snapshot` / `from_snapshot` / `diff`, `last_reasoning` (doc 03, doc 04).
- `horismos_core::error::HorismosError` — every variant renders
  `domain_text()` ("heartRate 1280 exceeds horis:max 300 (bpm) on :Patient")
  plus a machine-readable witness structure.
- The skeleton this PRP builds on: `crates/horismos-py` (a `cdylib` named
  `_horismos` that today exposes only `__version__`/`PROFILE_VERSION`),
  `python/horismos/` (the pure-Python package that re-exports it), maturin
  config in `pyproject.toml`, and the `make develop` / `make test-py` loop.

## Background — the ideas this PRP assumes

The audience here flips: docs 01–10 taught ontology concepts to a Python
developer; this PRP teaches *packaging and boundary* concepts to the same
reader, because the work is the boundary itself.

### Native extensions, PyO3, and who owns what

A Python *extension module* is a shared library (a `.so`/`.dylib`/`.pyd`) that
CPython loads exactly like a `.py` module. [PyO3](https://pyo3.rs) is the Rust
side of that contract: annotate a Rust struct with `#[pyclass]` and its methods
with `#[pymethods]`, and PyO3 generates the glue that makes them callable from
Python — reference counting, argument conversion, exception translation.

Horismos splits the surface in three, and the split is a hard rule
(`.claude/skills/pyo3-boundary`):

1. **`horismos-core`** decides everything. Gate checks, chaining, proof
   search — all of it runs in Rust, unchanged, whether called from Rust or
   Python.
2. **`crates/horismos-py`** (the `_horismos` native module) marshals and
   nothing else. If binding code contains an `if` about domain behavior, it
   is in the wrong crate.
3. **`python/horismos/`** (the pure-Python package) is the public surface:
   it re-exports the native handles, adds the Pythonic conveniences
   (keyword-argument construction, `snake_case` names, iteration protocols),
   and ships the type stubs. Users import `horismos`, never `horismos._horismos`.

*Python parallel*: this is the NumPy architecture — `numpy.core._multiarray_umath`
is the C engine nobody imports directly; `numpy/` is the Python face. sigil-lang's
`pyo3-ext` proved the same split for this codebase's ancestor.

### Marshaling, and why the boundary is "whole values, once"

Every value that crosses the Rust↔Python boundary is *converted* (marshaled):
a Rust `String` becomes a Python `str` by copying, a `Vec` becomes a `list`
element by element. Each individual crossing is cheap; the mistake is making
*many* of them. sigil-lang's benchmarks found marshaling dominating small
workloads when code chatted across the boundary per-triple.

Hence the rule (AGENTS.md, doc 14): **whole inputs in, whole outcomes out**.
`world.assert_(...)` hands the complete instance — class plus every property —
across in one call, and gets the complete outcome (a `Ref`, or one exception
carrying the whole witness) back in one call. There is no per-property
round-trip inside a transaction, and the transaction (gate → chain → commit)
runs entirely on the Rust side. The bindings never expose a half-committed
world because they physically cannot observe one.

### Wheels, maturin, and abi3

A *wheel* is Python's built-package format: a zip with a `.whl` suffix that
`pip install` unpacks — no compiler needed on the user's machine. For a native
package, someone has to compile the Rust once per platform and publish the
result; [maturin](https://www.maturin.rs) is the build backend that does this
for PyO3 crates (configured in `pyproject.toml` already).

Normally a native wheel also binds to one CPython version (3.11, 3.12, …)
because CPython's full C API changes each release. The *stable ABI* (“abi3”)
is the subset that doesn't: compile against `abi3-py311` (as
`crates/horismos-py` already does) and one wheel runs on every CPython ≥ 3.11.
The cost: APIs outside the stable subset are off limits — a constraint on how
the bindings are written, checked at compile time by PyO3's feature flag.

The developer loop, identical for humans and CI (`Makefile`):

```sh
make develop   # uv sync + maturin develop --uv → editable install into .venv
make test-py   # pytest against the freshly built module
```

### The GIL, and when to let go of it

CPython's *Global Interpreter Lock* serializes all Python bytecode: while any
Python thread runs, every other one waits. Native code holds the GIL by
default too — but pure-Rust work doesn't need it. PyO3 lets a binding release
the lock (`Python::detach`, the `allow_threads` pattern) around a Rust call,
so other Python threads (a web server's request handlers, a notebook's UI)
keep running while Rust chains to a fixed point.

The safety rule is symmetric: while detached, the binding must not touch any
Python object. That is trivially satisfiable here *because of* the marshaling
rule — by the time core is called, every input is already a Rust value.
Which calls can run long enough to matter (ontology load, assertion with
chaining, `prove`, verification) is decision **D3** below.

### Exceptions across the boundary

Rust signals failure with `Result<T, HorismosError>`; Python raises. The
bindings translate mechanically: every `Err` becomes one raised exception from
the `HorismosError` hierarchy (doc 11 § Errors), where `str(exc)` is exactly
`domain_text()` — the same "heartRate 1280 exceeds horis:max 300 (bpm) on
:Patient" sentence a Rust caller gets — and the machine-readable witness
rides along as exception attributes, with a fix `suggestion` where the gate
can offer one. Never a bare `RuntimeError`/`ValueError` from binding code:
a Python caller should be able to `except HorismosValidationError` and read
`.constraint` without string-parsing.

*Python parallel*: this is sigil-lang's error-experience convention carried
over — its Python users debugged gate rejections from the exception message
alone; that bar is the acceptance test for every error this PRP maps.

### Typing a dynamic surface: stubs, and the proxy question

The native module is opaque to type checkers — CPython can't introspect Rust.
Type *stubs* (`python/horismos/_horismos.pyi`) declare the surface so mypy and
IDEs see it; the repo rule is that stubs change in the same commit as the
native surface and `mypy --strict` is part of `make lint`.

Stubs cover the *static* API. The *ontology-derived* API — "a `Patient` has a
`heart_rate`" — only exists after `Ontology.load()` at runtime, which is why
key decision **D1** (generated proxy classes vs. dict-like generic
construction) is an ergonomics-vs-transparency question and why doc 11 leaves
a `[DECIDE]` marker on codegen-for-IDEs (`horismos stubgen`). Nothing about
D1 changes what the gate accepts; it changes only how the same
(class, properties) payload is *spelled* in Python.

## Scope

- **Package `horismos`** (per doc 11, minus the PRP-004 surface):
  - `Ontology.load(path, imports=[...])` → parse + profile-validate + rule
    parse via core; `.verify()` (a stub until PRP-005 — reports what load
    already checked, `ok=True` on any loaded ontology); `.namespaces` (the
    prefix table); `.ref(curie_or_iri)` (a node handle for a declared term —
    what query results compare equal to); rule introspection.
  - `World(onto)`: `.assert_()`, `.retract()`, `.query()` (per D2 — any
    query *evaluation* is a named `horismos-core::world` unit; the
    bindings only marshal), `.explain(subject, property)`, `.prove(goal)`,
    `.instances()`, `.value()`/`.values()`, `.snapshot()` /
    `World.from_snapshot()` / `.diff()`, `.subscribe(pattern, callback)` —
    ownership split decided at activation: PRP-003 ships registration +
    post-commit dispatch over the core-computed diff; the core-side
    transactional notification stage and runtime re-entrancy queueing stay
    with PRP-004 (`02-tasks-and-gaps.md` § Boundary contracts, B2/B6).
  - Boundary safety: interner tokens never cross — `Ref` is a frozen
    IRI-string handle, re-interned per receiving world (B1).
  - Proof trees as Python objects with `.render()` producing doc 04's
    indented form, byte-identical to the Rust rendering.
  - Query results as Python objects; whole inputs in, whole outcomes out.
- **Typed instance construction** per D1 — both candidate spellings
  prototyped against the triage quickstart, one shipped.
- **Errors**: the `HorismosError` hierarchy (doc 11 § Errors), every
  exception carrying `domain_text` as `str(exc)`, machine-readable fields,
  and `suggestion` where available. The structured attributes are the
  stable machine contract; `domain_text` is human-facing prose, not an API
  to parse (B3).
- **Wheels**: maturin + abi3 as configured; a local wheel build proven; the
  CI matrix *configuration* (Linux/macOS/Windows × 3.11–3.14) checked in.
  Matrix execution, publication, and CI gating are PRP-006's half of the
  split (B5) — deferred with a dated note, not silently.
- **Stubs & tests**: `.pyi` in sync, `mypy --strict` clean; pytest exercising
  every public symbol; Hypothesis property tests mirroring the Rust
  proptest invariants at the boundary (gate atomicity, determinism).

## Worked examples

All three run against the flagship fixture `examples/triage/triage.ttl`
(+ `snomed-ct-subset.ttl`) with shipped semantics; Example 1 is the DoD's
quickstart. Spelling of instance construction is shown in the D1-neutral
generic form; Example 3 shows the two D1 candidates side by side.

### Example 1 — the symbolic quickstart (structured input, no LLM)

```python
from horismos import Ontology, World

onto = Ontology.load("examples/triage/triage.ttl",
                     imports=["examples/triage/snomed-ct-subset.ttl"],
                     allow_incomplete=True)
assert not onto.verify().ok        # PRP-005 U7 completes the canonical partition
assert onto.namespaces[""] == "https://example.org/triage#"   # the : prefix

world = World(onto)
p = world.assert_(":Patient",
                  heart_rate=128,
                  has_symptom=[":CrushingChestPain"])
```

Hand-check what just happened, entirely on the Rust side of the boundary:

1. The gate admits the instance: `heartRate 128` is within
   `horis:min 0 / horis:max 300`; `CrushingChestPain` is accepted where the
   range says `Symptom` by subsumption (doc 03).
2. Forward chaining runs to the fixed point inside the same transaction.
   Rule `:r1` matches — `heartRate 128 exceeds 120` and `hasSymptom` is a
   `ChestPain` by subsumption — and asserts `acuity = Immediate` through the
   same gate with provenance `rule` (doc 04). `acuity` is
   `horis:assignedBy horis:RulesOnly`: no API caller could have set it.

```python
assert world.value(p, ":acuity") == onto.ref(":Immediate")
print(world.explain(p, ":acuity").render())
```

renders doc 04's indented proof form, byte-identical to the Rust rendering
(the exact text is already pinned by `crates/horismos-core/tests/reasoning_dod.rs`;
the bindings pass it through untouched):

```
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 128 exceeds 120   (asserted by the api)
  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)
```

### Example 2 — the error experience is the product

```python
try:
    world.assert_(":Patient", heart_rate=1280)
except HorismosValidationError as e:
    str(e)          # "heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"
    e.constraint    # "max" — machine-readable, no string parsing
    e.suggestion    # None today; the attribute is the doc-11 slot for fix
                    # suggestions, populated as core grows them
```

One call in, one exception out: the gate rejected atomically on the Rust
side, the world is byte-identical to before the call, and the whole witness
crossed the boundary once. A rules-only violation reads the same way:

```python
world.assert_(":Patient", acuity=":Immediate")
# HorismosValidationError: acuity is horis:assignedBy horis:RulesOnly;
# provenance api may not assert it
```

### Example 3 — the two D1 candidate spellings (decided 2026-08-18: B ships)

Both construct the *same* (class, properties) payload and cross the boundary
identically; the gate cannot tell them apart. Both were prototyped against
this quickstart; **candidate B is the shipped spelling** — the SNOMED import
floods A's namespace with numeric locals, and B keeps one vocabulary across
Turtle, errors, proofs, and code (`03-decision-instances.md` has the
evidence; A's prototype is preserved there).

```python
# Candidate A — generated proxy namespace (doc 11's lean):
T = onto.proxies()
p = world.assert_(T.Patient(heart_rate=128,
                            has_symptom=[T.CrushingChestPain]))

# Candidate B — dict-like generic construction:
p = world.assert_(":Patient",
                  heart_rate=128,
                  has_symptom=[":CrushingChestPain"])
```

A: discoverable (`T.<TAB>` lists the domain), typo-safe at construction time,
mirrors doc 11's `Patient(heart_rate=128, ...)` bet — but the classes exist
only at runtime unless a stubgen lands (the doc-11 `[DECIDE]`).
B: zero magic, greppable strings, no namespace object to teach — but a typo'd
class name reaches the gate before it fails, and IDEs see `str`.
The snake_case ↔ camelCase mapping (`heart_rate` ↔ `heartRate`) is automatic
and stable in both (doc 11).

## Non-goals

- **No `Agent`, workflows, bridges, or tools surface** — doc 11's `Agent`,
  `horismos.bridges`, `RunResult` belong to PRP-004 (which extends this
  package). The quickstart is therefore the *symbolic* one above, not doc
  11's agent quickstart.
- **No real verification** — `.verify()` returns a stub report; BDD checks
  with witnesses are PRP-005.
- **No PyPI publication and no full CI-matrix proof** — configuration ships
  here; running/publishing is PRP-006.
- **No async API** in v1 (revisit with the agent loop, PRP-004).
- **No trace/replay Python surface** (`horismos.replay`, training export) —
  it rides with the runtime that produces runs, PRP-004.

## Key decisions to make

1. **Generated proxy classes vs. dict-like generic instances** (D1; owns doc
   11's `[DECIDE: codegen for IDE/mypy support]`). Prototype *both* against
   the triage quickstart; the criterion is the ergonomics of the quickstart a
   newcomer types first. Evidence and the pick recorded in this folder.
2. **SPARQL-subset vs. Pythonic query builder** (D2; owns doc 03's
   `[DECIDE: exact subset grammar]`). Implement the minimal query surface
   the triage quickstart needs; defer the rest with a dated gap note.
3. **GIL strategy for long core calls** (D3). `allow_threads` around
   chaining/load/prove; escalate only if it forces API changes.

## Definition of done

`pip install` of the locally built wheel into a fresh venv → Example 1 (the
symbolic triage quickstart) runs **verbatim** with structured input, no LLM:
loads `triage.ttl` + the SNOMED subset, asserts the tachycardic chest-pain
patient, `acuity == Immediate` derived by `:r1` through the gate,
`explain(...).render()` produces the pinned proof text, the `heartRate 1280`
rejection raises `HorismosValidationError` with the domain-term sentence.
pytest exercises every public symbol; **parity is fixture-anchored** — the
proof rendering and rejection sentences are asserted byte-identical to the
Rust suite's own expectations, and the structured error attributes (not
just messages) are asserted; Hypothesis pins boundary invariants (atomic
rejection, determinism of two identically built worlds, value round-trip)
as a mandatory part of this DoD; stubs pass `mypy --strict`; the wheel
builds locally and the CI matrix config is checked in (matrix execution and
publication are PRP-006's, per the B5 split). `make verify` green.

## Glossary

- **Extension module** — a compiled shared library CPython imports like a
  `.py` file; here `horismos._horismos`, built from `crates/horismos-py`.
- **PyO3** — the Rust crate that generates the CPython glue for Rust types
  (`#[pyclass]`) and functions (`#[pymethods]`).
- **Marshaling** — converting a value between Rust and Python representations
  at the boundary; each crossing copies.
- **Wheel** — Python's installable binary package format; what `pip install`
  consumes without needing a compiler.
- **maturin** — the build backend that compiles a PyO3 crate into a wheel;
  `maturin develop` installs it editable into the current venv.
- **abi3 / stable ABI** — the version-stable subset of CPython's C API; one
  abi3 wheel serves every CPython ≥ its floor (3.11 here).
- **GIL** — CPython's Global Interpreter Lock; released around long pure-Rust
  calls so other Python threads keep running.
- **Type stub (`.pyi`)** — a declaration-only file telling mypy/IDEs the
  types of a module the checker can't introspect.
- **Proxy class** — a Python class generated from the loaded ontology
  (`T.Patient`) whose constructor spells an instance payload; a builder —
  nothing exists until the gate passes it (doc 11).
- **Witness** — the gate's machine-readable rejection record; renders as the
  domain-term sentence and rides on the raised exception.
