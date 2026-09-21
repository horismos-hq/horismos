# Horismos thin-pivot second opinion

> **Provenance.** Produced by scout task `horismos-thin-pivot-s1` on **2026-08-24** as an independent second opinion (a different model and session from the founding sweep) on the captain's "thin layer" thesis; scope: strategic assessment and a PRP/doc change map only — no implementation, no PR. Written against the repository as of [`1c7ce1c`](https://github.com/horismos-hq/horismos/blob/1c7ce1c) (the wiki as landed in PR #9, *before* the pivot commit `5c45c0a`), so every `file:line` reference below is to that state and links to it. Imported verbatim on 2026-09-03 by task `horismos-researchdocs-a1`: tags, tables and citations intact; the one workspace-local absolute path in the evidence trail was replaced with a repo-relative reference. **Where it stands now:** the captain adopted its Option C — the bounded projection kernel — and PR #10 applied the change maps below across PRP-000 – PRP-008 and framework docs 00 – 15; [pivot alignment](../neurosymbolic-landscape/pivot-alignment.md) re-verdicts the result every sweep.

**Scout:** `horismos-thin-pivot-s1`  
**Date:** 2026-08-24  
**Scope:** strategic assessment and PRP/doc change map only; no implementation or PR

## Executive verdict

**Verdict: partial on the thesis, but proceed with the pivot.** The strategic diagnosis is
right: Horismos needs to become a thin, standards-compatible decision and capability layer
that composes with systems of record. The literal implementation “make `World` a live view
over a FHIR server” would be wrong. The defensible design is a **bounded projection kernel**:

1. an adapter reads a versioned, explicitly scoped bundle from the external authority;
2. Horismos validates and normalizes that bundle into a local, closed transaction snapshot;
3. the existing gate, inference, truth-maintenance, proof, verification, and trace machinery
   runs unchanged over that snapshot;
4. the result is a typed proposed effect set; and
5. an adapter conditionally commits it to the external system, failing closed if the source
   versions changed.

Thus “thin” should mean **does not own durable records, the enterprise workflow, provider
clients, or standard terminology**, not “has no local World.” The local World is the isolation
boundary that makes the novel guarantees true. It becomes an evaluation workspace, not a
system of record.

The captain's strongest wording also overstates the coding-harness comparison. Claude Code
and Pi skills are indeed prompt-loaded Markdown; their instructions are followed by the model,
not enforced as semantic preconditions/effects. DeepSeek Harness, however, already has typed
events, a durable session log, swappable capability seams, monotonic guards, and a tool
execution pipeline. What it still does not have is Horismos's declarative semantic world,
fixed-point truth maintenance, or pre-execution proof that capability preconditions/effects
compose. The coding-agent opportunity is real, but the differentiation is **semantic contracts
and verified effects**, not simply “typed harness versus scratchpad.”

### One-sentence recommendation

Finish the current PRP-004 scenario/runtime work and build PRP-005's verification kernel;
immediately change the standing positioning and framework contracts; then activate a new
**PRP-008 — Standards & Host Integration** before packaging or the blank-slate authoring agent.

## What I reviewed and how I judged it

I reviewed the landed research wiki, especially the formalization history and scenario pages;
the triggering report's §§1.1, 1.2, and 1.7; every PRP from 000 through 007 plus the standing
skeleton; and framework docs 00–15. I also inspected the current official documentation for
Claude Code skills, Pi's coding agent/skills, and DeepSeek Harness's architecture and tool
pipeline.

The ranked criteria, written before selecting an option, were:

1. preserve determinism, gate integrity, the anti-wrapper boundary, and trace truthfulness;
2. make an honest integration claim that can survive concurrent external writers;
3. reuse real standards without pretending that standards alone supply Horismos semantics;
4. avoid invalidating the already-built PRP-001–004 core;
5. create a credible adoption path before polishing distribution and authoring;
6. minimize irreversible API/wire-contract changes.

The evidence base is unusually directional:

- Horismos's graph is in the Task-Network Model family, and the surveyed TNM systems did not
  reach sustained production multi-site use ([`research/formalization/00-overview.md:5-12`](../formalization/00-overview.md),
  [`research/formalization/tnm-family.md:5-8`](../formalization/tnm-family.md)).
- The comparatively successful standards were thin layers over FHIR and existing CIS/EHR
  infrastructure ([`research/formalization/cql-cds-hooks-fhir.md:5-9`](../formalization/cql-cds-hooks-fhir.md)). Even those layers retain
  substantial authoring burden ([`research/formalization/cql-cds-hooks-fhir.md:28-33`](../formalization/cql-cds-hooks-fhir.md)), so the
  pivot does not erase the problem PRP-007 targets.
- The gate and truth-maintenance mechanisms have no surveyed CIG analog and address documented
  historical failures ([`research/formalization/00-overview.md:55-81`](../formalization/00-overview.md); triggering report
  [`horismos-research-s1/report.md:75-110`](2026-08-24-founding-sweep.md)). Abandoning them would throw away the differentiated
  part of Horismos while retaining only a late integration story.
- The scenario catalog itself already rewards structured EHR facts, typed actions, and published
  artifacts ([`research/scenarios/00-catalog.md:5-13`](../scenarios/00-catalog.md)). VTE is the clearest first integration
  probe because its inputs are almost entirely structured and its actions are typed
  ([`research/scenarios/vte-prophylaxis.md:5-22`](../scenarios/vte-prophylaxis.md)).

## Options and adversarial conclusion

