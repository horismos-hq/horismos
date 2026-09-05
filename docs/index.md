---
title: Horismos
hide:
  - navigation
  - toc
---

<div class="horis-hero" markdown>

# Neural proposes. Symbolic decides.

<p class="horis-tagline">Horismos is an ontology-first decision kernel that sits <em>around</em> the agent you already run. Your rules make the call. The model translates language into typed facts. Every conclusion ships with a proof — and the kernel still runs with the model deleted.</p>

[Quickstart — 10 minutes, zero API keys](quickstart.md){ .md-button .md-button--primary }
[Coming from LangGraph](coming-from-langgraph.md){ .md-button }
[Governing a coding agent](governing-a-coding-agent.md){ .md-button }

<div class="horis-chips" markdown="span">
<span>Rust core · Python bindings</span>
<span>Standard Turtle, RDFS-plus</span>
<span>MIT OR Apache-2.0</span>
<span>Pre-alpha — see the [roadmap](roadmap.md)</span>
</div>

</div>

## The whole idea in ten lines

You have shipped an agent. Somewhere in a prompt it says *"patients with a heart rate over 120 and chest pain are immediate acuity"*, and somewhere in production the model skipped that line. In Horismos the sentence is a rule in a small ontology file, the model can only propose typed facts, and the rule — not the model — assigns the decision:

```python
from horismos import Ontology, World, HorismosValidationError

onto = Ontology.load("examples/triage/triage.ttl",
                     imports=["examples/triage/snomed-ct-subset.ttl"])
world = World(onto)

p = world.assert_(":Patient", subject=":p7",
                  heart_rate=128, has_symptom=[":CrushingChestPain"])

print(world.value(p, ":acuity"))              # derived by rule r1 inside the same transaction
print(world.explain(p, ":acuity").render())   # the proof, not a transcript
```

```text
Ref(:Immediate)
:acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"
  ├─ heartRate 128 exceeds 120   (asserted by the api)
  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)
```

No rule mentions *crushing* chest pain. The imported SNOMED subset says it is a kind of chest pain, and the proof shows that step. Now try to set the decision directly — the way a model, a tool, or a careless caller would:

```python
try:
    world.assert_(":Patient", subject=":p8", acuity=":Immediate")
except HorismosValidationError as e:
    print(e)
```

```text
acuity may only be assigned by rules (horis:assignedBy horis:RulesOnly); assertion came from the api
```

That refusal is the product. It applies to every path into the world — perception, rule, tool, API — with no back door. The snippets above run verbatim against this repository; the [quickstart](quickstart.md) walks through them with nothing to configure.

## What breaks in prompt-governed agents

These are the failure classes working agent teams keep meeting. The numbers are from the research wiki in this repo (`research/`), where each is tagged `[measured]` with its source.

