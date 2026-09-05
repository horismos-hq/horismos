# PRP-002 — Tasks & Gaps

Working surface per `../CONVENTIONS.md`. Seeded 2026-08-05 at folder conversion
(the teaching-grade objective outgrew one page); becomes authoritative when the
PRP activates. All decisions below are **open** — resolving them (via
`.claude/skills/brainstorming`, never ad hoc) is the activation gate; unit
shapes may be adjusted by those resolutions in the same commit that records
them. Expanded context for D1–D4 lives in `01-objective.md` § Key decisions.

## Decisions to resolve (before implementation)

- [ ] D1. **Rete-lite vs. naive match-loop first**: which matcher ships as the
      primary strategy, decided by criterion benchmarks (naive vs. Rete-lite
      alpha memories keyed on class + property) on the triage fixture and a
      larger synthetic ontology; the loser stays in-tree as the parity oracle.
      Recorded here and in the U10 unit shape once resolved.
- [ ] D2. **Proof-tree granularity**: per-condition `ProofNode`s (sigil-lang
      `backward.py` precedent; doc 04's rendered form) vs. per-rule nodes.
      Recorded here; doc 04's ProofNode section rewritten in shipped voice if
      the shape changes.
- [ ] D3. **Trace schema versioning policy**: what counts as breaking for
      `horismos.trace/1`, where converters live and how they are tested, how
      the reserved `loss_mask`/outcome-label fields may evolve; includes the
      doc-09 training-export tag set marker
      (`[DECIDE: exact tag set — PRP-002]`, doc 09). Recorded as a policy
      section in doc 09; the marker is deleted in the same commit.
- [ ] D4. **Negation semantics**: re-validate the pragmatic closed-world stance
      for the graph world and confirm ∃ vs. ∀ for multi-valued `horis:isNot`
      (must stay coherent with `horis:none` and the unassigned-property rows).
      Owns doc 04's `[DECIDE: confirm ∃ vs. ∀ …]` and doc 02's
      `[DECIDE: PRP-002]` closed-world marker — both deleted and rewritten in
      shipped voice on resolution; semantics pinned by U2's test matrix.
- [x] D5. **Retraction semantics for rule-derived facts**: resolved two ways.
      A caller still cannot retract a derived fact *directly* from non-rule
      provenance (forbid, as originally leaned) — but the base-fact side was
      previously ambiguous ("change only by re-running inference" did not
      specify that re-running inference *invalidates* what no longer holds,
      and the shipped code didn't). Captain's ruling, 2026-08-18
      (`data/horismos-prp-review-s1/decisions/tm-derived-facts.md`, prompted
      by the independent review's top finding 2): **recompute-on-base-
      mutation** — every external transaction resets and re-chains derived
      facts to the fixed point of its final base facts, cascading through
      chained (derived-from-derived) support. Not chosen:
      derivations-as-immutable-history. Recorded in doc 03 § Truth
      maintenance and `03-decisions.md` (full rationale, options considered,
      benchmark pointer).
- [ ] D6. **Retraction vs. `horis:immutable`**: retract-then-assert currently
      replaces an immutable value in two steps (PRP-001 gap, 2026-07-29). Owns
      the markers in doc 02 and `world/gate.rs`; recorded in doc 02's
      constraint table and enforced at the gate.
- [ ] D7. **sameAs-canonical class lookup**: anchored external IRIs are
      second-class as values (`Schema::class()` exact-IRI vs. sameAs-canonical
      subsumption — spurious rejects only; PRP-001 gap, 2026-07-29). Needs a
      conscious ruling before the matcher crosses anchors; recorded here and
      on the schema-lookup API docs.

## Units of work

- [ ] U1. Rule model + load-time validation: parse `horis:Rule`
      (`when`/`then`/`explain`/`priority`; the 11 operators, `all`/`any`/`none`)
      into typed conditions; reject malformed rules and unit-mismatched
      comparisons (PRP-001 D6 hand-off) with domain-term errors — `triage.ttl`
      `:r1` loads; a `heartRate` comparison in anything but `"bpm"` is a
      load-time error naming the rule.
- [ ] U2. Condition evaluation over the World: all 11 operators + combinators
      with doc 04's edge semantics as a table-driven test matrix (unassigned,
      multi-valued, `isAssigned`, `isNot` per D4's resolution).
- [ ] U3. Subsumption-aware matching: `horis:is` matches the `subClassOf`
      closure including imported, `owl:sameAs`-anchored hierarchies (per D7);
      each hierarchy hop reportable as an explicit subsumption step —
      `is :ChestPain` matches a `:CrushingChestPain` instance from the SNOMED
      fixture.
- [ ] U4. Naive match-loop: deterministic (rule, bindings) candidate
      enumeration over the graph — the parity oracle regardless of D1's
      outcome; candidate sets for the triage fixture asserted exactly.
- [ ] U5. Forward chaining to fixed point: priority selection, rule-IRI
      lexicographic tie-break, fired-set no-re-fire, conflict suppression
      recorded, iteration cap, derived facts asserted through the gate with
      provenance `rule` — reproduces `01-objective.md` Example 1 exactly
      (4 passes, one suppression).
- [ ] U6. Backward chaining `prove(goal)`: rule index by concluded property,
      recursive condition proving, cycle detection, depth cap 50 (sigil-lang
      `backward.py` port) — reproduces Example 2 including the unprovable
      case.
- [ ] U7. `explain(fact)` + `ProofNode::render()`: recorded derivations from
      the forward-chaining trace, rendered in doc 04's indented format —
      the r1 proof tree renders byte-identically to the documented form.
- [ ] U8. Constraint checking as continuous invariant: rule-derived violations
      surface as domain-term diagnostics, transactions stay atomic (doc 03) —
      Example 3's `oneOf` witness text asserted.
- [ ] U9. Traces: typed reasoning steps in `horismos.trace/1` (fired,
      suppressed, proofs, `loss_mask`, per-fact provenance); stored fact
      provenance travels the wire form (PRP-001 gap —
      `a_rules_only_fact_does_not_survive_the_wire_form_yet` flips to
      surviving); JSON export, compact and pretty, per D3's policy.
- [ ] U10. Second matcher strategy per D1 (Rete-lite alpha memories keyed on
      class + property, or deferral recorded): differential property tests
      against the U4 oracle; criterion benchmarks naive vs. Rete-lite — the
      DoD's benchmark evidence.
- [ ] U11. Property-test suite: determinism (same ontology + snapshot +
      transaction → byte-identical traces; trace equality as assertion),
      fixed-point invariance (re-chaining a committed world derives nothing),
      gate invariants under `rule` provenance.
- [ ] U12. DoD run: triage scenario end-to-end symbolically — structured facts
      in → subsumption + forward chaining → `acuity` derived → `explain`
      renders the r1 proof; traces serialize to the versioned schema; parity
      against the hand-computed fixed points of `01-objective.md`. Zero LLM
      anywhere.

## Gaps discovered during implementation

(append dated notes here; scope changes go back into `01-objective.md` by explicit edit)

- 2026-08-19 (truth maintenance, D5): full detail in `03-decisions.md` §
  Gaps. Summary: (1) `World::from_snapshot` doesn't restore `derivations`
  (rule identity/proof aren't in the wire form), so a resumed world won't
  truth-maintain a fact that was rule-derived before serialization until
  something else re-derives it — would need a v3 snapshot wire form, a
  separate decision, not implied by this ruling. (2) A multi-valued
  `horis:RulesOnly` property with several distinct standing rule-derived
  values only has its *last* one explained by `World::explain`
  (`derivations` overwrites per (subject, property)) — pre-existing, not a
  truth-maintenance correctness gap (removal still catches every rule-
  tagged value via `fact_meta`). (3) PRP-004's runtime/trace loop should
  call `reasoning_step` after retraction transactions too, now that
  `retract_as` populates `World::last_reasoning` — no new trace-schema
  surface needed, `Step::Reasoning` already covers it generically.
- 2026-08-19: the retract-derived-and-rechain approach does full-graph work
  on every external transaction (not just ones touching a derivation), by
  design — see `03-decisions.md` § Options considered. If
  `benches/matching.rs`'s `triage_replace_transaction` numbers show this is
  heavy at a scale the project cares about, the fix is dependency-tracked
  incremental invalidation (a real TMS), deliberately not built in this
  task — future-optimization gap, not a defect.
- 2026-09-05 **[DECIDE]** (proof attribution when rules agree): the forward
  chainer fires every matching rule in priority order, and
  `World::record_derivation` (`crates/horismos-core/src/world/mod.rs:640`)
  replaces the record per (subject, property) on each firing, so when two
  rules conclude the *same* value for a single-valued property the proof
  `explain()` cites is the *lowest*-priority rule's — the rule named is
  effectively arbitrary whenever rules agree. Surfaced by the gallery
  completeness work (branch `fm/horismos-gallery-completeness`): the change
  gate's first cover rule overlapped `:protectedNeedsHuman` and took over its
  proof on every protected-path proposal while `verify()` passed, because
  equal values are not a conflict. This is the single-valued same-value case
  of the `derivations` overwrite recorded in note (2) of 2026-08-19 above.
  It is a gap in the headline guarantee — the proof is meant to name the
  rule that decided — not a docs problem; disjoint covers avoid it overlay
  by overlay and nothing enforces them. Relabelling each same-valued rule to
  a distinct value and re-running `verify()` shows the shipped examples
  already carry such overlaps among their original rules (change gate:
  `rejectStaleBase`/`rejectFailingChecks`, `approveRoutineChange`/
  `approveReviewedChange`, `protectedNeedsHuman`/`largeBotDiffNeedsHuman`;
  claims: `denyInactivePolicy`/`denyNonCoveredService`,
  `pendMissingPriorAuth`/`pendHighDollar`), so a routine change that a human
  also approved is explained by the priority-10 rule, not the priority-20
  one. To decide here: whether a same-value
  re-derivation must leave a standing higher-priority proof in place, or
  the record should carry every supporting rule. No design or
  implementation in this note.
