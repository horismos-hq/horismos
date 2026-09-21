# Health

Three scenes: VTE prophylaxis (the standards-anchored vertical slice that was built for PRP-008), the triage flagship you met in the quickstart, and prior authorization — which is precisely the kind of decision that needs an appealable proof, and precisely why it is not the first build.

## VTE prophylaxis

### The scene

A patient is admitted. Hospital-acquired venous thromboembolism is a leading preventable cause of in-hospital death, and the intervention is well understood: stratify risk, check contraindications, order prophylaxis. A randomized trial of a computerized alert for exactly this decision (Kucher et al., NEJM 2005; 1,255 vs. 1,251 patients) markedly reduced DVT and PE — the mechanism class works when it fires reliably. An LLM agent reading the chart can tell you the risk factors; what it cannot give you is a guarantee that it applied the protocol the same way for every patient, or a reason a pharmacist can audit.

Our scenario ranking puts this one first for Horismos: the inputs are almost entirely structured EHR facts, the logic is bounded (a published score plus a contraindication check), and the action is a typed order. That is the structured-input path — no model in the loop at all.

### The overlay

The Padua score arithmetic is not in the rules. Summing risk factors is the job of your CQL library or adapter (PRP-008's position is to compose with an evaluator, not rebuild one); Horismos consumes the typed score and decides what it implies:

```turtle
--8<-- "examples/gallery/vte-prophylaxis.ttl"
```

### The decision, with proof

```python
from horismos import HorismosValidationError, Ontology, World

onto = Ontology.load("examples/gallery/vte-prophylaxis.ttl")
world = World(onto)

def admit(subject: str, **facts: object) -> None:
    ref = world.assert_(":Admission", subject=subject, **facts)
    proof = world.explain(ref, ":prophylaxis")
    print(proof.render() if proof else f"{subject}: no recommendation — facts incomplete")

admit(":adm-1", padua_score=5, active_bleeding=False, platelet_count=210,
      on_therapeutic_anticoagulation=False)
admit(":adm-2", padua_score=6, active_bleeding=False, platelet_count=38,
      on_therapeutic_anticoagulation=False)
admit(":adm-3", padua_score=2, active_bleeding=False, platelet_count=250,
      on_therapeutic_anticoagulation=False)
```

```text
:prophylaxis = :Pharmacologic   via highRisk — "Padua score 4 or more with no bleeding contraindication — pharmacologic prophylaxis"
  ├─ onTherapeuticAnticoagulation isNot true   (closed world)
  ├─ paduaScore 5 atLeast 4   (asserted by the api)
  ├─ activeBleeding false is false   (asserted by the api)
  └─ plateletCount 210 atLeast 50   (asserted by the api)

:prophylaxis = :MechanicalOnly   via highRiskWithContraindication — "High VTE risk but pharmacologic prophylaxis is contraindicated — mechanical prophylaxis only"
  ├─ onTherapeuticAnticoagulation isNot true   (closed world)
  ├─ paduaScore 6 atLeast 4   (asserted by the api)
  └─ any of:
     ├─ activeBleeding is true   (not held)
     └─ plateletCount 38 below 50   (asserted by the api)

:prophylaxis = :NoneIndicated   via lowRisk — "Padua score below 4 — routine prophylaxis not indicated"
  ├─ onTherapeuticAnticoagulation isNot true   (closed world)
  └─ paduaScore 2 below 4   (asserted by the api)
```

The second proof is worth a second look: a disjunction renders both branches and says which one held. A pharmacist reading it knows the recommendation rests on the platelet count, not on bleeding.

A high-risk admission whose bleeding contraindication was never assessed gets `:InsufficientData` — the overlay declines to recommend and the proof names the observations that are missing; an out-of-range lab value never enters; and nothing but a rule can write the recommendation:

```python
admit(":adm-4", padua_score=7)                    # no bleeding or platelet facts yet
for bad in (dict(platelet_count=-1), dict(padua_score=5, prophylaxis=":Pharmacologic")):
    try:
        world.assert_(":Admission", subject=":adm-5", **bad)
    except HorismosValidationError as e:
        print(f"[{e.constraint}] {e}")
```

```text
:prophylaxis = :InsufficientData   via contraindicationNotAssessed — "High VTE risk but the bleeding contraindication was not assessed — no recommendation until it is"
  ├─ onTherapeuticAnticoagulation isNot true   (closed world)
  ├─ paduaScore 7 atLeast 4   (asserted by the api)
  ├─ activeBleeding isNot true   (closed world)
  ├─ none of:
  │  └─ plateletCount below 50   (not held)
  └─ any of:
     ├─ activeBleeding isEmpty   (closed world)
     └─ plateletCount isEmpty   (closed world)

[min] plateletCount -1 is below horis:min 0 (10^9/L) on :Admission
[provenance_refused] prophylaxis may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api
```

When the labs arrive, the recommendation follows — in the same transaction, with no stale intermediate state:

```python
admit(":adm-4", active_bleeding=False, platelet_count=180, on_therapeutic_anticoagulation=False)
```

```text
:prophylaxis = :Pharmacologic   via highRisk — "Padua score 4 or more with no bleeding contraindication — pharmacologic prophylaxis"
  ├─ onTherapeuticAnticoagulation isNot true   (closed world)
  ├─ paduaScore 7 atLeast 4   (asserted by the api)
  ├─ activeBleeding false is false   (asserted by the api)
  └─ plateletCount 180 atLeast 50   (asserted by the api)
```

### Why verification matters here

The completeness check asks: *for every reachable combination of Padua score, bleeding status and platelet count, does some rule assign a recommendation?* For this overlay the answer was no, in two places: an admission with no Padua score recorded at all, and a high-risk admission whose bleeding contraindication was never assessed — `:highRisk` needs `activeBleeding is false` **and** `plateletCount atLeast 50`, so an unrecorded bleeding status or platelet count matched neither it nor `:highRiskWithContraindication`. Closing that gap is the most instructive thing in this example, because none of the four original values was an honest answer to it. `:NoneIndicated` would tell a ward to withhold prevention on the strength of a score nobody took; `:MechanicalOnly` would order a treatment from missing data. So the overlay gained a fifth outcome, `:InsufficientData`, and two rules that assign it — and the proof tree names the observation that is missing rather than a recommendation nobody can defend. A verification check that forces that conversation before deployment is doing more than proving a theorem. Conflict-freedom confirmed that `highRisk` and `highRiskWithContraindication` cannot both fire on one admission, and named three overlaps with `alreadyAnticoagulated` (priority 100) that priority already resolved; those three rules now say `onTherapeuticAnticoagulation isNot true` explicitly. Dead-rule detection catches a threshold that `horis:min`/`horis:max` make unreachable. The assumption manifest states which SNOMED edition and which FHIR profile the closure assumed.

What it would not do: say anything about whether the Padua thresholds are clinically right, whether the upstream score computation is correct, or whether the EHR record was complete. Those live outside the declared closure, and the manifest says so.

### Status and honest edges

- The overlay runs today with structured input. Hydrating it from FHIR (Observation, Condition, MedicationStatement) with source versions in the proof, and committing a proposed order back conditionally, was PRP-008's vertical slice, and VTE is the scenario it was built for: `examples/vte` runs that slice over a recorded bundle.
- Thresholds here are illustrative. Padua ≥ 4 is the published high-risk cutoff; the contraindication values are teaching numbers.
- No head-to-head clinical trial of a hard-gated ontology system exists in the literature we surveyed. Horismos's safety argument is architectural, not yet measured for itself.

## Triage (the flagship)

The quickstart runs it end to end. The point it makes for health specifically: a rule written against `:ChestPain` fires for a SNOMED-coded unstable angina it never mentioned, and the proof shows the subsumption step. See `examples/triage/triage.ttl` and the [Quickstart](../quickstart.md).

## Prior authorization — later, on purpose

Prior authorization has the strongest FHIR prior art of anything surveyed (Da Vinci CRD/DTR/PAS is literally the use case), CMS-0057-F mandates FHIR PA APIs, and Epic put hook-triggered third-party coverage-requirements services into production in August 2026. The shape fits Horismos exactly: structured inputs, bounded rules, a typed determination, and a proof tree that makes every denial appealable.

We nonetheless rule it out as a *first* build: the documented harms — a litigated "known 90% error rate" in one Medicare Advantage tool, 300,000 claims batch-denied at about 1.2 seconds each in another — came from opaque automation in an adversarial payer workflow, and a pre-alpha framework has no business there until it has a trust record. The [Insurance](insurance.md) page shows the adjudication shape with a proof per denial; the operating decision about when to deploy it is a human one.
