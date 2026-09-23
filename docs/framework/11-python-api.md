# 11 · Python API Reference

Package: `horismos` (wheels via maturin/PyO3 over `horismos-core`). Python ≥ 3.11.
Everything here is importable from the top level unless noted.

The PRP-008 host SPI and adapters are Rust-only in v1. Whole-bundle Python entry/exit
surfaces are not exported yet; FHIR/repository parsing and mapping logic stay outside
`horismos-py` and this package (no logic in bindings). Everything below remains the current shipped Python API.

## Quickstart (the agent loop)

Natural language in, typed facts asserted, the rules deciding. This runs verbatim against
`examples/vte/` (the test suite holds it to that) on the scripted mock bridge:

```python
from datetime import UTC, datetime

from horismos import Agent, Ontology, World
from horismos.bridges import MockBridge

onto = Ontology.load("examples/vte/perception.ttl")   # vte.ttl + a perception workflow
note = "78yo, post-op day 2, on warfarin, bed-bound"
bridge = MockBridge({note: {
    "instances": [{"class": ":VteCase", "reducedMobility": True, "activeCancer": False,
                   "priorVte": False, "bleedingRisk": True}],
    "confidence": 0.93}})

agent = Agent(World(onto), workflow=":NoteToRecommendation", bridges={"jev": bridge})
result = agent.run(note, run_id="doc-11", now=datetime(2026, 9, 23, tzinfo=UTC))
print(result.decision)          # Ref(:HoldProphylaxis)
print(result.proof.render())    # via bleeding-contraindication, a leaf asserted by perception
```

`bridges={"jev": JevBridge.from_env()}` perceives with the live model instead. The model
is shown `:VteCase`'s four booleans and never `:recommendation`, which is
`horis:RulesOnly`; below the step's `horis:confidence` the run takes its fallback edge and
asserts nothing. The triage workflow also fires tools, whose bindings are Rust functions,
so its full run is the Rust demo (`make demo`); tools are not bindable from Python.

## Quickstart (symbolic — structured input, no LLM)

The engine beneath the agent, exercised directly (PRP-003's shipped
surface; this snippet runs verbatim against `examples/triage/`, and the
test suite holds it to that):

```python
from horismos import Ontology, World

onto = Ontology.load("examples/triage/triage.ttl",
                     imports=["examples/triage/snomed-ct-subset.ttl"])
assert onto.verify().ok
assert onto.namespaces[""] == "https://example.org/triage#"   # the : prefix

world = World(onto)
p = world.assert_(":Patient",
                  heart_rate=128,
                  has_symptom=[":CrushingChestPain"])

assert world.value(p, ":acuity") == onto.ref(":Immediate")   # derived by :r1
print(world.explain(p, ":acuity").render())
```

## `Ontology`

| Member | Signature | Notes |
|--------|-----------|-------|
| `load` | `(path, *, imports=None, allow_incomplete=False) -> Ontology` | Parses the root and explicit imports as one unit, then verifies; failure raises `HorismosVerificationError` unless the explicit authoring escape is set |
| `verify` | `() -> VerificationReport` | Re-run on demand |
| `classes` / `properties` / `rules` / `workflows` | introspection collections | Iterable, indexable by CURIE |
| `ns` | `NamespaceMap` | `onto.ns.triage.Patient` → IRI |
| `schema_for` | `(class_or_step) -> dict` | The derived JSON Schema (doc 06) — inspectable |
| `diff` | `(other) -> OntologyDiff` | Domain-term rendered diff |
| `version` / `hash` | `str` | Deployment identity, stamped into traces — deferred (2026-08-18): no ontology-level version vocabulary exists to bind yet, and PRP-004's trace stamping is the consumer that would need it |

### `VerificationReport`

`VerificationReport` is an immutable Python value marshaled from one complete core JSON
payload. It exposes `ok`, `complete`, `conflicts`, `dead_rules`, `terminating`,
`workflows`, `witnesses`, `assumptions`, `exit_code`, and `to_json()`. Every witness has
`kind`, `severity` (`"error"` fails the report; `"info"` is advisory), `domain_text`,
`region`, and `suggestion`; ordered region bounds retain their exact authored lexical
values and units. Each `Conflict` names both rules, their values and priorities, the
overlap region, and the `winner` whose value stands under priority order (`None` when
both rules share a priority, since the outcome is then order-dependent).

