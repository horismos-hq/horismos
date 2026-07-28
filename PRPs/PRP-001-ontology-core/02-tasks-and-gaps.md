# PRP-001 — Tasks & Gaps

Working surface per `../CONVENTIONS.md`. Seeded 2026-07-28 at folder conversion;
becomes authoritative when the PRP activates.

## Decisions to resolve (before implementation)

- [ ] D1. **Turtle parser: `oxigraph` / `sophia` vs. minimal in-house.** Spike:
      benchmark loading `examples/triage/triage.ttl` + a synthetic ~350k-triple file
      through both crates; measure load time, memory, and how much of each crate we'd
      actually use. Record in `03-parser-spike.md`.
- [ ] D2. **Triple-store representation**: interned IRIs + index layout (port
      sigil-lang's string-interning experience from the VM chunk design).
- [ ] D3. **Which SHACL subset exactly**; how shape violations render in domain terms.
- [ ] D4. **Snapshot semantics**: persistent structural sharing (docs lean this way)
      vs. copy-on-write. Includes the binary snapshot format
      (docs/framework/03-world.md **[DECIDE]**).
- [ ] D5. **External-vocabulary import strategy**: full load vs. lazy subset
      extraction (SNOMED CT ≈ 350k concepts)
      (docs/framework/02-ontology.md **[DECIDE]**).
- [ ] D6. **Unit handling**: auto-convert compatible units vs. reject mismatches
      (docs/framework/02-ontology.md **[DECIDE]**).

## Units of work

- [ ] U1. IRI interning + term/value model (`Iri`, `Ref`, `Value` mirroring the XSD
      subset) — property-tested round-trips.
- [ ] U2. Turtle parse/serialize (per D1) — round-trips `triage.ttl` byte-stably
      modulo formatting.
- [ ] U3. RDFS-plus profile validation at load: reject out-of-profile constructs
      with domain-term errors.
- [ ] U4. Schema model: classes, subclassOf closure, property domain/range,
      constraint vocabulary — queryable.
- [ ] U5. Imports into named graphs + `owl:sameAs` anchoring + prefix management.
- [ ] U6. Instance store (`World`): assert/retract through the validation gate;
      the five gate checks in documented order; atomic rejection.
- [ ] U7. `sigil:assignedBy` provenance policy enforced at the gate
      (perception | rule | tool | api).
- [ ] U8. Snapshots + diff (per D4); `World::from_snapshot`.
- [ ] U9. Pattern queries (tier 1: `instances`, `value`); SPARQL subset deferred
      to its own unit behind the `sparql` feature.
- [ ] U10. Property-test suite against the gate (proptest): no sequence of valid
      operations produces an invalid world; every rejection names its constraint.
- [ ] U11. DoD run: load `triage.ttl` + a SNOMED subset fixture, reject
      schema-violating instances with domain-term errors, round-trip Turtle,
      snapshot, answer pattern queries. Zero LLM anywhere.

## Gaps discovered during implementation

(append dated notes here; scope changes go back into `01-objective.md` by explicit edit)
