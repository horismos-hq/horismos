# Horismos PRPs — Phase Plans

Horismos is an **ontology-first neuro-symbolic decision and capability kernel**: Rust core,
Python bindings. A small RDFS-plus overlay declares the typed decision/capability fragment;
the runtime evaluates it as a verified, versioned projection over an existing system of
record. LLMs are transducers at declared boundaries; the symbolic runtime decides. See
`../docs/framework/` for the full framework definition, `PRP-000-vision-positioning.md` for
the bounded-projection-kernel positioning (pivot adopted 2026-08-24), and
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
| [PRP-005](PRP-005-guarantees-verification/01-objective.md) | Guarantees & verification | BDD checks with witnesses, an assumption manifest, and non-vacuity, surfaced in both APIs |
| [PRP-008](PRP-008-standards-host-integration/01-objective.md) | Standards & host integration | FHIR hydration/commit, terminology provenance, CDS-Hooks/PlanDefinition/CQL composition, the candidate-artifact boundary, the generic host SPI |
| [PRP-006](PRP-006-packaging-distribution/01-objective.md) | Packaging & distribution | crates.io, PyPI, CI, docs site — activated 2026-09-21 on PRP-008's integration surface; the PyPI release path first |
| [PRP-007](PRP-007-authoring-agent.md) | Ontology authoring agent | Import/align/overlay-first; SMEs converse; Claude drafts Turtle; the verifier gatekeeps |
| [PRP-009](PRP-009-physical-ai.md) | Physical AI | Physical AI as the lead story: shield-first docs and quickstart, a MuJoCo demo, a published latency number — sequencing against 006/007 is a captain decision |

## Conventions

Full rules in [CONVENTIONS.md](CONVENTIONS.md) (lifecycle, folder form for active
PRPs, tracks, the dependency-DAG sequencing rule). At-a-glance state: [BOARD.md](BOARD.md).

- The v1 security/trust workstream (captain-authorized 2026-08-18, per independent-
  review findings R2/D4) is cross-cutting, not a new PRP number:
  [`docs/framework/15-security.md`](../docs/framework/15-security.md) is the v1
  threat model; PRP-004, PRP-006, and PRP-007 each carry a **Security acceptance
  surface** subsection stating what their DoD must prove, plus owned **[DECIDE]**
  markers for the genuinely open mechanism choices. PRP-008 implements the fifth trust
  boundary (the external system of record/host harness, doc 15) as a bounded prototype.

- PRPs are **very high level** — intent, scope, non-goals, key decisions, definition
  of done. Far-future Drafts are one page each; from PRP-002 onward the next-up/Active
  PRP is teaching-grade (background, worked triage examples, Python-parallel notes,
  glossary — see CONVENTIONS.md). Sprint-level detail is produced when a PRP is
  activated (the PRP converts to a folder: `01-objective.md` + `02-tasks-and-gaps.md`).
- **Sequencing is a dependency DAG, not strict numeric order** (captain ruling,
  2026-08-24, `data/horismos-thin-pivot-s1/decisions/captain-pivot-ruling.md` item 2):
  PRP-008 activates after PRP-005 and before PRP-006/007 even though its number is
  higher — numbers are stable identifiers assigned at draft time, not an activation
  queue. Established PRPs are never renumbered to preserve a strict sequence; the
  Reading order table above states the actual activation order.
- The running example across all PRPs is **clinical triage** (`docs/framework/` uses it
  throughout), extended by PRP-008 with a second, standards-anchored vertical slice
  (VTE prophylaxis). The proving-ground benchmark decision (τ-Bench
  airline vs. triage-first) is recorded in PRP-000 as an open decision.
- Prior art: `../../sigil-lang` is the proving ground this framework inherits from
  (Rete-lite matcher, BDD verifier, constraint solver, trace design, neural bridge,
  NL-to-DSL authoring loop — all built and tested there). PRPs cite what they port.
