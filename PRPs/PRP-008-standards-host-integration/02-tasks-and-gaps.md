# PRP-008 — Tasks & Gaps

## Run record

- Plan: `PRPs/PRP-008-standards-host-integration/01-objective.md`
- Branch: `fm/horismos-prp008-a1`
- Worktree: `/Users/billaram/.treehouse/horismos-86ae5f/3/horismos`
- Baseline: `5b87889bc723bd22cf5b9476ed87034f151b4d8f`
- Baseline gate: `make develop && make verify` green on 2026-09-02 (the initial
  `make verify` exposed only a stale local extension, then the Makefile rebuild restored
  the source-clean baseline).
- Pacing: six dependency-ordered units, approved by the captain's launch brief. PRP-005
  remains a separate parallel lane; no PRP-005 file or verifier implementation is touched.
- Task U1: complete (commit `9e8147c`, review Approved with minors; one coherent-fixture
  minor carried to U3).
- Task U2: complete (commit `8d3f403`, senior review Approved after two hardening passes).
- Task U3: complete (commit `616a79b`, senior review Approved after fail-closed and
  conditional-write hardening).
- Task U4: complete (commit `eac7d67`, senior review Approved after evaluator/envelope/
  linkage hardening).
- Task U5: complete (commit `576915d`, senior review Approved after patient/evidence/
  artifact integrity hardening).
- Task U6: complete (commit `2dd771e`, senior review Approved with no findings).
- Final review: Approved with two documentation minors, both closed before the completion
  commit (runtime evidence term count and foundational-skeleton historical wording).
- Final validation: `make verify` green on 2026-09-03, including `make check-boundary`
  and `make no-model-gate`.

## Decisions resolved before implementation

- [x] D1. **FHIR baseline and deployment profiles** (decided 2026-09-02): accept FHIR
  R4 4.0.1 JSON on the wire and conformance-test the VTE fixture against an explicitly
  named local R4 profile set; import only an R6-shaped PlanDefinition subset. No US Core
  certification claim is made. Fallback: add an adapter-local R4B/R5 translation layer,
  never widen the kernel or silently accept constructs.
- [x] D2. **Mapping representation** (decided 2026-09-02): use an owned, deterministic,
  RDF-aligned mapping profile compiled to typed path extractors; reject modifier extensions
  and report every ignored/unsupported path as mapping loss. StructureMap is not a runtime
  dependency. Fallback: exchange the extractor implementation behind the same loss-report
  contract if a normative mapping standard stabilizes.
- [x] D3. **Source consistency** (decided 2026-09-02): hydrate an exact resource-version
  vector, probe `CapabilityStatement` at adapter construction for `versioned-update` and
  transaction support, normalize strong and weak ETags to version identity, and classify
  409/412 alike as drift. No automatic conflict retry. Fallback: recommendation-only mode
  on servers that cannot prove versioned conditional writes.
- [x] D4. **Closed-world boundary** (decided 2026-09-02): a decision declares its complete
  prefetch closure before hydration; every member is separately `present`,
  `verified-absent`, `not-authorized`, or `not-fetched`. Only the first two satisfy a
  required closure, and admission is atomic. Fallback: explicitly widen and rehydrate a
  new decision epoch; never fetch opportunistically inside evaluation.
- [x] D5. **FHIR output posture** (decided 2026-09-02): CDS Hooks returns a recommendation
  card and may attach a labeled proposed FHIR resource. Conditional write is a separate,
  explicitly authorized adapter call; proposed resources carry a proof reference and are
  never mistaken for accepted record state. Fallback: recommendation-only deployment.
- [x] D6. **PlanDefinition subset** (decided 2026-09-02): import R4 JSON carrying the
  stable, R6-shaped fields `url`, `version`, `status`, and flat leaf `action` entries with
  applicability conditions expressed only as named external CQL boolean results. Nested
  actions, dynamic values, related actions, transforms, and unknown modifiers fail closed
  at load. Fallback: add one construct at a time with a typed fixture and loss rule.
