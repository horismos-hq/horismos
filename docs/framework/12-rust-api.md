# 12 · Rust API Reference

Crates: `horismos-core` (the symbolic engine — **zero LLM dependencies**),
`horismos-bridges` (provider clients, feature-gated), `horismos-py` (PyO3 layer, not for
direct use). MSRV: latest stable minus 2.

**Declared-future (PRP-008)**: pure, format-neutral `ExternalSnapshot`/`ProposedEffectSet`
values and traits live in `horismos-core` only where genuinely core-agnostic; FHIR/HTTP
implementations live in a new adapter crate (e.g. `horismos-fhir`), never in
`horismos-core` itself. A direct `World<FhirClient>` — a World parameterized on a live
external client — is an explicit **non-goal**: it would break doc 03's and doc 14's
network-free-transaction invariant. Current dependency and deterministic feature gates are
unchanged by this addition.

## Quickstart

```rust
use horismos_core::{Ontology, World, Agent, RunInput};
use horismos_bridges::OpenRouterBridge;

let onto = Ontology::load_with_imports_and_options(
    "triage.ttl",
    ["snomed-ct-subset.ttl"],
    Default::default(),
)?;                                              // load verifies and fails closed
assert!(onto.verify().ok());                     // deterministic re-verification

let mut world = World::new(&onto);
let agent = Agent::builder(&onto)
    .workflow("triage:Triage")
    .bridge("claude", OpenRouterBridge::from_env())
    .tool("ehr.assign_queue", assign_queue)      // impl Tool
    .build()?;                                   // unbound declarations = build error

let outcome = agent
    .run(&mut world, RunInput::new("run_01J…", "58M, crushing chest pain…", now))
    .await?;
println!("{}", outcome.trace.to_json_pretty());
```

## Core types

| Type | Role | Notes |
|------|------|-------|
| `Ontology` | Immutable, loaded+verified schema/rules/workflows | Cheap to clone (`Arc` internals) |
| `World` | Mutable instance store | `&mut` for transactions; `Snapshot` is an immutable value |
| `reason` (module) | Deterministic inference; driven through `World` | `world.prove(subject, prop, &value)`, `world.explain(subject, prop)` |
| `Agent` | Workflow interpreter | Built per-(ontology, bindings); `run` borrows a `World`, returns `RunOutcome` |
| `Trace` | The trajectory | `serde` to/from `horismos.trace/1` or `/2`; `RunOutcome.trace` is always a `/2` |
| `VerificationReport` | Deterministic BDD and workflow findings | `ok()`, typed witnesses/regions, canonical `to_json()`, `exit_code()`, and an always-present five-field `AssumptionManifest` |
| `Iri`, `Ref`, `Value` | Interned identifiers and typed values | `Value` mirrors the XSD subset + IRIs |

## Key traits

```rust
pub trait Tool: Send + Sync {
    fn call(&self, args: &BTreeMap<String, Value>) -> Result<BTreeMap<Iri, Value>, String>;
    // v1 ships sync only; an async variant behind a future `async-tools` feature is deferred
}

#[async_trait]
pub trait Bridge: Send + Sync { /* perceive / narrate / rank — doc 08 */ }
```

Tool arguments arrive resolved from the `horis:takes` declaration; the returned
property→value map is validated against the declared `horis:effect` before
assertion — implementors cannot bypass the gate.

## Feature flags

| Feature | Default | Contents |
|---------|---------|----------|
| `verify` | on | BDD verification (biodivine-lib-bdd) |
| `sparql` | on | The query subset |
| `bridges-anthropic` / `-openai` / `-onnx` / `-openrouter` | off | Provider clients (in `horismos-bridges`); `-openrouter` ships `OpenRouterBridge` |
| `mock` | on (dev) | MockBridge + testing utilities |
| `wasm` | off | wasm-bindgen surface; mutually exclusive with `pyo3` |
| `pyo3` | off | Only `horismos-py` enables this |

The flag discipline is inherited directly from sigil-lang's proven `pyo3-ext`/`wasm`
isolation: PyO3-touching code is gated at module level; the symbolic core compiles
for `wasm32-unknown-unknown` with `--no-default-features --features verify`.

Ordinary `Ontology::load` and `Ontology::load_from_str` verify and fail closed.
`LoadOptions { allow_incomplete: true }` and the named `*_allow_incomplete` helpers are
the explicit authoring/debug escape; verification still runs, `verify()` remains
available, and any agent trace records `ontology.allow_incomplete: true`. Re-verification
is currently a complete deterministic pass. Numeric discretization compares exact
authored values only within the same opaque unit; v1 performs no unit conversion.

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

Errors are `thiserror`-derived domain errors. Verification failure is
`OntologyError::VerificationFailed { report }`; validation, workflow, bridge, and tool
errors likewise preserve domain vocabulary through `domain_text()` rather than reducing
findings to opaque strings.
