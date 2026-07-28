# 04 · Reasoning

The Reasoner is deterministic inference over the World. Same facts, same ontology →
same conclusions, always, with the derivation recorded.

## Subsumption

Class-hierarchy reasoning is always on. `sigil:is :ChestPain` matches instances of
`:ChestPain` **and every subclass**, including subclasses that arrived via imports:

```
:CrushingChestPain ⊑ :ChestPain ⊑ :Symptom        (local + SNOMED-imported axioms)
```

This is the quiet superpower of ontology-first: rule `r1` fires for crushing chest
pain although no rule mentions it. Property inference is limited to `rdfs:domain` /
`rdfs:range` typing of instances. There is no complex-class inference (no OWL).

## Forward chaining

Runs inside every World transaction, to a fixed point:

```
loop (max N iterations, default 100):
    candidates = rules whose conditions hold in the current graph
                 minus rules already fired on the same bindings
    if empty: fixed point reached → done
    fire highest-priority candidate (ties: documented deterministic order —
        rule IRI lexicographic; no reliance on load order)
    assert its conclusions through the gate (provenance: rule)
```

Semantics (inherited from sigil-lang, re-stated for the graph world):

- **No re-fire**: a (rule, bindings) pair fires at most once per transaction.
- **Priority** orders candidate selection, not rule truth — all applicable rules
  eventually fire unless a conflict policy stops them.
- **Conflict policy**: two rules concluding *different values for the same
  single-cardinality property* on the same subject is a **conflict**. Runtime policy:
  higher priority wins, the losing conclusion is recorded in the trace as suppressed.
  Statically, verification flags these pairs at load time (doc 10) so runtime
  suppression is a last resort, not a design pattern.
- **Iteration cap** exists as a tripwire; termination is separately *verified* (doc 10).

### Condition evaluation

The 11 operators evaluate over the graph with these edge semantics:

| Situation | Result |
|-----------|--------|
| Property unassigned, any comparison (`exceeds`, `is`, …) | `false` |
| Property unassigned, `sigil:isNot` | `true` |
| Property unassigned, `sigil:isAssigned` | `false` |
| Multi-valued property (`hasSymptom`), `sigil:is X` | `true` if **any** value matches (∃) |
| Unit mismatch in comparison | Load-time verification error, not a runtime surprise |

This is a pragmatic closed-world reading of an open-world data model. It is a
deliberate, documented divergence from RDF semantics — Sigil worlds are operational
state, not the open web. **[DECIDE: confirm ∃ vs. ∀ for multi-valued `isNot` — PRP-002]**

## Backward chaining

Goal-directed proving, exposed two ways:

```python
proof = world.prove(":acuity :Immediate", subject=p)
#  → ProofNode | None.  Drives rule evaluation backwards from the goal:
#    finds rules concluding the goal, recursively proves their conditions
#    (from facts, or from other rules). Cycle detection; depth cap 50.

tree = world.explain(p, "acuity")
#  → the recorded derivation of an EXISTING fact (from the forward-chaining trace).
```

`prove` answers "could this be established?"; `explain` answers "why is this
established?". Both return the same `ProofNode` structure:

```
ProofNode
├── goal:        (:patient-7 :acuity :Immediate)
├── via:         :r1                       # or "asserted" (base fact) or None (unprovable)
├── explain:     "Tachycardia with chest pain — immediate track"
├── conditions:  [ProofNode(128 exceeds 120, via: asserted),
│                 ProofNode(hasSymptom ⊒ ChestPain, via: subsumption(:CrushingChestPain))]
└── bindings:    {…}
```

Note `via: subsumption(...)` — hierarchy steps are explicit proof nodes, so "why did
this fire for *crushing* chest pain?" has a visible answer.

## Rendering proofs

`proof.render()` produces the human form used in traces, narration (doc 06), and the
authoring loop (doc 13):

```
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 128 exceeds 120            (asserted by perception, conf 0.94)
  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (SNOMED subsumption)
```

Every leaf carries provenance — down to the perception confidence of the fact it
rests on. An auditor can see that a symbolic conclusion ultimately rests on a neural
extraction, and exactly how confident that extraction was. That is the neuro-symbolic
seam, made visible.

## Determinism guarantees

- No wall-clock, no randomness, no iteration-order dependence anywhere in inference.
- `sigil:now` (used by tool effects) is transaction-stamped: injected by the runtime
  at transaction open, constant within it, recorded in the trace — replay reproduces it.
- The same (ontology, world snapshot, input transaction) triple always yields the
  same (derived facts, proof trees, trace) — this is tested property-style in CI.