- [x] D7. **CQL boundary** (decided 2026-09-02): consume pinned external evaluator results
  as typed, provenance-bearing inputs; Horismos does not parse or compile CQL. Fallback:
  add another external evaluator adapter, not a second CQL engine.
- [x] D8. **Terminology policy** (decided 2026-09-02): runtime is offline over licensed,
  build-time expansions pinned by canonical URL, code-system version, expansion timestamp
  supplied by the package, and expansion parameters including `check-system-version`.
  Missing versions fail closed. Fallback: regenerate the package through an authorized
  terminology service and create a new mapping/package identity.
- [x] D9. **Trace evolution** (decided 2026-09-02): standards/host evidence uses
  `horismos.integration-trace/1`, a deterministic adapter-layer envelope around an
  unchanged core trace, because remote commit lifecycle is outside the core transaction
  and is not a new meaning for `horismos.trace/1` or `/2`. It carries closure, source
  vector, mapping/package identity, proof reference, proposed effects, and commit receipt.
  Fallback: migrate this envelope explicitly if a later core trace version absorbs it.
- [x] D10. **Generic host SPI** (decided 2026-09-02): a pure `horismos-host` crate owns
  format-neutral `ExternalSnapshot`, closure/version/mapping evidence, `ExternalFact`, and
  `ProposedEffectSet`; each effect is typed `Bufferable`, `Reversible`, or `Irreversible`
  and includes an idempotency key plus verification posture. FHIR and coding-harness crates
  depend on this crate and `horismos-core`; neither dependency points back inward.

## Controller resolutions

- `horismos-host` is the pure companion crate for SPI and candidate-artifact runtime
  policy. It may depend on `horismos-core`, `serde`, `serde_json`, and `thiserror`; it has
  no HTTP, provider, filesystem mutation, clock, or randomness dependency.
- `ExternalSnapshot` contains resolved strings and typed core `Value`s, not interned IRIs;
  admission interns once and uses the public atomic World transaction surface.
- PRP-005 coupling is isolated in `horismos-host::assumptions`: five fixed fields named
  `input_closure`, `external_versions`, `mapping_version`, `supported_subset`, and
  `unverified_effects`. It can convert to the provisional `report.assumptions` shape after
  PRP-005 lands without changing adapter APIs.
- Candidate-artifact authoring payload stays opaque to the runtime. PRP-008 mints only
  runtime evidence constants for policy validation, sandbox results, and authorization;
  PRP-007 owns the authoring envelope vocabulary.
- FHIR I/O is behind a synchronous `FhirTransport` trait; the DoD uses deterministic
  in-memory and recorded-Bundle transports. A future HTTP implementation belongs in the
  adapter crate behind a feature and cannot affect the no-model gate.
- All adapter outputs use `BTreeMap`/ordered vectors. Caller-supplied transaction stamps,
  run ids, source versions, and hashes preserve trace equality.

## Units of work

- [x] U1. **Generic host SPI** — create `horismos-host`; implement closure states,
  exact version vectors, mapping/package evidence, atomic `ExternalSnapshot::admit`,
  reversibility-typed proposed effects, assumption-manifest bridge, integration trace,
  and exact domain-term errors. Test: two format-neutral snapshots serialize identically;
  a bad fact rejects admission without partial World state. **Done 2026-09-03**:
  `horismos-host` ships the pure SPI, validated wire values, atomic private-World
  admission, forward-reference ordering/cycle rejection, five-field assumption bridge,
  terminology expansion evidence, reversibility-typed effects, structured per-effect
  receipts, and `horismos.integration-trace/1`; 9 focused tests and `make verify` green;
  senior review Approved with minors.
- [x] U2. **Candidate-artifact runtime pipeline** — typed opaque envelope → deterministic
  path/base-hash/syntax/permission/kind policy → sandbox checks → symbolic authorization, with
  proof events and no semantic-correctness claim. Test: one seeded patch is accepted and
  one stale-base/path-invalid proposal is rejected before sandbox/application. **Done
  2026-09-03**: runtime-minted evidence is sealed; bounded V4A updates bind their embedded
  path to the validated target; executable policy rejects shells for both artifacts and
  declared checks; sandbox sees only ordered typed checks; all ten success/rejection paths
  pin structured stages and domain text. Senior review Approved.