The always-present `AssumptionManifest` has exactly five fields:
`input_closure`, `external_profile_versions`, `mapping_version`, `supported_subset`,
and `unverified_external_effects`. Their current collection/string types are
provisional pending PRP-008; the fields themselves are permanent.

For CI and authoring tools:

```console
python -m horismos verify examples/triage/triage.ttl
```

The command prints the canonical report JSON and exits `0` for a passing report or `1`
for verification findings. `--import path` may be repeated. There is no switch that
disables verification; `allow_incomplete=True` is the explicit authoring/debug escape,
and every agent trace produced from such an ontology records
`ontology.allow_incomplete: true`.

## Typed instance construction

Instances are constructed in the ontology's own vocabulary — the class as
the CURIE it was authored as, properties as `snake_case` keywords (the
snake_case ↔ ontology camelCase mapping is automatic and stable; list
values fan out over a multi-valued property):

```python
p = world.assert_(":Patient", heart_rate=128,
                  has_symptom=[":CrushingChestPain"])
```

One vocabulary end to end: these are the same CURIEs the author wrote in
the Turtle and the same rendering every domain-term error and proof tree
prints. A misspelled keyword fails on the one boundary crossing with a
domain-term error and a fix suggestion (`no property named "heart_rat" is
declared by the ontology` · `did you mean 'heart_rate'?`); a name two
namespaces share must be written as a CURIE.

Generated proxy classes (`T.Patient(...)`) were prototyped against this
exact quickstart and deliberately not shipped: imported vocabularies flood
the proxy namespace (the SNOMED subset alone contributes ~50 numeric
locals, which are not even legal Python attribute names), and a second
Python-side naming system would diverge from what errors and proofs
render. Decided 2026-08-18 in PRP-003 on a prototyped-and-rejected
generated-proxy design — revisit alongside `horismos stubgen` codegen
when PRP-007's authoring tooling generates ontology artifacts anyway.

## `World`

| Member | Signature | Notes |
|--------|-----------|-------|
| `assert_` | `(class_, *, subject=None, **properties) -> Ref` | One crossing: full gate + chaining, atomic transaction |
| `retract` | `(ref, property, value)` | Policy-checked (doc 03) |
| `instances` | `(class_) -> list[Ref]` | Subsumption-aware |
| `value` / `values` | `(ref, property)` | Single / multi-valued |
| `query` | `(class_, **patterns) -> list[Ref]` | Class-scoped conjunctive patterns (doc 03 § Queries); a textual SPARQL front end is deferred beyond v1 (decided 2026-08-18 in PRP-003) |
| `prove` | `(ref, property, value) -> ProofNode \| None` | Backward chaining |
| `explain` | `(ref, property) -> ProofNode \| None` | Derivation of an existing fact |
| `snapshot` / `from_snapshot` / `diff` | | Persistent snapshots |
| `subscribe` | `(property, callback, *, value=ANY) -> Subscription` | Post-commit dispatch over the committed diff; the runtime's transactional notification stage is PRP-004's |

`ProofNode` exposes `render()`, `rule` (the concluding rule, on a conclusion), `children`,
and `provenance` — on a leaf resting on a stored fact, who asserted it: `"perception"`,
`"rule"`, `"tool"` or `"api"` (the same token the trace records).

## `Agent`

