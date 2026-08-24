# PRP-001 — Ontology Core (Rust)

**Status**: ✅ **Done** (2026-07-29) — D1–D6 resolved, U1–U11 built and green
under `make verify`; DoD run ships as `crates/horismos-core/tests/dod.rs`
(see `02-tasks-and-gaps.md`, incl. gaps handed to PRP-002/006)
**Depends on**: PRP-000

## Pivot note (2026-08-24) — bounded projection kernel

Captain-adopted pivot (`horismos-thin-pivot-s1/report.md`, PRP-001 row; PRP-000 §"Snapshot-
scoped guarantees"): this PRP's historical DoD and shipped tests are unchanged and remain the
record of what was built and verified. The durable description is reinterpreted, not
rewritten: `World` is a **transaction projection** — an admitted, versioned local snapshot
bound to one decision package — not the deployment's enterprise memory of record. `World`
gains no FHIR client and no network I/O; the interop story moves from `owl:sameAs`-only
anchoring to versioned standard mappings plus terminology closure (owned by PRP-008). PRP-008
adds a core-native, format-neutral `ExternalSnapshot`/batch-admission value carrying source/
version/mapping provenance on top of this crate's existing gate and store — that work,
including any new core types, is owned by PRP-008, not retrofitted here.

## Intent

The foundation crate: load, validate, and hold an RDFS-plus ontology; store typed
instances (the world); admit only validated mutations. Everything else — reasoning,
workflows, agents — builds on these types.

## Scope

- **RDFS-plus profile** (the formalism decision, made concrete here):
  - Base: RDF triples, RDFS classes/subclassOf/domain/range, Turtle syntax in/out.
  - Constraints: SHACL-inspired shapes — `horis:min/max`, `horis:unit`, cardinality,
    `horis:oneOf`, datatype enforcement.
  - Rules vocabulary: `horis:Rule` with `when/then/explain/priority` and the 11
    condition operators inherited from sigil-lang (exceeds, below, between, is, isNot,
    isAssigned, contains, inList, atLeast, atMost, isEmpty) — Datalog-shaped, keeping
    the Type-5 door open.
  - Workflow vocabulary: `horis:Workflow`, step classes, transitions (executed in
    PRP-004; *defined and validated* here).
  - `owl:sameAs`-based anchoring to external vocabularies (SNOMED, FIBO, schema.org);
    imports with prefix management.
- **Instance store (`World`)**: typed instance graph, assert/retract through the
  validation gate, snapshots/versioning, pattern queries (SPARQL subset).
- Turtle parser/serializer decision: evaluate `oxigraph`/`sophia` vs. minimal in-house
  (sigil-lang precedent favors owning the core; ecosystem crates are mature here —
  **decide early, benchmark both**).

## Non-goals

- No inference (PRP-002). No Python surface (PRP-003). No workflow execution (PRP-004).
- No full OWL DL — we take RDFS + named extensions only; reasoning stays tractable.

## Key decisions to make

1. Triple store representation: interned IRIs + index layout (port sigil-lang's
   string-interning experience from the VM chunk design).
2. Which SHACL subset exactly; how shape violations render in domain terms.
3. Snapshot semantics: persistent (immutable structural sharing, matching sigil-lang's
   frozen-AST philosophy) vs. copy-on-write.
4. External-vocabulary import strategy: full load vs. lazy subset extraction (SNOMED CT
   is ~350k concepts; we need subset extraction).

## Definition of done

`horismos-core` crate: loads `triage.ttl` + a SNOMED subset, rejects schema-violating
instances with domain-term errors, round-trips Turtle, snapshots the world, answers
pattern queries. Property-tested (proptest) against the validation gate. No LLM anywhere.