- [x] U3. **FHIR adapter** — create `horismos-fhir`; capability probing, prefetch closure
  hydration, RDF-aligned bounded mapping with loss report/modifier-extension rejection,
  exact version evidence, proposed-resource labeling, and conditional commit with 409/412/
  weak-ETag drift equivalence. Test: deterministic recorded Bundle plus mutated-source
  fixture; unsupported/missing data asserts machine witness and exact domain text. **Done
  2026-09-03**: the R4 adapter probes write posture, retains recommendation-only hydration,
  enforces Bundle/closure/ETag consistency, emits total mapping loss, rejects recursive
  modifiers, and requires a sealed proof-linked proposed-resource authorization. Its
  transport contract atomically enforces the full read vector; weak ETags and both 409/412
  conflict forms fail closed. Eleven focused tests and senior review Approved.
- [x] U4. **CDS Hooks and PlanDefinition/CQL boundary** — CDS-Hooks-shaped request/service
  response, bounded R4 PlanDefinition importer/executor, and typed external CQL results.
  Test: unsupported construct fails at load; VTE invocation returns a recommendation card
  with proof-linked proposed resource. **Done 2026-09-03**: sealed manually validated
  PlanDefinition imports accept only flat applicability actions over named
  `text/cql-identifier` booleans; execution pins the evaluator artifact and exact source
  vector. CDS Hooks wire types emit every authorized proposal and rebuild logical
  resource-targeted proof Provenance for suggestion actions. Five focused tests and senior
  review Approved.
- [x] U5. **VTE prophylaxis vertical slice** — package ontology, recorded FHIR Bundle,
  closure/profile/terminology/mapping manifest, standalone runner, and end-to-end tests.
  Test: zero-network/zero-model decision, versions in proof/integration trace, drift abort,
  and a benign-completion/fail-closed count reporting the closure utility tax. **Done
  2026-09-03**: a patient-bound six-member closure admits atomically and drives the
  higher-priority VTE risk/contraindication rules, a bounded PlanDefinition, and a
  CDS Hooks-shaped recommendation. Real SHA-256 pins cover the ontology, local R4
  StructureDefinition, owned mapping, PlanDefinition, offline terminology expansion,
  and composite package. Proof/trace carry all identities and exact source versions;
  sealed traced commits record acceptance or drift, recommendation-only posture remains
  usable, and nine VTE integrity/E2E tests plus the standalone replay pass. Senior review
  Approved after all patient-binding, outcome-coherence, profile, and digest findings.
- [x] U6. **Coding-harness adapter** — create `horismos-harness`; materialize branch,
  dirty paths, base hashes, test status, and path scopes through the same SPI and exercise
  the candidate-artifact pipeline. Test: both adapters satisfy one shared host-contract
  suite, proving the SPI has no FHIR-shaped fields. **Done 2026-09-03**: the pure harness
  adapter materializes branch/head, every file hash, dirty paths, typed test results, and
  path scopes with an exact deterministic version vector. Only its exact current snapshot
  can enter the existing policy → sandbox → symbolic authorization pipeline; reversible
  artifact handoffs reject forged effects, proposal-id rebinding, changed/missing/added
  sources, and preserve idempotent receipt coherence. One generic hydration-contract test
  runs unchanged against both FHIR and harness adapters. Seven focused tests and senior
  review Approved with no findings.

## Gaps discovered during implementation

- 2026-09-03, U1 minor (closed in U3): the FHIR fixture supplies coherent proposed-effect
  and receipt ids, while the generic receipt representation remains independently
  round-tripped by the SPI contract tests.
- 2026-09-03, integration dependency: PRP-005 was still being built on its isolated lane,
  so this branch deliberately does not run its not-yet-merged verifier against the VTE
  ontology. The only coupling is the five-field `horismos_host::assumptions` module; its
  conversion to the final `report.assumptions` type must be reconciled when PRP-005 lands.
  This does not block PRP-008's adapter/vertical-slice DoD, but it does remain a required
  integration check before the combined branches land.
