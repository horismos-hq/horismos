# Horismos PRPs — Phase Plans

Horismos is an **ontology-first neuro-symbolic agent framework**: Rust core, Python bindings.
The ontology (RDFS-plus) is simultaneously the typed world-state, the reasoning substrate,
and the agent loop/workflow definition. LLMs are transducers at declared boundaries; the
symbolic runtime decides. See `../docs/framework/` for the full framework definition and
`../planning/horismos-plan.html` for the living plan (published as a Claude artifact).

## Reading order

| PRP | Phase | One-liner |
|-----|-------|-----------|
| [PRP-000](PRP-000-vision-positioning.md) | Vision & positioning | What we claim, to whom, and what we deliberately do not claim |
| [Foundational skeleton](foundational-skeleton.md) | Infrastructure (standing) | Repo layout, build tooling, AGENTS.md, Claude skills — the substrate every phase assumes |
| [PRP-001](PRP-001-ontology-core/01-objective.md) | Ontology core | RDFS-plus schema model, typed instance store, validated mutations (Rust) |
| [PRP-002](PRP-002-reasoning-substrate/01-objective.md) | Reasoning substrate | Chaining, subsumption, constraints, proof trees, training-ready traces |
| [PRP-003](PRP-003-python-bindings/01-objective.md) | Python bindings & API | PyO3 wheels; the Pythonic front door |
| [PRP-004](PRP-004-agent-loop/01-objective.md) | Ontology-first agent loop | Workflow execution, perception/narration boundaries, tools, Bridge |
| [PRP-005](PRP-005-guarantees-verification.md) | Guarantees & verification | BDD checks with witnesses, surfaced in both APIs |
| [PRP-006](PRP-006-packaging-distribution.md) | Packaging & distribution | crates.io, PyPI, CI, docs site |
| [PRP-007](PRP-007-authoring-agent.md) | Ontology authoring agent | SMEs converse; Claude drafts Turtle; the verifier gatekeeps |

## Conventions

Full rules in [CONVENTIONS.md](CONVENTIONS.md) (lifecycle, folder form for active
PRPs, tracks). At-a-glance state: [BOARD.md](BOARD.md).

- The v1 security/trust workstream (captain-authorized 2026-08-18, per independent-
  review findings R2/D4) is cross-cutting, not a new PRP number:
  [`docs/framework/15-security.md`](../docs/framework/15-security.md) is the v1
  threat model; PRP-004, PRP-006, and PRP-007 each carry a **Security acceptance
  surface** subsection stating what their DoD must prove, plus owned **[DECIDE]**
  markers for the genuinely open mechanism choices.

- PRPs are **very high level** — intent, scope, non-goals, key decisions, definition
  of done. Far-future Drafts are one page each; from PRP-002 onward the next-up/Active
  PRP is teaching-grade (background, worked triage examples, Python-parallel notes,
  glossary — see CONVENTIONS.md). Sprint-level detail is produced when a PRP is
  activated (the PRP converts to a folder: `01-objective.md` + `02-tasks-and-gaps.md`).
- Numbering is sequence. Each PRP assumes the previous ones are done (PRP-000 is a
  standing document, revised as positioning sharpens).
- The running example across all PRPs is **clinical triage** (`docs/framework/` uses it
  throughout). The proving-ground benchmark decision (τ-Bench airline vs. triage-first)
  is recorded in PRP-000 as an open decision.
- Prior art: `../../sigil-lang` is the proving ground this framework inherits from
  (Rete-lite matcher, BDD verifier, constraint solver, trace design, neural bridge,
  NL-to-DSL authoring loop — all built and tested there). PRPs cite what they port.
