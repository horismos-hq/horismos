# 01 · Concepts

## The two principles

1. **Neural proposes, symbolic decides.** LLMs perceive language and propose typed
   content; every proposal passes a deterministic gate before it affects anything.
   No decision — no rule firing, no workflow transition, no tool call — is ever made
   by a model.
2. **Traces are output, not logging.** Every run produces a typed, replayable
   trajectory: what was perceived (and with what confidence), what fired (and why,
   as a proof tree), what executed (and under which satisfied preconditions). The
   audit log and the training corpus are the same object.

## The seven nouns

| Noun | What it is | Lives in |
|------|-----------|----------|
| **Ontology** | The authored artifact: classes, properties, constraints, rules, workflows, tool contracts — one RDFS-plus graph | `.ttl` files, versioned like code |
| **World** | The typed instance store: facts as instances of ontology classes; all mutation passes the validation gate | Memory (snapshots serializable) |
| **Reasoner** | Deterministic inference over World: subsumption, forward/backward chaining, constraint checking | `sigil-core` |
| **Workflow** | The agent loop, declared as ontology data: steps, transitions, goals, preconditions, fallbacks | The same `.ttl` |
| **Agent** | The runtime that executes a Workflow against a World, calling Bridges and Tools where the ontology says to | `sigil-core` |
| **Bridge** | A model client behind a uniform contract (Anthropic, OpenAI, local, mock) with confidence and timeout semantics | `sigil-bridges` |
| **Trace** | The trajectory of a run: every step, typed | Emitted by every run |

Dependency direction is strict: `Trace ← Agent ← (Workflow, Bridge, Tools)`,
`Agent ← Reasoner ← World ← Ontology`. The Ontology depends on nothing; the symbolic
chain (`Ontology → World → Reasoner`) never depends on Bridge — that is the enforced
anti-wrapper boundary (see [Architecture](14-architecture.md)).

## How this differs from an agent framework

| | Conventional agent stack (LangGraph, agent SDKs) | Sigil |
|---|---|---|
| State | Prompt context + ad-hoc scratch | Typed instance graph, schema-validated |
| Control flow | LLM decides next action each turn | Ontology-declared workflow; deterministic transitions |
| Tool calls | LLM emits tool + args from text | Runtime fires tools when preconditions hold; args read from the graph |
| "Why did it do that?" | Read the transcript, guess | `world.explain(...)` returns a proof tree |
| Invalid states | Discovered in production | Unrepresentable (validation gate) or caught at load (verification) |
| Behavior change | Edit prompts, redeploy, pray | Edit ontology; re-verify; diffs in domain terms |
| Remove the LLM | Nothing works | Everything but NL input works |

## What Sigil is not

- **Not a rules engine with an LLM feature.** The workflow/agent model and the
  perception contract are first-class, not bolted on.
- **Not an LLM orchestrator.** There is exactly one Bridge call per PerceptionStep —
  no chains, no self-reflection loops, no model-to-model negotiation.
- **Not OWL-DL reasoning.** Sigil takes RDFS plus named, tractable extensions
  ([02](02-ontology.md)); full description-logic inference is out of scope forever.
- **Not (yet) differentiable.** Honest Kautz placement is Type 2–3. The trace format
  is deliberately training-grade to fund the future bets (see PRP-000's research
  appendix), but nothing in v1 claims Type 5.

## Lifecycle of a deployment

```
author (SME + authoring agent, doc 13)
  → verify (BDD checks + witnesses, doc 10)        [design time]
  → load (Ontology + World, docs 02–03)
  → run  (Agent executes Workflow, docs 05–08)     [runtime, repeated]
  → trace (audit / replay / training, doc 09)
  → revise (ontology diff, re-verify, redeploy)
```