| Option | Strongest case | Why it fails or survives |
|---|---|---|
| **A. Stay vertically integrated** | The current model is coherent, deterministic, and already partly built. One artifact makes verification and replay tractable. | It has no credible EHR/CIS or repository-harness adoption path and repeats the ownership posture the historical evidence warns about. It makes every deployment first replace state, vocabulary, and workflow rather than add one decision layer. Reject. |
| **B. Fully thin pass-through** | Direct queries and writes against FHIR minimize duplication and make the external record immediately current. | Network reads, server ordering, partial responses, and concurrent mutation enter the inference transaction. The gate cannot validate writes by other clients; TMS cannot maintain facts it does not control; trace equality ceases to be meaningful. Reject. |
| **C. Bounded projection kernel** | External standards and systems own durable state and the outer workflow; Horismos owns a versioned decision snapshot and a conditional effect commit. Existing core work survives. | Requires explicit freshness, closure, mapping, and commit contracts. Guarantees become scoped rather than global. This is extra adapter engineering, but it is the only option that passes the ranked criteria. **Recommend.** |

## Where thinness is free, and where it threatens guarantees

### The guarantee boundary

The current docs say the World is the “agent's memory of record” and that every external
mutation ends at the fixed point ([`docs/framework/03-world.md:3-5,50-76`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/03-world.md)). That is stronger than
an external-system deployment can truthfully claim. Under the pivot, the formal claim should be:

> Given a verified decision package, a declared input closure, a source-version vector, and an
> admitted input bundle, Horismos deterministically produces a fixed-point decision snapshot,
> proof, trace, and proposed effect set. A commit succeeds only if the external resources still
> match that source-version vector. Mutations outside Horismos are outside the guarantee and
> invalidate the projection until it is refreshed.

| Mechanism | Thin model | Required contract / honest limit |
|---|---|---|
| **RDFS-plus profile and `horis:` execution terms** | **Keep as a small executable overlay.** Domain identity can come from FHIR profiles, SNOMED CT, LOINC, RxNorm, published PlanDefinitions, and repository/tool schemas. | FHIR/SNOMED do not define Horismos's rule, workflow, provenance, or gate semantics. “Compatible with standards” is accurate; “the ontology is FHIR” is not. |
| **Uniform validation gate** | **Survives for admission and Horismos-mediated effects.** It validates the hydrated bundle and every proposed write before commit. | It cannot retroactively make a FHIR server valid or intercept writes by other clients. Invalid incoming resources must fail closed or be quarantined; no decision may silently run over a partially admitted bundle. |
| **Truth maintenance** | **Survives inside one projection.** Each new external version is a base-fact replacement transaction that resets derivations and re-chains. | It does not stay live across unseen external mutations. A subscription/change feed reduces staleness but is not the proof boundary. Only re-hydration plus source-version checks restores a current guarantee. |
| **Determinism** | **Survives after materialization.** Sort and canonicalize the complete input bundle; stamp the evaluation epoch/source versions; do no network I/O inside the core transaction. | A live/lazy `World` backed by FHIR breaks the claim. Paging, `_include`, terminology expansion, server clocks, and network failures must complete outside core and be captured as inputs. |
| **Atomicity** | **Survives locally; becomes conditional externally.** The local transaction is atomic. FHIR writes should use a transaction Bundle and conditional version checks where supported. | Horismos cannot promise distributed atomicity across multiple systems/tools. The trace must distinguish “decision produced,” “commit attempted,” “commit accepted,” and compensation/failure. |
| **Verification** | **Mostly survives and gains a useful assumption boundary.** Rules, local workflow fragments, mappings, declared tool effects, and PlanDefinition-subset compilation can be checked before execution. | It cannot prove data completeness in a future EHR response, external terminology correctness, tool implementation behavior, or absence of concurrent writers. Those become explicit assumptions and runtime checks, not silent proof claims. |
| **Proof trees** | **Keep.** Base leaves gain source resource/version/profile/mapping provenance. | A proof is valid for the recorded snapshot, not forever for the patient or repository. |
| **Trace** | **Keep and strengthen.** Stamp package hash, mapping version, source resource `versionId`/ETag vector, request identity, and commit outcome. | FHIR payloads and coding-repo contents raise retention/secrets concerns. Preserve the versioned wire policy; add a new trace version if the required fields are not additive under `trace::migrate`. |
| **Perception/narration boundaries** | **Mostly free.** Structured EHR/repository facts use the existing no-Bridge bypass; NL remains an optional transducer. | Mapping structured standards into facts must be deterministic adapter code, never an LLM perception shortcut. |
| **External vocabulary** | **Free in principle, nontrivial operationally.** Use canonical code systems, versions, ValueSets/ECL, and mappings rather than local near-duplicates. | The current single-parent, unversioned SNOMED fixture demonstrates why `owl:sameAs` alone is insufficient ([`research/terminology/snomed-ct.md:24-41`](../terminology/snomed-ct.md)). Preserve polyhierarchy and edition/expression provenance ([`research/terminology/snomed-ct.md:60-70`](../terminology/snomed-ct.md)). |
| **Workflow ownership** | **Can become optional.** Horismos may execute a small deterministic decision microflow invoked by CDS Hooks, or compile a supported PlanDefinition subset. The host owns the longitudinal patient/repository workflow. | Totality applies to the invoked fragment, not the entire care pathway or coding session. Unsupported PlanDefinition constructs must fail at load with a domain-term report, never be approximated. |

