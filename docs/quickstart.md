# Quickstart: 10 minutes, zero API keys

By the end of this page you will have loaded a shield for a robot's motion commands, asserted proposed commands as typed facts, watched the validation gate refuse a command above the mechanical limit in the units the ontology declares, read every verdict with its proof, watched a standing verdict recomputed in the transaction that engaged the emergency stop, and read the verification report that proves the shield assigns a verdict to every reachable command. Nothing on this page needs a model or an API key. The shield is entirely symbolic: the planner that proposes each command is upstream of it, and the shield sees typed facts. Where a model does come in — a perception step turning text into typed facts, the agent loop, going live — the [clinical quickstart](quickstart-clinical.md) picks up with the triage example.

Every Python block on this page is executed verbatim by the test suite (`python/tests/test_devdocs_snippets.py`). If it renders here, it ran.

## 0. Install

Horismos is on PyPI:

```sh
pip install horismos          # or: uv pip install horismos
```

`pip install horismos` installs the latest stable release; pre-releases install only on request (`pip install --pre horismos`, `uv pip install --prerelease allow horismos`).

Each release ships one abi3 wheel per platform, serving CPython 3.11 and newer: Linux x86_64 (glibc 2.17 or newer), Linux aarch64 (glibc 2.28 or newer), macOS universal2 (Apple Silicon and Intel), and Windows x86_64. The release pipeline installs each of the four wheels and runs the symbolic quickstart from the [Python API reference](framework/11-python-api.md) and `python -m horismos demo` against it on Python 3.11 through 3.14; the aarch64 wheel runs under emulation. Every wheel includes the Jev and OpenRouter model bridges. Anything else installs from the source archive, which compiles the Rust core locally, so it needs a Rust toolchain and a few minutes.

### The clinical demo

The package carries its own examples, so a first run needs nothing else: no clone, no key, no network. The release gate's smoke run is the clinical demo:

```sh
python -m horismos demo
```

It reads two clinical notes through a perception workflow on the scripted mock bridge: the first clears the step's `horis:confidence 0.85` gate and a rule decides, the second falls short and goes to a clinician. That example — perception, the agent loop, an imported vocabulary, and the one variable that switches to a live model — is walked through on the [clinical quickstart](quickstart-clinical.md). This page stays with the shield, which needs no model at all.

The snippets below load the shield from the same installed package, through `horismos.examples.path`, so they run as pasted, from any directory.

## 1. Load the shield

The running example is a shield for a mobile manipulator: a learned policy proposes a motion command, and this overlay decides, with a proof, whether the command may reach the actuator. **It is not a safety-rated component.** Horismos is a supervisory decision layer between a planner and an actuator interface, not a safety PLC or a control loop; it makes no claim against ISO 10218, ISO/TS 15066 or IEC 61508 and has not been assessed under them. The thresholds below are teaching values; a real cell takes its own from its risk assessment.

The whole overlay is one Turtle file of nine rules ([Physical AI](use-cases/physical-ai.md) shows all of it). The parts that matter for the next five minutes are a range-constrained speed, a decision property that only rules may assign, and two of the rules that assign it:

