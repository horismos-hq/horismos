---
name: cargo-workspace-map
description: Use before creating, moving, or placing any Rust code in the horismos workspace — answers which crate and module owns a piece of functionality (ontology, world, reasoner, workflow, bridge, trace, verify, PyO3), which dependency edges are allowed, and which feature flag gates it. Also use when adding a dependency to any Cargo.toml.
---

# Cargo workspace map

Layout authority: `docs/framework/14-architecture.md`. When this skill and that doc
disagree, the doc wins — then fix this skill.

## Ownership table

| Functionality | Crate · module | Spec doc | Owning PRP |
|---|---|---|---|
| Turtle parse/serialize, RDFS-plus profile, imports, IRI interning | `horismos-core::ontology` | 02 | PRP-001 |
| Instance store, validation gate, snapshots, transactions, queries, subscriptions | `horismos-core::world` | 03 | PRP-001 |
| Subsumption, forward chaining (Rete-lite), backward chaining, proof trees | `horismos-core::reason` | 04 | PRP-002 |
| Step interpreter, edges/fallbacks, run outcomes, tools contract | `horismos-core::workflow` | 05–07 | PRP-004 |
| BDD checks + witnesses | `horismos-core::verify` (feature `verify`) | 10 | PRP-005 |
| `horismos.trace/1` emit + replay | `horismos-core::trace` | 09 | PRP-002 |
| `Bridge` **trait only** (perceive/narrate/rank) | `horismos-core::bridge` | 08 | PRP-004 |
| `HorismosError` hierarchy, `domain_text()` | `horismos-core::error` | 12 | PRP-001 |
| WASM `HorismosRuntime` | `horismos-core::wasm` (feature `wasm`) | 12 | PRP-006 |
| Bridge implementations (mock, anthropic, openai, onnx) | `horismos-bridges` | 08 | PRP-004 |
| Python marshaling — **no logic** | `horismos-py` (`_horismos` cdylib) | 11 | PRP-003 |
| Pure-Python surface, proxies, stubs | `python/horismos/` | 11 | PRP-003 |

## Dependency rules (mechanically enforced)

```
horismos-core     → nothing model-related, no network, no provider SDKs. EVER.
horismos-bridges  → horismos-core + provider SDKs (feature-gated)
horismos-py       → horismos-core + horismos-bridges
```

- Adding any dependency to `crates/horismos-core/Cargo.toml`: it must not be a
  provider SDK or HTTP client. `make check-boundary` greps for violations;
  `make no-model-gate` builds/tests core with `--no-default-features --features verify`.
- New shared dependencies go in `[workspace.dependencies]` in the root Cargo.toml.
- `horismos-core` must keep compiling for `wasm32-unknown-unknown` under the
  no-model-gate feature set.

## Feature flags (horismos-core)

`verify` (default), `sparql` (default), `mock` (dev/testing utilities), `wasm`
(mutually exclusive with the PyO3 path). Provider flags (`bridges-*`) live in
`horismos-bridges`, never in core.

## Before you finish

Run `make verify`. If you added a module, its top doc-comment must state the spec
doc and owning PRP, matching the style of existing `mod.rs` files.
