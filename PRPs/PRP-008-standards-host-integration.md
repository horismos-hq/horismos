# PRP-008 — Standards & Host Integration

**Status**: Draft (new, 2026-08-24 — captain-adopted bounded-projection pivot)
**Depends on**: PRP-001, PRP-002, PRP-004, PRP-005 (consumes all four; preserves every
core invariant they established — no changes to `horismos-core`'s gate, chaining, or
determinism semantics, only new adapter-side types and crates around it).
**Sequencing**: dependency DAG, not numeric — activates **after PRP-005** (the verification
kernel it needs) and the current PRP-004 branch land, **before PRP-006** (packaging — needs
a real integration surface to document) **and PRP-007** (authoring — its default job needs
PRP-008's mapping/package contracts to overlay against). Numbering stays sequential per
`PRPs/CONVENTIONS.md`; this PRP does not renumber 005–007.

## Origin

Captain ruling (`decisions/captain-pivot-ruling.md`, item 2 — `sequence-prp008-before-ship-
author`), adopting `horismos-thin-pivot-s1/report.md`'s recommended sequence and its
"PRP-008 — Standards & Host Integration" row in the PRP-by-PRP change map. See
`PRPs/PRP-000-vision-positioning.md` §"Snapshot-scoped guarantees" for the formal claim this
PRP exists to make concrete, and `docs/framework/03-world.md` for the World-as-projection
mechanism it builds adapters around.

## Intent

Own everything the bounded-projection pivot requires that is *not* a change to
`horismos-core`'s reasoning kernel: hydrating a versioned, explicitly scoped external
bundle into a local World projection, validating and conditionally committing typed
proposed effects back to the system of record, and the generic host-adapter surface that
lets both FHIR/CIS deployments and coding-harness deployments plug into the same kernel. A
credible prototype against **one vertical slice** is the bar — not certification, not
hospital deployment, not full CQL/PlanDefinition coverage. VTE prophylaxis is the source
report's recommended first slice; which scenario (VTE / NEWS2 / sepsis-compliance) is
actually built first is still open, not preempted by this pass — a separate captain call
(see `PRPs/BOARD.md`).

## Reference data flow (unchanged from the adopted design)

```text
FHIR/CIS or repo/harness
  -> adapter fetches declared closure + versions
  -> canonical ExternalSnapshot / ImportBatch
  -> World admission gate (all-or-nothing)
  -> fixed-point reasoning + local workflow fragment
  -> proof + trace + ProposedEffectSet
  -> adapter re-checks source versions
  -> conditional transaction/tool commit
  -> commit receipt or conflict/failure appended to trace
```

No adapter performs a network read from inside matching, chaining, verification, or a
`World` transaction — this preserves the anti-wrapper boundary
(`docs/framework/14-architecture.md`) and the whole-input/whole-outcome boundary
(`docs/framework/14-architecture.md`, `.claude/skills/pyo3-boundary`) exactly as built.

## Scope

- **FHIR hydration/mapping/conditional commit**: an adapter (new crate, e.g.
  `horismos-fhir`, outside `horismos-core`) that reads a declared, versioned resource
  closure from a FHIR server into a canonical `ExternalSnapshot`, maps it to ontology
  facts with recorded mapping-loss/totality reporting, and conditionally commits a
  `ProposedEffectSet` back (transaction Bundle / conditional-update where the server
  supports it), failing closed on source-version drift.
- **Terminology provenance**: canonical code systems, versions, ValueSets/ECL, and
  mappings — replacing `owl:sameAs`-only anchoring (PRP-001's current, insufficient
  interop story per `research/terminology/snomed-ct.md`) with versioned, polyhierarchy-
  preserving provenance carried through proof and trace.
- **CDS-Hooks-style service surface**: Horismos as the invoked decision service behind a
  CDS-Hooks-shaped contract — hook fires, Horismos evaluates a projection, a
  recommendation/card (or proposed FHIR resource) returns. The host keeps owning the hook
  lifecycle and UI.
- **Bounded PlanDefinition importer/executor**: a deliberately narrow, explicitly listed
  supported construct subset; anything outside it fails at load with a domain-term report,
  never a silent approximation.
- **CQL composition story**: compose with an external CQL evaluator (consume its typed
  results as inputs) rather than building a second CQL engine; native compilation stays a
  non-goal pending evidence.
- **Generic host adapter SPI**: the minimum format-neutral `ExternalSnapshot` and
  `ProposedEffectSet` types (core-native, no FHIR/HTTP dependency) that both a FHIR adapter
  and a coding-harness adapter implement against — this is what keeps FHIR-specific and
  repository-specific code out of `horismos-core`.
- **The candidate-artifact boundary** (captain-adopted design, ruling item 3): PRP-008
  owns the **host-adapter/runtime side**. A model-generated patch/file/command proposal is
  a typed envelope; deterministic policy validates target paths, base hashes, syntax, and
  permissions; a sandbox executes required tests/checks; only then does a symbolic
  authorization step permit application. Semantic correctness of the generated content
  remains a review/test claim, never a gate theorem — the gate proves the *proposal* was
  policy-legal and passed its declared checks, not that the code is right. PRP-007 mints
  the envelope's ontology vocabulary jointly (see PRP-007's pivot note); this PRP builds
  the validation/sandbox/authorization pipeline that consumes it.
