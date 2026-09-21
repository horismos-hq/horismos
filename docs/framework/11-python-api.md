# 11 · Python API Reference

Package: `horismos` (wheels via maturin/PyO3 over `horismos-core`). Python ≥ 3.11.
Everything here is importable from the top level unless noted.

The PRP-008 host SPI and adapters are Rust-only in v1. Whole-bundle Python entry/exit
surfaces are not exported yet; FHIR/repository parsing and mapping logic stay outside
`horismos-py` and this package (no logic in bindings, unchanged). Everything below remains the current shipped Python API.

## Quickstart (the canonical triage run)

```python
from horismos import Ontology, World, Agent
from horismos.bridges import AnthropicBridge

onto = Ontology.load("triage.ttl", imports=["snomed-ct-subset.ttl"])
assert onto.verify().ok

world = World(onto)
agent = Agent(world,
              workflow="triage:Triage",
              bridges={"claude:claude-fable-5": AnthropicBridge.from_env()},
              tools={"ehr.assign_queue": ehr_client.assign_queue})

result = agent.run("58M, crushing chest pain 20 min, HR 128, BP 92/60, diaphoretic")
print(result.decision)          # triage:Immediate
print(result.proof.render())
```

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

## `Agent`

| Member | Signature | Notes |
|--------|-----------|-------|
| `__init__` | `(world, *, workflow, bridges={}, tools={}, budget=None)` | Missing bridge/tool for a declared step → construction-time error, not runtime surprise |
| `run` | `(text_or_none, *, input=None) -> RunResult` | `text` enters at the first PerceptionStep; `input=` asserts structured facts and may skip perception entirely |
| `step` | `() -> StepResult` | Single-step execution for debugging/notebooks |
| `resume` | `(run_id, with_input=…)` | Continue a run paused at a `WaitStep` |

### `RunResult`

`decision`, `terminal`, `goal_reached`, `proof`, `trace` (see docs 05, 09),
`world` (the post-run world), `steps` (list of `StepResult`).

## `horismos.bridges`

`AnthropicBridge`, `OpenAIBridge`, `HFBridge`, `ONNXBridge`, `CallableBridge`,
`MockBridge` — all implement the `Bridge` protocol (doc 08). `MockBridge(script=…)`
is the CI workhorse.

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