```turtle title="examples/gallery/safety-envelope.ttl (excerpt)"
:requestedSpeed a rdf:Property ;
    rdfs:domain :MotionCommand ; rdfs:range xsd:decimal ;
    horis:unit "m/s" ;
    horis:min 0 ; horis:max 2.0 ;          # the gate refuses anything above the mechanical limit
    horis:cardinality "1" .

:verdict a rdf:Property ;
    rdfs:domain :MotionCommand ; rdfs:range :Verdict ;
    horis:cardinality "0..1" ;
    horis:assignedBy horis:RulesOnly .   # the policy network cannot mark its own command safe

:blockOnEStop a horis:Rule ;
    horis:priority 100 ;
    horis:when [
        horis:subject :MotionCommand ; horis:property :eStopEngaged ; horis:is true
    ] ;
    horis:then [ horis:property :verdict ; horis:value :Blocked ] ;
    horis:explain "Emergency stop is engaged — no motion" .

:attenuateInSharedCell a horis:Rule ;
    horis:priority 50 ;
    horis:when [
        horis:all (
            [ horis:subject :MotionCommand ; horis:property :eStopEngaged ; horis:is false ]
            [ horis:subject :MotionCommand ; horis:property :zone ; horis:is :SharedCell ]
            [ horis:subject :MotionCommand ; horis:property :requestedSpeed ; horis:exceeds 0.25 ]
            [ horis:subject :MotionCommand ; horis:property :nearestHumanDistance ; horis:atLeast 0.5 ]
        )
    ] ;
    horis:then [ horis:property :verdict ; horis:value :Attenuated ] ;
    horis:explain "Shared cell with a person present — speed is capped, command is attenuated" .
```

Loading parses the Turtle, validates the RDFS-plus profile, builds the schema and rules, and runs the verifier — one native call, GIL released. An overlay with a coverage gap does not load at all; section 7 shows that. `path` finds the file inside the installed package, and the rules come back highest priority first:

```python
from horismos import (
    HorismosValidationError,
    HorismosVerificationError,
    Ontology,
    Snapshot,
    World,
)
from horismos.examples import path

onto = Ontology.load(path("gallery/safety-envelope.ttl"))
print(onto.namespaces[""])
print(len(onto.rules), [r.curie for r in onto.rules])
print(onto.rules[0].priority, onto.rules[0].explain)
```

```text
https://example.org/shield#
9 [':blockOnEStop', ':blockRestrictedZone', ':blockTooClose', ':noProximityReadingBlocks', ':attenuateInSharedCell', ':attenuateOpenFloorNearHuman', ':permitOpenFloor', ':permitSlowOpenFloorNearHuman', ':permitSlowSharedCell']
100 Emergency stop is engaged — no motion
```

## 2. Propose a command, get a verdict with a proof

Properties are spelled as `snake_case` keywords that map onto the ontology's `camelCase` names; classes and values are the same CURIEs the author wrote. The whole command crosses the boundary once; the gate and forward chaining run inside that one transaction, so the verdict is standing by the time the call returns:

```python
world = World(onto)
cmd = world.assert_(":MotionCommand", subject=":cmd-1",
                    requested_speed=1.2, nearest_human_distance=3.4,
                    e_stop_engaged=False, zone=":OpenFloor")

print(world.value(cmd, ":verdict") == onto.ref(":Permitted"))
print(world.explain(cmd, ":verdict").render())
```

```text
True
:verdict = :Permitted   via permitOpenFloor — "Open floor with clear separation — command permitted as proposed"
  ├─ eStopEngaged false is false   (asserted by the api)
  ├─ zone :OpenFloor is :OpenFloor   (asserted by the api)
  └─ nearestHumanDistance 3.4 atLeast 1.0   (asserted by the api)
```

Every leaf names its provenance. Here each base fact was asserted by the API — the adapter between the planner and the shield. When a fact comes from a perception step instead, the leaf says so and carries the extraction confidence; the neuro-symbolic seam stays visible down to the fact.

## 3. Attenuate, block, and refuse silence

The same call, three more proposals. In a shared cell with a person present the speed is capped, so a command at 0.8 m/s is attenuated rather than passed. A person inside the minimum separation distance blocks the command whatever else is true. And a command that arrives with no proximity reading at all is blocked *explicitly*, with a proof that names the reading that never arrived:

```python
proposals = {
    ":cmd-2": dict(requested_speed=0.8, nearest_human_distance=1.1,
                   e_stop_engaged=False, zone=":SharedCell"),
    ":cmd-3": dict(requested_speed=0.2, nearest_human_distance=0.3,
                   e_stop_engaged=False, zone=":SharedCell"),
    ":cmd-4": dict(requested_speed=0.1, e_stop_engaged=False, zone=":OpenFloor"),
}
for subject, state in proposals.items():
    ref = world.assert_(":MotionCommand", subject=subject, **state)
    print(world.explain(ref, ":verdict").render())
```

