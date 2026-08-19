# 12 · Rust API Reference

Crates: `horismos-core` (the symbolic engine — **zero LLM dependencies**),
`horismos-bridges` (provider clients, feature-gated), `horismos-py` (PyO3 layer, not for
direct use). MSRV: latest stable minus 2.

## Quickstart

```rust
use horismos_core::{Ontology, World, Agent, RunInput};
use horismos_bridges::AnthropicBridge;

let onto = Ontology::load("triage.ttl")?
    .with_import("snomed-ct-subset.ttl")?;
onto.verify()?.assert_ok()?;                     // refuse to run unverified

let mut world = World::new(&onto);
let agent = Agent::builder(&onto)
    .workflow("triage:Triage")
    .bridge("claude:claude-fable-5", AnthropicBridge::from_env()?)
    .tool("ehr.assign_queue", assign_queue)      // impl Tool
    .build()?;                                   // unbound declarations = build error

let outcome = agent.run(&mut world, RunInput::text("58M, crushing chest pain…")).await?;
println!("{}", outcome.proof().render());
```

## Core types

| Type | Role | Notes |
|------|------|-------|
| `Ontology` | Immutable, loaded+verified schema/rules/workflows | Cheap to clone (`Arc` internals) |
| `World` | Mutable instance store | `&mut` for transactions; `Snapshot` is an immutable value |
| `reason` (module) | Deterministic inference; driven through `World` | `world.prove(subject, prop, &value)`, `world.explain(subject, prop)` |
| `Agent` | Workflow interpreter | Built per-(ontology, bindings); `run` borrows a `World` |
| `Trace` | The trajectory | `serde` to/from `horismos.trace/1` |
| `Iri`, `Ref`, `Value` | Interned identifiers and typed values | `Value` mirrors the XSD subset + IRIs |

## Key traits

```rust
pub trait Tool: Send + Sync {
    fn invoke(&self, args: ToolArgs, cx: &ToolCx) -> Result<ToolOutput, ToolError>;
    // v1 ships sync only; an async variant behind a future `async-tools` feature is deferred
}

#[async_trait]
pub trait Bridge: Send + Sync { /* perceive / narrate / rank — doc 08 */ }
```

Tool arguments arrive typed (`ToolArgs::get::<f64>("queue")` resolves from graph
values); outputs are validated against the declared `horis:effect` before assertion —
implementors cannot bypass the gate.

## Feature flags

| Feature | Default | Contents |
|---------|---------|----------|
| `verify` | on | BDD verification (biodivine-lib-bdd) |
| `sparql` | on | The query subset |
| `bridges-anthropic` / `-openai` / `-onnx` | off | Provider clients (in `horismos-bridges`) |
| `mock` | on (dev) | MockBridge + testing utilities |
| `wasm` | off | wasm-bindgen surface; mutually exclusive with `pyo3` |
| `pyo3` | off | Only `horismos-py` enables this |

The flag discipline is inherited directly from sigil-lang's proven `pyo3-ext`/`wasm`
isolation: PyO3-touching code is gated at module level; the symbolic core compiles
for `wasm32-unknown-unknown` with `--no-default-features --features verify`.

## WASM

```rust
// horismos_core::wasm — behind `wasm`
let rt = HorismosRuntime::new(ttl_source)?;     // load + verify
let out = rt.run_json(input_json)?;          // structured input → outcome + trace (JSON)
```

Browser/edge deployments run the **full symbolic stack** — gate, chaining,
verification, workflows with structured input and local tools. Bridge-dependent steps
require either a host-provided fetch shim or ontologies whose workflows avoid
perception (the structured-input pattern, doc 06).

## Threading & determinism

- `Ontology` is `Send + Sync`; share one across threads.
- `World` is single-writer by design (transactions are serial per world). Parallelism
  pattern: many worlds (one per case/session), one ontology — triage runs are
  independent by nature. A multi-world manager utility (a registry keyed by case or
  session id, spawning and reaping worlds) is deferred past v1 — the pattern above is
  simple enough for a caller to implement directly until a deployment needs one built
  in.
- All inference is deterministic (doc 04); `Trace` equality is a meaningful test
  assertion and CI uses it.

## Errors

`HorismosError` enum mirroring the Python hierarchy (`OntologyError`,
`VerificationFailed { report }`, `ValidationRejected { witness }`, `WorkflowError`,
`BridgeError`, `ToolError`), all `thiserror`-derived, all carrying `domain_text()`.
