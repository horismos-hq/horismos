# Benchmarks

The criterion benches in this directory (`cargo bench -p horismos-core`) and their
recorded runs. Everything below is a **measurement on one named machine, not a
product claim**: which number the docs may state, at which percentile, for which
surface and world size, is PRP-009 decision 2 (the captain's), and doc 14's
performance-posture rows are unchanged by this file.

| Bench | What it measures | Owner |
|---|---|---|
| `matching.rs` | Naive vs Rete-lite matching on triage and a synthetic 60-rule chain, first and second transaction | PRP-002 |
| `verification.rs` | Triage load + default verification | PRP-005 |
| `shield.rs` | The motion-command shield's per-decision cost (criterion means) | PRP-009 P1 |
| `shield_percentiles.rs` | The same decisions, timed one by one: p50 / p99 / max | PRP-009 P1 |
| `shield_common/` | The shared shield fixture (not a target) | PRP-009 P1 |

`scripts/bench_shield.py` is the Python twin of `shield_percentiles.rs`, through the
public `horismos` API.

## PRP-009 P1: shield decision latency (recorded 2026-09-23)

### Machine and build

| | |
|---|---|
| CPU | Apple M4, 10 cores, 16 GiB |
| OS | macOS 26.3.1 (Darwin 25.3.0), arm64 |
| Rust | rustc 1.97.1 (stable), cargo 1.97.1 |
| Python | CPython 3.13.13, uv 0.11.14 |
| Horismos | `0.1.0-alpha.4` (`0.1.0a4`), commit `c4c8429` plus these benches |
| Rust profile | `bench` (`cargo bench`: optimised, the release profile's settings) |
| Python module | `uv run maturin develop --uv --release`; the default `make develop` is a debug build and is **not** the measured path (its numbers appear once below, labelled, for contrast) |
| Load | `examples/gallery/safety-envelope.ttl`, fail-closed (`Ontology::load`), 9 rules |

Commands, in the order they were run:

```text
cargo bench -p horismos-core --bench shield_percentiles      # per-decision p50/p99/max
cargo bench -p horismos-core --bench shield                  # criterion means
uv run maturin develop --uv --release
uv run python scripts/bench_shield.py                        # Python, release module
```

The machine was not otherwise idle: a `cargo test --workspace` and later a `rustc` from
another worktree ran on other cores for parts of the session. Every timed program here
is single-threaded on a 10-core machine; the Rust percentile table was run twice, an
hour apart, and the second run's p50s were within 0.1 µs of the first on every row
(the max column moved, as a max does). The first run is the one recorded, except the
`fresh_subject` rows (both tables): a review found the first harness paying the cloned
world's interner copy-on-write inside the timed section (see the shape's description
below), so those rows were re-run on the same machine, same day, with the harness
fixed; the other rows of that re-run were within noise of the first, and the max
column again moved (`world_size/99` reached 1.586 ms).

### What one decision is

One transaction (doc 03): assert the `:MotionCommand`'s whole state in one call, and
inside that call the five-check gate runs and forward chaining reaches the fixed
point; then read `:verdict`. `explain` (the recorded derivation) is timed apart, never
inside a decision. Three shapes:

- `replace_standing`: the steady state. One standing command `:cmd` is re-asserted with
  the tick's state. The world's size is constant; each tick pays truth maintenance
  (doc 03 § Truth maintenance) for its own verdict.
- `fresh_subject`: a new anonymous command each tick into a cloned world (the world
  grows by one). The clone is set-up, outside the timed section, and so is the
  interner copy it would otherwise pay on its first intern: `World::clone` shares the
  `Arc<Interner>` and `Arc::make_mut` deep-copies every ontology IRI on the first
  write, a cost a real loop (which never clones) never sees, so the harness forces it
  in set-up (`shield_common::detached_clone`) and drops the clone after the clock is
  read. `cold_world` (criterion, Python) is the first transaction of a brand-new
  `World`, the shape PRP-002's `triage_transaction` measures.
- `world_size/N`: `replace_standing` with N further standing commands, each carrying a
  derived verdict that truth maintenance resets and re-derives on every transaction.

The three verdict paths are the Physical AI page's: open floor 2.4 m at 1.8 m/s →
`:Permitted`; shared cell 0.8 m at 1.2 m/s → `:Attenuated`; no proximity reading →
`:Blocked` via `noProximityReadingBlocks`. Each harness asserts the verdict it read.

### Rust, per decision (`shield_percentiles`)

Warm-up 1 000 untimed decisions, then each decision timed on its own with `Instant`.

| case | samples | p50 | p99 | max |
|---|---:|---:|---:|---:|
| `replace_standing/permitted_open_floor` | 20000 | 4.67 µs | 9.12 µs | 58.33 µs |
| `replace_standing/attenuated_shared_cell` | 20000 | 4.50 µs | 5.71 µs | 18.46 µs |
| `replace_standing/blocked_no_reading` | 20000 | 3.92 µs | 4.92 µs | 15.00 µs |
| `fresh_subject/permitted_open_floor` | 20000 | 6.29 µs | 6.67 µs | 19.21 µs |
| `fresh_subject/attenuated_shared_cell` | 20000 | 6.58 µs | 6.79 µs | 23.17 µs |
| `fresh_subject/blocked_no_reading` | 20000 | 5.75 µs | 5.96 µs | 42.08 µs |
| `explain/attenuated_shared_cell` | 20000 | below timer resolution (≈ 41 ns tick); criterion: 11.8 ns | | |
| `world_size/1` | 2000 | 6.75 µs | 8.79 µs | 22.08 µs |
| `world_size/10` | 2000 | 32.38 µs | 41.83 µs | 54.04 µs |
| `world_size/50` | 2000 | 166.29 µs | 188.38 µs | 254.33 µs |
| `world_size/99` | 2000 | 360.96 µs | 394.17 µs | 1.163 ms |
| `world_size/100` | — | refused | refused | refused |
| `world_size/1000` | — | refused | refused | refused |

### Rust, criterion means (`shield`)

Point estimate with criterion's 95 % interval, 100 samples per bench.

| bench | mean |
|---|---:|
| `shield_decision/replace_standing/permitted_open_floor` | 4.73 µs [4.65, 4.83] |
| `shield_decision/replace_standing/attenuated_shared_cell` | 4.44 µs [4.43, 4.46] |
| `shield_decision/replace_standing/blocked_no_reading` | 3.88 µs [3.85, 3.93] |
| `shield_decision/fresh_subject/permitted_open_floor` | 7.77 µs [7.72, 7.82] |
| `shield_decision/fresh_subject/attenuated_shared_cell` | 7.77 µs [7.72, 7.81] |
| `shield_decision/fresh_subject/blocked_no_reading` | 6.76 µs [6.71, 6.80] |
| `shield_decision/cold_world/permitted_open_floor` | 8.13 µs [7.70, 8.67] |
| `shield_decision/cold_world/attenuated_shared_cell` | 7.59 µs [7.46, 7.74] |
| `shield_decision/cold_world/blocked_no_reading` | 6.92 µs [6.82, 7.04] |
| `shield_explain/explain` | 11.8 ns [11.6, 12.1] |
| `shield_explain/prove` | 340 ns [339, 341] |
| `shield_world_size/1` | 6.70 µs |
| `shield_world_size/10` | 32.5 µs |
| `shield_world_size/50` | 171.8 µs |
| `shield_world_size/99` | 361.8 µs |
| `shield_world_size/100`, `/1000` | refused (below) |

### Python, per decision (`scripts/bench_shield.py`, release module)

Same shapes through `World.assert_` / `World.value`; `cold_world` builds the `World`
outside the timed section.

| case | samples | p50 | p99 | max |
|---|---:|---:|---:|---:|
| `replace_standing/permitted_open_floor` | 20000 | 8.04 µs | 8.67 µs | 78.38 µs |
| `replace_standing/attenuated_shared_cell` | 20000 | 8.08 µs | 8.42 µs | 22.42 µs |
| `replace_standing/blocked_no_reading` | 20000 | 6.92 µs | 8.62 µs | 166.75 µs |
| `cold_world/permitted_open_floor` | 20000 | 7.67 µs | 8.04 µs | 22.62 µs |
| `cold_world/attenuated_shared_cell` | 20000 | 7.75 µs | 38.92 µs | 1.292 ms |
| `cold_world/blocked_no_reading` | 20000 | 6.92 µs | 27.38 µs | 127.17 µs |
| `explain/attenuated_shared_cell` | 20000 | 4.04 µs | 7.21 µs | 136.04 µs |
| `world_size/1` | 2000 | 10.50 µs | 18.42 µs | 113.21 µs |
| `world_size/10` | 2000 | 36.08 µs | 47.00 µs | 108.67 µs |
| `world_size/50` | 2000 | 177.58 µs | 207.42 µs | 346.71 µs |
| `world_size/99` | 2000 | 369.42 µs | 710.75 µs | 1.294 ms |
| `world_size/100` | — | refused | refused | refused |
| `world_size/1000` | — | refused | refused | refused |

The Python decision costs roughly 3.5 µs over the Rust one (marshaling the state in and
the `Ref` out); `explain` costs 4 µs against 12 ns because the proof tree is marshaled
into Python objects. The Python max column carries interpreter pauses (the 1.29 ms
outliers) that the Rust rows do not.

### Reading the numbers

- **Steady state, one standing command:** 4–5 µs p50 in Rust, ≈ 8 µs in Python, with
  p99 under 10 µs on both and a max in the tens of µs (Rust) or, rarely, above 1 ms
  (Python, interpreter pauses). A fresh anonymous subject per tick costs ≈ 2 µs more
  than the standing-command shape in Rust (6–7 µs p50): the world grows by one
  instance, and the transaction re-derives the one extra standing verdict.
- **The world-size axis is linear and it has a ceiling.** Cost grows ≈ 3.6 µs per
  standing verdict, because every transaction re-derives every standing derivation
  (doc 03 § Truth maintenance). At **more than 100 standing derived verdicts the
  kernel refuses every transaction**: `reason::forward::ITERATION_CAP` is 100 and
  counts rule *firings* per transaction, admitting exactly 100 and refusing the
  101st; re-chaining 100 standing verdicts is 100 firings (the `world_size/99` row,
  99 padding verdicts plus `:cmd`), and one more verdict is one firing too many.
  The error is the gate's own, verbatim:

  ```text
  forward chaining exceeded 100 iterations without reaching a fixed point (last fired rule https://example.org/shield#permitOpenFloor); the transaction was rolled back
  ```

  For a control loop this decides PRP-009 decision 7: a fresh subject per tick reaches
  the ceiling on the 101st tick; the standing-command shape never does. Neither the
  cap nor truth maintenance is changed here (doc 04 owns the cap as a tripwire).
- **`replace_standing` cannot express a sensor dropout on its own.** Re-asserting a
  named instance *merges* properties (single-valued ones are replaced), so a tick that
  omits `nearestHumanDistance` keeps the previous reading; the no-reading `:Blocked`
  row uses a standing command that never had one. A live loop needs a `retract` (or a
  fresh subject) for the dropout tick; its cost is not measured here.
- The two docs-14 target rows this touches — gate + chain transaction < 1 ms, and
  `prove` / `explain` < 1 ms — are met at p50 and p99 by every measured row that the
  cap admits, Rust and Python, at up to 99 standing verdicts. They are **not** met at
  the max: the `world_size/99` max is 1.163 ms in Rust and 1.294 ms in Python, both
  over the gate + chain target, and Python's `cold_world` max is over it as well
  (1.292 ms, an interpreter pause). Whether any of this becomes a published number,
  and at which percentile, is decision 2.

## The "21 s at 55 rules" figure, attributed

PRP-009 cites an outside timing, not reproduced, of 21 s for a 55-rule ontology, and
names `reason::backward`'s depth-cap test as the candidate. That test builds a 55-rule
ladder (`MAX_DEPTH` + 5 rules, each concluding the flag the next one needs) with
`Ontology::load_from_str_allow_incomplete`, asserts one instance, and proves the top
flag; the search hits the depth cap and returns `None`.

Reproduced on this machine (`cargo test -p horismos-core --lib the_depth_cap_bounds_the_search`,
timed on a second run so no compilation is included):

| profile | test wall time |
|---|---:|
| debug (`cargo test`, the default, what `make test` runs) | 300.8 s (315.6 s on the first run) |
| release (`cargo test --release`) | 10.2 s |

So the figure is **not the decision, and not the depth-capped search**. A scratch
program (not checked in) that builds the same ladder and times its phases separately:

| phase, 55-rule ladder | release | debug |
|---|---:|---:|
| `load_from_str_allow_incomplete` (parse + verify) | 10.66 s | 302.3 s |
| `load_from_str` (fail-closed; the ladder verifies clean) | 10.38 s | 299.2 s |
| first assertion (gate + forward chain to fixed point) | 60 µs | 176 µs |
| `prove` of the top flag (backward, depth-capped, unprovable) | 24 µs | 148 µs |

The whole cost is **load-time verification**, and it grows steeply with chain length
(release, parse + verify only):

| rules in the ladder | 10 | 20 | 30 | 40 | 50 | 55 | 60 |
|---|---:|---:|---:|---:|---:|---:|---:|
| load + verify | 9.9 ms | 112 ms | 656 ms | 2.35 s | 6.52 s | 10.19 s | 15.27 s |

That is roughly quartic. `sample(1)` on the 60-rule run puts every sample inside
`verify::rules::verify_ontology`, and the shape matches `SymbolicSpace::established_rule_formulas`:
the symbolic firing fixed point restarts its scan from rule 0 on every update (mirroring
the runtime's restart-after-fire), recomputing every rule's standing formula each time,
so a chain of *n* rules costs O(n³) BDD operations on formulas that themselves grow with
the chain. The Python path is the same code: `Ontology.load` of the same ladder took
10.7 s through the release module and 296.3 s through the debug one, and its assert
and `prove` rounded to 0.000 s.

**Attribution:** the 21 s is the load-time verification of a ~55-rule *sequential
chain* — plausibly this test, or an ontology of that shape, on a release build of a
slower machine (this M4 does it in 10 s; a debug build takes 300 s, not 21). It is not
per-decision latency: the same ladder decides in 60 µs and proves in 24 µs once
loaded. Two caveats belong with it. First, it does sit inside doc 14's "~50 rules"
scale and misses the "load + verify < 1 s" target by an order of magnitude for
*this* shape (a long dependent chain); the shipped ontologies are not that shape.
Through the release Python module, 20 loads each: `safety-envelope.ttl` (9 rules) p50
3.53 ms, max 5.04 ms; `triage.ttl` p50 1.03 ms, max 1.92 ms. Second, `make test` runs
this test in debug on every run, at ~5 minutes on an M4. Neither is changed here: the
verifier is PRP-005's, and the test's cost is a `make test` concern outside P1's scope.
Both are reported to the PRP.

### The debug build, for contrast only

`make develop` builds the Python module without `--release`. The same script against
that build (5 000 samples, 500 for the sweep) is **not the measured path**; it is here
so nobody mistakes it for one:

| case (debug module) | p50 | p99 | max |
|---|---:|---:|---:|
| `replace_standing/permitted_open_floor` | 58.08 µs | 62.29 µs | 120.21 µs |
| `replace_standing/attenuated_shared_cell` | 59.12 µs | 63.71 µs | 109.12 µs |
| `replace_standing/blocked_no_reading` | 52.29 µs | 57.71 µs | 108.38 µs |
| `explain/attenuated_shared_cell` | 23.88 µs | 26.62 µs | 30.96 µs |
| `world_size/99` | 3.842 ms | 4.191 ms | 5.095 ms |

About 7× the release numbers per decision; the world-size ceiling is the same.
