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
- [ ] D5. **Retraction semantics for rule-derived facts**: cascade vs. forbid;
      doc 03's lean is forbid (derived facts are retracted only by re-running
      inference after base-fact changes). Owns doc 03's marker; recorded in
      doc 03 § Assertion and the validation gate.
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
