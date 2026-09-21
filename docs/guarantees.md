# The guarantees, honestly

This page is the trust-builder, so it is written the way a reviewer would want it: what is claimed, in what scope, what `verify()` proves, and what is explicitly **not** claimed. Sources: [Verification](framework/10-verification.md), [Security & Trust](framework/15-security.md), the project's positioning decisions on snapshot-scoped guarantees, and the standing assessment of the design against the field from the weekly research sweep (verdict of 2026-09-02: *aligned-with-adjustments*; summarised in § "Why we think the shape is right" below).

## The claim, verbatim

> Given a verified decision package, a declared input closure, a source-version vector, and an admitted input bundle, Horismos deterministically produces a fixed-point decision snapshot, proof, trace, and proposed effect set. A commit succeeds only if the external resources still match that source-version vector. Mutations outside Horismos are outside the guarantee and invalidate the projection until it is refreshed.

And the one-sentence positioning every design document inherits:

> Horismos is a deterministic, ontology-first decision and capability kernel that evaluates a versioned projection of an existing system, proves why an effect is allowed, and conditionally commits through standards-native adapters.

## What stays true unconditionally — inside one evaluated projection

These hold today, for the world a transaction was given, with no qualifier:

| Property | What it means | Where it is enforced |
|---|---|---|
| **The gate has no back door** | Every mutation — perception, rule, tool, API — passes the same five checks (class, domain/range, constraints, cardinality, provenance policy) and is rejected atomically on the first failure | `horismos-core::world`, property-tested; the [Quickstart](quickstart.md) shows the refusals |
| **Truth maintenance re-chains to a fixed point** | Every external transaction ends with the world at the forward-chained fixed point of its final base facts; a derived fact cannot outlive its support, through any depth of chained derivation | [The World](framework/03-world.md) § Truth maintenance, which carries the decision record |
| **Determinism over materialized inputs** | No wall-clock, randomness, or iteration-order dependence anywhere in inference; `horis:now` is transaction-stamped; the same (ontology, snapshot, input) yields the same (facts, proofs, trace) | [Reasoning](framework/04-reasoning.md) § Determinism; trace equality is a test assertion in CI |
| **Proofs and traces are exact for the recorded snapshot** | Every conclusion carries its derivation; every leaf carries its provenance (and confidence, for perception) | [Reasoning](framework/04-reasoning.md), [Traces](framework/09-traces.md) |

Determinism is cheap to check yourself. Two independently built worlds produce byte-identical snapshots and byte-identical proofs:

```python
from horismos import Ontology, World

onto = Ontology.load("examples/triage/triage.ttl",
                     imports=["examples/triage/snomed-ct-subset.ttl"])

def run() -> tuple[bytes, str]:
    world = World(onto)
    p = world.assert_(":Patient", subject=":p7",
                      heart_rate=128, has_symptom=[":CrushingChestPain"])
    return world.snapshot().to_bytes(), world.explain(p, ":acuity").render()

assert run() == run()
print("two independent worlds, identical bytes")
```

```text
two independent worlds, identical bytes
```

## What is explicitly scoped

The pre-pivot framing implied a global, always-current guarantee. The adopted design says plainly what is scoped:

- **Freshness** — only as current as the last hydration plus its version check. A projection is valid *as of* its source-version vector; a write made directly against the system of record by another client is invisible to Horismos until re-hydration, and a proposed commit against drifted source fails closed.
- **Completeness** — bounded by a declared input closure, never implicit. The pragmatic closed-world reading ("property unassigned → condition false") is correct for a property the closure declared in scope and genuinely absent, and a category error for one that was simply never fetched. Keeping *absent* and *not-fetched* distinct is the adapter's declared job (PRP-008); our survey of the field found nothing else that treats that distinction as a verification concern.
- **Atomicity** — local transactions are atomic; cross-system commits are conditional, not distributed-atomic. The trace records "decision produced", "commit attempted", "commit accepted", and "commit failed/conflicted" as separate, separately observable states.

## What `verify()` proves

`Ontology.verify()` ports the BDD verifier proven in the sigil-lang project and adapts it to ontology semantics (constraints tighten the variable domains; subsumption widens rule applicability). It runs at load, in the authoring loop, and in CI, and every finding is a **witness** — a domain-term sentence plus a machine-readable region.