- **Coding-harness adapter (second domain proof, not clinical generalization)**: after the
  generic host SPI exists, a prototype adapter materializing repository state (branch,
  dirty files, test status, path scopes) as a versioned World projection, proving the SPI
  is genuinely format-neutral rather than FHIR-shaped in disguise.

## Non-goals

- **Not a FHIR wrapper.** Horismos does not become a live view over a FHIR server —
  `World` never gets a network client; every read is materialized into a closed,
  versioned snapshot before core ever sees it (doc 03, doc 14 — unchanged).
- **Not distributed atomicity.** No cross-system atomic-commit claim. The trace
  distinguishes "decision produced," "commit attempted," "commit accepted," and
  compensation/failure as distinct, separately observable states.
- **Not a second CQL engine** until composing with an existing evaluator is demonstrated
  insufficient — building one is explicitly deferred pending evidence (D7 below).
- **Not full PlanDefinition coverage.** The bounded subset is a permanent v1 design
  position, not a placeholder for "eventually everything."
- **Not certification, hospital deployment, or a production-grade terminology service** —
  this PRP's bar is a credible prototype on one vertical slice.
- **No semantic-correctness gate theorem for candidate artifacts.** The candidate-artifact
  boundary proves policy compliance and passing declared checks; it does not and cannot
  prove the generated content is semantically correct — that stays a review/test claim,
  stated explicitly wherever the boundary is described (doc 06, doc 07, doc 10).

## Required decisions (D-items — resolved at activation, not smuggled into code)

Ten D-items carried from the adopted design report, each needing the repo's brainstorming
protocol (`.claude/skills/brainstorming`) before implementation, never decided ad hoc:

1. **FHIR baseline and deployment profile(s)**: R4/R4B/R5, US Core or another national
   base, resource closure, and what is merely accepted vs. conformance-tested.
2. **Mapping representation**: generated mapping code, declarative maps, FHIR RDF,
   StructureMap, or a deliberately smaller owned mapping profile; include round-trip/loss
   reporting either way.
3. **Source consistency**: exact resource-version vector, transaction Bundle/conditional-
   update requirements, conflict retry policy, and which servers cannot support them.
4. **Closed-world boundary**: what exact search/bundle constitutes complete input per
   decision; missing / not-authorized / not-fetched / absent must remain distinct states,
   never collapsed into one (feeds PRP-005's input-closure declaration and assumption
   manifest).
5. **FHIR output posture**: recommendation-only CDS Hooks cards vs. proposed FHIR
   resources vs. authorized conditional writes.
6. **PlanDefinition subset and fail-closed behavior**: the explicit list of supported vs.
   unsupported constructs.
7. **CQL boundary**: external evaluator results as typed inputs vs. any supported direct
   compilation (default: compose, do not rebuild — see Non-goals).
8. **Terminology service/build-time expansion policy**: versions, licenses, and offline
   reproducibility.
9. **Trace schema evolution**: source vectors, mapping/package identity, and remote commit
   receipts — a new `horismos.trace/N` version if these fields are not additive under
   `trace::migrate` (doc 09's versioning policy is unchanged and binding here).
10. **Generic host SPI**: the minimum format-neutral `ExternalSnapshot`/`ProposedEffectSet`
    shape that keeps FHIR and coding-harness clients out of `horismos-core`.

Plus the vocabulary D-item shared with PRP-007 (its Key decision 5): the candidate-artifact
envelope's `horis:` terms, minted jointly, not by either PRP alone.

## Definition of done

Below, "the vertical slice" is written against **VTE prophylaxis** (per the report's
recommended sequence — structured EHR inputs, bounded scoring/contraindications, typed
recommendation/order proposal, published prior art) as the illustrative first candidate.
Which scenario (VTE / NEWS2 / sepsis-compliance) is actually built is still open, not
preempted by this pass — a separate captain call (see `PRPs/BOARD.md`); the DoD below
applies to whichever scenario is chosen. The vertical slice runs end to end:

- one declared FHIR input closure hydrates atomically into an `ExternalSnapshot`;
- resource/profile/terminology/mapping versions appear in the proof and the trace;
- a conflicting external update (source version drifted since hydration) aborts the
  commit — verified by a fixture that mutates the source between hydration and commit;
- a CDS Hooks-style invocation returns a recommendation or proposed FHIR resource;
- the same decision runs standalone from a recorded Bundle with **zero network, zero
  model** — the anti-wrapper/no-model-gate discipline (doc 14) extends to the adapter path,
  not just the core;
- unsupported or missing data fails in domain terms, never a silent guess;
- the candidate-artifact boundary's policy-validation → sandbox-test → symbolic-
  authorization pipeline runs against at least one seeded model-generated patch proposal,
  both an accepted and a rejected case;
- the generic host SPI is exercised by two independent adapters (FHIR and a coding-
  harness prototype) proving it is genuinely format-neutral, not FHIR-shaped.

`make verify`, `make check-boundary`, and `make no-model-gate` stay green throughout —
this PRP adds crates and adapter code, never a `horismos-core` dependency on the network or
a provider SDK.
