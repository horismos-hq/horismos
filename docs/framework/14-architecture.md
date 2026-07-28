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
│   └── horismos-py/               # PyO3 bindings only — no logic
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
horismos-bridges     →  provider SDKs
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

## Performance posture

| Path | Target (triage-scale: ~50 rules, ~10² facts) | Rationale |
|------|------------------------------------------------|-----------|
| Gate + chain transaction | < 1 ms | In-core, interned, rete-lite indexed |
| `prove` / `explain` | < 1 ms | Depth-capped, indexed rule heads |
| Load + verify | < 1 s | Authoring-loop interactivity (doc 13) |
| Perception step | Bridge latency + < 5 ms overhead | The model call dominates by orders of magnitude |

Benchmarks (criterion) are CI-tracked from PRP-002 onward; regressions fail builds.
Scaling beyond (10³+ rules, 10⁵+ facts) is measured, not promised — the Rete-lite →
full-Rete decision point is data-driven (PRP-002).

## Concurrency model

One `World` = one serial transaction stream (single-writer). Horizontal pattern:
one world per case/session/conversation, sharing an immutable `Ontology`. This
matches the domain shape (a triage case, a claim, an incident) and eliminates
lock discipline from the mental model. Cross-world aggregation is a query/reporting
concern, outside the runtime.

## Security & trust boundaries

- **Untrusted**: NL input, LLM output, tool return values → all pass gates.
- **Trusted after sign-off**: the ontology (it *is* the program — its supply chain is
  git + SME sign-off + verification, doc 13).
- **Sandboxing tools** is the deployer's concern (bindings run with process
  privileges); MCP bindings inherit MCP's isolation. `horis:redactInput` for
  sensitive NL (doc 09).

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

Every **[DECIDE]** marker in these docs is owned by exactly one PRP; activating a PRP
begins by resolving its markers.