### Reference data flow

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

No adapter performs a network read from inside matching, chaining, verification, or a World
transaction. That preserves the anti-wrapper architecture in
[`docs/framework/14-architecture.md:31-46`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/14-architecture.md) and the whole-input/whole-outcome boundary in
[`docs/framework/14-architecture.md:48-63`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/14-architecture.md).

## PRP-by-PRP change map

Effort is rough engineering effort after design decisions, not calendar time: **S** = 1–3 days,
**M** = 1–2 engineer-weeks, **L** = 3–6 engineer-weeks, **XL** = 6–10+ engineer-weeks. The
rows distinguish changes to an already-shipped PRP record from work that belongs in PRP-008.

| PRP | Keep | Change under the thin model | Ownership and rough effort |
|---|---|---|---|
| **PRP-000 — Vision & positioning** | The two principles; Type 2–3 honesty; researchers, regulated builders, and SMEs; “delete the LLM” claim. | Replace “one artifact owns domain + rules + workflows” ([`PRPs/PRP-000-vision-positioning.md:11-18`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/PRP-000-vision-positioning.md)) with “a verified decision/capability package overlays existing records, vocabularies, tools, and workflows.” Add FHIR/CQL/CDS Hooks/PlanDefinition and coding-harness comparisons. State snapshot-scoped guarantees. | Standing document, change now. **S**. This is the product decision; do not bury it in PRP-008. |
| **Foundational skeleton** | Rust core, Python marshaling boundary, no-model/boundary gates, Makefile workflow. | Add an integration crate/service boundary such as `horismos-fhir` or `horismos-adapters`; forbid it from entering `horismos-core`. Add fixtures/gates proving no network inside evaluation. Update skills roster later for capability-fragment authoring. | Standing skeleton plus doc 14. **S–M** for layout/gates; actual adapter is PRP-008. |
| **PRP-001 — Ontology core (Done)** | Parser/profile, typed values, gate, immutable snapshots, imports, deterministic local store. The existing decision to own adjacency storage remains correct ([`PRPs/PRP-001-ontology-core/02-tasks-and-gaps.md:13-16`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/PRP-001-ontology-core/02-tasks-and-gaps.md)). | Reinterpret `World` as a transaction projection, not the enterprise memory of record. Add a core-native, format-neutral `ExternalSnapshot`/batch-admission value carrying source/version/mapping provenance. Do **not** give `World` a FHIR client. Replace `owl:sameAs`-only interop messaging with versioned standard mappings and terminology closure. | Amend the Done PRP's durable description without rewriting its historical DoD: **S**. New core value/provenance support under PRP-008: **M**. |
| **PRP-002 — Reasoning substrate (Done)** | Subsumption, forward/backward chaining, full recomputation TMS, proof trees, trace schema discipline. Its immutable read / gated write split is exactly the needed kernel ([`PRPs/PRP-002-reasoning-substrate/01-objective.md:399-416`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/PRP-002-reasoning-substrate/01-objective.md)). | Scope fixed-point claims to one admitted projection. Base proof leaves carry external resource/version/mapping provenance. Refresh is a replacement of the external base set followed by full re-chain. Add an explicit input-closure declaration so closed-world negation does not mistake “not fetched” for “absent.” | PRP record/docs **S**; closure/provenance implementation and properties **M–L**, owned by PRP-008 with core changes reviewed as PRP-002 invariants. |
| **PRP-003 — Python API (Done)** | Whole values across PyO3; generic CURIE/value surface; no Python logic; snapshots/proofs/errors. | Add whole-bundle entry/exit surfaces only after core types exist: hydrate an external snapshot in one call; return a decision plus proposed effect set/commit receipt. FHIR parsing/mapping remains outside `horismos-py`; no per-resource chatter inside a transaction. | API/stubs/tests under PRP-008, **M**. Existing API remains supported. |
| **PRP-004 — Agent loop (Active/current work)** | Deterministic step transitions, Bridge boundary, tool contracts/effect gate, mock, trace, runtime-fired tools. These are reusable as a decision microflow. | Stop claiming the ontology must own the outer loop ([`PRPs/PRP-004-agent-loop/01-objective.md:13-20`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/PRP-004-agent-loop/01-objective.md)). Add host-invoked/stateless execution and adapter bindings after the current branch lands. Align ActionStep effects with external conditional commits; distinguish local effect admission from remote commit. The host may invoke one workflow fragment via CDS Hooks or PlanDefinition rather than surrender its longitudinal workflow. | Finish the current scenario/PR first. Spec amendment **S**; runtime service/commit integration belongs to PRP-008, **L**. Do not expand the active PRP mid-flight. |
| **PRP-005 — Guarantees & verification (Draft)** | The four rule checks, workflow checks, domain-term witnesses, BDD approach ([`PRPs/PRP-005-guarantees-verification.md:13-29`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/PRP-005-guarantees-verification.md)). | Make verification reports declare their assumptions and scope: input closure, external profile/version, mapping version, supported PlanDefinition subset, and unverified external effects. Add explicit non-vacuity checks recommended by the research. Verify adapter mappings and capability fragments for totality/conflicts/dead contracts where decidable. Never label external data quality or tool code “verified.” | Core PRP remains next after 004 and should proceed. Base verifier **L** as planned; assumption manifest/non-vacuity **M**; standards/capability checks staged in PRP-008/007 **L**. |
| **PRP-006 — Packaging, distribution & docs (Draft)** | Crates/PyPI, mock-only CI, supply-chain gates, docs publication, trace privacy. | Make integration a release criterion: publish the adapter package/crate, standards compatibility matrix, test FHIR server fixtures, CDS Hooks conformance fixture, and a no-model structured-EHR example. Replace triage-only newcomer DoD ([`PRPs/PRP-006-packaging-distribution.md:46-50`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/PRP-006-packaging-distribution.md)) with both standalone and host-integrated paths. | Defer activation until PRP-008 produces a real surface. Existing packaging **M**; added conformance/docs **M**. |
| **PRP-007 — Ontology authoring agent (Draft)** | Draft -> parse/gate -> verify -> witnesses -> human sign-off; bounded refinement; domain-term review; no autonomous deployment ([`PRPs/PRP-007-authoring-agent.md:13-35`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/PRP-007-authoring-agent.md)). Keep the hard gate; the literature supports it. | Change the default job from “invent a whole ontology from conversation” to “import, align, and generate a small verified overlay/delta over FHIR profiles, terminology, PlanDefinitions, tool schemas, or repo conventions.” Add non-vacuity and wrong-but-consistent probes. Add a capability/skill compiler mode with a formalization coverage report. Trace mining may propose edits but uses the same gated authoring loop. | Rewrite Draft after PRP-008 contracts stabilize, **S–M** planning and **L–XL** implementation. Do not build blank-slate authoring first. |
| **PRP-008 — Standards & Host Integration (new)** | Consumes 001/002/004/005; preserves all core invariants. | Own FHIR hydration/mapping/conditional commit, terminology/value-set provenance, a CDS Hooks service surface, a deliberately bounded PlanDefinition/ActivityDefinition importer/executor, and a CQL composition story (call a standards engine or ingest its results; do not casually build another CQL engine). Also owns a generic host adapter SPI reusable by coding harnesses. | **XL (6–10+ engineer-weeks)** for a credible prototype, excluding certification, hospital deployment, and full CQL/PlanDefinition coverage. Activate before 006/007. Update the “numbering is sequence” convention to a dependency DAG rather than renumbering established PRPs. |

