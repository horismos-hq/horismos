# PRP-008 — Activation decisions

Decided 2026-09-02. Each pass followed `.claude/skills/brainstorming`: statement and
blast radius, evidence before options, distinct alternatives, adversarial house-rule
check, criteria ranked before scoring, then a recorded decision and fallback. The standing
evidence page `research/neurosymbolic-landscape/pivot-alignment.md` (sweep #2, especially
X3/X7 and A1–A7) was consulted in every pass. The corresponding checked resolution lines
and implementation pins live in `02-tasks-and-gaps.md`.

No framework `[DECIDE]` marker is owned by D1–D10: these questions existed only in the
draft's Required decisions section. That section is rewritten as resolved and this record
is the durable strike. PRP-007 Key decision 5 remains authoring-owned; this activation
pins only the runtime/authoring mechanical boundary and does not design its vocabulary.

## D1 — FHIR baseline and deployment profiles

**Falsifiable statement.** The prototype is interoperable enough to validate its bounded-
projection claim using FHIR R4 4.0.1 JSON plus an explicitly named local VTE profile set,
while a bounded importer adopts the stable R6 PlanDefinition shape. Blast radius:
`horismos-fhir`, U3–U5, docs 03/14, later packaging; no core/no-model edge.

**Evidence.** Doc 14 requires adapters outside core. Pivot X7/A5 reports CDS Hooks 3.0,
CQL engines, and AI Transparency on R4 while R6 PlanDefinition is normative-ballot stable.
The PRP explicitly rejects certification and full PlanDefinition coverage.

**Options.** (A) R4-only everywhere maximizes deployed compatibility but ignores the
stable future PlanDefinition shape. (B) R4 wire plus R6-shaped bounded import separates
host compatibility from artifact shape. (C) R5/R6 wire first follows the newest spec but
misses the deployed CDS Hooks ecosystem. **Adversarial pass:** all can remain deterministic;
C creates adoption risk, A creates immediate importer churn. **Ranked criteria:** boundary
invariants → deployed compatibility → explicit conformance claim → upgrade path → cost.
**Decision:** B. Conformance-tested means only the named fixtures/profile manifest, never
US Core certification. Fallback: adapter-local R4B/R5 translation.

## D2 — Mapping representation

**Falsifiable statement.** A small owned RDF-aligned mapping profile can map the VTE R4
closure totally enough for the prototype while naming every loss and rejecting modifier
extensions. Blast radius: `horismos-fhir::mapping`, U3/U5, docs 02/10/15.

**Evidence.** Doc 06 requires deterministic mapping, never perception. X7/A4 says FHIR RDF
is normative in R6 while StructureMap moved to an incubator and RDF deliberately omits
modifier extensions because of non-monotonicity. **Options:** generated Rust mapping
(fast but opaque to authoring); full StructureMap (standard-shaped but unstable and broad);
raw FHIR RDF (standard but too permissive for a tiny closure); owned RDF-aligned descriptors
(bounded and reportable). **Adversarial:** full engines enlarge attack/dependency surface;
generated code hides losses; raw RDF can admit modifiers silently. **Criteria:** deterministic
totality/loss evidence → modifier safety → bounded surface → future replaceability → cost.
**Decision:** owned RDF-aligned descriptors compiled to typed extractors. Fallback: replace
extractors behind the same loss contract when a normative mapping language stabilizes.

## D3 — Source consistency

**Falsifiable statement.** Exact version-vector equality plus server-enforced conditional
writes is sufficient to detect all source drift relevant to one declared closure. Blast
radius: SPI versions, FHIR transport/init/commit, U1/U3/U5, integration trace.

**Evidence.** Doc 03 requires fail-closed freshness. Pivot C1/C2 and A2 cite FHIR
`versioned-update`, `If-Match`, and observed 409/412 variation; X3 says Horismos has the
declared read set that makes OCC appropriate. A1 motivates effect reversibility.
**Options:** timestamp freshness (cheap, unsound); exact vector with adapter-only check
(portable but TOCTOU remains); exact vector plus conditional server write (strongest);
locks (strong but not broadly available). **Adversarial:** retries may reinterpret changed
clinical state; weak ETags/status variations can cause false handling. **Criteria:** fail-
closed correctness → provider enforcement → observable errors → portability → cost.
**Decision:** exact vector, init probe, normalized weak/strong ETags, 409/412 drift, no
automatic retry; effects typed by reversibility. Fallback: recommendation-only.

## D4 — Closed-world boundary

**Falsifiable statement.** A predeclared prefetch closure whose members retain four presence
states can support Horismos's pragmatic closed-world evaluation without confusing absence
with access/fetch failure. Blast radius: SPI closure/assumptions, hydration, CDS request,
U1/U3–U5, PRP-005 bridge.

**Evidence.** Doc 03 requires absence states to remain distinct. Pivot X9 notes this is a
novel claim; A3 favors prefetch over service-side fetch; A14 says assumptions must be
tested. **Options:** infer closure from Bundle contents (simple but collapses states);
service-side lazy fetch (complete-looking but violates transaction boundary); predeclared
closure with state per member; always fail on any empty search (safe but unusable).
**Adversarial:** strict manifests impose the measured utility tax in X4; closure labels can
still lie unless checked. **Criteria:** semantic honesty → no evaluation-time I/O →
testability → utility → ergonomics. **Decision:** predeclared, four states, only present or
verified-absent complete required members. Fallback: explicit widen-and-rehydrate.

## D5 — FHIR output posture

