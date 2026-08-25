# Horismos Framework Documentation

> **Horismos** is an ontology-first neuro-symbolic decision and capability kernel. Rust
> core, Python bindings. The ontology overlay declares a bounded decision/capability
> fragment, evaluated as a versioned projection over an existing system of record.
> Neural proposes; symbolic decides.

> **Status**: framework *definition*, v0.1 — written in shipped voice so every contract
> is concrete and arguable, reviewed before implementation. Where behavior is still an
> open decision it is marked **[DECIDE]** and cross-referenced to a PRP in `../../PRPs/`.
> **Pivot adopted 2026-08-24** (bounded projection kernel — `PRPs/PRP-000-vision-
> positioning.md` §"Snapshot-scoped guarantees"): docs below are updated at the contract
> level for the new positioning; deep host-integration mechanism (FHIR adapters, CDS
> Hooks, PlanDefinition) is owned by the new `PRPs/PRP-008-standards-host-integration.md`
> and marked **declared-future** where it appears here.

## The one-paragraph mental model

You author a small RDFS-plus overlay (`.ttl`) that declares the decision/capability
fragment your deployment needs: classes and constraints, rules, and an optional decision
workflow. The runtime loads it, formally verifies it, and evaluates it as a **bounded
projection** — a versioned, explicitly scoped snapshot hydrated from your existing system
of record (a FHIR server, a repository, a tool registry), never a live connection to it.
Natural language enters through a single declared perception boundary where an LLM
extracts typed instances — validated before they become facts. Everything downstream —
inference, workflow transitions, tool calls — is deterministic, traced, and replayable
*for that snapshot*; a typed proposed effect set commits back to the system of record
conditionally, failing closed if the source has moved on. Delete the LLM and the
framework still runs. Horismos does not own your durable records, your enterprise
workflow, your provider clients, or your standard terminology — it overlays them.

## Documentation map

| # | Doc | What it covers |
|---|-----|----------------|
| 01 | [Concepts](01-concepts.md) | The nouns, the two principles, how Horismos differs from agent frameworks and adjacent standards |
| 02 | [The Ontology](02-ontology.md) | RDFS-plus profile: classes, constraints, the full `horis:` vocabulary, imports, standard-artifact anchoring |
| 03 | [The World](03-world.md) | Instance store as an admitted projection: assertion, validation gate, queries, snapshots, freshness, conditional commit |
| 04 | [Reasoning](04-reasoning.md) | Rules, forward/backward chaining, subsumption, proof trees, scoped to one input closure |
| 05 | [Workflows](05-workflows.md) | Step types, transitions, preconditions, fallbacks, termination — optional decision microflows a host may invoke |
| 06 | [LLM Boundaries](06-llm-boundaries.md) | Perception & narration: schema derivation, confidence, fallback contracts |
| 07 | [Tools](07-tools.md) | Tool contracts, bindings (Python/Rust/HTTP/MCP), effect validation, proposals, adapter-mediated commits |
| 08 | [The Bridge](08-bridge.md) | Model backends, registry, timeouts, the mock backend, provider mechanics |
| 09 | [Traces](09-traces.md) | The trajectory schema, replay, audit, training-corpus export |
| 10 | [Verification](10-verification.md) | BDD checks, witnesses, the assumption manifest, non-vacuity, workflow verification, CI usage |
| 11 | [Python API](11-python-api.md) | The `horismos` package reference |
| 12 | [Rust API](12-rust-api.md) | The `horismos-core` crate reference, feature flags, WASM |
| 13 | [Authoring](13-authoring.md) | The authoring agent: import/align/overlay-first, SMEs converse, Claude drafts, the verifier gatekeeps |
| 14 | [Architecture](14-architecture.md) | Internals: crate layout, PyO3 boundary, performance, PRP mapping, adapter/service crate boundary |
| 15 | [Security & Trust](15-security.md) | The v1 threat model: assets, trust boundaries, threats, mitigations vs. deployer-owned concerns; the host-harness boundary |

## Reading paths

- **"Show me the product"** → 01 → 05 → 06 → the quickstarts in 11.
- **"I'm reviewing before we build"** → in order, 01–15; argue with every **[DECIDE]**.
- **"I'm reviewing trust/deployment risk"** → 15, then 06/07/08/09/13 for the boundaries it cites.
- **"I build agents today"** → 01 (the comparison table), 07 (tools inversion), 06.
- **"I'm the SME"** → 13 only. You will never read Turtle unless you want to.
- **"I'm integrating against FHIR/CIS or a coding harness"** → `PRPs/PRP-000-vision-
  positioning.md`'s comparison table, then `PRPs/PRP-008-standards-host-integration.md`
  for the mechanism this PRP owns; docs 03 and 14 for how the projection boundary works
  today.

## The running example

All documentation uses one worked domain: **clinical triage** (`triage.ttl`) — a
patient-intake workflow that extracts vitals and symptoms from natural language,
derives acuity through rules over a SNOMED-anchored symptom hierarchy, and routes to
a queue via an EHR tool. It exercises every framework feature. The full file is built
up progressively in docs 02–07.

## Research wiki

`../../research/` reality-checks this design against the field (CIG formalisms,
neuro-symbolic AI, clinical validation practice, terminology engineering,
agent-authored ontologies) — see `research/README.md`.

## Prior art

Horismos inherits designs proven in the `sigil-lang` project (same authors): Rete-lite
matching, BDD verification, constraint solving, trace semantics, the pluggable neural
bridge, and the validate-and-refine authoring loop — collectively ~1,270 tests of
prior art. Horismos is not a rewrite of sigil-lang; it inverts it: the engine is the
product, and the language surface is gone (an authoring agent writes standard Turtle
instead).
