# Horismos Framework Documentation

> **Horismos** is an ontology-first neuro-symbolic agent framework. Rust core, Python
> bindings. The ontology is simultaneously the typed world-state, the reasoning
> substrate, and the agent workflow. Neural proposes; symbolic decides.

> **Status**: framework *definition*, v0.1 — written in shipped voice so every contract
> is concrete and arguable, reviewed before implementation. Where behavior is still an
> open decision it is marked **[DECIDE]** and cross-referenced to a PRP in `../../PRPs/`.

## The one-paragraph mental model

You author one artifact: an RDFS-plus ontology (`.ttl`) that declares your domain's
classes and constraints, its rules, and its workflows. The runtime loads it, formally
verifies it, and executes it. Natural language enters through a single declared
perception boundary where an LLM extracts typed instances — validated before they
become facts. Everything downstream — inference, workflow transitions, tool calls —
is deterministic, traced, and replayable. Delete the LLM and the framework still runs.

## Documentation map

| # | Doc | What it covers |
|---|-----|----------------|
| 01 | [Concepts](01-concepts.md) | The seven nouns, the two principles, how Horismos differs from agent frameworks |
| 02 | [The Ontology](02-ontology.md) | RDFS-plus profile: classes, constraints, the full `horis:` vocabulary, imports |
| 03 | [The World](03-world.md) | Instance store: assertion, validation gate, queries, snapshots, subscriptions |
| 04 | [Reasoning](04-reasoning.md) | Rules, forward/backward chaining, subsumption, proof trees |
| 05 | [Workflows](05-workflows.md) | Step types, transitions, preconditions, fallbacks, termination |
| 06 | [LLM Boundaries](06-llm-boundaries.md) | Perception & narration: schema derivation, confidence, fallback contracts |
| 07 | [Tools](07-tools.md) | Tool contracts, bindings (Python/Rust/HTTP/MCP), effect validation, proposals |
| 08 | [The Bridge](08-bridge.md) | Model backends, registry, timeouts, the mock backend, provider mechanics |
| 09 | [Traces](09-traces.md) | The trajectory schema, replay, audit, training-corpus export |
| 10 | [Verification](10-verification.md) | BDD checks, witnesses, workflow verification, CI usage |
| 11 | [Python API](11-python-api.md) | The `horismos` package reference |
| 12 | [Rust API](12-rust-api.md) | The `horismos-core` crate reference, feature flags, WASM |
| 13 | [Authoring](13-authoring.md) | The authoring agent: SMEs converse, Claude drafts, the verifier gatekeeps |
| 14 | [Architecture](14-architecture.md) | Internals: crate layout, PyO3 boundary, performance, PRP mapping |

## Reading paths

- **"Show me the product"** → 01 → 05 → 06 → the quickstarts in 11.
- **"I'm reviewing before we build"** → in order, 01–14; argue with every **[DECIDE]**.
- **"I build agents today"** → 01 (the comparison table), 07 (tools inversion), 06.
- **"I'm the SME"** → 13 only. You will never read Turtle unless you want to.

## The running example

All documentation uses one worked domain: **clinical triage** (`triage.ttl`) — a
patient-intake workflow that extracts vitals and symptoms from natural language,
derives acuity through rules over a SNOMED-anchored symptom hierarchy, and routes to
a queue via an EHR tool. It exercises every framework feature. The full file is built
up progressively in docs 02–07.

## Prior art

Horismos inherits designs proven in the `sigil-lang` project (same authors): Rete-lite
matching, BDD verification, constraint solving, trace semantics, the pluggable neural
bridge, and the validate-and-refine authoring loop — collectively ~1,270 tests of
prior art. Horismos is not a rewrite of sigil-lang; it inverts it: the engine is the
product, and the language surface is gone (an authoring agent writes standard Turtle
instead).
