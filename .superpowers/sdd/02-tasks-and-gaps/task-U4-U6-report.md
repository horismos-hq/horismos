# Implementer report — U4 (forward chaining), U5 (explain/render), U6 (prove)

Commits: `c0efc2f` (U4 + wire v2), `fd8f2fa` (U5), `88f21a7` (U6). `make verify`
green at each head. Briefs for these units are the ledger's unit-plan entries
(progress.md) — no separate brief files were written for B2/B3; treat the unit
plan text + controller pins + D-item statements as the brief.

## What was built

- `reason::forward` — doc 04's loop verbatim: one firing per pass, no-re-fire
  on (rule, subject), conflict policy (suppress at fire time; late
  higher-priority override re-marks the earlier firing `Overridden`), cap 100
  with whole-transaction rollback, conclusions through the full gate with
  `Provenance::Rule`, gate refusals recorded as `Rejected` diagnostics.
  Integrated into `World::assert_instance_as` between gate-commit and return.
- `reason::proof` — per-condition `ProofNode` (D-2), subsumption hops explicit,
  provenance on leaves, doc-04 `render` (pinned byte-exactly in U5 tests).
- Stored fact provenance (in-memory `fact_meta` + snapshot wire v2 with
  per-fact provenance; gate's restoring path re-gates each fact under its
  recorded provenance). PRP-001's pinned limitation test flipped to a
  round-trip success + a forged-provenance-cannot-smuggle test added.
- `World::explain` / `World::last_reasoning` / `World::fact_provenance` /
  `World::prove`.
- `reason::backward` — backward.py ported: rule index by conclusion match,
  `_proving` cycle set keyed (subject, property), MAX_DEPTH 50, hypothesis =
  cloned world mutated only through the gate.

## TDD evidence

Example-1 parity test written from the PRP's hand computation before the loop
was wired; first run of `a_gate_refused_conclusion_is_a_diagnostic_not_a_
rollback` FAILED (0 rejected vs 1 expected) — a genuine behavioural surprise
that exposed the conflict-policy/gate ordering question, resolved as a
recorded ruling (suppression shields the gate; test rewritten to the PRP's
actual Example-3 setup). The dod.rs suite failure after integration exposed
the wire-form provenance gap end-to-end (RulesOnly acuity broke snapshot
round-trip) and drove wire v2.

## Named risks (check these specifically)

1. **Conflict-policy ordering** (forward.rs): suppress-before-gate means a
   suppressed conclusion is never gate-checked at all. Is that defensible
   against doc 04 ("assert its conclusions through the gate")? My reading:
   only conclusions that survive conflict selection are proposed.
2. **`standing` map only tracks rule-set values** — a rule replacing a
   *base-fact* value in the same slot is plain gate-replace semantics, not a
   conflict. Confirm this matches doc 04's "two rules concluding …" wording.
3. **Rollback completeness** (assert_instance_as error path): instances,
   fact_meta, derivations, next_skolem, interner truncation — is anything
   else observable left behind? (`last_reasoning` deliberately keeps the
   previous committed record.)
4. **prove's hypothesis via `apply_rule_conclusion`** — it asserts on the
   REAL instance in a cloned world; check no path writes to the original
   (aliasing via Arc/imbl structural sharing is copy-on-write — verify no
   `Arc::make_mut` on shared state can leak).
5. **`concludes_goal` literal match uses `==`** (not numeric cross-typed
   value_eq) for literal conclusions — Integer 1 goal vs Decimal 1.0
   conclusion would not match. Defensible (conclusions are authored,
   goals should quote them) or a latent surprise?
6. **Wire v2 trust**: forged provenance claims — the new test pins
   value-invalid smuggling; is there a hole for provenance-only forgery that
   matters (e.g., a fact claiming Perception to look neural in proofs)?

## Out of scope

Traces/serde (U7), Rete-lite + benchmarks (U8), retraction rulings D-5/D-6
gate checks (U9), DoD property suite (U10).
