# PRP-002 — Decision record: truth maintenance for derived facts

Resolves D5 (`02-tasks-and-gaps.md`) and doc 03's truth-maintenance marker. Recorded
per `CONVENTIONS.md`'s "03-*.md — decision records" slot; the ruling itself was made
above this PRP (captain's decision, below) — this file records it and the
implementation approach chosen to satisfy it, per `.claude/skills/brainstorming`'s
evidence/options/resolution shape.

## The problem (falsifiable statement)

> A derived fact, once concluded, survives changes to the base facts that support
> it — a derived `:Immediate` acuity from `heartRate` 128 remains standing after
> `heartRate` is replaced by 96 or retracted.

**CONFIRMED**, by the independent plan-surface review (2026-08-18, top finding 2 of
`data/horismos-prp-review-s1/report.md`) and reproduced as a failing regression
before this fix (`crates/horismos-core/tests/truth_maintenance.rs`, all three cases
red against pre-fix code). Root cause: `assert_instance_as` re-chained without first
clearing stale derivations (so an untouched derivation just sat there unless some
rule happened to overwrite it), and `retract_as` never re-chained at all.

This contradicted doc 03/04's stated fixed-point promise and — because the PRP-002
tasks ledger's D5 still showed the retraction-cascade question as **open** even
after the code and docs had settled *part* of it (no direct retraction of a derived
fact) — the review separately flagged the ledger itself as stale (D2 in the review).

## The ruling

Captain's decision, 2026-08-18 (`data/horismos-prp-review-s1/decisions/tm-derived-facts.md`):

> Derived facts MUST be recomputed/invalidated when their supporting facts change or
> are retracted... Semantics: truth maintenance / recompute-on-base-mutation,
> matching doc 03's fixed-point promise. Not chosen: derivations-as-immutable-history.

This settles the *semantics*. Two things it does **not** change, both already
correctly decided by PRP-002 and left as-is: a caller still cannot retract a derived
fact directly from non-rule provenance (the remedy is to change its base-fact
support, which now genuinely works); and the rule path itself remains admitted to
retract what a rule asserted (PRP-001 U7).

## Options considered for the implementation approach

1. **Incremental TMS** — track a dependency graph (which base facts support which
   derivations) and invalidate only the reachable subgraph on a change. Correct and
   the eventual right answer at scale; explicitly out of scope for this task (the
   brief says do not build it here) — real engineering cost (a new dependency-edge
   data structure, its own invariants and tests) for a benefit not yet measured to
   be needed.
2. **Retract-derived + rechain (chosen)** — on every external transaction, after the
   requested base-fact change commits, reset every currently-standing rule-derived
   value and run the existing `forward::chain` fixed-point loop again from that
   base-only state. Simple: no new data structure, reuses the exact algorithm
   already proven correct for a fresh world. Cost: full-graph work every
   transaction, not just near the change (see Benchmarks below).
3. **Derivations-as-immutable-history** — stop retracting/replacing derived facts at
   all; treat every derivation as a permanent event, queryable but never removed.
   Rejected by the captain's ruling outright (doesn't match doc 03's promise that
   the *world* — as opposed to the *trace* — reflects current truth).

Option 2 was chosen. It is the approach the task brief itself names as sound, and it
is the smallest change that provably satisfies the ruling: correctness follows
directly from `forward::chain`'s existing fixed-point guarantee, not from new logic
whose own correctness would need separate proof.

## Implementation

`World::retract_all_derived` (`crates/horismos-core/src/world/mod.rs`) strips every
value `derivations` currently records — using `fact_meta`'s per-value `Provenance`
as the removal test, not `derivations` alone, so a multi-valued `horis:RulesOnly`
property does not leave earlier conclusions behind just because a later one
overwrote the one explanation `derivations` remembers. It is **not** keyed off "any
fact tagged `Provenance::Rule`": `assert_instance_as`/`apply_rule_conclusion` both
admit a directly-stated `Provenance::Rule` without that call being the reasoner's
own write path (exercised by existing tests), so a fact can carry the Rule tag
without being a derivation the gate itself would call "derived"
(`gate::check_retraction` keys its own refusal off `derivations`, not the tag) — the
two are related but not interchangeable.

`assert_instance_as` and `retract_as` both call `retract_all_derived` immediately
after committing the requested base-fact change and before building the matcher for
`forward::chain`, so the matcher (which scans the world fresh at construction) only
ever sees this transaction's base-only state. `retract_as` gained the same
transaction checkpoint/rollback `assert_instance_as` already had, because it can now
trip the iteration cap too. Public signatures are unchanged; `retract_as` newly
populates `World::last_reasoning` (previously always left `None` by a retraction) —
additive, not breaking.

## Regressions and properties (the evidence)

- `crates/horismos-core/tests/truth_maintenance.rs`: (a) replacing a supporting
  `heartRate` invalidates a stale `:Immediate`, re-deriving to the correct new fixed
  point; (b) retracting it removes the now-unsupported derivation outright; (c) a
  fact derived from a derived fact (`needsCardiacMonitor` from `:acuity
  :Immediate`) cascades away too. All three were red before this fix, green after.
