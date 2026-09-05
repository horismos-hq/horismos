# Coding

Two scenes for an agent that writes and ships code: what it may do about a red build, and whether its change may land. The second is the [change gate](../governing-a-coding-agent.md), covered in depth on its own page; this page adds CI triage and shows how the two compose.

## CI failure triage

### The scene

The agent's PR goes red. Left to a prompt, the agent reruns the job (sometimes twice), or "fixes" the test, or edits the deploy script that failed. Each of those is a judgment a model made, none of it is recorded as a decision, and the only artifact is a chat log. The disposition of a failed build should be policy: known-flaky tests get exactly one rerun, a flaky test that fails again is not flaky anymore, deploy failures roll back before anyone investigates.

### The overlay

```turtle
--8<-- "examples/gallery/ci-triage.ttl"
```

### The decision, with proof

```python
from horismos import HorismosValidationError, Ontology, World

onto = Ontology.load("examples/gallery/ci-triage.ttl")
world = World(onto)

def triage(subject: str, **observed: object) -> None:
    ref = world.assert_(":BuildFailure", subject=subject, **observed)
    proof = world.explain(ref, ":disposition")
    print(proof.render() if proof else f"{subject}: no disposition — needs a person")

triage(":run-901", failing_stage=":Test", known_flaky=True, prior_reruns=0)
triage(":run-902", failing_stage=":Lint", known_flaky=False)
triage(":run-903", failing_stage=":Deploy", known_flaky=False)
```

```text
:disposition = :RerunOnce   via rerunKnownFlakyOnce — "Known-flaky test, not yet rerun — one rerun is allowed"
  ├─ failingStage :Test is :Test   (asserted by the api)
  ├─ knownFlaky true is true   (asserted by the api)
  └─ priorReruns 0 below 1   (asserted by the api)

:disposition = :FixForward   via deterministicFailuresFixForward — "Deterministic lint/test/build failure — the agent may propose a fix through the change gate"
  ├─ failingStage :Lint inList (:Lint :Test :Build)   (asserted by the api)
  └─ knownFlaky false is false   (asserted by the api)

:disposition = :RollBack   via deployFailuresRollBack — "A failed deploy stage rolls back first and investigates second"
  └─ failingStage :Deploy is :Deploy   (asserted by the api)
```

The rerun happens, the failure reproduces. The agent does not get to decide the test is "still flaky"; the facts change and the disposition follows:

```python
triage(":run-901", prior_reruns=1, reproduces_on_rerun=True)
try:
    world.assert_(":BuildFailure", subject=":run-904",
                  failing_stage=":Test", disposition=":RerunOnce")
except HorismosValidationError as e:
    print(f"[{e.constraint}] {e}")
```

```text
:disposition = :EscalateToOwner   via flakyStillFailingEscalates — "The flaky test failed again on rerun — it is no longer flaky, escalate to the owner"
  ├─ knownFlaky true is true   (asserted by the api)
  └─ reproducesOnRerun true is true   (asserted by the api)

[provenance_refused] disposition may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api
```

In a full workflow, `:RerunOnce` is an `ActionStep` whose tool the runtime fires — the rerun is never a model's call — and `:FixForward` hands the agent's patch to the change gate.

### Why verification matters here

Two rules here overlapped on purpose: `rerunKnownFlakyOnce` (priority 50) requires `priorReruns` below 1, and `flakyStillFailingEscalates` (priority 60) requires `reproducesOnRerun` true, so both matched a known-flaky test failure recorded with `priorReruns = 0` and `reproducesOnRerun = true` — a state the CI adapter should never produce, and priority resolved it in favour of escalation if it did. The conflict check named that pair and its region, along with a second overlap nobody had intended: `deployFailuresRollBack` (priority 100) against the same escalation rule, because `flakyStillFailingEscalates` carried no stage condition at all. That is the check earning its place — one of the two overlaps was a design decision, the other was an oversight, and they are indistinguishable by eye. Both rules now carry explicit guards (`failingStage isNot :Deploy`, `reproducesOnRerun isNot true`) that state the resolution priority was making silently. Completeness reported the regions with no disposition, and they shared a shape: a failure nobody had classified. A build whose `knownFlaky` status was never recorded matched nothing, because `deterministicFailuresFixForward` needs `knownFlaky is false` and the rerun rule needs `true`; so did a flaky-listed lint or build failure, and a test already out of reruns. `:EscalateToOwner` is the default in all of them — letting `:FixForward` cover an unclassified red build would have the agent proposing patches against a failure whose cause is unknown. Non-vacuity flags a rule whose conclusion never differs from what a higher-priority rule already assigns.

### Status and honest edges

Runs today with structured input. Materializing the CI state as a versioned projection (run id, stage, flaky-list version) and committing "rerun" or "roll back" as a conditional effect is the coding-harness adapter in PRP-008.

## Merge and release gating

The [change gate](../governing-a-coding-agent.md) is the merge decision: 25 terms, eight rules, and a `RulesOnly` decision the bot cannot write. Release gating is the same shape with different facts — signed artifact, passing smoke suite, change-freeze window, on-call acknowledged — and the same guarantee: the release is approved by a rule you can read, not by whichever model was on duty.

What composes them is the fail-closed default. `FixForward` does not mean *merge*; it means the agent may propose a change, which then has to clear the gate on its own facts.