```text
:verdict = :Attenuated   via attenuateInSharedCell — "Shared cell with a person present — speed is capped, command is attenuated"
  ├─ eStopEngaged false is false   (asserted by the api)
  ├─ zone :SharedCell is :SharedCell   (asserted by the api)
  ├─ requestedSpeed 0.8 exceeds 0.25   (asserted by the api)
  └─ nearestHumanDistance 1.1 atLeast 0.5   (asserted by the api)

:verdict = :Blocked   via blockTooClose — "A person is inside the minimum separation distance"
  └─ nearestHumanDistance 0.3 below 0.5   (asserted by the api)

:verdict = :Blocked   via noProximityReadingBlocks — "No proximity reading — the minimum separation distance cannot be established, so motion is refused"
  ├─ eStopEngaged false is false   (asserted by the api)
  ├─ zone isNot :Restricted   (closed world)
  └─ nearestHumanDistance isEmpty   (closed world)
```

The last one is the design decision to remember. An earlier draft of this overlay gave a command with no distance reading no verdict at all and called that safe, because an actuator that receives nothing does not move. That is safety resting on silence: it holds only as long as every consumer downstream — the controller, the retry logic, the operator's dashboard — happens to treat absence as refusal, and nothing in the system says so. Here the refusal is a fact: recomputed with the world, written into the trace, readable by an auditor. Section 7 shows how the gap was found.

## 4. Watch the gate refuse a command

Four bad proposals. Each fails atomically, in domain terms and in the units the ontology declared, with a machine-readable structure alongside the sentence:

```python
bad = [
    dict(requested_speed=3.5, nearest_human_distance=5.0,
         e_stop_engaged=False, zone=":OpenFloor"),      # above the mechanical limit
    dict(requested_speed=0.1, nearest_human_distance=5.0,
         e_stop_engaged=False, zone=":OpenFloor",
         verdict=":Permitted"),                          # a decision property — rules only
    dict(requested_speed=0.1, nearest_human_distance=5.0,
         zone=":OpenFloor"),                             # no e-stop reading
    dict(requested_speed=0.1, nearest_human_distance=5.0,
         e_stop_engaged=False, zone=":Hallway"),         # not a declared zone
]
for props in bad:
    try:
        world.assert_(":MotionCommand", subject=":cmd-5", **props)
    except HorismosValidationError as e:
        print(f"[{e.constraint}] {e}  data={e.data}")

print(len(world.instances(":MotionCommand")))   # nothing partial landed
```

```text
[max] requestedSpeed 3.5 exceeds horis:max 2.0 (m/s) on :MotionCommand  data={'bound': '2.0', 'unit': 'm/s'}
[provenance_refused] verdict may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api  data={'policy': 'horis:RulesOnly'}
[cardinality_min] eStopEngaged is required by horis:cardinality "1" but no value is asserted on :MotionCommand  data={'bound': '1'}
[one_of] zone :Hallway is not one of horis:oneOf (:OpenFloor :SharedCell :Restricted) on :MotionCommand  data={'members': [':OpenFloor', ':SharedCell', ':Restricted']}
4
```

The first line is refused *before* any rule runs: a command above the mechanical limit never becomes a fact, so nothing downstream ever sees it. The second is the one to remember. `horis:assignedBy horis:RulesOnly` means no policy network, no tool result, and no API caller can mark a command `:Permitted`. The same check runs for every provenance; there is no privileged path.

## 5. Query the verdicts

The pattern query is class-scoped; an undeclared class or property is an error, never a silently empty list:

```python
print(world.query(":MotionCommand", verdict=":Blocked"))
print(world.query(":MotionCommand", verdict=":Permitted"))
```