### Required PRP-008 decisions

These are D-items, not decisions to smuggle into code:

1. FHIR baseline and deployment profile(s): R4/R4B/R5, US Core or another national base,
   resource closure, and what is merely accepted versus conformance-tested.
2. Mapping representation: generated mapping code, declarative maps, FHIR RDF, StructureMap,
   or a deliberately smaller owned mapping profile; include round-trip/loss reporting.
3. Source consistency: exact resource-version vector, transaction Bundle/conditional update
   requirements, conflict retry policy, and servers that cannot support them.
4. Closed-world boundary: what exact search/bundle constitutes complete input for each decision;
   missing, not-authorized, not-fetched, and absent must not collapse into one state.
5. FHIR output posture: recommendation-only CDS Hooks cards versus proposed FHIR resources versus
   authorized conditional writes.
6. PlanDefinition subset and fail-closed behavior; explicitly list unsupported constructs.
7. CQL boundary: external evaluator results as typed inputs versus supported direct compilation.
8. Terminology service/build-time expansion policy, versions, licenses, and offline reproducibility.
9. Trace schema evolution for source vectors, mapping/package identity, and remote commit receipts.
10. Generic host SPI: the minimum format-neutral `ExternalSnapshot` and `ProposedEffectSet` types
    that keep FHIR and coding-harness clients out of core.

## Framework doc-by-doc change map