**Falsifiable statement.** Returning a recommendation card plus an optional labeled
proposed resource, with write as a separate authorized conditional operation, is the
safest useful VTE prototype posture. Blast radius: CDS response, effect model, FHIR commit,
proof/trace, U3–U5.

**Evidence.** Doc 07 distinguishes proposed effects from remote acceptance. X8 warns that
unreviewed writes change regulatory posture; A3 favors prefetch; A6 requires labels and
Provenance pointing to proof. **Options:** cards only (safest, less concrete); cards plus
proposal (reviewable and useful); automatic conditional write (complete but premature).
**Adversarial:** a proposal may look accepted; a card can omit actionable structure;
writes can partially fail. **Criteria:** human review/safety → state honesty → typed utility
→ reversibility → cost. **Decision:** card + labeled proposal by default, explicit separate
authorized commit. Fallback: card only.

## D6 — PlanDefinition subset

**Falsifiable statement.** Flat leaf actions gated only by named external CQL boolean
results are enough to import and execute the VTE artifact without silently approximating
FHIR orchestration semantics. Blast radius: importer, CQL boundary, supported-subset
assumption, U4/U5.

**Evidence.** PRP scope makes a permanent bounded subset the v1 posture. A5 favors the R6
shape, while D7's ecosystem evidence rejects a home CQL engine. **Options:** metadata-only
import (too weak); flat actions/named conditions (small executable subset); nested
RequestOrchestration subset (richer, much larger); full PlanDefinition (explicit non-goal).
**Adversarial:** named results can be misbound; unsupported fields could be ignored by
serde. **Criteria:** fail-closed fidelity → executable VTE slice → explicit subset manifest
→ simplicity. **Decision:** flat actions, applicability conditions, named typed booleans;
deny unknown/unsupported semantic fields. Fallback: one-fixture-at-a-time expansion.

## D7 — CQL boundary

**Falsifiable statement.** Pinned typed outputs from an external CQL evaluator provide all
CQL composition the first slice needs. Blast radius: U4/U5, assumptions/provenance; no core
or parser dependency.

**Evidence.** The PRP says compose, do not rebuild. Pivot C3 reports active CQL Release 2
and two frequently shipping engines; the research inference says release velocity alone
settles build versus compose. **Options:** external typed results; compile a tiny CQL subset;
embed an engine. **Adversarial:** external results can be stale or wrongly typed, addressed
by source/evaluator versions; direct support creates a second semantics. **Criteria:** no
duplicate engine → provenance/type safety → offline replay → cost. **Decision:** external
typed inputs only. Fallback: another evaluator adapter.

## D8 — Terminology policy

**Falsifiable statement.** A build-time, licensed, version-pinned expansion package can make
VTE evaluation reproducible offline and expose enough provenance for audit. Blast radius:
FHIR mapping/package identity, snapshot assumptions, trace, U3/U5.

**Evidence.** Docs 02/10 say extracted subsets and no terminology-correctness claim. A7
requires `check-system-version` and expansion parameters; terminology releases change on
independent cadences. **Options:** live expansion each run (fresh but nondeterministic/I/O);
unversioned checked-in codes (simple but unauditable); pinned offline package (reproducible);
vendor terminology SDK (wrapper risk). **Adversarial:** checked-in expansions can stale and
licenses constrain redistribution. **Criteria:** offline determinism → exact identity →
license honesty → refreshability → size. **Decision:** pinned package/parameters, reject
missing versions. Fallback: authorized regeneration creates a new identity.

## D9 — Trace evolution

**Falsifiable statement.** A versioned integration-trace envelope around the unchanged core
trace represents source, mapping, proposed-effect, and remote-commit lifecycle without
changing either existing core wire contract. Blast radius: `horismos-host::trace`, U1/U3–
U6; future migration coupling isolated.

**Evidence.** Doc 09 forbids reinterpreting trace/1 and requires distinct commit states.
Doc 03 says remote commit is outside World atomicity. Existing core already has trace/2;
PRP-005 is concurrently changing verifier-owned assumptions. **Options:** mutate trace/2
(wire break); mint core trace/3 now (correct but couples parallel work and external I/O into
core); adapter integration envelope (separates lifecycles); log separately (violates
"traces are output"). **Adversarial:** two schemas can fragment tooling; a wrapper must
embed, not merely reference, the decision trace. **Criteria:** wire compatibility → boundary
truth → one auditable object → rebase isolation → migration path. **Decision:**
`horismos.integration-trace/1` embeds the core trace and all host evidence. Fallback:
explicit converter into a future core trace version.

## D10 — Generic host SPI

**Falsifiable statement.** One pure companion crate with resolved-string facts, explicit
closure/version/mapping evidence, and reversibility-typed effects is format-neutral enough
for both FHIR and repository adapters. Blast radius: new `horismos-host`, both adapters,
U1–U6, doc 14 boundary, PRP-005 coupling module.

**Evidence.** Doc 14 fixes dependency direction and whole-input/outcome boundaries. Pivot
A1 requires bufferable/reversible/irreversible effects; A11 widens coding effects beyond
paths. The PRP requires two adapter proofs. **Options:** types in core (central but risks
host concerns leaking inward); FHIR-first traits generalized later (likely FHIR-shaped);
pure companion crate now; dynamic JSON envelope (format-neutral but untyped). **Adversarial:**
a companion can become a wrapper crate; typed enums can overfit the first effect set.
**Criteria:** anti-wrapper pass/fail → format neutrality → determinism → compile-time
effect safety → minimal API. **Decision:** pure `horismos-host`, typed values/effects and
generic adapter trait; test against both adapters. Fallback: split transport-specific
traits while preserving the shared value types.
