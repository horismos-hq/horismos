# PRP-003 — Tasks & Gaps

Working surface per `../CONVENTIONS.md`, seeded 2026-08-18 at activation.
Expanded context for D1–D3 lives in `01-objective.md` § Key decisions. Each
D-item is resolved (via `.claude/skills/brainstorming`, never ad hoc) before
the unit it blocks; the boundary-plumbing units it does not block may land
first — the D1/D2 prototypes need a working boundary to prototype against.

## Decisions to resolve (before implementation)

- [ ] D1. **Generated proxy classes vs. dict-like generic instances** (owns
      doc 11's `[DECIDE: codegen for IDE/mypy support (horismos stubgen)]`).
      Prototype both spellings against the triage quickstart (Example 3 of
      `01-objective.md`); criterion: ergonomics of the quickstart a newcomer
      types first. Blocks U6. Recorded here + `03-decision-instances.md`
      (evidence: both prototypes runnable), and doc 11's marker struck in the
      same commit. If the prototypes leave it a genuine coin-flip, escalate
      rather than coin-flip.
- [ ] D2. **Query surface: SPARQL subset vs. Pythonic builder** (owns doc
      03's `[DECIDE: exact subset grammar]`). Ship the minimal surface
      Example 1 needs; everything further deferred with a dated gap note.
      Blocks U7. Recorded here; doc 03's marker struck (its § Queries
      rewritten in shipped voice) in the same commit.
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
- [ ] U7. **Query surface per D2.** Whatever D2 picks, results as Python
      objects, whole outcome in one crossing. Test target: `test_query.py`
      — "patients whose acuity is Immediate" returns exactly the asserted
      patient; unknown terms are domain-term errors, not empty results.
- [ ] U8. **Subscriptions + GIL policy (D3).** `subscribe(pattern,
      callback)`: post-commit notification per doc 03 § Subscriptions
      (callbacks fire after commit, never mid-chaining; re-entrant
      mutations queue as new transactions); GIL released per D3 around the
      long native calls. Test target: `test_subscribe.py` — callback fires
      once per matching commit with the bound subject, not at all on a
      rejected transaction; a chaining call started in one thread does not
      block a second Python thread (release observable).
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