```text
[Ref(:cmd-3), Ref(:cmd-4)]
[Ref(:cmd-1)]
```

## 6. Engage the emergency stop, watch the verdict follow

Derived facts are never left stale. `:cmd-1` is standing as `:Permitted`. Recording the emergency stop on it withdraws that verdict and re-derives from the new base facts in the same transaction — there is no window in which a previously permitted command remains permitted under a changed world. Releasing the stop returns `:Permitted`, and there is no intermediate state in which the verdict is unassigned, because the rule set is total and every transaction ends at the fixed point:

```python
world.assert_(":MotionCommand", subject=":cmd-1", e_stop_engaged=True)
print(world.explain(cmd, ":verdict").render())
world.assert_(":MotionCommand", subject=":cmd-1", e_stop_engaged=False)
print(world.value(cmd, ":verdict"))
```

```text
:verdict = :Blocked   via blockOnEStop — "Emergency stop is engaged — no motion"
  └─ eStopEngaged true is true   (asserted by the api)

Ref(:Permitted)
```

Try to retract the verdict directly and the gate tells you which rule owns it. The remedy is to change the base facts, which is what you just did:

```python
try:
    world.retract(cmd, ":verdict", ":Permitted")
except HorismosValidationError as e:
    print(f"[{e.constraint}] {e}")
```

```text
[derived_fact_retraction] verdict :Permitted was derived by :permitOpenFloor; derived facts change only by re-running inference after base-fact changes; retraction came from the api
```

## 7. Prove the coverage

A shield is only as good as its coverage, and coverage is what `verify()` proves. Completeness asks whether every reachable combination of zone, speed, distance and e-stop state gets a verdict; conflict-freedom asks whether two rules can derive different verdicts for one command; dead-rule detection asks whether any rule can never fire. `Ontology.load` already ran all of it in section 1. The report is yours to read:

```python
report = onto.verify()
print(report.ok, report.complete, report.conflicts, report.dead_rules, report.terminating)
print([(w.severity, w.domain_text) for w in report.witnesses])
```

```text
True {':verdict': True} () () True
[]
```

`ok=True` is a proof obligation discharged, not an absence of checking. Take a rule away and the verifier names what it was covering. Write a copy of the overlay without `:noProximityReadingBlocks` into the current directory and try to load it:

```python
from pathlib import Path

source = path("gallery/safety-envelope.ttl").read_text()
start = source.index(":noProximityReadingBlocks a horis:Rule")
end = source.index(" .\n", start) + 3
gap = Path("shield-with-a-gap.ttl")
gap.write_text(source[:start] + source[end:])

try:
    Ontology.load(gap)
except HorismosVerificationError as e:
    print(f"refused: {e}")

report = Ontology.load(gap, allow_incomplete=True).verify()
print(report.ok, report.complete)
for w in report.witnesses:
    print(f"[{w.severity}] {w.kind}: {w.domain_text} — {w.suggestion}")
```

```text
refused: no verdict assigned when requestedSpeed ∈ [0, 0.25) and no nearestHumanDistance assigned and eStopEngaged is false and zone is :SharedCell
False {':verdict': False}
[error] completeness: no verdict assigned when requestedSpeed ∈ [0, 0.25) and no nearestHumanDistance assigned and eStopEngaged is false and zone is :SharedCell — add a rule assigning verdict for this region
```

The load fails closed: an overlay with an error-severity witness does not load, and `allow_incomplete=True` is the only way past it — an authoring escape that is stamped into every trace the resulting ontology produces. The witness names the region a command with no proximity reading falls into: `:blockTooClose` needs `below 0.5`, the shared-cell rules need `atLeast 0.5`, and a condition over an unassigned property is false, so nothing matched. `:noProximityReadingBlocks` is what closes it, in every zone but `:Restricted`, which `:blockRestrictedZone` already covers. Every worked example on this site passes the check; [Guarantees](guarantees.md) says what it proves and what it never will.