| Doc | Change |
|---|---|
| **00 Index** | Rewrite the one-paragraph model ([`docs/framework/00-index.md:11-18`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/00-index.md)) around a verified overlay and bounded projection. Add a host-integrator reading path and the PRP-008 integration doc(s). |
| **01 Concepts** | Add **System of Record**, **Adapter**, **Decision Package**, and **Projection/Evaluation Epoch**. Redefine World from durable memory to local transaction state. Keep the two principles verbatim ([`docs/framework/01-concepts.md:3-12`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/01-concepts.md)). Change the conventional-stack comparison from “we replace your state/control flow” to “we gate selected high-stakes decisions/effects inside it.” |
| **02 Ontology** | Keep the executable RDFS-plus overlay and `horis:` vocabulary. Add standard artifact identity, versions, mappings, profiles, ValueSets/ECL, and polyhierarchy provenance. Avoid claiming `owl:sameAs` alone buys full interoperability ([`docs/framework/02-ontology.md:124-138`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/02-ontology.md)). Specify that FHIR JSON/RDF mappings are compiled before runtime. No new vocabulary terms until PRP-008 owns their D-items. |
| **03 World** | This is the largest semantic rewrite. World is an admitted projection bound to one decision package plus a source-version vector. Add hydration rejection/quarantine, closure, freshness states, external-conflict invalidation, and conditional-commit semantics. Preserve the local gate and TMS exactly. Replace global “no back door” with “no back door inside an evaluation/commit mediated by Horismos”; explicitly exclude other external writers. |
| **04 Reasoning** | Keep inference semantics. State that pragmatic closed-world negation ([`docs/framework/04-reasoning.md:58-64`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/04-reasoning.md)) applies only to declared-complete input closures. Proof leaves include source/mapping provenance. Determinism is a property of materialized inputs, not of the external server. |
| **05 Workflows** | Recast workflows as optional decision microflows. Add host invocation and a supported PlanDefinition mapping. Totality applies to the fragment. The host/EHR may own waits, longitudinal state, scheduling, and user interaction. Keep deterministic branches, fallbacks, and local composition. |
| **06 LLM Boundaries** | Mostly unchanged. Promote FHIR/repository ingestion as the primary structured-input path ([`docs/framework/06-llm-boundaries.md:53-63`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/06-llm-boundaries.md)). State that standards mapping is deterministic and cannot be delegated to perception. Add candidate-patch/content authoring as a future explicitly gated boundary for coding agents rather than pretending arbitrary code is a normal Tool argument. |
| **07 Tools** | Keep declared takes/effects and runtime firing. Add adapter-mediated external commits, expected source versions, remote receipt/effect reconciliation, authorization scopes, and distributed-failure semantics. Align where useful with ActivityDefinition/FHIR operations without forcing every tool into FHIR. Correct the current implication that contract narrowing makes all bad calls impossible: it constrains visible args/effects, not hidden side effects ([`docs/framework/07-tools.md:88-100`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/07-tools.md)). |
| **08 Bridge** | Essentially unchanged; model providers remain outside core. Add no FHIR/repository responsibilities. A host adapter is not a Bridge because it supplies authoritative structured state rather than neural proposals. |
| **09 Traces** | Preserve schema permanence. Add package/mapping hashes, source resource/version vector, closure declaration, external request correlation, proposed effects, commit status, and conflicts. Separate “decision complete” from “effect committed.” Revisit redaction because FHIR resource references and repo paths/content can be sensitive. |
| **10 Verification** | Add an explicit assumptions/scope section and machine-readable assumption manifest. Add mapping loss/totality, imported-artifact subset support, capability composition, and non-vacuity checks. Strengthen “does not claim” to include external record completeness/freshness, terminology server correctness, and tool/host implementations ([`docs/framework/10-verification.md:69-77`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/10-verification.md)). |
| **11 Python API** | Preserve existing standalone API. Add whole-value host adapter APIs and service handlers after PRP-008. No FHIR logic in PyO3. Return proposed effects and commit receipts distinctly. |
| **12 Rust API** | Add pure, format-neutral values/traits in core only where needed; FHIR/HTTP implementations live in a new adapter crate. Keep the current dependency and deterministic feature gates. A direct `World<FhirClient>` is a non-goal. |
| **13 Authoring** | Rewrite from blank-slate ontology drafting to import/align/overlay-first. Render source anchors, mapping losses, formalized versus prose-only clauses, and assumption diffs. Keep verifier + human sign-off ([`docs/framework/13-authoring.md:34-35`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/13-authoring.md)). Add the skills-to-fragment loop below. |
| **14 Architecture** | Add adapter/service crates and the two-boundary data flow: hydrate before core, conditional commit after core. Keep network/provider dependencies out of core. Split performance targets into deterministic kernel time and external I/O latency. Change one-World-per-case to one projection per decision epoch; the system of record owns case lifetime. |
| **15 Security & Trust** | Add a fifth boundary: external system of record/host harness. Cover stale or incomplete reads, authorization/SMART scopes, tenant/patient mix-up, malicious/compromised server responses, webhook/subscription spoofing, TOCTOU between decision and commit, mapping/package supply chain, and over-broad write scopes. Keep the existing honest deployer-owned limits ([`docs/framework/15-security.md:233-248`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/15-security.md)). |

## Coding-agent generalization: skills as verified capability fragments

### What current harnesses actually do