| Check | The question it answers | Example witness |
|---|---|---|
| **Completeness** | Does some rule assign the decision property for every reachable input region? | "No acuity assigned when heartRate ∈ (100, 120] and no Symptom present" |
| **Conflict-freedom** | Can two rules derive different values for a single-cardinality property on an overlapping region? | "r1 → :Immediate but r4 → :Urgent when heartRate ∈ (120, 140] ∧ ChestPain — r1 wins by priority; confirm intended" |
| **Dead rules** | Is any rule's condition unsatisfiable given constraints and higher-priority shadowing? | "r7 can never fire: requires heartRate > 350, but horis:max is 300" |
| **Termination** | Can chaining cycle? | The cycle, listed rule by rule |
| **Workflow totality** | Is every step reachable, does every path terminate, does every fallible step declare its failure edge, can every `horis:requires` ever hold? | Composed with the rule-set BDDs: a workflow is proven never to stall only because its rules are proven to assign what its steps require |
| **Non-vacuity** | Is a rule satisfiable but inert (its conclusion never differs from what a higher-priority rule already assigns)? Does a constraint exclude any reachable value? | Same witness shape, first-class in `report.witnesses` |

Alongside `report.ok`, every report carries an **assumption manifest**: the input closure it treated as complete, the external profile and terminology versions it assumed, the mapping version, the supported subset of any imported artifact, and the tool or adapter effects it checked only for declared-contract compliance. `report.ok` must never be read as "the domain is correct" or "the external data is complete", and the manifest exists so nobody has to infer that from silence.

### What `verify()` reports

```python
report = onto.verify()
print(report.ok, report.complete, report.conflicts, report.dead_rules, report.terminating)
print([(w.severity, w.domain_text) for w in report.witnesses])
```

```text
True {':acuity': True} () () True
[('info', 'hasSymptom horis:cardinality "0..*" excludes no value count on :Patient')]
```

`ok=True` here is a proof obligation discharged, not an absence of checking: `:acuity`
is assigned for every input in the declared closure, no two rules derive different
values for it on an overlapping region, no rule is unreachable, and chaining
terminates. The one `info` witness is advisory — an unbounded cardinality that
excludes nothing — and does not fail the report.

`ok=False` is not advisory. `Ontology.load` refuses an ontology with any
error-severity witness, so a rule set with a completeness gap cannot be loaded and
then quietly run; `allow_incomplete=True` is the only way past it, and it is stamped
into every trace the resulting ontology produces. Every worked example on this site
passes the check — which is why several of them changed shape when it landed. The
[gallery](use-cases/index.md) says, per overlay, what the witnesses reported and what
closing them required.

## What is not claimed

Permanent design positions, not gaps waiting to be filled:

- **Nothing about LLM behaviour.** Perception sits behind the gate and a mandatory fallback edge; verification proves the symbolic system is total and sound around whatever perception does. Prompt-level injection immunity is not promised — the mitigation is architectural containment: an attacker's best case is a wrong-but-schema-valid fact, bounded by the confidence threshold, never an arbitrary action.
- **Nothing about tool or host implementation behaviour** beyond declared-contract compliance. Contract narrowing constrains what an `ActionStep` can ask for and what result can land as a fact; it cannot prove a bound implementation has no side effect outside its declared effect.
- **No semantic-correctness theorem for generated code.** The candidate-artifact boundary (PRP-008) proves a proposal was policy-legal and passed its declared checks — never that the code is right.
- **Nothing probabilistic.** Properties are proven over the discretized symbolic space exactly, or reported with a witness. There are no "95% verified" states. Confidence gates acceptance; it never weights inference.
- **Nothing about external record completeness or freshness**, terminology-server correctness, or whether a formal rule set means what its author intended — the last is a structural ceiling of formalization itself, which our survey puts in numbers: NL-to-formal translation reaching 24–35% semantic correctness in one survey, and an 11.2% semantic-alignment pass rate against a 61.8% compile rate in another.
- **Not a security boundary for processes.** Sandboxing tool execution, MCP servers, and the authoring agent is the deployer's; encryption-at-rest and access control for the default trace store are pluggable, not built in. The full v1 threat model, boundary by boundary, is [Security & Trust](framework/15-security.md).
- **Not yet measured for itself.** No head-to-head clinical trial of a hard-gated ontology system exists in the surveyed literature. The evidence on the landing page is for the mechanism class; Horismos's own benchmark number (τ-bench airline) does not exist yet.

## Why we think the shape is right

Our standing research sweep re-assesses the design against new literature every week. Its current verdict is that every load-bearing commitment above — snapshot-scoped guarantee, fail-closed conditional commit, thin overlay on existing standards, gate as proof-emitter, symbolic decides — now has independent confirmation from at least two of the database, formal-methods, agent-safety, and HL7 standards communities, and that where the field moved differently it moved toward the neural side of the exact line Horismos draws, with the measured evidence in those same papers favouring this side. It also lists the adjustments it recommends, and they are on the [roadmap](roadmap.md) rather than hidden: typing proposed effects by reversibility, measuring the utility tax of tight closures, and a deterministic conclusion monitor at the narration boundary. This section is re-stated with each sweep; the date above is the verdict it currently reflects.