## 8. Snapshots, diffs, and the wire form

Snapshots are O(1) values with structural sharing. Diffs are deterministic. The binary form is versioned, and every fact re-enters the gate when it is loaded back. A snapshot of the shield's world is every command and every verdict that stood, in one value:

```python
before = world.snapshot()
r = world.assert_(":MotionCommand", subject=":cmd-6",
                  requested_speed=0.2, nearest_human_distance=0.7,
                  e_stop_engaged=False, zone=":OpenFloor")
for fact in world.diff(before).added:
    prop = onto.compact(fact.property) if fact.property else "-"
    print(fact.kind, fact.subject.curie, prop, fact.value)

wire = world.snapshot().to_bytes()
restored = World.from_snapshot(onto, Snapshot.from_bytes(onto, wire))
print(restored.value(r, ":verdict"))
```

```text
class :cmd-6 - https://example.org/shield#MotionCommand
property :cmd-6 :requestedSpeed 0.2
property :cmd-6 :nearestHumanDistance 0.7
property :cmd-6 :eStopEngaged False
property :cmd-6 :zone Ref(:OpenFloor)
property :cmd-6 :verdict Ref(:Permitted)
Ref(:Permitted)
```

## 9. Subscribe to committed verdicts

Callbacks fire after the transaction commits, matched against the committed diff. A rejected transaction notifies nobody, and a permitted command does not wake a subscriber to `:Blocked`:

```python
world.subscribe(":verdict",
                lambda subject, value: print("hold the actuator for", subject.curie),
                value=onto.ref(":Blocked"))
world.assert_(":MotionCommand", subject=":cmd-7",
              requested_speed=0.5, nearest_human_distance=0.4,
              e_stop_engaged=False, zone=":OpenFloor")
world.assert_(":MotionCommand", subject=":cmd-8",
              requested_speed=0.5, nearest_human_distance=4.0,
              e_stop_engaged=False, zone=":OpenFloor")
```

```text
hold the actuator for :cmd-7
```

## 10. Where the model comes in

Everything above is the symbolic kernel driven from Python with structured input, and for a shield that is the whole story: the planner is upstream, the shield sees typed facts, and no model is asked anything. The parts of the framework a shield does not need have a clinical running example, and the [clinical quickstart](quickstart-clinical.md) walks them end to end on the triage overlay:

- **An imported vocabulary.** A rule written against `:ChestPain` fires for a SNOMED CT code it never mentioned, and the proof shows the subsumption step: [section 6](quickstart-clinical.md#6-reach-through-the-imported-vocabulary).
- **The agent loop.** A perception step turns a clinical note into typed facts behind a `horis:confidence` threshold, a rule decides, a tool fires, a narration step explains — on the deterministic mock bridge, with no network and no key, from `make demo` in a source checkout and, for the perception-and-reasoning version, from Python with `horismos.Agent`: [section 9](quickstart-clinical.md#9-run-the-whole-agent-loop-mock-bridge-no-network-no-key).
- **Going live.** One environment variable switches the same loop to a real model, and the decision and the proof do not change: [section 10](quickstart-clinical.md#10-go-live-with-one-environment-variable), then [Go live in 2 minutes](go-live.md) for a captured live trace, step by step.

## Where next

- [Physical AI](use-cases/physical-ai.md) — the whole shield overlay, what `verify()` reported before its last three rules existed, and the honest edges.
- [Clinical quickstart](quickstart-clinical.md) — triage, SNOMED CT subsumption, the mock-bridge agent loop, and going live.
- [Coming from LangGraph](coming-from-langgraph.md) — wrap a graph node around the kernel in ten lines.
- [Governing a coding agent](governing-a-coding-agent.md) — the change gate: a 28-term overlay where the bot cannot approve itself.
- [The guarantees, honestly](guarantees.md) — what `verify()` proves, and what it never will.
- [Python API reference](framework/11-python-api.md) and [The World](framework/03-world.md) for the full surface.
