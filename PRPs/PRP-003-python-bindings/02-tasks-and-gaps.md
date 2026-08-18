# PRP-003 — Tasks & Gaps

Working surface per `../CONVENTIONS.md`, seeded 2026-08-18 at activation.
Expanded context for D1–D3 lives in `01-objective.md` § Key decisions. Each
D-item is resolved (via `.claude/skills/brainstorming`, never ad hoc) before
the unit it blocks; the boundary-plumbing units it does not block may land
first — the D1/D2 prototypes need a working boundary to prototype against.

## Boundary contracts

Pinned at activation; sharpened 2026-08-18 per the independent plan review
(lens 2 S2, lens 5 Q3, top finding 2).

- **B1 — no `Iri` tokens across the boundary.** Interner tokens are
  world-relative; only full IRI *strings* cross. `Ref` is a frozen string
  handle (full IRI + compacted form), re-interned against whichever world
  receives it — a `Ref` minted from one world is safe against another by
  construction.
- **B2 — subscriptions ownership (003 vs 004), decided.** PRP-003 ships the
  *registration surface and post-commit dispatch*: `subscribe(pattern,
  callback)` lives in the pure-Python layer, matches against the
  **core-computed diff** of each committed transaction, and fires strictly
  after commit on the mutating thread — no transaction semantics in Python,
  no core hooks. The core-side transactional notification slot (doc 03's
  "notify subscribers" stage, re-entrant mutation queueing inside the
  runtime loop) remains **PRP-004**, as `world/mod.rs` already records.
- **B3 — the structured error contract is the machine API.** The exception
  attributes (`constraint` tag, per-tag `data` fields, `subject`, `class_`,
  `property`, `value`, `provenance`, `operation`, `suggestion`) are the
  versioned machine-readable surface, documented in the stubs and doc 11.
  `domain_text` / `str(exc)` is for humans and is **not** a stability
  surface — tests may pin it, programs must not parse it.
- **B4 — parity is tested from the same fixtures.** The triage fixture pins
  both APIs: the r1 proof rendering and the gate rejection sentences are
  asserted byte-identical to the Rust suite's expectations, and Hypothesis
  boundary invariants (atomic rejection, determinism, value round-trip) are
  DoD-mandatory, not optional hardening.
- **B5 — CI ownership split with PRP-006.** PRP-003 owns the local wheel
  build proof and the matrix workflow *configuration file*
  (Linux/macOS/Windows × 3.11–3.14). PRP-006 owns matrix *execution*,
  publication, and CI gating. Neither claims the other's half.
- **B6 — any query evaluation lives in core.** Whatever D2 picks, the
  evaluator is a named `horismos-core::world` unit; `horismos-py` and the
  pure-Python layer only marshal the question and the rows.

## Decisions to resolve (before implementation)

- [x] D1. **Generated proxy classes vs. dict-like generic instances**
      (decided 2026-08-18): the **dict-like generic form ships** — CURIE
      class term + snake_case keyword properties; generated proxies are
      deliberately not shipped in v1. Both candidates were prototyped and
      run against the triage quickstart; the decider on the stated
      criterion (quickstart ergonomics) was that imported vocabularies
      flood the proxy namespace (~50 numeric SNOMED locals, not legal
      Python attribute names), while the generic form keeps one vocabulary
      end to end (Turtle = errors = proofs = code). Not a coin-flip; no
      escalation. Evidence + preserved prototype:
      `03-decision-instances.md`. Doc 11's stubgen marker struck in the
      same commit (§ Typed instance construction rewritten in shipped
      voice); stubgen codegen deferred with a dated gap note below.
- [x] D2. **Query surface: SPARQL subset vs. Pythonic builder** (decided
      2026-08-18): v1 ships a **class-scoped conjunctive pattern query**
      whose evaluator is the named core unit `World::query_pattern`
      (`horismos-core::world`, B6) — exact grammar stated in
      `03-decision-query.md`; undeclared terms are domain-term errors,
      never empty results. The **textual SPARQL subset is deferred beyond
      v1** (dated note below); if built, it compiles to this tier. Doc
      03's marker struck, § Queries rewritten in shipped voice, same
      commit.
- [ ] D3. **GIL release policy**. Which native calls detach from the
      interpreter (load / assert-with-chaining / prove / snapshot codecs)
      and the proof that none touches Python while detached. Blocks U8.
      Recorded here and in `crates/horismos-py`'s module docs; escalate only
      if it forces an API change.

## Units of work

- [ ] U1. **Native handles: `Ontology` across the boundary.** `_horismos`
      exposes an ontology handle: `load(path, imports)` (core `load` +
      `with_import`, whole call one crossing), `namespaces`, `ref`,
      rule introspection (IRI, priority, explain), `version`.
      Test target: `python/tests/test_ontology.py` loads
      `examples/triage/triage.ttl` + SNOMED import, sees the `triage`
      prefix, `ref("triage:Immediate")` round-trips, and a malformed TTL
      path raises (mapped in U2).
- [ ] U2. **The `HorismosError` hierarchy.** Exception types per doc 11
      § Errors defined in the native module, raised from every fallible
      binding; `str(exc)` == `domain_text()`, witness fields
      (`constraint`, `subject`, `property_`, `value`, `provenance`,
      `operation`) + `suggestion` (None-able) as attributes; never a bare
      `RuntimeError`/`ValueError`. Test target: `test_errors.py` pins
      Example 2's exact message for `heart_rate=1280` and the RulesOnly
      rejection, and asserts `except HorismosValidationError` catches as
      `HorismosError` too.
