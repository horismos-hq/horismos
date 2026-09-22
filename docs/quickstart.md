# Quickstart: 10 minutes, zero API keys

By the end of this page you will have loaded an ontology, asserted typed facts, watched the validation gate refuse bad data in domain terms, read a rule-derived conclusion with its proof, run the full agent loop on the mock bridge, and seen the one line that switches it to a live model. Nothing here needs an API key until the last step, and that step reads the key from your environment.

Every Python block on this page is executed verbatim by the test suite (`python/tests/test_devdocs_snippets.py`). If it renders here, it ran.

## 0. Build

You need a stable Rust toolchain (`rustup`), [uv](https://docs.astral.sh/uv/), and Python 3.11 or newer.

```sh
git clone https://github.com/horismos-hq/horismos.git && cd horismos
make develop        # uv sync + maturin develop: builds the extension into .venv
```

The first build compiles the Rust core; later ones are incremental. Run the snippets below with `uv run python` (paste them into a file such as `quickstart.py` at the repository root and run `uv run python quickstart.py`), because the fixture paths are repo-relative.

## 1. Load the triage ontology

The running example is clinical triage. The whole overlay is one Turtle file plus an imported SNOMED CT subset; the parts that matter for the next five minutes are these:

```turtle
:heartRate a rdf:Property ;
    rdfs:domain :Patient ; rdfs:range xsd:decimal ;
    horis:unit "bpm" ; horis:min 0 ; horis:max 300 ;
    horis:cardinality "0..1" .

:ChestPain rdfs:subClassOf :Symptom ;
    owl:sameAs sct:29857009 .            # SNOMED CT anchor — its subtypes come along

:acuity a rdf:Property ;
    rdfs:domain :Patient ; rdfs:range :AcuityLevel ;
    horis:cardinality "0..1" ;
    horis:assignedBy horis:RulesOnly .   # perception may NEVER set this field

:r1 a horis:Rule ;
    horis:priority 10 ;
    horis:when [ horis:all (
        [ horis:subject :Patient ; horis:property :heartRate  ; horis:exceeds 120 ]
        [ horis:subject :Patient ; horis:property :hasSymptom ; horis:is :ChestPain ] ) ] ;
    horis:then [ horis:property :acuity ; horis:value :Immediate ] ;
    horis:explain "Tachycardia with chest pain — immediate track" .
```

Loading parses the Turtle, resolves imports, validates the RDFS-plus profile, and builds the schema and rules — one native call, GIL released:

```python
from horismos import (
    HorismosOntologyError,
    HorismosValidationError,
    Ontology,
    Snapshot,
    World,
)

onto = Ontology.load("examples/triage/triage.ttl",
                     imports=["examples/triage/snomed-ct-subset.ttl"])
print(onto.namespaces[""])
print(len(onto.rules), [r.curie for r in onto.rules])
print(onto.rules[0].priority, onto.rules[0].explain)
```

```text
https://example.org/triage#
6 [':r1', ':r2', ':r3', ':r4', ':r6', ':r5']
10 Tachycardia with chest pain — immediate track
```

## 2. Assert typed facts, get a decision with a proof

Properties are spelled as `snake_case` keywords that map onto the ontology's `camelCase` names; classes and values are the same CURIEs the author wrote. The whole payload crosses the boundary once; the gate and forward chaining run inside that one transaction:

```python
world = World(onto)
p = world.assert_(":Patient", subject=":p7",
                  heart_rate=128, has_symptom=[":CrushingChestPain"])

print(world.value(p, ":acuity") == onto.ref(":Immediate"))
print(world.explain(p, ":acuity").render())
```

```text
True
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 128 exceeds 120   (asserted by the api)
  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)
```

Every leaf names its provenance. When the fact comes from a perception step instead of the API, the leaf says so and carries the extraction confidence — the neuro-symbolic seam stays visible down to the fact.

## 3. Watch the gate reject bad data

Four bad assertions. Each fails atomically, in domain terms, with a machine-readable structure alongside the sentence:

```python
bad = [
    dict(heart_rate=1280),        # outside horis:min/max
    dict(acuity=":Immediate"),    # a decision property — rules only
    dict(heart_rate="fast"),      # not an xsd:decimal
    dict(heart_rat=120),          # a typo
]
for props in bad:
    try:
        world.assert_(":Patient", **props)
    except HorismosValidationError as e:
        print(f"[{e.constraint}] {e}  data={e.data}")
    except HorismosOntologyError as e:
        print(f"[ontology] {e}  suggestion={e.suggestion}")

print(len(world.instances(":Patient")))   # nothing partial landed
```

```text
[max] heartRate 1280 exceeds horis:max 300 (bpm) on :Patient  data={'bound': '300', 'unit': 'bpm'}
[provenance_refused] acuity may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api  data={'policy': 'horis:RulesOnly'}
[ontology] "fast" is not a valid xsd:decimal lexical form  suggestion=None
[ontology] no property named "heart_rat" is declared by the ontology  suggestion=did you mean 'heart_rate'?
1
```

The second line is the one to remember. `horis:assignedBy horis:RulesOnly` means no perception output, no tool result, and no API caller can set `acuity`. The same check runs for every provenance; there is no privileged path.

## 4. Query the decision

The pattern query is class-scoped and subsumption-aware; an undeclared class or property is an error, never a silently empty list:

```python
print(world.query(":Patient", acuity=":Immediate"))
```

```text
[Ref(:p7)]
```

## 5. Change a base fact, watch the conclusion follow

Derived facts are never left stale — and never left empty either. Replacing the heart rate that `r1` rested on withdraws its conclusion and re-derives from the new base facts in the same transaction: at 90 bpm `r3` fires instead, so `:Immediate` becomes `:Standard`. Putting the heart rate back returns `:Immediate`. There is no intermediate state in which the property is unassigned, because the rule set is total and every transaction ends at the fixed point:

```python
world.assert_(":Patient", subject=":p7", heart_rate=90)
print(world.value(p, ":acuity"))
world.assert_(":Patient", subject=":p7", heart_rate=130)
print(world.value(p, ":acuity"))
```

```text
Ref(:Standard)
Ref(:Immediate)
```

Try to retract the derived fact directly and the gate tells you which rule owns it — the remedy is to change the base facts, which is what you just did.

## 6. Reach through the imported vocabulary

No rule mentions unstable angina. Assert the SNOMED code and rule `r1` still fires, because the imported hierarchy places `sct:4557003` under chest pain and the anchor in the overlay connects the two:

```python
q = world.assert_(":Patient", subject=":p8",
                  heart_rate=132, has_symptom=["sct:4557003"])
print(world.explain(q, ":acuity").render())
```

```text
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 132 exceeds 120   (asserted by the api)
  └─ hasSymptom sct:4557003 ⊑ :ChestPain   (subsumption)
```

This is the quiet advantage of ontology-first: rules are written against concepts, and coverage grows with the vocabulary rather than with the prompt.

## 7. Snapshots, diffs, and the wire form

Snapshots are O(1) values with structural sharing. Diffs are deterministic. The binary form is versioned, and every fact re-enters the gate when it is loaded back:

```python
before = world.snapshot()
r = world.assert_(":Patient", subject=":p9",
                  heart_rate=140, has_symptom=[":ChestPain"])
for fact in world.diff(before).added:
    prop = onto.compact(fact.property) if fact.property else "-"
    print(fact.kind, fact.subject.curie, prop, fact.value)

wire = world.snapshot().to_bytes()
restored = World.from_snapshot(onto, Snapshot.from_bytes(onto, wire))
print(restored.value(r, ":acuity"))
```

```text
class :p9 - https://example.org/triage#Patient
property :p9 :heartRate 140
property :p9 :hasSymptom Ref(:ChestPain)
property :p9 :acuity Ref(:Immediate)
Ref(:Immediate)
```

## 8. Subscribe to committed facts

Callbacks fire after the transaction commits, matched against the committed diff. A rejected transaction notifies nobody:

```python
world.subscribe(":acuity",
                lambda subject, value: print("page the charge nurse for", subject.curie),
                value=onto.ref(":Immediate"))
world.assert_(":Patient", subject=":p10",
              heart_rate=150, has_symptom=[":CrushingChestPain"])
```

```text
page the charge nurse for :p10
```

## 9. Run the whole agent loop — mock bridge, no network, no key

Everything above is the symbolic kernel driven from Python with structured input. The full loop — a perception step that turns text into typed facts, a reasoning step, an action step that fires a tool, a narration step — is declared in the same Turtle file as a workflow and executed by the Rust interpreter. `make demo` runs it against the deterministic mock bridge:

```sh
make demo
```

```text
Horismos triage agent demo — mock
input: "58M, crushing chest pain, heart rate 128"

terminal: :Done
fallback taken: false
decision: :acuity = :Immediate

proof (:acuity):
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 128 exceeds 120   (asserted by perception)
  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)

narration:
  [:Explain] Patient triaged as immediate acuity: heart rate 128 bpm with crushing chest pain meets the tachycardia-plus-chest-pain rule.

trajectory (4 steps) written to /var/folders/…/horismos_triage_demo_trajectory.json
```

Note the proof leaf: `asserted by perception`. The same rule, the same proof shape, but the base fact now came through the model — and the trace file records that step with its model id, confidence, threshold and every fact it asserted. Open the trajectory JSON: it is `horismos.trace/2`, the schema described in [Traces](framework/09-traces.md), with `loss_mask` already set per step.

!!! note "Python drives the kernel; the loop is Rust today"
    The workflow interpreter lives in `horismos-core::workflow::Agent` and is exercised from Rust (`make demo`, `crates/horismos-bridges/tests/agent_loop_dod.rs`). A Python `Agent.run(...)` wrapper is a PRP-004 follow-up. From Python you drive the symbolic kernel directly — which is also the primary path for structured, standards-hydrated input.

## 10. Go live with one environment variable

`make demo-live` switches to the live bridge on exactly one signal: the `OPENROUTER_API_KEY` environment variable is set. The bridge itself also accepts a key from a local file named by `OPENROUTER_API_KEY_FILE`, but that is the bridge's own fallback — it does not put the demo binary into live mode, so with only the file variable set the demo runs the mock bridge. With the variable unset the same command does not fail and prints no warning: it runs the mock bridge, and the first line of output reads `Horismos triage agent demo — mock` — check that line to confirm which bridge ran. The bridge never logs the key, never writes it to a trace, and nothing in the repository ever contains it. Pick a model with `OPENROUTER_MODEL` if you do not want the inexpensive default:

```sh
export OPENROUTER_API_KEY=...        # your existing key; never hardcode or commit it
make demo-live                      # same demo, OpenRouterBridge::from_env() swapped in; unset key → mock, check the first output line
```

The ontology names a model prefix (`horis:model "claude:…"`); deployment maps that prefix to a bridge instance. Swapping providers is configuration, and the trace records the bridge actually invoked, so an override is visible after the fact.

!!! tip "See a real run, step by step"
    [Go live in 2 minutes](go-live.md) runs exactly this command with a key and walks through the captured trace: the perception call and the typed facts it produced, the gate they passed, the rule that fired, and the proof — with `terminal`, `decision` and the proof tree identical to the mock run above, character for character.

## Where next

- [Coming from LangGraph](coming-from-langgraph.md) — wrap a graph node around the kernel in ten lines.
- [Governing a coding agent](governing-a-coding-agent.md) — the change gate: a 28-term overlay where the bot cannot approve itself.
- [The guarantees, honestly](guarantees.md) — what `verify()` proves, and what it never will.
- [Python API reference](framework/11-python-api.md) and [The World](framework/03-world.md) for the full surface.
