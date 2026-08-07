# Implementer report — U7 (traces), U8 (Rete-lite + D-1), U9 (retraction
# rulings), U10 (DoD + property suite)

Commits: `080ea1a` (U7), `9af7089` (U8), `d8cf777` (U9), `3ccb657` (U10), plus
a small self-review fixup commit (Provenance wire-contract doc). `make verify`
green at each head; two intermediate verify failures (a clippy manual-map in
U9, a clippy type-complexity in U10) were caught by the gated verify-then-
commit flow and fixed before their commits landed.

## What was built

- `trace::{schema,emit,migrate}` — horismos.trace/1 as serde types (all four
  doc-09 step kinds are v1; reasoning emitted now), compact+pretty JSON with
  omitted-never-null optionals, byte-exact equality pinned; D-3's versioning
  policy written in `migrate`'s module doc with a rehearsed v1→v2 dry-run
  test and the pinned export tag set.
- `reason::matcher::ReteLite` — per-rule match memories + property-keyed
  invalidation; `MatcherKind`/`World::use_matcher`; differential tests
  (incl. randomized proptest) pin byte-identical traces and stores vs the
  naive oracle. D-1 resolved by the prescribed criterion run: wash at triage
  scale, 12.6× at chain-60 scale (naive breaches the 1 ms posture there);
  ReteLite is the default. Evidence: 03-benchmarks.md.
- Gate: `ImmutableRetraction` (D-6) and `DerivedFactRetraction` (D-5, with
  the Rule-provenance exception PRP-001 U7 pinned) checks + witnesses;
  sameAs-canonical oneOf/range lookups (D-7). Docs 02/03 markers struck;
  single-typed ruling recorded in doc 03.
- `tests/reasoning_dod.rs` — the PRP DoD end to end over triage + the
  imported SNOMED hierarchy; exact r1 proof render; trace round-trips and
  byte-equality; prove's failing and provable cases.
- `tests/inference_properties.rs` — proptest invariants: convergence under
  the cap, no re-fire, byte-identical traces across runs and across
  independently loaded ontologies.

## Named risks

1. **Trace v1 completeness**: perception/action/narration shapes are frozen
   from doc 09's example without a consumer yet; if PRP-004 needs a
   different field shape, that is a v2 under the D-3 policy. Mitigation:
   fields are near-all optional; additive-optional evolution stays legal.
2. **Provenance variant order is snapshot-wire-load-bearing** (postcard
   encodes indices). Now documented on the enum; nothing enforces it
   mechanically — a reorder would fail the round-trip tests, which is the
   pinning we have.
3. **ReteLite invalidation completeness**: correctness rests on "condition
   truth depends only on the subject's own property values and immutable
   class facts". The differential proptest is the guard; if a future
   operator reads other instances' mutable state, the invalidation index
   must widen.
4. **Deferred status surfaces** (PRP Status flip, BOARD, U-item boxes) —
   deliberate, blocked on PR #3's folder conversion; recorded in the ledger
   and to be called out in the PR body.