- `crates/horismos-core/tests/truth_maintenance_properties.rs`: a bounded proptest
  over random replace/retract sequences on one subject's `heartRate` asserts the
  resulting world is a byte-identical `Snapshot` to a fresh world built from that
  same final base-fact set — the cleanest correctness statement, and the one this
  ruling's "must match a fresh world" framing asks for directly.
- Three existing backward-chaining tests (`reason/backward.rs`,
  `tests/reasoning_dod.rs` § f) had relied on "derive, then manually retract the
  conclusion with `Provenance::Rule`" to construct an *open goal* for `prove` to
  explore — a technique truth maintenance retires, since that retraction now
  immediately re-derives the same conclusion. Rewritten to use a same-priority-slot
  "spoiler" rule that legitimately outranks the rule under test, so its conclusion
  never lands in the *real* fixed point while its own conditions still genuinely
  hold — `prove`'s rule-search path (as opposed to the top-level "already holds"
  leaf shortcut) is now reached through states that are actually reachable via the
  public API, not manufactured ones.

## Benchmarks

`crates/horismos-core/benches/matching.rs` adds `triage_replace_transaction` and
`chain60_replace_transaction`: a *second* transaction (replacing a single-valued
base fact) against a world already at its fixed point, which now pays for
`retract_all_derived` plus a full matcher rebuild and re-chain — versus each
fixture's existing first-transaction benchmark, which is unaffected
(`retract_all_derived` is a no-op scan when nothing has been derived yet).
Local run, both matchers (naive is the differential oracle, Rete-lite is the
default — `MatcherKind`):

| Benchmark | naive | rete |
|---|---|---|
| `triage_transaction` (fresh, 1 rule, 1 patient) | 5.4 µs | 5.6 µs |
| `triage_replace_transaction` (2nd txn, same fixture) | 3.4 µs | 3.7 µs |
| `chain60_transaction` (fresh, 60-rule chain, 31 patients) | 1.66 ms | 132 µs |
| `chain60_replace_transaction` (2nd txn, same fixture) | 1.69 ms | 158 µs |

At triage scale the replace transaction is *cheaper* than a fresh one (one rule,
one prior derivation — the reset is trivial), both matchers well under the
"gate + chain transaction < 1 ms" posture (`docs/framework/14-architecture.md`).
At the synthetic 60-rule scale, Rete-lite's replace transaction costs about what a
*fresh* transaction already costs (~+20%) — because both the matcher rebuild and
the full re-chain already ran fresh on *every* external transaction before this
fix (`MatcherKind::build` has always been called once per transaction, not once
per world); the truth-maintenance reset adds a bounded, comparatively small
increment on top of work that was already being redone. Naive naturally absorbs it
in the noise (naive was already documented "over budget" at this synthetic scale
regardless of this fix — that is a pre-existing D-1 tradeoff, not new here).
Rete-lite, the shipped default, stays comfortably under budget with the fix.

**This is deliberately the "note it, don't build incremental TMS" outcome the task
brief asked for**: if a real workload's derivation graph grows large enough that
the per-transaction reset+rechain cost matters, the fix is option 1 above
(dependency-tracked invalidation), not micro-tuning this reset — tracked as a
future-optimization gap below, not built here.

## Gaps discovered during implementation

(mirrors `02-tasks-and-gaps.md`'s dated gap-note; recorded here too since it is
truth-maintenance-specific)

- 2026-08-19: `World::from_snapshot` restores `fact_meta` (so a restored fact's
  provenance survives) but not `derivations` (rule identity + proof aren't in the
  wire form — a pre-existing, deliberate PRP-001/PRP-002 scope boundary). Because
  `retract_all_derived` is keyed off `derivations` (see Implementation above, and
  why), a world resumed from a snapshot will not truth-maintain a fact that was
  genuinely rule-derived before serialization until *something* re-derives it for
  another reason. Not fixed here — it would need a v3 snapshot wire form carrying
  rule identity/proof, which is its own decision, not implied by this ruling.
- 2026-08-19: a multi-valued `horis:RulesOnly` property with several *different*
  standing rule-derived values only has the *last* one explained by `derivations`
  (`record_derivation` overwrites per `(subject, property)`) — pre-existing, not
  introduced by this fix (`World::explain` already had the same limitation). Truth
  maintenance still removes every rule-tagged value at that (subject, property)
  correctly (it filters `fact_meta` per-value, not per-`derivations`-entry), so this
  is an `explain()`-fidelity gap, not a truth-maintenance correctness gap.
- 2026-08-19: PRP-004's runtime/trace-append loop should confirm it calls
  `reasoning_step` after retraction transactions too, not only assertion ones —
  `World::retract_as` now populates `World::last_reasoning` with a real
  `ReasoningRecord` (previously always `None`), so a retraction's re-derivation is
  observable and traceable through the existing `Step::Reasoning` shape with no new
  trace-schema surface needed; this is a wiring note for PRP-004, not a schema gap
  (trace v1's Step::Reasoning already generically represents "a chaining pass
  happened," independent of what triggered it).
