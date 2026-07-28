# PRP-000 — Vision & Positioning

**Status**: Draft · standing document
**Depends on**: —

## Intent

Fix what Sigil claims, to whom, and — as important — what it deliberately does not claim,
so every later PRP inherits a stable story.

## The pitch

Sigil is an ontology-first neuro-symbolic agent framework. A subject-matter expert authors
one artifact — an RDFS-plus ontology declaring the domain, its rules, *and* its workflows —
and the runtime executes it. LLMs exist only at two declared transducer boundaries
(perception: NL → typed instances; narration: proof → NL). Every conclusion carries a proof
tree; every rule set is formally verified before it runs. Delete the LLM and the framework
still runs.

**Tagline candidates** (settle here): "The symbolic spine for LLM agents" ·
"Agents that live inside an ontology" · "Neural proposes. Symbolic decides."

## Personas

1. **Researchers** building neuro-symbolic systems who today glue Scallop/Prolog/ad-hoc
   rules to LLM calls by hand.
2. **Agent builders** in regulated or correctness-critical domains (health, finance,
   compliance) who cannot ship "the model felt confident."
3. **SMEs** (clinicians, adjusters, compliance officers) — not direct users of the API,
   but the authors of the ontology via the authoring agent (PRP-007).

## Honest placement

Kautz **Type 2–3**: deterministic symbolic orchestrator, neural perception at the edges.
We say so plainly. The comparison set and one-line differentiation:

- **Scallop** — differentiable Datalog (Type 5), research-oriented; no agent loop, no
  workflow model, no LLM boundary contracts. Sigil is the productionized Type 2–3 layer.
- **Apollo-1 (AUI)** — closed, typed-state commercial agent runtime; validates the market.
  Sigil is open, ontology-standard (RDFS-plus), and verifiable.
- **LangGraph / agent SDKs** — orchestration where the LLM decides; no typed world, no
  proofs, no verification. Sigil inverts control.
- **GoRules/Drools** — rules engines without an agent model, ontology, or neural boundary.

## Research appendix (not a phase)

The differentiable extension (see plan artifact, Vision tab): trace fine-tuning
(TIR/CIR-style SFT+GRPO on Sigil trajectories — Type 2 cloned into Type 3 weights),
Scallop-style provenance semirings over the Datalog-shaped rule subset (genuine Type 5),
and GNN guidance over the instance graph (open question). Obligation on v1: traces are
training-corpus-grade from day one (PRP-002).

## Open decisions

- **Proving ground**: τ-Bench airline (public scoreboard, direct Apollo-1 comparison)
  vs. clinical triage (docs' running example, SNOMED interop showcase). Recommendation
  on record: τ-Bench for the benchmark number; triage for the docs narrative.
- **Naming**: "Sigil" for the framework (recommended) vs. reserving it for a future DSL.
- **License**: research-friendly permissive (MIT/Apache-2.0 dual, Rust convention) —
  confirm.

## Definition of done

A `README.md` + docs-site landing page that states the pitch, personas, comparison table,
and honest taxonomy — reviewed and stable enough that PRP-001–007 never re-litigate it.
