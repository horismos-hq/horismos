# PRP-002 U8 — matching benchmarks (D-1 evidence)

Recorded 2026-08-07 · `cargo bench -p horismos-core --bench matching` ·
Apple Silicon dev machine, release profile, criterion 0.7.
(Transfer into `PRPs/PRP-002-reasoning-substrate/` as `03-benchmarks.md` when
the folder-form PR lands.)

| Benchmark | Naive | Rete-lite | Ratio |
|---|---|---|---|
| `triage_transaction` (flagship fixture: 1 rule, one patient asserted, full gate+chain) | 5.30 µs | 5.47 µs | 0.97× (wash) |
| `chain60_transaction` (60-rule sequential chain, 31 instances, one assertion lights the fuse) | **1.60 ms** | **126.9 µs** | **12.6×** |

Reading:

- At triage scale the strategies are indistinguishable and both sit ~200×
  under doc 14's "gate + chain transaction < 1 ms" posture; the Rete build
  cost (one naive-cost pass + memory upkeep) is noise.
- At the synthetic scale the naive loop **breaches the 1 ms posture**
  (re-scanning 60 rules × 31 subjects each of ~61 passes) while Rete-lite
  stays 8× under it — each firing changes one (subject, property), so the
  invalidation index re-checks exactly one rule for one subject.
- Decision (D-1): default `MatcherKind::ReteLite`; `NaiveMatcher` stays
  selectable (`World::use_matcher`) as the differential-testing oracle,
  pinned by `reason::matcher::differential_tests` (byte-identical traces
  and stores under both strategies, including a randomized proptest).
- Doc 14's "Rete-lite → full-Rete decision point is data-driven" stays open
  for a future phase; nothing measured here motivates beta joins (v1 rules
  bind one subject — there is nothing to join yet).