| Member | Signature | Notes |
|--------|-----------|-------|
| `__init__` | `(world, *, workflow, bridges)` | `bridges` maps each `horis:model` prefix (`"jev"` for `"jev:jev-1.13"`) to a bridge; with exactly one bound, every step uses it. An undeclared workflow, a prefix no bridge serves, or a `rust:` tool binding → `HorismosWorkflowError` here, never mid-run. A binding scheme no build implements (`python:`, `http:`, `mcp:`) is not refused here: its step takes `horis:onError` when reached |
| `__init__(…, tools=, budget=)` | | Deferred (2026-09-23): tool implementations are Rust functions, so a workflow with a `rust:` binding runs from Rust today (`make demo`); a Python-callable binding waits on doc 07's `python:` scheme |
| `run` | `(text, *, run_id=None, now=None) -> RunResult` | One whole run per call, `horis:startsAt` to a terminal step, every step's transaction, gate and chaining in core, the World locked for the whole run (bridge calls included). `run_id` defaults to a fresh UUID and `now` (timezone-aware) to the current UTC instant, read once; pass both for a reproducible trace. Subscribers receive the run's committed facts once, after it finishes — also when a later step fails and the run raises |
| `run(…, input=)` | | Deferred (2026-09-23): structured facts entering a run alongside, or instead of, perception |
| `step` / `resume` | | Deferred (2026-09-23): single-step execution and resuming at a `WaitStep` need the core loop to pause, and `WaitStep` is itself deferred there |

Perception runs only inside a run: a `PerceptionStep`'s bridge is shown the schema derived
from `horis:extracts` (doc 06), and its proposal is accepted only at or above the step's
`horis:confidence`, then asserted through the validation gate with perception provenance.
Low confidence, any bridge error, or a gate rejection takes the step's `horis:fallback`
edge. There is no call that hands a bridge's answer to the World any other way.

### `RunResult`

`terminal` (CURIE), `decision` and `decides` (the workflow's `horis:decides` value and
property, when the run set it), `fallback_taken`, `goal_reached`, `case` (the instance
perception asserted), `proof` (the decision's derivation, read in the same crossing as
`decision`), and `trace` — the `horismos.trace/2` trajectory as a dict (doc 09).
`world` and `steps` are deferred (2026-09-23): the World the agent was given is the
post-run world, and each step's record is in `trace["steps"]`.

## `horismos.bridges`

| Bridge | Construction | Notes |
|--------|--------------|-------|
| `MockBridge` | `(script=None, *, narrate=None, id="mock:default")` | Scripted by exact input text or `"*"`; unscripted input gets the low-confidence default. `invocations()` returns every request the runtime sent, schema included. The CI workhorse |
| `JevBridge` | `(transport="openrouter", *, model=None, api_key=None, hints=None)`, `.from_env(hints=None)` | TypeSafe's text → decisions model (doc 08 § `jev:`). `hints` maps a property's local name to a `PropertyHint(instructions, when_true, when_false, ordinal)`. The key is never echoed into a repr or trace |
| `OpenRouterBridge` | `(model)`, `.from_env()` | Any OpenRouter-routed chat model, forced into the derived schema |

Every bridge the published wheel ships is compiled in; `compiled()` names them
(`{"mock", "jev", "openrouter"}`). A source build made without a provider bridge's
`bridges-*` feature refuses to construct it with a `HorismosBridgeError` naming the
feature. A bridge's HTTP connections belong to the process that opened them: after
`os.fork()`, construct the child's bridges in the child.

`AnthropicBridge`, `OpenAIBridge`, `HFBridge` and `ONNXBridge` are deferred
(2026-09-23): doc 08's routing table names those backends, and none has an
implementation yet. `CallableBridge` (a Python callable as a bridge) is deferred too: a run
executes with the interpreter released, so a bridge written in Python needs a designed
re-entry point first.

## Errors

```
HorismosError
├── HorismosOntologyError        # parse/profile/import problems
├── HorismosVerificationError    # .report attached, witnesses inside
├── HorismosValidationError      # gate rejection; .constraint, .domain_text
├── HorismosWorkflowError        # missing edges, unbound tools/bridges
├── HorismosBridgeError          # provider failures (usually handled via fallback)
└── HorismosToolError            # binding invocation/effect validation
```

Every error carries `domain_text` (human), `data` (machine), and where applicable
`suggestion` — the sigil-lang error-experience conventions, inherited.

## Module utilities

- `horismos.replay(trace, …)` — doc 09.
- `horismos.export.to_training_jsonl(...)` — doc 09.
- `horismos.testing` — `MockBridge`, world fixtures, trace assertions
  (`assert_fired(trace, "r1")`, `assert_no_bridge_calls(trace)`).