- [ ] U3. **`World` mutations: whole-transaction `assert_` / `retract`.**
      World handle over core: `assert_` (class + properties in one crossing
      → `Ref` out; gate + chaining inside core), `retract`. Value
      marshaling both directions for the full `Value` enum (int, decimal,
      str, bool, datetime, duration, ref). Test target: `test_world.py` —
      Example 1's assert derives `acuity == Immediate` (chaining ran inside
      the transaction), a rejected assert leaves the world unchanged
      (atomicity observable from Python), retraction of a derived fact
      raises with the PRP-002 ruling's message.
- [ ] U4. **Read surface + proofs.** `instances(class)` (subsumption-aware),
      `value`/`values`, `explain(subject, prop)`, `prove(subject, prop,
      value)`, proof trees as Python objects with `.render()` passing
      through the Rust rendering. Test target: `test_reasoning.py` pins
      Example 1's rendered r1 proof byte-identically to
      `crates/horismos-core/tests/reasoning_dod.rs`' expectation, and
      `prove` returns `None` for an unprovable goal.
- [ ] U5. **Snapshots.** `snapshot()` / `World.from_snapshot()` / `diff()`
      + `to_bytes`/`from_bytes` (bytes cross whole). Test target:
      `test_snapshots.py` — mutate-after-snapshot diff shows exactly the
      added facts; a corrupted byte-string raises the snapshot-format error;
      restoring under a different ontology raises the mismatch error.
- [ ] U6. **Typed instance construction per D1** — both prototypes live in
      the D1 spike, the winner ships (the loser's prototype stays in
      `03-decision-instances.md` as evidence, not in the package). Test
      target: `test_instances.py` — the quickstart spelling constructs the
      same world state as the raw U3 form, snake_case↔camelCase mapping
      pinned (`heart_rate` ↔ `heartRate`), unknown property/class fails
      with a domain-term error.
- [ ] U7. **Query surface per D2.** Whatever D2 picks, the evaluator is a
      named `horismos-core::world` unit (B6) with its own Rust tests;
      results cross as Python objects, whole outcome in one crossing. Test
      target: `test_query.py` — "patients whose acuity is Immediate"
      returns exactly the asserted patient; unknown terms are domain-term
      errors, not empty results — plus the core unit's own Rust test.
- [ ] U8. **Subscriptions (per B2) + GIL policy (D3).** `subscribe(pattern,
      callback)` as B2 scopes it: pure-Python registration + post-commit
      dispatch over the core-computed transaction diff (callbacks fire
      after commit, never mid-chaining; a mutation from a callback is a
      new transaction — it runs after the dispatching one completed).
      Core-side transactional hooks stay with PRP-004. GIL released per D3
      around the long native calls. Test target: `test_subscribe.py` —
      callback fires once per matching commit with the bound subject, not
      at all on a rejected transaction; a chaining call started in one
      thread does not block a second Python thread (release observable).
- [ ] U9. **Stubs, strictness, property tests.** `_horismos.pyi` + public
      package stubs complete; `mypy --strict` green over `python/`;
      Hypothesis suites: gate atomicity from Python (any rejected payload
      leaves state byte-identical — snapshot-diff empty), determinism (two
      identically built worlds render identical proofs / snapshots
      byte-identically), value round-trip (marshal in → read back equal).
      Test target: `make lint` + `test_properties.py`.
- [ ] U10. **Wheel + CI matrix config + the DoD run.** `maturin build`
      wheel installs into a fresh venv; `.github/workflows` matrix config
      (Linux/macOS/Windows × 3.11–3.14) checked in with a dated deferral
      note (full-matrix proof → PRP-006); `test_quickstart.py` runs
      Example 1 **verbatim** (the file is quoted from `01-objective.md`,
      kept in sync by the test itself). pytest imports and exercises every
      public symbol (`test_public_surface.py` walks `horismos.__all__`).
      `make verify` green — the PRP's DoD.

## Gaps discovered during implementation

(append dated notes here; scope changes go back into `01-objective.md` by explicit edit)

- 2026-08-18 (D2 deferral): the textual SPARQL SELECT/ASK subset (FILTER,
  OPTIONAL, ORDER BY/LIMIT over basic graph patterns) is deferred beyond
  v1 — no v1 consumer needs a SPARQL string (evidence in
  `03-decision-query.md`). If a later phase wants it, it is a front-end
  compiling to `World::query_pattern`, owned by whichever PRP claims it
  then; doc 03 § Queries records the same.
- 2026-08-18 (D1 deferral): `horismos stubgen` codegen for IDE/mypy
  support of ontology-derived names is deferred — no v1 unit ships it.
  Revisit when PRP-007's authoring tooling generates ontology artifacts;
  the decision record (`03-decision-instances.md`) carries the context.
- 2026-08-18 (U3): `xsd:duration` values cross the boundary *outbound* as
  their lexical form (`"PT2H"` as `str`), not a dedicated handle — inbound
  strings parse through core for duration-ranged properties, so the
  round-trip is lossless but type-asymmetric. A dedicated wrapper type is
  deferred until something needs arithmetic on it.
- **2026-08-18 — OPEN CAPTAIN HOLD: truth maintenance for derived facts.**
  The independent plan review (top finding 2) confirmed PRP-002 shipped no
  invalidation of derived facts when their supporting base facts are later
  replaced or retracted (a derived `:Immediate` acuity can outlive the
  heart rate that justified it). Whether that is the intended semantics
  (derived facts as historical events) or a defect (recompute/cascade) is
  an **open captain decision, held above this PRP**. PRP-003 therefore
  binds current core behavior *without endorsing it*: the Python surface
  and its docs stay silent on the fate of derived facts after base-fact
  mutations, no test pins post-mutation derived-fact persistence, and the
  `[DECIDE]`-owning PRP for the ruling is assigned by the captain, not
  here. Tests that pin the merged retraction gate (a derived fact cannot
  be api-retracted) remain — that rule survives either resolution.
