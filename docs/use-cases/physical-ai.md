# Physical AI

**Where this sits.** Horismos is a supervisory decision layer: it sits between a planner and an actuator interface and decides, with a proof, whether a proposed command may pass. It is not a safety-rated component, a safety PLC or a control loop, and it does not replace the safety-rated stops and interlocks a cell already has. Horismos makes no claim against ISO 10218, ISO/TS 15066 or IEC 61508 and has not been assessed under them. The thresholds in this overlay are teaching values; a real cell takes its own from its risk assessment. What the shield adds is a refusal with a proof above that layer, not a substitute for it.

## The scene

A learned policy — a VLA model, an RL controller, an LLM planner — proposes a motion command for a mobile manipulator. The proposal is probably fine. "Probably" is not the standard for a robot arm moving near a person. The established pattern is a **shield**: a deterministic layer between the policy and the actuator that checks each proposed action against a specification of safe states and blocks or attenuates the ones that violate it (Alshiekh et al., "Safe Reinforcement Learning via Shielding", AAAI 2018). The policy stays free to be clever inside the envelope; the envelope is not negotiable.

The 2026 agent-safety literature converges on the same shape for software agents — deterministic pre-execution gates over declared constraints and current state: SMT policies checked before a call, temporal-logic monitors over action traces, in-kernel enforcement because "tool-call guardrails cannot intercept system actions that bypass the tool layer," and freshness-bounded approval because a verdict "correct at check time" can be "stale by actuation." A robot makes every one of those concerns physical.

## The overlay

Typed preconditions over the world state gate the command. The gate's range constraint on `requestedSpeed` means a command above the mechanical limit never becomes a fact at all; the rules decide the rest:

```turtle
--8<-- "examples/gallery/safety-envelope.ttl"
```

## The decision, with proof

```python
from horismos import HorismosValidationError, Ontology, World
from horismos.examples import path

onto = Ontology.load(path("gallery/safety-envelope.ttl"))
world = World(onto)

def shield(subject: str, **state: object) -> None:
    ref = world.assert_(":MotionCommand", subject=subject, **state)
    proof = world.explain(ref, ":verdict")
    print(proof.render() if proof else f"{subject}: no verdict — command does not proceed")

shield(":cmd-1", requested_speed=1.2, nearest_human_distance=3.4,
       e_stop_engaged=False, zone=":OpenFloor")
shield(":cmd-2", requested_speed=0.8, nearest_human_distance=1.1,
       e_stop_engaged=False, zone=":SharedCell")
shield(":cmd-3", requested_speed=0.2, nearest_human_distance=0.3,
       e_stop_engaged=False, zone=":SharedCell")
```

```text
:verdict = :Permitted   via permitOpenFloor — "Open floor with clear separation — command permitted as proposed"
  ├─ eStopEngaged false is false   (asserted by the api)
  ├─ zone :OpenFloor is :OpenFloor   (asserted by the api)
  └─ nearestHumanDistance 3.4 atLeast 1.0   (asserted by the api)

:verdict = :Attenuated   via attenuateInSharedCell — "Shared cell with a person present — speed is capped, command is attenuated"
  ├─ eStopEngaged false is false   (asserted by the api)
  ├─ zone :SharedCell is :SharedCell   (asserted by the api)
  ├─ requestedSpeed 0.8 exceeds 0.25   (asserted by the api)
  └─ nearestHumanDistance 1.1 atLeast 0.5   (asserted by the api)

:verdict = :Blocked   via blockTooClose — "A person is inside the minimum separation distance"
  └─ nearestHumanDistance 0.3 below 0.5   (asserted by the api)
```

Three things a prompt cannot give you. A command over the mechanical limit is refused *before* any rule runs, in the units the ontology declared. The policy network cannot mark its own command safe. And a command with no distance reading is refused *explicitly*. An earlier draft of this overlay gave that command no verdict at all and called it safe, on the grounds that an actuator which receives nothing does not move. That is safety resting on silence: it holds only for as long as every consumer downstream — the controller, the retry logic, the operator's dashboard — happens to treat absence as refusal, and nothing in the system says so. `verify()` reported the region, and closing it required `:noProximityReadingBlocks`: a `:Blocked` verdict with a proof that names the reading that never arrived. At an actuator boundary that is the stronger design, because the refusal is now a fact — recomputed with the world, written into the trace, and readable by the auditor — rather than an inference each consumer has to make, correctly, about a value it never saw:

