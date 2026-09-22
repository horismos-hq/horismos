# 14 · Architecture

Internals, invariants, and the mapping from this definition to build phases.

## Crate & package layout

```
horismos/                          (workspace)
├── crates/
│   ├── horismos-core/             # THE product: ontology, world, reasoner, agent, traces, verify
│   │   ├── ontology/           #   turtle parse, profile, imports, interning
│   │   ├── world/              #   store, gate, snapshots, transactions, queries
│   │   ├── reason/             #   subsumption, forward (rete-lite), backward, proofs
│   │   ├── workflow/           #   step interpreter, edges, outcomes
│   │   ├── verify/             #   BDD checks + witnesses (port of sigil-lang verifier)
│   │   ├── trace/              #   horismos.trace/1 emit + replay
│   │   └── wasm.rs             #   [wasm] HorismosRuntime
│   ├── horismos-bridges/          # provider clients ([bridges-*] features), mock
│   ├── horismos-py/               # PyO3 bindings only — no logic
│   ├── horismos-host/             # format-neutral snapshot/effect SPI + artifact policy
│   ├── horismos-fhir/             # R4 hydration/mapping/conditional commit + CDS surface
│   └── horismos-harness/          # recorded repository projection + artifact handoff
├── python/horismos/               # pure-Python surface: proxies, testing utils, export
├── docs/framework/             # ← you are here (becomes the docs site)
├── PRPs/                       # phase plans
├── examples/
│   ├── triage/                 # flagship (ttl + runs + tests)
│   ├── no-llm-monitoring/      # the anti-wrapper demo
│   ├── mcp-tools/
│   └── taubench-harness/
└── planning/                   # living plan artifact
```

## The dependency invariant (anti-wrapper, enforced)

```
horismos-core        →  (nothing model-related)          # compiles & tests with zero LLM deps
horismos-host        →  horismos-core                     # pure format-neutral host values
horismos-bridges     →  provider SDKs
horismos-fhir        →  horismos-host + horismos-core     # transport remains adapter-local
horismos-harness     →  horismos-host                     # no filesystem/process execution
horismos-py          →  horismos-core (+ optional bridges)
```

Mechanical enforcement, not convention:

1. `horismos-core` has **no dependency** on `horismos-bridges` — the `Bridge` trait lives in
   `horismos-core::bridge` as a pure interface; implementations live outside.
2. CI job `no-model-gate`: builds `horismos-core --no-default-features --features verify`,
   runs the full symbolic test suite **with no network and no keys**. Red = the
   anti-wrapper claim broke.
3. CI job `wasm-gate`: `wasm32-unknown-unknown` build of the same.

## Data flow (one perception transaction)

```
text ──Bridge──▶ JSON (schema-forced) ──gate──▶ base facts
                                                  │ forward chain (rete-lite, in-core)
                                                  ▼
                                        derived facts + proofs
                                                  │ constraint re-check
                                                  ▼
                              commit ── snapshot ── trace append ── subscribers
```

Everything inside the box after the Bridge call is allocation-light Rust over
interned IRIs. There is **no cross-language chatter inside a transaction** — the
sigil-lang benchmark lesson (PyO3 marshaling dominated small workloads) is designed
out: Python/WASM hand a whole input over, get a whole outcome back.

## Two-boundary data flow for a standards-hydrated deployment

The diagram above is the *inner* box, unchanged. A standards-hydrated deployment wraps it
with two more boundaries, both outside `horismos-core`, both entirely before/after the
transaction above — never interleaved with it:

```
FHIR/CIS or repo/harness ──Adapter (hydrate)──▶ ExternalSnapshot ──▶ [ inner box above ]
                                                                            │
                            [ inner box above ] ──▶ ProposedEffectSet ──Adapter (commit,
                                                       conditional on source version)──▶
                                                       FHIR/CIS or repo/harness
```

This is the reference data flow of the adopted bounded-projection design. It changes
nothing about the
dependency invariant above: the format-neutral contract lives in `horismos-host`; adapters
live in their own crates (`horismos-fhir` and `horismos-harness`) and depend inward, never
the reverse. A future HTTP transport may exist only in the FHIR adapter behind a feature;
the shipped recorded transport and harness prototype have no network, filesystem-mutation,
or process-execution dependency.

## Performance posture

| Path | Target (triage-scale: ~50 rules, ~10² facts) | Rationale |
|------|------------------------------------------------|-----------|
| Gate + chain transaction | < 1 ms | In-core, interned, rete-lite indexed |
| `prove` / `explain` | < 1 ms | Depth-capped, indexed rule heads |
| Load + verify | < 1 s | Authoring-loop interactivity (doc 13) |
| Perception step | Bridge latency + < 5 ms overhead | The model call dominates by orders of magnitude |
| Adapter hydration/commit | External I/O latency, not core-budgeted | A live FHIR read/write is bounded by the external system, not the kernel — measured and reported separately, never blended into the deterministic-kernel numbers above |

Benchmarks (criterion) are CI-tracked from PRP-002 onward; regressions fail builds.
Scaling beyond (10³+ rules, 10⁵+ facts) is measured, not promised — the Rete-lite →
full-Rete decision point is data-driven (PRP-002). **Split, going forward**: the table's
existing rows measure *deterministic kernel time* only; any future adapter benchmark is a
separate, clearly-labeled *external I/O latency* number — conflating the two would hide a
network-bound adapter behind an in-core performance claim.

## Concurrency model

One `World` = one serial transaction stream (single-writer). Horizontal pattern:
one world per case/session/conversation, sharing an immutable `Ontology`. This
matches the domain shape (a triage case, a claim, an incident) and eliminates
lock discipline from the mental model. Cross-world aggregation is a query/reporting
concern, outside the runtime.

**Amended (2026-08-24, bounded-projection pivot)**: for a standards-hydrated deployment,
read "one world per case" as **one projection per decision epoch** — a World's lifetime is
bound to one hydration/decision, not to the case's own lifetime, which the external system
of record continues to own. A long-running case may span many projections (one per
decision point, each re-hydrated), rather than one World living as long as the case does.
This is a refinement of the existing pattern, not a different one: a standalone deployment
where the case genuinely *is* the World's lifetime (the shipped triage example) is
unaffected.

## Security & trust boundaries

- **Untrusted**: NL input, LLM output, tool return values → all pass gates.
- **Trusted after sign-off**: the ontology (it *is* the program — its supply chain is
  git + SME sign-off + verification, doc 13).
- **Sandboxing tool-execution processes** (including MCP servers and the authoring
  agent) is the deployer's concern. This is the deployer-process-isolation boundary
  specifically — for the v1 threat model in full (assets, every trust boundary,
  per-boundary mitigations vs. what stays deployer-owned), see doc 15.

## Definition → build mapping

| Docs | PRP |
|------|-----|
| 02 Ontology, 03 World | PRP-001 |
| 04 Reasoning, 09 Traces (schema) | PRP-002 |
| 11 Python API | PRP-003 |
| 05 Workflows, 06 Boundaries, 07 Tools, 08 Bridge | PRP-004 |
| 10 Verification | PRP-005 |
| 12 Rust API surface polish, packaging, docs site | PRP-006 |
| 13 Authoring | PRP-007 |
| 15 Security & Trust | PRP-004, PRP-006, PRP-007, PRP-008 (per boundary — see doc 15) |
| Standards/host-integration additions across 00–03, 06, 07, 09, 10, 13, 14 | PRP-008 |

Every **[DECIDE]** marker in these docs is owned by exactly one PRP; activating a PRP
begins by resolving its markers.