| Failure class | What was measured | What Horismos does instead |
|---|---|---|
| **Silent wrong-state failures** — the run "succeeds", the state is wrong, no tool errored | In τ²-bench airline, 78% of observed failures were silent wrong-state failures with no tool error; deterministic read-only pre-execution gates raised task success from 29.6% to 42.0% ([arXiv:2607.07405](https://arxiv.org/abs/2607.07405)) | Every write passes a five-check gate in domain terms; rule-derived facts are recomputed, never left stale ([The World](framework/03-world.md)) |
| **Stale approvals** — correct when checked, stale when acted on | Approval verdicts changed in 5.3–48.4% of cases after an eight-step replay shift ([arXiv:2608.26306](https://arxiv.org/abs/2608.26306)); models produced stale-aligned decisions 74.7–77.3% of the time, while version-stamp checks were 100% deterministic ([arXiv:2608.25553](https://arxiv.org/abs/2608.25553), [arXiv:2609.00243](https://arxiv.org/abs/2609.00243)) | Decisions are scoped to a versioned snapshot; a commit against drifted source state fails closed ([Guarantees](guarantees.md)) |
| **Prompt injection flipping the conclusion** | Under injection, five open models inverted a verified conclusion in 61% of trials; a hardened prompt cut that to 14%; only enforcement drove it to zero ([arXiv:2606.19588](https://arxiv.org/abs/2606.19588)). Six LLMs repeated a single planted fake lab value in 50–83% of clinical vignettes ([Communications Medicine, 2025](https://www.nature.com/articles/s43856-025-01021-3)) | Perception can only propose schema-forced facts; decision properties are unrepresentable in its output; narration renders only from the proof ([LLM Boundaries](framework/06-llm-boundaries.md)) |
| **Policies that cannot see state** | Codex rules "evaluate command prefixes only — they cannot reference repository state"; after PocketOS lost production data to a blanket-scope token, the post-mortem read "guardrails must be implemented at the API gateway and token-permission level, not in advisory text" (`research/neurosymbolic-landscape/coding-agent-harnesses.md`) | Preconditions are declared over typed world state and checked by the runtime before any tool fires ([Tools](framework/07-tools.md)) |

## What you get

- **A typed world, not a scratchpad.** Facts are instances of ontology classes; ranges, units, cardinality, closed enumerations and provenance policy are enforced at the gate. Rejections are sentences an auditor can read: `heartRate 1280 exceeds horis:max 300 (bpm) on :Patient`.
- **Rules that decide, with proofs.** Deterministic forward and backward chaining, subsumption over your class hierarchy and imported vocabularies, a proof tree per conclusion, and truth maintenance that re-derives when base facts change.
- **LLMs at two declared boundaries only.** Perception (text → typed instances, schema-forced, confidence-gated, with a mandatory fallback edge) and narration (proof → prose, never written back). Structured input skips the model entirely.
- **Tools the runtime fires.** Declared contracts: typed arguments read from the graph, preconditions checked, effects validated through the same gate. The model never picks a tool or builds its arguments.
- **Traces as the product.** Every run emits a typed, replayable trajectory — the audit log and the training corpus are the same object, with loss masks precomputed.
- **Verification before deployment.** Completeness, conflict-freedom, dead rules, termination and workflow totality, with counterexamples rendered in domain terms. `Ontology.load` refuses an ontology that fails them (see [Guarantees](guarantees.md) for exactly what is claimed).

## It surrounds your stack

Horismos does not replace LangGraph, your agent SDK, your coding harness, or your system of record. It gates one decision inside them.

<div class="horis-flow" markdown>

```text
 your text, or your structured system-of-record data
        │
        │  perception: ONE schema-forced model call, confidence-gated, typed output
        │  (structured input takes the same door with no model at all)
        ▼
 ┌────────────────────────────┐
 │  five-check validation gate │  class · domain/range · constraints · cardinality · provenance
 └─────────────┬──────────────┘   rejects atomically, in domain terms
               ▼
 ┌────────────────────────────┐
 │  typed world  ⇄  rules      │  forward chaining to a fixed point, proof per conclusion,
 └─────────────┬──────────────┘   derived facts recomputed on every base-fact change
               ▼
   preconditions hold ─► the runtime fires YOUR tool; its effects re-enter the gate
               ▼
 ┌────────────────────────────┐
 │  proof + trace              │  narration renders the proof for humans; the trace is
 └────────────────────────────┘   replayable, auditable, training-grade
```

</div>

See [Architecture](architecture.md) for the bounded projection kernel in one diagram, and [Coming from LangGraph](coming-from-langgraph.md) for the honest split of who does what.

## What Horismos does not claim

Stating the edges plainly is part of the offer.

- **It does not verify the LLM.** Verification proves the symbolic system is total and sound around whatever perception does; the mitigation for a bad extraction is the gate, the confidence threshold and the fallback edge — not a promise about the model.
- **It does not prove generated code is correct.** For coding agents, the planned candidate-artifact boundary proves a proposal was policy-legal and passed its declared checks. Semantic correctness stays a review and test claim.
- **Its guarantees are snapshot-scoped.** Inside one evaluated projection the gate has no back door and determinism holds. Freshness and completeness are only as good as the last hydration and the declared input closure.
- **It is not a system of record, an orchestrator, or OWL-DL.** It overlays your records, runs one bounded decision, and takes RDFS plus a small executable vocabulary — full description-logic inference is out of scope forever.
- **It is pre-alpha.** The symbolic core, the Python bindings, the Rust agent loop, verification (PRP-005) and standards/host integration (PRP-008) are all merged; packaging and distribution (PRP-006) is not. The [roadmap](roadmap.md) says exactly what runs today.

## Where next

<div class="grid cards" markdown>

- **[Quickstart](quickstart.md)** — `make develop`, load the triage ontology, watch the gate reject bad data, print a proof, run the full agent loop on the mock bridge, then go live with one environment variable.
- **[Go live in 2 minutes](go-live.md)** — the same demo against a real model through OpenRouter: one environment variable, a real captured trace, and a decision and proof identical to the mock run.
- **[Coming from LangGraph](coming-from-langgraph.md)** — the same decision written both ways, and the ten lines that let a graph node call the kernel.
- **[Governing a coding agent](governing-a-coding-agent.md)** — a 28-term change gate where the bot cannot grant itself merge permission.
- **[Use cases](use-cases/index.md)** — health, coding, insurance, physical AI, customer operations: one template, five runnable overlays.
- **[The guarantees, honestly](guarantees.md)** — what `verify()` proves, what is explicitly not claimed.
- **[Framework reference](framework/00-index.md)** — the shipped-voice definition, docs 00–15.

</div>
