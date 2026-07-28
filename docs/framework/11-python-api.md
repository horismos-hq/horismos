# 11 · Python API Reference

Package: `sigil` (wheels via maturin/PyO3 over `sigil-core`). Python ≥ 3.11.
Everything here is importable from the top level unless noted.

## Quickstart (the canonical triage run)

```python
from sigil import Ontology, World, Agent
from sigil.bridges import AnthropicBridge

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

## `Ontology`

| Member | Signature | Notes |
|--------|-----------|-------|
| `load` | `(path, *, imports=[], verify=True, allow_incomplete=False) -> Ontology` | Parses Turtle, resolves imports, verifies (doc 10) |
| `verify` | `() -> VerificationReport` | Re-run on demand |
| `classes` / `properties` / `rules` / `workflows` | introspection collections | Iterable, indexable by CURIE |
| `ns` | `NamespaceMap` | `onto.ns.triage.Patient` → IRI |
| `schema_for` | `(class_or_step) -> dict` | The derived JSON Schema (doc 06) — inspectable |
| `diff` | `(other) -> OntologyDiff` | Domain-term rendered diff |
| `version` / `hash` | `str` | Deployment identity, stamped into traces |

## Typed instance proxies

Loading an ontology generates proxy classes (also available dynamically):

```python
from sigil import proxies
T = proxies(onto)                       # namespace object
p = T.Patient(heart_rate=128, systolic=92, diastolic=60,
              symptoms=[T.CrushingChestPain, T.Diaphoresis])
```

Property names snake_case ↔ ontology camelCase mapping is automatic and stable.
Proxies are *builders*: nothing exists until `world.assert_(p)` passes the gate.
**[DECIDE: codegen for IDE/mypy support (`sigil stubgen triage.ttl`) — PRP-003]**

## `World`

| Member | Signature | Notes |
|--------|-----------|-------|
| `assert_` | `(instance \| triple, *, provenance="api") -> Ref` | Full gate; atomic transaction incl. chaining |
| `retract` | `(ref_or_triple)` | Policy-checked (doc 03) |
| `instances` | `(cls) -> list[Ref]` | Subsumption-aware |
| `value` / `values` | `(ref, prop)` | Single / multi-valued |
| `query` | `(sparql: str) -> Rows` | The SELECT/ASK subset |
| `prove` | `(goal, *, subject=None) -> ProofNode \| None` | Backward chaining |
| `explain` | `(ref, prop) -> ProofNode` | Derivation of an existing fact |
| `snapshot` / `from_snapshot` / `diff` | | Persistent snapshots |
| `subscribe` | `(pattern, callback) -> Subscription` | Post-commit notification |

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

## `sigil.bridges`

`AnthropicBridge`, `OpenAIBridge`, `HFBridge`, `ONNXBridge`, `CallableBridge`,
`MockBridge` — all implement the `Bridge` protocol (doc 08). `MockBridge(script=…)`
is the CI workhorse.

## Errors

```
SigilError
├── SigilOntologyError        # parse/profile/import problems
├── SigilVerificationError    # .report attached, witnesses inside
├── SigilValidationError      # gate rejection; .constraint, .domain_text
├── SigilWorkflowError        # missing edges, unbound tools/bridges
├── SigilBridgeError          # provider failures (usually handled via fallback)
└── SigilToolError            # binding invocation/effect validation
```

Every error carries `domain_text` (human), `data` (machine), and where applicable
`suggestion` — the sigil-lang error-experience conventions, inherited.

## Module utilities

- `sigil.replay(trace, …)` — doc 09.
- `sigil.export.to_training_jsonl(...)` — doc 09.
- `sigil.testing` — `MockBridge`, world fixtures, trace assertions
  (`assert_fired(trace, "r1")`, `assert_no_bridge_calls(trace)`).