**Claude Code.** The official skills documentation says a skill is a `SKILL.md` containing
instructions; Claude loads and uses it when relevant or on explicit invocation. Skill bodies are
prompt material loaded on demand, and bundled skills are explicitly described as prompt-based
instructions that let Claude orchestrate with tools. Frontmatter can influence discovery,
invocation, tool availability, hooks, and subagent execution, but those features do not turn the
skill's natural-language claims into verified preconditions/effects. Source: [Claude Code skills
documentation](https://code.claude.com/docs/en/skills).

**Pi.** Pi describes itself as a minimal terminal coding harness with built-in read/bash/edit/
write/grep/find/ls tools and extension points through TypeScript extensions, skills, prompt
templates, and packages. Its skill docs say startup scans names/descriptions into the system
prompt; when a task matches, the agent reads the full `SKILL.md` and follows its instructions,
with the candid note that models do not always load it. Pi implements the same Agent Skills
shape and can consume Claude/Codex skill directories. Source: [Pi coding agent README](https://github.com/earendil-works/pi/tree/main/packages/coding-agent)
and [Pi skills documentation](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/skills.md).

**DeepSeek Harness.** It is not accurately described as only a scratchpad. Its official
architecture makes the model adapter, tool registry, session log, and agent loop replaceable
plugins. The durable session log is the source from which model context is projected, and
model-visible inputs must be reconstructable from that log. It also defines capability seams,
typed events, request/tool waterfalls, monotonic guards, approval policy, sandbox providers, and
an immutable tool outcome observation point. However, tool schemas still join model prompt
assembly; the architecture does not declare a semantic domain graph with verified cross-tool
preconditions/effects or maintain a fixed point over repository state. Sources: [DeepSeek
Harness architecture](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/architecture.md)
and [tool execution pipeline](https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/tool-execution-pipeline.md).

The market gap is therefore:

> Existing harnesses provide extensibility, typed call schemas, logs, hooks, approvals, and
> sandboxes. Horismos can add a declarative semantic layer that proves when a capability is
> eligible, constrains its observable effect, recomputes eligibility after state changes, and
> emits a proof of why it ran.

### Proposed mechanism

Call the authored unit a **Capability Fragment** (working name, not approved vocabulary). It is a
small ontology overlay attached to an existing skill/tool/plugin, not a replacement for that
package.

1. **Ingest without trusting.** Read `SKILL.md`, frontmatter, referenced scripts, tool JSON
   Schemas, repo [`AGENTS.md`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/AGENTS.md), manifests, and CI config. Hash every source.
2. **Deterministically lift explicit structure.** Convert machine-readable names, tool schemas,
   allowed tool sets, file globs, exit codes, timeouts, and script entry points into candidate
   classes, typed arguments, bindings, and constraints. No model is needed for facts already
   explicit in metadata.
3. **Let a model propose the prose mapping.** The authoring agent proposes workflow steps,
   `horis:requires` conditions, declared effects, fallbacks, human gates, and relevant repo-state
   facts. Each clause links back to source text. This is PRP-007's gated-authoring pattern, not a
   trusted compiler pass.
4. **Expose holes, do not guess.** The compiler emits a coverage report: formalized clauses,
   machine-checkable clauses, test-backed opaque claims, prose-only guidance, and contradictions.
   Missing effects or unbounded shell behavior block an “enforced” label but need not block using
   the original skill as advisory text.
5. **Verify and probe.** Profile validation, rule/workflow verification, explicit non-vacuity,
   permission/path checks, and generated scenario witnesses run before sign-off. Execute safe
   probes in a sandbox when static evidence is insufficient.
6. **Human/project-owner sign-off.** Only a signed fragment becomes runtime policy. This follows
   the research conclusion that schema changes require a hard human gate and that traces should
   feed proposals, not self-modification ([`horismos-research-s1/report.md:411-422`](2026-08-24-founding-sweep.md)).
7. **Runtime enforcement.** A host adapter materializes repository state (branch, dirty files,
   language, manifest, test status, approvals, path scopes) as a versioned World projection.
   The reasoner computes eligible capabilities. The model may rank or propose among already-legal
   candidates; the runtime checks arguments/effects and records the proof/trace.
8. **Reconcile.** After a tool/patch, refresh changed repo facts, discard standing derivations,
   and re-chain. Compare the expected git/tree version before applying an effect, exactly like the
   FHIR source-version contract.

### Can an existing Claude skill be compiled?

**Partially, and “compiled” must never imply semantic completeness.** The repository's own
`brainstorming` skill is a useful worked example:

| Skill text | Candidate formalization | Limit |
|---|---|---|
| A design question has exactly one owning PRP ([`.claude/skills/brainstorming/SKILL.md:8-21`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/.claude/skills/brainstorming/SKILL.md)). | Preconditions: one D-item, exactly one owner; zero/two owners produce a blocked diagnostic. | Deciding whether something really is a design question versus an implementation choice still needs interpretation. |
| Produce a falsifiable statement and a blast-radius record (`:23-42`). | Required output shape with crates, units, downstream PRPs, docs, and gates; completeness cardinalities. | “Could be wrong” and accurate blast radius are semantic judgments. |
| Evidence sweep precedes options (`:44-66`). | Deterministic workflow ordering; required evidence categories and citations. | Source quality and relevance are not mechanically decidable. |
| Options -> adversarial pass -> ranked criteria (`:68-101`). | Step graph, minimum/maximum option cardinality, hard gate failures, required ranking artifact. | “Genuinely distinct,” “steelmanned,” and non-criteria-fitted are irreducibly qualitative without human review. |
| Record four artifacts in one commit (`:103-114`). | Observable effects and completion preconditions; git-diff checks can enforce all four changed together. | Whether the rewritten shipped voice faithfully captures the decision is not a filesystem fact. |
| Spike has a question, metric, kill criterion, time box, and record (`:116-126`). | Typed spike plan and a gate that forbids execution until fields are present. | Picking a discriminating metric and kill threshold is domain judgment. |

That is still valuable. Today, a model can forget Step 0 or claim completion without all four
records. With a signed fragment, the runtime can make those omissions impossible. It cannot make
the evidence insightful.

### Formalizable versus irreducibly prose

| Usually formalizable | Usually not formalizable without a domain-specific verifier |
|---|---|
| Capability/tool identity and version; argument/result JSON types; path/host/tool allowlists; required files; environment predicates; workflow order; finite choices; observable preconditions; declared observable effects; timeouts/retries; approval gates; exit-code policy; required tests/lints; artifact existence; provenance; exact completion checklist. | User intent; code quality; architectural taste; “investigate thoroughly”; whether evidence is persuasive; whether options are genuinely distinct; semantic correctness of novel code; the real side effects of arbitrary shell/scripts; visual quality; business/clinical correctness; faithful prose summaries; unstated environment assumptions. |

Scripts remain opaque tool implementations unless instrumented or sandboxed. Static analysis may
suggest effects, but an author must declare and test them. A Markdown `allowed-tools` field says
what the model may call; it does not prove when the call is appropriate or what the tool changes.

### The arbitrary-code problem

The current Horismos contract says the model never constructs tool arguments; arguments are read
from the graph ([`docs/framework/07-tools.md:25-31`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/docs/framework/07-tools.md)). A useful coding agent necessarily generates
novel source text, patches, commands, and sometimes filenames. Pretending this fits an ordinary
ActionStep would weaken the first principle by stealth.

The honest extension is a new, explicitly declared **candidate-artifact boundary**: model output
is a patch/file/command proposal with a typed envelope; deterministic policy validates target
paths, base hashes, syntax, permissions, and required checks; a sandbox executes tests; only then
does a symbolic step authorize application. Semantic correctness remains a review/test claim,
not a gate theorem. This design question belongs to PRP-007/008 and needs a `[DECIDE]` owner before
new vocabulary is minted.

## Does the thin pivot strengthen the coding-agent use case?

**Yes, substantially, if the ontology is task-scoped and overlays the repository/harness.** A
coding team will not replace its git repository, CI, IDE, tool registry, or session harness with
a World. It may adopt a small policy/decision layer that says, for this task and repo state,
which capability may run, what it requires, what it may touch, and what proves completion.

This also solves a current product-shape mismatch. Clinical longitudinal workflows invite the
same “own the CIS” trap as GLIF/SAGE. Coding tasks are naturally short-lived bounded projections:
checkout at commit X + task + tool registry + policy + current diff. That matches snapshot-scoped
verification and conditional application unusually well.

“The agent defines an ontology scoped to the task” fits report §1.7 **only as a proposal into the
gated authoring loop**:

1. the host supplies trusted base vocabularies and non-negotiable policy;
2. the agent proposes a task-local overlay and maps repo/tool observations into it;
3. validation, verification, non-vacuity, and scenario probes run;
4. a human or pre-authorized policy signs the overlay for this task/expiry/base commit;
5. runtime facts may then change freely through the gate, but schema/rules do not self-modify;
6. traces may later propose a reusable fragment revision, never install it directly.

This is exactly the report's recommended “trace-mining as proposal source” rather than a
self-reinforcing ontology loop ([`horismos-research-s1/report.md:411-422`](2026-08-24-founding-sweep.md)). NELL's documented
schema drift and Wikidata's human schema governance reinforce the separation
([`research/ontology-authoring/autonomous-agents-failure-modes.md:5-23`](../ontology-authoring/autonomous-agents-failure-modes.md)).

Task scoping does not eliminate vacuous verification or the formalization gap. An agent can still
write a trivial task ontology that permits everything or omits the dangerous effect. Require:

- a fixed platform policy that task overlays can narrow but not weaken;
- non-vacuity and mutation tests;
- explicit “unformalized clauses” output;
- source hashes/base commit and short expiry;
- no schema change after execution starts;
- a sign-off tier proportionate to risk (human for writes/commands; reusable pre-approved
  fragments for read-only analysis).

## Risks of pivoting

1. **Guarantee laundering.** Marketing may continue saying “every mutation passes the gate” even
   though other FHIR clients or shell processes mutate state. Mitigation: publish the projection/
   conditional-commit claim verbatim and test conflicts.
2. **Adapter becomes the new monolith.** FHIR mappings, terminology, PlanDefinition, CQL, CDS
   Hooks, auth, and server quirks can consume the roadmap. Mitigation: one scenario, one FHIR
   baseline, one PlanDefinition subset, and recommendation-only service output first.
3. **Semantic impedance.** FHIR resources/profiles and SNOMED concepts do not map losslessly to a
   single-typed, subject-major RDFS-plus World. Mitigation: mapping-loss reports, fail-closed
   unsupported cases, and no claim of full FHIR semantics.
4. **Closed-world errors.** A FHIR search omission can be interpreted as negation. Mitigation:
   declared input closure and distinct absent/not-fetched/not-authorized states.
5. **TOCTOU and distributed partial failure.** A valid decision may commit against changed data or
   only some systems. Mitigation: source-version vectors, conditional writes, transaction Bundles,
   explicit commit receipts, and no cross-system atomicity claim.
6. **Standards conformance theater.** Importing a FHIR IRI or PlanDefinition label is not
   interoperability. Mitigation: executable conformance fixtures against real test servers and a
   published supported-subset matrix.
7. **Core contamination.** Network/auth/provider dependencies may creep into `horismos-core`.
   Mitigation: adapter crates, existing boundary gates, and a new no-network-in-transaction test.
8. **Clinical scope overwhelms product learning.** Certification and institutional integration
   can hide whether anyone wants the semantic kernel. Mitigation: build the generic host SPI and a
   coding-harness adapter in parallel conceptually, but implement one clinical slice first.
9. **Capability-fragment false confidence.** A partially compiled skill may be called “verified.”
   Mitigation: formalization coverage and explicit opaque/prose-only clauses in every report.

## Risks of not pivoting

1. **Repeating the TNM adoption history.** Horismos asks adopters to replace domain vocabulary,
   state, workflow, and runtime together—the highest-friction side of the historical comparison.
2. **No answer to the first serious clinical question.** The research explicitly predicts EHR/
   vendor integration as the immediate credibility gap
   ([`research/formalization/00-overview.md:104-106`](../formalization/00-overview.md)).
3. **Authoring before adoption surface.** PRP-007 may optimize creation of artifacts no host can
   consume, amplifying rather than reducing knowledge-acquisition cost.
4. **Demo overfitting.** Triage exercises internal features but does not prove data mapping,
   concurrent update safety, authorization, or service integration.
5. **Terminology drift.** Hand-picked subsets and local vocabulary continue to diverge from
   maintained standard releases.
6. **Coding-agent opportunity remains rhetorical.** Without an adapter/overlay posture, applying
   Horismos to existing repos and harnesses would require replacing their state and control loop,
   recreating the same adoption problem outside health.
7. **The genuinely novel pieces become invisible.** Reviewers can dismiss Horismos as another
   all-owning guideline language before engaging with the gate/TMS/trace/verification contribution.

## Recommended sequence

### Change now (knowledge/design work, before more scope hardens)

1. Record the captain-held product decision in PRP-000 and rewrite docs 00/01/03/14 at the
   contract level. This report recommends; it does not resolve the D-item itself.
2. Create PRP-008 as Draft with the ten D-items above. Change PRP dependency planning from a
   strict numeric sequence to an explicit DAG; do not renumber established PRP-005–007 references.
3. Amend PRP-005 now with assumption manifests and non-vacuity so its verifier does not bake in
   an all-owning-world claim.
4. Amend PRP-007's Draft now to be overlay/import-first and add the existing research open items;
   do not implement it yet.

### Finish before pivot implementation

5. Finish and merge the current PRP-004 scenario/runtime work without dragging FHIR into the
   active branch. Its deterministic workflow/tool kernel is useful under either product shape.
6. Implement PRP-005's core verification work. It is the guarantee engine PRP-008 needs and
   depends only on 001/002 ([`PRPs/PRP-005-guarantees-verification.md:3-11`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/PRP-005-guarantees-verification.md)).

### First pivot proof

7. Activate PRP-008 with **VTE prophylaxis** as the vertical slice: structured EHR inputs, bounded
   scoring/contraindications, typed recommendation/order proposal, and published prior art. The
   milestone is not “FHIR supported”; it is:

   - one declared FHIR input closure hydrates atomically;
   - resource/profile/terminology/mapping versions appear in proof and trace;
   - a conflicting external update aborts the commit;
   - a CDS Hooks-style invocation returns a recommendation or proposed resource;
   - the same decision runs standalone from a recorded Bundle with zero network/model;
   - unsupported/missing data fails in domain terms.

8. Only after that slice, add the deliberately bounded PlanDefinition importer/executor. Compose
   with a CQL engine before considering native CQL execution.

### Then ship and author

9. Activate PRP-006 after PRP-008 so public docs/packages include a real integration story and
   conformance matrix.
10. Activate PRP-007 after the mapping/package contracts are stable. Its first acceptance case
    should modify a standards-anchored VTE overlay, not regenerate triage from scratch.
11. Prototype the coding-harness adapter and `brainstorming` capability fragment after the generic
    host SPI exists. Treat it as a second domain proof, not as permission to generalize clinical
    vocabulary ad hoc.

## Evidence trail and reproducibility

Repository inspection commands (read-only) included:

```text
rg --files research PRPs docs/framework .claude/skills
# originally the untracked founding-sweep report file, imported to research/reports/ on 2026-09-03
nl -ba research/reports/2026-08-24-founding-sweep.md | sed -n '1,111p;340,440p;810,837p'
nl -ba research/formalization/{00-overview,glif,tnm-family,cql-cds-hooks-fhir}.md
nl -ba research/scenarios/{00-catalog,vte-prophylaxis,news2-deterioration,sepsis-bundle-compliance,ruled-out}.md
nl -ba PRPs/PRP-000-vision-positioning.md PRPs/PRP-005-guarantees-verification.md PRPs/PRP-006-packaging-distribution.md PRPs/PRP-007-authoring-agent.md
nl -ba PRPs/PRP-00{1,2,3,4}-*/{01-objective,02-tasks-and-gaps}.md
nl -ba docs/framework/00-index.md ... docs/framework/15-security.md
rg -n '\[DECIDE' docs/framework PRPs
git status --short --branch
```

Observed repository state: detached `HEAD`, no worktree changes before this external report was
written. Key current-plan evidence: PRP-001/002/003 are Done, PRP-004 is Active, and 005/006/007
are Draft ([`PRPs/BOARD.md:102-117`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/PRPs/BOARD.md)).

Current harness documentation was inspected through `chrome-devtools-axi`, per the task rule:

```text
chrome-devtools-axi open https://code.claude.com/docs/en/skills
chrome-devtools-axi open https://github.com/earendil-works/pi/tree/main/packages/coding-agent
chrome-devtools-axi open https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/skills.md
chrome-devtools-axi open https://github.com/deepseek-ai/deepseek-harness
chrome-devtools-axi open https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/architecture.md
chrome-devtools-axi open https://github.com/deepseek-ai/deepseek-harness/blob/master/docs/tool-execution-pipeline.md
```

The DeepSeek material was especially recent (developer preview, compatibility-breaking changes
explicitly expected), so conclusions about its current architecture should be rechecked when a
PRP actually depends on them. No GitHub mutation, push, issue, or PR was performed. I also rendered
a temporary Lavish review surface using the project's [`planning/board.html`](https://github.com/horismos-hq/horismos/blob/1c7ce1c/planning/board.html) design tokens; Chrome's
layout check reported `scrollWidth == clientWidth` (1100 px), and the full PRP table and decision
cards were accessible. The session was ended and the disposable HTML removed after review.

## Bottom line

The right pivot is not “Horismos becomes a FHIR wrapper” and not “delete World.” It is:

> **Horismos is a deterministic, ontology-first decision and capability kernel that evaluates a
> versioned projection of an existing system, proves why an effect is allowed, and conditionally
> commits through standards-native adapters.**

That framing preserves the work that is genuinely novel, directly answers the historical adoption
failure, makes clinical interoperability concrete, and gives the coding-agent case a much better
fit. The cost is a narrower, more honest guarantee: Horismos proves its decision epoch and its
mediated effects—not the correctness of an entire EHR, repository, tool, or world it does not own.