```python
for bad in (dict(requested_speed=3.5, nearest_human_distance=5.0, e_stop_engaged=False, zone=":OpenFloor"),
            dict(requested_speed=0.1, e_stop_engaged=False, zone=":OpenFloor", verdict=":Permitted")):
    try:
        world.assert_(":MotionCommand", subject=":cmd-4", **bad)
    except HorismosValidationError as e:
        print(f"[{e.constraint}] {e}")

shield(":cmd-5", requested_speed=0.1, e_stop_engaged=False, zone=":OpenFloor")
```

```text
[max] requestedSpeed 3.5 exceeds horis:max 2.0 (m/s) on :MotionCommand
[provenance_refused] verdict may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api
:verdict = :Blocked   via noProximityReadingBlocks — "No proximity reading — the minimum separation distance cannot be established, so motion is refused"
  ├─ eStopEngaged false is false   (asserted by the api)
  ├─ zone isNot :Restricted   (closed world)
  └─ nearestHumanDistance isEmpty   (closed world)
```

The emergency stop is engaged while `:cmd-1` is standing. Its verdict is recomputed in the transaction that recorded the stop — there is no window in which a previously permitted command remains permitted under a changed world:

```python
shield(":cmd-1", e_stop_engaged=True)
```

```text
:verdict = :Blocked   via blockOnEStop — "Emergency stop is engaged — no motion"
  └─ eStopEngaged true is true   (asserted by the api)
```

## Why verification matters here

A shield is only as good as its coverage, and coverage is what `verify()` proves. Completeness asks whether every reachable combination of zone, speed, distance and e-stop state gets a verdict, and when the check first ran against this overlay it returned two regions that had none. The first: *"no verdict assigned when requestedSpeed ∈ [0, 0.25) and nearestHumanDistance < 0 and eStopEngaged is false and zone is :SharedCell"* — a shared-cell command with no proximity reading at all, because `:blockTooClose` needs `below 0.5` and both permit rules need `atLeast 0.5`, and a condition over an unassigned property is false. The second was narrower and easier to miss by eye: an open-floor command with a person between 0.5 m and 1.0 m, falling between `:blockTooClose` and `:permitOpenFloor`'s 1.0 m of required clearance. Closing them is what `:noProximityReadingBlocks` and the two `:OpenFloor` rules in the overlay are for — a missing measurement is refused outright, and a person inside the shared-cell distance is treated as a shared cell whatever the zone label says. Conflict-freedom names every pair of rules that can match one command with different verdicts, and it named three: `blockOnEStop` (priority 100) against each of `attenuateInSharedCell`, `permitOpenFloor` and `permitSlowSharedCell` — including the open-floor-with-clear-separation case that is exactly the `:cmd-1` transition above. Priority already resolved all three in favour of blocking; the witness (*"blockOnEStop wins by priority; confirm intended"*) asked for that ordering to be confirmed rather than assumed, and the three rules now carry an explicit `eStopEngaged is false` guard that says in the condition what priority was saying silently. Dead-rule detection catches a speed threshold above `horis:max`; non-vacuity flags a constraint that excludes no reachable value.

What it does not claim: anything about the sensor that produced `nearestHumanDistance`, the controller that executes an attenuated command, or the physical world between check and actuation. Those are the adapter's and the deployer's, and the assumption manifest lists the unverified external effects explicitly.

## Status and honest edges

- Runs today with structured input at the decision level. Nothing in Horismos runs at control-loop rates or replaces a certified safety PLC; this is the supervisory decision layer between a planner and an actuator interface, and the framework's own posture is that OS- and hardware-level isolation stays the deployer's.
- Measured, not promised. One shield decision takes about 5 µs in Rust and about 8 µs from Python (p99 under 10 µs) for a single standing command on an Apple M4, release build; cost grows about 3.6 µs per additional standing verdict, and the kernel refuses a transaction that would fire more than 100 rules. That is the cost of one supervisory decision on structured input, `explain` apart; the benchmark, the machine and every recorded run are in `crates/horismos-core/benches/README.md`.
- The freshness concern is the physical version of the snapshot-scoped guarantee: a verdict is exact for the world state it was given, and the runtime recomputes on every change it is told about. What it is not told about, it cannot gate ([Guarantees](../guarantees.md)).
- Thresholds are teaching values. A real cell derives them from its risk assessment and the applicable standards; the overlay shape is what stays the same.
