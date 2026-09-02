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

## The nouns

Four nouns added by the bounded-projection pivot (2026-08-24) sit alongside the original
seven; host-integration ones name mechanism `PRPs/PRP-008-standards-host-integration/01-objective.md`
owns, not new `horis:` vocabulary minted here (`docs/framework/02-ontology.md`):

| Noun | What it is | Lives in |
|------|-----------|----------|
| **Ontology** | The authored artifact: classes, properties, constraints, rules, an optional decision workflow, tool contracts — one RDFS-plus overlay graph | `.ttl` files, versioned like code |
| **World** | The typed instance store: an **admitted projection** — facts as instances of ontology classes, bound to one decision package and hydration; all mutation passes the validation gate | Memory (snapshots serializable) |
| **Reasoner** | Deterministic inference over World: subsumption, forward/backward chaining, constraint checking — scoped to the current projection | `horismos-core` |
| **Workflow** | An optional decision microflow, declared as ontology data: steps, transitions, goals, preconditions, fallbacks — a host may invoke a fragment rather than surrender its whole longitudinal process | The same `.ttl` |
| **Agent** | The runtime that executes a Workflow against a World, calling Bridges and Tools where the ontology says to | `horismos-core` |
| **Bridge** | A model client behind a uniform contract (Anthropic, OpenAI, local, mock) with confidence and timeout semantics | `horismos-bridges` |
| **Trace** | The trajectory of a run: every step, typed | Emitted by every run |
| **System of Record** *(declared-future, PRP-008)* | The external authority Horismos overlays — a FHIR server, a repository/tool registry — which owns durable state and the enterprise workflow; Horismos never replaces it | Outside Horismos, reached only through an adapter |
| **Adapter** *(declared-future, PRP-008)* | The boundary code that hydrates a declared, versioned closure from a System of Record into an `ExternalSnapshot`, and conditionally commits a `ProposedEffectSet` back | A new crate outside `horismos-core` (e.g. `horismos-fhir`) |
| **Decision Package** *(declared-future, PRP-008)* | The versioned unit Horismos evaluates: an ontology overlay + a source-version vector + a declared input closure | Produced by an Adapter, consumed by World admission |
| **Projection / Evaluation Epoch** *(declared-future, PRP-008)* | The bound between one admitted World and the Decision Package/source-version vector it was hydrated for; a new external mutation invalidates the epoch until re-hydration | `docs/framework/03-world.md` |

Dependency direction is strict: `Trace ← Agent ← (Workflow, Bridge, Tools)`,
`Agent ← Reasoner ← World ← Ontology`. The Ontology depends on nothing; the symbolic
chain (`Ontology → World → Reasoner`) never depends on Bridge — that is the enforced
anti-wrapper boundary (see [Architecture](14-architecture.md)). Adapters sit *outside*
this chain entirely: they produce the input a World transaction admits and consume its
output as a proposed effect to commit — they never call into matching, chaining,
verification, or a transaction in progress (doc 03, doc 14).

## How this differs from an agent framework — and from adjacent standards

| | Conventional agent stack (LangGraph, agent SDKs) | Horismos |
|---|---|---|
| State | Prompt context + ad-hoc scratch | Typed instance graph, schema-validated |
| Control flow | LLM decides next action each turn | Ontology-declared workflow; deterministic transitions |
| Tool calls | LLM emits tool + args from text | Runtime fires tools when preconditions hold; args read from the graph |
| "Why did it do that?" | Read the transcript, guess | `world.explain(...)` returns a proof tree |
| Invalid states | Discovered in production | Unrepresentable (validation gate) or caught at load (verification) |
| Behavior change | Edit prompts, redeploy, pray | Edit ontology; re-verify; diffs in domain terms |
| Remove the LLM | Nothing works | Everything but NL input works |
| Relationship to your existing state | Replaces it, or bolts state onto its own scratch | Overlays it: gates selected high-stakes decisions/effects inside your system of record, doesn't replace your state or control flow |

Horismos is not "we replace your state and control flow" — it is "we prove and gate one
decision inside it." See `PRPs/PRP-000-vision-positioning.md` §"Comparison" for how this
overlay posture differs specifically from FHIR, CQL, CDS Hooks, PlanDefinition, and
coding-agent harnesses.

## What Horismos is not

- **Not a rules engine with an LLM feature.** The workflow/agent model and the
  perception contract are first-class, not bolted on.
- **Not an LLM orchestrator.** There is exactly one Bridge call per PerceptionStep —
  no chains, no self-reflection loops, no model-to-model negotiation.
- **Not OWL-DL reasoning.** Horismos takes RDFS plus named, tractable extensions
  ([02](02-ontology.md)); full description-logic inference is out of scope forever.
- **Not (yet) differentiable.** Honest Kautz placement is Type 2–3. The trace format
  is deliberately training-grade to fund the future bets (see PRP-000's research
  appendix), but nothing in v1 claims Type 5.

## Lifecycle of a deployment

```
author (SME + authoring agent, doc 13 — import/align/overlay-first)
  → verify (BDD checks + witnesses + assumption manifest, doc 10)   [design time]
  → [declared-future, PRP-008] hydrate (Adapter → ExternalSnapshot, doc 03)
  → load (Ontology + World, docs 02–03)
  → run  (Agent executes Workflow, docs 05–08)     [runtime, repeated]
  → [declared-future, PRP-008] commit (ProposedEffectSet → System of Record, conditional)
  → trace (audit / replay / training, doc 09)
  → revise (ontology diff, re-verify, redeploy)
```
