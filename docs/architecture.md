# Architecture

The bounded projection kernel in one picture, then the pieces. Depth lives in the framework reference — this page tells you which document to open.

## The kernel in one diagram

<div class="horis-diagram">
<svg viewBox="0 0 980 520" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Horismos bounded projection kernel: a system of record is hydrated by an adapter into a snapshot; the kernel gates it, reasons to a fixed point, emits proof and trace; a proposed effect set commits back conditionally. LLMs sit only at perception and narration.">
  <defs>
    <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="currentColor"/>
    </marker>
  </defs>
  <style>
    .box { fill: var(--md-code-bg-color); stroke: currentColor; stroke-width: 1.3; }
    .kernel { fill: var(--md-code-bg-color); stroke: currentColor; stroke-width: 2; }
    .future { stroke-dasharray: 5 4; }
    .t { font-family: inherit; font-size: 13px; fill: currentColor; }
    .s { font-family: inherit; font-size: 11px; fill: currentColor; opacity: 0.75; }
    .h { font-family: inherit; font-size: 14px; font-weight: 700; fill: currentColor; }
    .l { stroke: currentColor; stroke-width: 1.3; fill: none; marker-end: url(#arrow); }
  </style>

  <!-- System of record -->
  <rect class="box" x="20" y="185" width="150" height="110" rx="8"/>
  <text class="h" x="95" y="213" text-anchor="middle">System of record</text>
  <text class="s" x="95" y="235" text-anchor="middle">FHIR server · repository</text>
  <text class="s" x="95" y="251" text-anchor="middle">tool registry · your DB</text>
  <text class="s" x="95" y="277" text-anchor="middle">owns durable state</text>

  <!-- Adapter: hydrate -->
  <rect class="box future" x="200" y="200" width="120" height="80" rx="8"/>
  <text class="t" x="260" y="227" text-anchor="middle">Adapter</text>
  <text class="s" x="260" y="245" text-anchor="middle">hydrate a declared,</text>
  <text class="s" x="260" y="259" text-anchor="middle">versioned closure</text>
  <text class="s" x="260" y="296" text-anchor="middle">PRP-008</text>
  <path class="l" d="M 170 240 L 198 240"/>
  <path class="l" d="M 320 240 L 348 240"/>
  <text class="s" x="334" y="230" text-anchor="middle">snapshot</text>

  <!-- Kernel -->
  <rect class="kernel" x="350" y="120" width="440" height="240" rx="12"/>
  <text class="h" x="570" y="145" text-anchor="middle">horismos-core — the kernel (no network, no model)</text>

  <rect class="box" x="470" y="160" width="200" height="36" rx="6"/>
  <text class="t" x="570" y="183" text-anchor="middle">Ontology (.ttl) — verified</text>
  <text class="s" x="570" y="214" text-anchor="middle">classes · constraints · rules · workflow · tool contracts</text>

  <rect class="box" x="370" y="230" width="90" height="56" rx="6"/>
  <text class="t" x="415" y="253" text-anchor="middle">Gate</text>
  <text class="s" x="415" y="271" text-anchor="middle">5 checks</text>

  <rect class="box" x="480" y="230" width="90" height="56" rx="6"/>
  <text class="t" x="525" y="253" text-anchor="middle">World</text>
  <text class="s" x="525" y="271" text-anchor="middle">typed facts</text>

  <rect class="box" x="590" y="230" width="90" height="56" rx="6"/>
  <text class="t" x="635" y="253" text-anchor="middle">Rules</text>
  <text class="s" x="635" y="271" text-anchor="middle">fixed point</text>

  <rect class="box" x="700" y="230" width="72" height="56" rx="6"/>
  <text class="t" x="736" y="253" text-anchor="middle">Proof</text>
  <text class="s" x="736" y="271" text-anchor="middle">+ trace</text>

  <path class="l" d="M 460 258 L 478 258"/>
  <path class="l" d="M 570 258 L 588 258"/>
  <path class="l" d="M 680 258 L 698 258"/>

  <text class="s" x="570" y="318" text-anchor="middle">truth maintenance: every transaction re-chains to the fixed point of its base facts</text>
  <text class="s" x="570" y="336" text-anchor="middle">deterministic · no wall-clock · trace equality is a test assertion</text>

  <!-- Perception (LLM) -->
  <rect class="box" x="360" y="30" width="200" height="60" rx="8"/>
  <text class="t" x="460" y="52" text-anchor="middle">Perception — LLM</text>
  <text class="s" x="460" y="70" text-anchor="middle">text → typed instances, schema-forced,</text>
  <text class="s" x="460" y="83" text-anchor="middle">confidence-gated, fallback edge required</text>
  <path class="l" d="M 415 90 L 415 228"/>
  <text class="s" x="408" y="108" text-anchor="end">proposes</text>
  <text class="s" x="408" y="122" text-anchor="end">the gate decides</text>

  <!-- Narration (LLM) -->
  <rect class="box" x="640" y="30" width="180" height="60" rx="8"/>
  <text class="t" x="730" y="52" text-anchor="middle">Narration — LLM</text>
  <text class="s" x="730" y="70" text-anchor="middle">proof → prose for an audience;</text>
  <text class="s" x="730" y="83" text-anchor="middle">never written back</text>
  <path class="l" d="M 736 228 L 736 92"/>

  <!-- Tools -->
  <rect class="box" x="370" y="395" width="400" height="60" rx="8"/>
  <text class="t" x="570" y="417" text-anchor="middle">Your tools — fired by the runtime when preconditions hold</text>
  <text class="s" x="570" y="435" text-anchor="middle">arguments read from the graph · effects validated through the gate · never chosen by a model</text>
  <path class="l" d="M 635 360 L 635 393"/>
  <path class="l" d="M 415 393 L 415 360"/>
  <text class="s" x="645" y="381" text-anchor="start">preconditions</text>
  <text class="s" x="405" y="381" text-anchor="end">effects</text>

  <!-- Proposed effects + conditional commit -->
  <rect class="box future" x="820" y="200" width="140" height="80" rx="8"/>
  <text class="t" x="890" y="227" text-anchor="middle">Adapter</text>
  <text class="s" x="890" y="245" text-anchor="middle">conditional commit:</text>
  <text class="s" x="890" y="259" text-anchor="middle">fails closed on drift</text>
  <text class="s" x="890" y="296" text-anchor="middle">PRP-008</text>
  <path class="l" d="M 790 240 L 818 240"/>
  <text class="s" x="804" y="230" text-anchor="middle">effects</text>
  <path class="l" d="M 890 280 L 890 485 L 95 485 L 95 297"/>
  <text class="s" x="492" y="505" text-anchor="middle">commit receipt or conflict appended to the trace — no cross-system atomicity claim</text>
</svg>
</div>

Solid boxes are merged today. Dashed boxes are the host-integration mechanism owned by PRP-008: hydrate a declared, versioned closure into an all-or-nothing snapshot; run the unchanged kernel over it; commit a typed proposed effect set back only if the source still matches the recorded versions. Nothing inside the kernel box performs a network read, ever — that is the anti-wrapper boundary, enforced mechanically (`make check-boundary`, `make no-model-gate`).

## One transaction, end to end

```text
text ──Bridge──▶ JSON (schema-forced) ──gate──▶ base facts
                                                  │ forward chain (Rete-lite, in-core)
                                                  ▼
                                        derived facts + proofs
                                                  │ constraint re-check
                                                  ▼
                              commit ── snapshot ── trace append ── subscribers
```

Everything after the Bridge call is allocation-light Rust over interned IRIs. Python and (planned) WASM hand a whole input over and get a whole outcome back — there is no cross-language chatter inside a transaction, a lesson measured in the sigil-lang project this framework inherits from.

## The pieces and where they live

| Piece | Crate / package | Read |
|---|---|---|
| Turtle parsing, RDFS-plus profile validation, schema and subsumption, imports and anchoring | `horismos-core::ontology` | [The Ontology](framework/02-ontology.md) |
| The typed instance store, five-check gate, provenance, snapshots, diff, binary form, queries | `horismos-core::world` | [The World](framework/03-world.md) |
| Rule model, the 11 condition operators, naive and Rete-lite matchers, forward and backward chaining, proof trees | `horismos-core::reason` | [Reasoning](framework/04-reasoning.md) |
| The workflow interpreter (`Agent`), step types, tool contracts and effect validation | `horismos-core::workflow` | [Workflows](framework/05-workflows.md), [Tools](framework/07-tools.md) |
| The `Bridge` trait — a pure interface, no provider code | `horismos-core::bridge` | [The Bridge](framework/08-bridge.md), [LLM Boundaries](framework/06-llm-boundaries.md) |
| `horismos.trace/1` and `/2`, emission, migration policy | `horismos-core::trace` | [Traces](framework/09-traces.md) |
| BDD verification with witnesses, assumption manifest, non-vacuity | `horismos-core::verify` (PRP-005) | [Verification](framework/10-verification.md) |
| `MockBridge` (the CI workhorse) and `OpenRouterBridge` (feature `bridges-openrouter`) | `horismos-bridges` | [The Bridge](framework/08-bridge.md) |
| PyO3 extension — marshaling only, no logic | `horismos-py` | [Python API](framework/11-python-api.md) |
| The `horismos` Python package: naming sugar over the native handle | `python/horismos` | [Python API](framework/11-python-api.md) |
| FHIR and coding-harness adapters, host SPI (`ExternalSnapshot`, `ProposedEffectSet`) | new crates outside `horismos-core` (PRP-008) | `PRPs/PRP-008-standards-host-integration.md` |

## The invariants that make it a kernel

- **Anti-wrapper boundary.** `horismos-core` never depends on `horismos-bridges`, a provider SDK, or the network. `make no-model-gate` builds and tests the core with `--no-default-features --features verify`; `make check-boundary` greps the manifest. Red means the "runs with the LLM deleted" claim broke.
- **Determinism.** No wall-clock, randomness, or hash-order dependence in inference. `horis:now` is stamped once per transaction. Two runs of the same input produce byte-identical trajectories, and the test suite asserts it.
- **One vocabulary end to end.** The CURIEs an author writes are the CURIEs the API takes, the errors print, and the proofs render. No second naming system in the bindings (a generated-proxy design was prototyped and rejected on evidence; `PRPs/PRP-003-python-bindings/03-decision-instances.md`).
- **Whole inputs in, whole outcomes out.** Each Python call is one transaction: gate, chaining, fixed point, diff, all under one lock acquisition with the GIL released.
- **Two wire contracts, versioned forever.** The trace schema evolves only by minting a new version with a converter; the snapshot binary form has a versioned header.
- **One world per case.** A `World` is a serial transaction stream; parallelism is many worlds sharing one immutable `Ontology`. Under the projection model, one world per *decision epoch*.

## Performance posture

Triage-scale targets (about 50 rules, hundreds of facts): gate-plus-chain transaction under 1 ms, `prove`/`explain` under 1 ms, load-plus-verify under 1 s, perception overhead under 5 ms on top of the model call. Criterion benchmarks are tracked from PRP-002 onward; adapter I/O is measured separately and never blended into kernel numbers. Scaling beyond that is measured, not promised — see [Architecture internals](framework/14-architecture.md).

## Go deeper

[Concepts](framework/01-concepts.md) for the nouns and the two principles; [Architecture internals](framework/14-architecture.md) for the crate layout, the dependency invariant, and the concurrency model; [Security & Trust](framework/15-security.md) for the five trust boundaries and what each one leaves to the deployer.
