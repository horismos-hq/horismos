# PRP-001 — Tasks & Gaps

Working surface per `../CONVENTIONS.md`. Seeded 2026-07-28 at folder conversion;
becomes authoritative when the PRP activates.

## Decisions to resolve (before implementation)

- [x] D1. **Turtle parser: `oxigraph`.** Decided 2026-07-28 (human call, spike
      waived). We consume oxigraph's parsing/serialization (`oxttl`/`oxrdf`), not its
      store — our interned triple store (D2) stays ours. Load-time/memory validated
      empirically in U2; if oxigraph proves heavy there, the fallback is its parser
      crates alone, not a re-litigation of this decision.
- [x] D2. **Store representation** (decided 2026-07-28): `u32`-newtype interned IRIs
      in an arena; instance data as adjacency maps (subject → property → values),
      matching gate/rule access patterns. No generic SPO/POS/OSP indexes up front —
      secondary indexes arrive benchmark-driven with PRP-002's matcher.
- [x] D3. **Constraints** (decided 2026-07-28): the documented `horis:` vocabulary
      exactly (doc 02) — SHACL-inspired, not SHACL-conformant. Violations render per
      the doc-03 template + machine-readable structure. SHACL export mapping deferred
      to PRP-006 interop.
- [x] D4. **Snapshots** (decided 2026-07-28): persistent structural sharing
      (HAMT-style, `im` crate or equivalent) — O(1) `snapshot()`, structural `diff()`.
      Binary form: postcard-serialized with a versioned header.
- [x] D5. **Imports** (decided 2026-07-28): pre-extracted subset files only in v1 —
      a subset `.ttl` is an ordinary import into a named graph; no I/O in the core,
      no lazy loading. Extraction tooling belongs to PRP-007. U11 ships a ~50-concept
      chest-pain SNOMED subset fixture.
- [x] D6. **Units** (decided 2026-07-28): strict — units are opaque labels, exact
      match required between rule conditions and property declarations; mismatch is a
      load-time verification error in domain terms. No auto-conversion in v1.

## Units of work

- [x] U1. IRI interning + term/value model (`Iri`, `Ref`, `Value` mirroring the XSD
      subset) — property-tested round-trips.
- [x] U2. Turtle parse/serialize (per D1) — round-trips `triage.ttl` byte-stably
      modulo formatting.
- [x] U3. RDFS-plus profile validation at load: reject out-of-profile constructs
      with domain-term errors.
- [x] U4. Schema model: classes, subclassOf closure, property domain/range,
      constraint vocabulary — queryable.
- [x] U5. Imports into named graphs + `owl:sameAs` anchoring + prefix management.
- [x] U6. Instance store (`World`): assert/retract through the validation gate;
      the five gate checks in documented order; atomic rejection.
- [x] U7. `horis:assignedBy` provenance policy enforced at the gate
      (perception | rule | tool | api).
- [x] U8. Snapshots + diff (per D4); `World::from_snapshot`.
- [x] U9. Pattern queries (tier 1: `instances`, `value`); SPARQL subset deferred
      to its own unit behind the `sparql` feature.
- [x] U10. Property-test suite against the gate (proptest): no sequence of valid
      operations produces an invalid world; every rejection names its constraint.
- [x] U11. DoD run: load `triage.ttl` + a SNOMED subset fixture, reject
      schema-violating instances with domain-term errors, round-trip Turtle,
      snapshot, answer pattern queries. Zero LLM anywhere.

## Gaps discovered during implementation

(append dated notes here; scope changes go back into `01-objective.md` by explicit edit)

- 2026-07-29 — **wasm32 build of core is broken by transitive `getrandom`**
  (oxrdf's blank-node ids, imbl's RandomState). `make verify` does not gate
  wasm today; the wasm-gate CI job is PRP-006's, and the candidate fix there is
  getrandom's `wasm_js` backend. Parked, not load-bearing for PRP-001/002.
- 2026-07-29 — **retract-then-assert defeats `horis:immutable`.** Deliberately
  not decided in code; marked `[DECIDE: retraction vs horis:immutable —
  PRP-002]` in `world/gate.rs` and doc 02, alongside PRP-002's existing
  retraction-semantics decision.
- 2026-07-29 — **`owl:sameAs`-anchored IRIs are second-class as values**:
  `Schema::class()` is exact-IRI while subsumption is sameAs-canonical, so an
  anchored external IRI is accepted as a *class* but its sameAs twin is the
  one the gate's `oneOf`/range logic sees. Spurious rejects only (never
  accepts); needs a conscious ruling when queries start crossing anchors —
  owned by PRP-002 alongside the reasoner's subsumption work.
- 2026-07-29 — **xsd:dateTime requires an explicit timezone offset** (the
  `chrono::DateTime<FixedOffset>` pin). Determinism-friendly; revisit only if
  a fixture needs offset-less local times.
- 2026-07-29 — **instances are single-typed** (`ClassConflict` on a second
  `rdf:type`): a modeling ruling made at the gate, to be recorded in doc 03's
  next edit pass (PRP-002 touches doc 03/04 anyway).
- 2026-07-29 — **fact provenance is not stored** (only checked at the gate),
  so `from_bytes` re-validates everything as `Provenance::Api` — a snapshot
  holding a `RulesOnly` fact will not restore until fact provenance travels
  with the wire form. Owned by PRP-002 (traces record provenance anyway).
  Pinned by `a_rules_only_fact_does_not_survive_the_wire_form_yet` and
  documented on `Snapshot::from_bytes`.
- 2026-07-29 (final review) — **`Iri` tokens carry no world provenance**: a
  token minted by one world, used against another, resolves to a silently
  wrong IRI whenever the index is in range (documented on
  `Interner::resolve`). Mitigated for snapshots by `from_snapshot`'s
  interner-prefix check; the general question is
  **[DECIDE: `Iri` world-provenance tagging — PRP-003]** (the PyO3 surface
  makes cross-world tokens easy to produce from Python).
