# Pivot alignment: is the bounded-projection kernel where the field is going?

<!-- Standing page. Re-assessed each weekly sweep against the commitments listed below;
     the verdict line is re-issued (not silently edited) — see "Verdict history" below. -->

## Claim

**[inference — the verdict; every fact it rests on is tagged and cited below]**
**Verdict (2026-09-02, sweep #2): aligned-with-adjustments.** Every load-bearing
commitment of the bounded-projection pivot adopted 2026-08-24 — a snapshot-scoped
guarantee over a versioned hydration, a conditional commit that fails closed on
source-version drift, a thin overlay on CDS Hooks/FHIR/CQL rather than a replacement
record, a gate that emits proofs rather than booleans, and a deterministic
pre-execution gate over declared constraints for tools and model-proposed artifacts —
now has independent, post-August-2026 confirmation from at least two of the database,
formal-methods, agent-safety, and HL7 standards communities. Where the field moved
differently, it moved toward the *neural* side of the exact line Horismos draws (model
classifiers at the approval step, self-refining ontologies without a human gate,
LLM-adjudicated write conflicts), and the measured evidence in those same papers
favours Horismos's side. The adjustments are real but shape-preserving: type proposed
effects by reversibility, state and measure the utility tax of tight closures, harden
the narration boundary with a deterministic conclusion monitor, and pin several PRP-008
D-items to evidence that now exists. None requires changing the pivot's shape.

## The commitments assessed

**[measured — local repo]** The pivot is stated in [`PRPs/PRP-000-vision-positioning.md`](https://github.com/billaram/horismos/blob/main/PRPs/PRP-000-vision-positioning.md)
§"Snapshot-scoped guarantees", [`PRPs/PRP-008-standards-host-integration/01-objective.md`](https://github.com/billaram/horismos/blob/main/PRPs/PRP-008-standards-host-integration/01-objective.md),
[`docs/framework/03-world.md`](https://github.com/billaram/horismos/blob/main/docs/framework/03-world.md) §"World as a transaction projection",
[`docs/framework/06-llm-boundaries.md`](https://github.com/billaram/horismos/blob/main/docs/framework/06-llm-boundaries.md) §"Candidate-artifact authoring",
[`docs/framework/07-tools.md`](https://github.com/billaram/horismos/blob/main/docs/framework/07-tools.md) §"Adapter-mediated external commits",
[`docs/framework/10-verification.md`](https://github.com/billaram/horismos/blob/main/docs/framework/10-verification.md) §"The assumption manifest", and
[`docs/framework/13-authoring.md`](https://github.com/billaram/horismos/blob/main/docs/framework/13-authoring.md) §"Skills-to-fragment loop and trace-mining". This page
assesses eight commitments drawn from those texts:

- **C1 — Snapshot-scoped guarantee**: hydrate a declared, versioned input closure into a
  local projection; reason to a fixed point; guarantees hold for that snapshot only.
- **C2 — Conditional commit**: a typed `ProposedEffectSet` commits only if the source
  still matches the recorded source-version vector; fail closed on drift; no
  cross-system atomicity claim; "decision produced / commit attempted / accepted /
  failed" are distinct trace states.
- **C3 — Thin overlay**: behind a CDS-Hooks-shaped service, over FHIR resources, composing
  with an external CQL evaluator, importing a bounded PlanDefinition subset.
- **C4 — Gate as proof-emitter; traces are output**: every admitted fact passes the
  five-check gate; every conclusion carries a proof; the trace is the product.
- **C5 — Neural proposes, symbolic decides**: no rule firing, transition, tool call, or
  commit is made by a model; LLMs only at perception and narration.
- **C6 — Capability fragments**: a small overlay attached to an existing skill/tool that
  proves eligibility, constrains observable effect, emits a proof — without replacing
  the harness's own state, sandbox, or approval machinery.
- **C7 — Candidate-artifact boundary**: a model-proposed patch is a typed envelope →
  deterministic policy (paths, base hashes, syntax, permissions) → sandbox checks →
  symbolic authorization; policy-legality is proven, semantic correctness is not.
- **C8 — Assumption manifest and the gated ontology loop**: verification reports declare
  their scope (input closure, versions, supported subset, unverified effects); ontology
  changes — including trace-mined proposals — enter only through verifier + SME sign-off.

## What the new literature CONFIRMS

### C1/C2 — snapshot-scoped guarantee and fail-closed conditional commit

**[measured]** The multi-agent-systems literature now names the failure class the
version vector exists to exclude. A position paper argues "many MAS failures are
fundamentally concurrency control problems: agents concurrently read and write shared
state, and long LLM inference windows amplify the risk of stale reads, lost updates, and
inconsistent outcomes" ([arXiv:2608.18092](https://arxiv.org/abs/2608.18092); the
abstract page's submission date reads June 2026 despite the August identifier). A
verified consistency hierarchy for multi-agent LLM systems — pessimistic locking,
serializable snapshot isolation, snapshot isolation — realized in three Rust runtimes
with 274 Verus proof obligations and TLA+/TLC models reports 0/1000 vs 1000/1000
anomalies between protected and unprotected modes
([arXiv:2606.17182](https://arxiv.org/abs/2606.17182)); its companion S-Bus validates a
reconstructed read set at commit with TLAPS-proved read-set soundness across 884,110
commit attempts ([arXiv:2605.17076](https://arxiv.org/abs/2605.17076)). "Agentic
Transaction" reinterprets ACID for agent execution as semantic atomicity, consistency,
isolation, and durability ([arXiv:2608.13900](https://arxiv.org/abs/2608.13900)).

**[measured]** Three August-2026 papers independently find that leaving staleness
handling to the model fails and mechanical version checks work. "Invalidation
Contracts" attaches version stamps and cacheability hints to cached suggestions:
version-stamp validity was 100% deterministic across ~9,400 episodes, while
model-honoured hints ranged from 100% down to 11% depending on the model
([arXiv:2609.00243](https://arxiv.org/abs/2609.00243)). "When Stale Constraints Go
Unchecked" finds models inspected a superseded record's provenance path in roughly one
episode in five and produced stale-aligned decisions 74.7–77.3% of the time; rule-based
budget allocation recovered +89.3 points
([arXiv:2608.25553](https://arxiv.org/abs/2608.25553)). "Approved Too Late" (ACSOS 2026)
frames guardrail approval as "an Execute-stage TOCTOU hazard" — "correct at check
time" but "stale by actuation" — measuring verdict-change rates of 5.3–48.4% at an
eight-step replay shift and proposing a "freshness contract" recording checked
context, grounding time, expiry rule, and fallback
([arXiv:2608.26306](https://arxiv.org/abs/2608.26306)). AID-Guard "revalidates the
approved request and provider state at commit" and "requires the provider to enforce the
approved version, ETag, or lock precondition when authorization depends on mutable
resource state," failing closed on mismatch
([arXiv:2608.21159](https://arxiv.org/abs/2608.21159)). TOKI's verdict matrix over eight
memory systems finds "every baseline that keeps a language-model judge on the write path
admits at least one of three write-time anomalies (replay inconsistency, belief-drift
skew, audit erasure)" ([arXiv:2606.06240](https://arxiv.org/abs/2606.06240)).

**[measured]** On the standards side the conditional-commit mechanism is
spec-sanctioned and discoverable. FHIR R6's HTTP page (Standards Status: Normative)
states "If the version id given in the `If-Match` header does not match, the server
returns a `412 Precondition Failed`," servers "SHOULD provide full version support," and
"Servers can require that clients provide an `If-Match` header by returning `400 Bad
Request`" ([build.fhir.org/http.html](https://build.fhir.org/http.html));
`Bundle.entry.request.ifMatch` carries the precondition inside a transaction
([bundle-definitions](https://build.fhir.org/bundle-definitions.html)); and
`CapabilityStatement.rest.resource.versioning = versioned-update` advertises "using
e-tags for version integrity in the API"
([capabilitystatement-definitions](https://build.fhir.org/capabilitystatement-definitions.html)).
Oracle Health Millennium's R4 API makes `If-Match` *required* on update
([docs.oracle.com](https://docs.oracle.com/en/industries/health/millennium-platform-apis/mfrap/op-condition-id-put.html)).
An expired individual IETF draft proposed the same shape generically: "State-changing
operations on AST resources MUST provide a precondition (If-Match)"
(draft-jurkovikj-httpapi-agentic-state-00, 2025-12-10, expired 2026-06-13).

### C3 — thin overlay on CDS Hooks / FHIR / CQL / PlanDefinition

**[measured]** Epic put hook-triggered, third-party decision services into production
at scale on 2026-08-17: Da Vinci Coverage Requirements Discovery is live at Ochsner,
Froedtert ThedaCare, Denver Health and Summit Health with UnitedHealthcare, Aetna and
Network Health, "16 additional health insurers undergoing sandbox integration testing,"
five months ahead of the CMS 2027-01-01 deadline
([Epic newsroom](https://www.epic.com/epic/newsroom/); [HIT
Consultant](https://hitconsultant.net/2026/08/17/epic-deploys-real-time-crd-prior-authorization-apis-ochsner-froedtert/);
[TechTarget](https://www.techtarget.com/revcyclemanagement/news/366649207/Health-systems-test-real-time-prior-auths-through-Epic)).
The coverage does not name CDS Hooks; that the Da Vinci CRD IG is a CDS Hooks profile
is the sweep's own knowledge of the IG, not stated in those pieces. CDS Hooks 3.0.0 is
in an "R3 Normative Ballot" whose core (request/response, hook model, security) is
normative on FHIR 4.0.1, with new discovery `version`/`hookVersion` fields and no
AI/LLM-specific hook anywhere in the changelog
([build.fhir.org/ig/HL7/cds-hooks](https://build.fhir.org/ig/HL7/cds-hooks/); 2.0.1
remains the published version as of 2026-09-02). FHIR R6's second full Normative
ballot (v6.0.0-ballot5) marks PlanDefinition, ActivityDefinition and
RequestOrchestration Normative with "All Changes: None" for the Clinical Reasoning
module this cycle ([ballot intro](https://hl7.org/fhir/6.0.0-ballot5/ballot-intro.html);
[plandefinition](https://build.fhir.org/plandefinition.html)). CQL Release 2 published
as STU on 2026-08-12 with "no breaking changes" and new directives, model qualifiers,
and in-language model definitions ([standups.hl7.org](https://standups.hl7.org/2026/08/12/stu-publication-of-hl7-specification-clinical-quality-language-cql-release-2/);
[cql.hl7.org](https://cql.hl7.org/)); the CQL-on-FHIR IG's STU3 ballot adds a
`$evaluate` operation and `evaluatedResource` references — a typed result surface a
consumer can carry as provenance. Two engines ship weekly-to-monthly: cqframework CQL
5.2.0 (2026-08-19, Kotlin multiplatform) and Firely CQL SDK 2.14.0 (2026-08-20, with
"closed-world semantics when a value set is resolved locally" and exact-opaque version
matching) ([cqframework releases](https://github.com/cqframework/clinical_quality_language/releases);
[Firely releases](https://github.com/FirelyTeam/firely-cql-sdk/releases)).
**[inference]** The build-vs-compose question for CQL (PRP-008 D7) is settled by release
velocity alone: a home-built engine would chase a language that gained three feature
classes this quarter.

### C4 — gate as proof-emitter; traces are output

**[measured]** "Evidence-Carrying Validation for Knowledge Graphs" argues that programs
consuming a KG they don't maintain — "applications, authoring platforms, and LLM
agents" — need satisfaction traces and failure witnesses, not conformance bits; every
selected SHACL node-shape check "returns either a satisfaction trace or failure
witness," at a median 1.54–2.07x the cost of conformance-only validation
([arXiv:2608.15948](https://arxiv.org/abs/2608.15948)). "Trace Integrity for LLM Data
Agents" quantifies why answer-level evaluation cannot substitute for a checkable trace:
on BIRD Mini-Dev, answer accuracies of 20–24% coexist with correct-answer-invalid-trace
rates of 45.8–59.1% ([arXiv:2608.26036](https://arxiv.org/abs/2608.26036)). Agent Zero
Memory reads answers under a "citation lock" so the system "may cite only evidence its
reader actually opened; fabrication is structurally excluded"
([arXiv:2608.29606](https://arxiv.org/abs/2608.29606)). Quipu makes "the governance
specification Σ, the trace, and signed verdicts … facts in the store they govern" and
reports gated writes catching 6/6 planted defects vs 0/6 ungated
([arXiv:2608.16813](https://arxiv.org/abs/2608.16813)). AGENT-O's scoring of 279
clinical-agent papers found governance/safety unreported in 82.8% and
provenance/reproducibility in 78.1% ([arXiv:2608.28345](https://arxiv.org/abs/2608.28345)).

### C5 — neural proposes, symbolic decides

**[measured]** "Reason Less, Verify More" finds "78% of observed failures are silent
wrong-state failures with no tool error" in τ²-bench airline and that "deterministic,
read-only pre-execution gates that inspect the proposed call and current state before
allowing a write" raise success 29.6% → 42.0% (+12.4 pp, P=0.0012, replicated), while
stating exactly Horismos's scope: gates "do not guarantee task success, but they can
deterministically prevent a known class of silent policy-violating writes"
([arXiv:2607.07405](https://arxiv.org/abs/2607.07405)). In industrial asset operations,
LLM agents over flat documents scored 65%, LLM-generated Cypher over a typed KG 82–83%,
and "native graph and optimization primitives, with no LLM" 99% on graph-answerable
scenarios ([arXiv:2605.26874](https://arxiv.org/abs/2605.26874)). MemStrata shows
similarity retrieval cannot tell a contradicted fact from a duplicate (AUROC 0.59) while
a deterministic supersession rule "with no similarity threshold and no LLM call" drives
served-stale-value rates from 15–40% to ~0%
([arXiv:2606.26511](https://arxiv.org/abs/2606.26511)). A KG-validation benchmark finds
LLM validators "lack sufficient stability" ([arXiv:2602.10748](https://arxiv.org/abs/2602.10748)).
Self-evolving world-model work concedes its own premise — "unreliable foresight can be
ignored, misused, or even degrade downstream decision-making" — and gates predictions by
confidence before they influence decisions
([arXiv:2606.30639](https://arxiv.org/abs/2606.30639)).

### C6 — capability fragments

**[measured]** Five independent 2026 groups converge on a deterministic pre-execution
gate over declared constraints and current state: SMT-LIB policies over
"agent-observable state and tool arguments" checked by Z3
([arXiv:2603.20449](https://arxiv.org/abs/2603.20449)); first-order LTL over tool-call
traces with online block-and-warn ([arXiv:2607.02599](https://arxiv.org/abs/2607.02599));
ABAC at an MCP proxy so "unauthorized tools never reach the model context" (0% vs 4–37%
violation rates for prompt-only) ([arXiv:2605.18414](https://arxiv.org/abs/2605.18414));
in-kernel eBPF enforcement of agent-declared policies because "tool-call guardrails
cannot intercept system actions that bypass the tool layer"
([arXiv:2606.25189](https://arxiv.org/abs/2606.25189)); and "Reason Less, Verify More"
above. Ontology-typed tool parameters cut domain-identifier hallucination from 43% to
0% over 72 invocations ([arXiv:2605.11234](https://arxiv.org/abs/2605.11234)). No shipped
harness declares world-state preconditions: OpenAI's Codex rules doc states rules
"evaluate command prefixes only — they cannot reference repository state, file contents,
or environmental conditions" ([learn.chatgpt.com](https://learn.chatgpt.com/docs/agent-configuration/rules));
Gemini CLI's policy engine predicates over `toolName`, `argsPattern`, `commandPrefix`,
`mcpName`, and `toolAnnotations` only
([policy-engine.md](https://github.com/google-gemini/gemini-cli/blob/main/docs/reference/policy-engine.md));
the MCP maintainers write that annotations "are not guaranteed to faithfully describe
tool behavior" and clients should "implement deterministic controls rather than relying
solely on hints" ([MCP blog](https://blog.modelcontextprotocol.io/posts/2026-03-16-tool-annotations/)).
Full harness survey: [`coding-agent-harnesses.md`](coding-agent-harnesses.md).

### C7 — candidate-artifact boundary

**[measured]** No core harness ships base-hash-checked patch application. Claude Code
issue #25775 proposing per-line content hashes was closed "not planned"; OpenCode's
experimental hashline edit "silently returns success when LINE#ID anchors are stale"
and was likewise closed; the only working implementations are third-party tool
overrides ([#25775](https://github.com/anthropics/claude-code/issues/25775);
[opencode #15424](https://github.com/anomalyco/opencode/issues/15424);
[pi-hashline-edit](https://pi.dev/packages/pi-hashline-edit)). Codex's `apply_patch`
envelope is typed (`create_file | update_file | delete_file`, V4A context anchors) but
defines no base hash, version, or policy stage — mismatches surface only as `failed`
([OpenAI docs](https://developers.openai.com/api/docs/guides/tools-apply-patch)).
Claude Code v2.1.257 fixed "file tools … following a symlink swapped inside the
working directory after the permission check" — a TOCTOU between authorization and
effect — and its worktree isolation ships a non-disableable "command shape" check that
"blocks a Bash or Monitor command it can't verify stays inside the worktree"
([changelog](https://code.claude.com/docs/en/changelog);
[worktrees](https://code.claude.com/docs/en/worktrees)). A single-author paper (ATM)
independently articulates admit → record base hash → re-read → apply-or-reroute
([arXiv:2607.00041](https://arxiv.org/abs/2607.00041)). KG-grounded code generation for
HEP software lifts Claude Code final success from 90.5% to 96.0% via structural
grounding plus execution-guided repair, leaving a 4–9% residual after repair
([arXiv:2609.01095](https://arxiv.org/abs/2609.01095)). The 2025–26 incident record
(PocketOS: a blanket-scope token issued a `volumeDelete` that destroyed production data
and its backups; "Guardrails must be implemented at the API gateway and
token-permission level, not in advisory text") shows the failures are below the prompt
([AIID 7311](https://incidentdatabase.ai/reports/7311)).

### C8 — assumption manifest and the gated ontology loop

**[measured]** FORGE (Datalog policies for agentic systems) states its Verdict
Correctness theorem only "when the environment contract holds" — an assume/guarantee
contract with Soundness and Sufficiency — and runs four static analyses before
deployment (contradiction, redundancy, subsumption, conditional reachability)
([arXiv:2602.16708](https://arxiv.org/abs/2602.16708)). Proof-carrying certificates for
LLM pipelines ship each certificate with assumptions "declared and partitioned by tier
(mathematical placeholders, cryptographic assumptions, ML/human oracles)"
([arXiv:2605.16407](https://arxiv.org/abs/2605.16407)). AID-Guard states six explicit
provider-contract assumptions A1–A6. A PRISMA survey of 38 agent-safety studies finds
"no existing approach simultaneously achieves soundness, scalability, semantic
correctness, and task-level safety preservation" and that NL→formal translation
"achieves only 24% to 35% semantic correctness"
([arXiv:2608.14590](https://arxiv.org/abs/2608.14590)). ShadowBench (EMNLP 2026) puts
the formalization gap in numbers: Claude Opus 4.8 reaches 61.8% Lean compile rate but
11.2% semantic-alignment pass ([arXiv:2608.29270](https://arxiv.org/abs/2608.29270)).
**[inference]** Each of these is the "no semantic-correctness theorem" posture stated by
someone else; the assumption manifest has an emerging peer group even though the term
itself appears nowhere in the surveyed literature.

**[measured]** On the gated ontology loop: GrOIL produces an OWL TBox "without any
unconstrained generation step" — LLM choices restricted to closed vocabularies by
guided decoding, deterministic compilation, and "low-confidence assignments … flagged
automatically for human review and … not committed to the TBox until inspected"
([arXiv:2608.22135](https://arxiv.org/abs/2608.22135)). Practitioner guidance converges:
Cognee's 2026-08-31 memory-graph guide says "Automate merges only under a defined
confidence policy … Leave ambiguous references separate until more evidence or human
review resolves them" ([cognee.ai](https://www.cognee.ai/blog/fundamentals/memory-graph-for-ai-agents));
Packkildurai prescribes that new relationships be "hypotheses … separate from verified
edges" with lifecycle states
([dataengineeringweekly.com](https://www.dataengineeringweekly.com/p/an-ontology-for-ai-agents-is-a-system));
KGC 2026 notes record "The ontology stops being the output of the LLM and becomes the
harness around it" and a seven-step method ending in an explicit Release step
([Futia, Medium](https://medium.com/@giuseppefutia/notes-from-kgc-2026-c9b4ac8569e5)).
Detail: [`../ontology-authoring/autonomous-agents-failure-modes.md`](../ontology-authoring/autonomous-agents-failure-modes.md) (sweep #2 delta).

## What it CHALLENGES, or where the field moved differently

**X1 — The approval step is going neural.** **[measured]** Codex v0.149.0 removed the
deterministic `approval_policy = "untrusted"` tier with no deprecation period (issue
#39973 open, unanswered) while `approvals_reviewer = "auto_review"` routes approvals to
"an automatic reviewer agent"
([openai/codex #39973](https://github.com/openai/codex/issues/39973);
[security doc](https://learn.chatgpt.com/docs/agent-approvals-security)). Claude Code
added `auto` mode (a model classifier) and `prompt`/`agent` hook handler types that put
a model inside the permission decision
([permission-modes](https://code.claude.com/docs/en/permission-modes);
[hooks](https://code.claude.com/docs/en/hooks)). Cursor calls its run modes, up to the
Auto-review classifier, "best-effort guardrails rather than a hard security boundary"
([cursor.com/docs/agent/security](https://cursor.com/docs/agent/security)).
**[inference]** This is the opposite direction from "symbolic decides" at the exact
step where a capability fragment would sit — and the incident record plus the
staleness-compliance numbers above are the argument for Horismos's side. The
positioning consequence: a fragment must be presented as the deterministic hook that
runs *before* any classifier, not as an alternative to it.

**X2 — Self-refining ontology loops without a human gate report benchmark gains.**
**[measured]** OaK ("ontology-as-a-kernel", 2026-08-24) refines a task ontology with an
LLM judge plus a HermiT consistency check and no human review, reporting TravelPlanner
59.31% vs ReAct 44.29% and CRMArenaPro 78.38% vs MemP 66.44%
([arXiv:2608.22974](https://arxiv.org/abs/2608.22974)). ASKS lets a schema/hub layer
emerge from embedding geometry, controlling drift by measuring churn rather than by a
gate ([arXiv:2608.29612](https://arxiv.org/abs/2608.29612)). SymbolLKG routes
LLM-extracted rules straight to solvers with no stated verification step
([arXiv:2608.26836](https://arxiv.org/abs/2608.26836)). For QA over policy text, "an
open, LLM-discovered schema matches or exceeds the formal ontology"
([arXiv:2604.27713](https://arxiv.org/abs/2604.27713)). **[measured]** The
counterweight is a security result: "single-axiom OWL edits achieve 93.3% SIV success
with 100% Pellet-consistency stealth" — a reasoner-consistency check is not a security
gate ([arXiv:2606.17223](https://arxiv.org/abs/2606.17223)). **[inference]** The human
gate must therefore be defended on safety grounds (stealth edits, drift), not on
accuracy — and the verifier's consistency pass must be complemented by SME review of
the semantic diff and by non-vacuity, exactly the PRP-007 open items sweep #1 filed.

**X3 — Conflict handling: the field explores both weaker and stronger mechanisms than
version-equality.** **[measured]** CoAgent argues locks and OCC fit LLM agents poorly
("read sets are broad and opaque rather than statically inferable"; OCC "discards
minutes of work on every conflict") and lets "the LLM inside each agent … judge whether
a conflicting write invalidates its plan" with saga-style inverses
([arXiv:2606.15376](https://arxiv.org/abs/2606.15376)); Agentic Transaction's isolation
is LLM-judged via "confidence divergence" ([arXiv:2608.13900](https://arxiv.org/abs/2608.13900)).
In the other direction, Quipu evaluates its gate "against the pending post-state" and
Atomix commits tool effects "only after per-resource frontiers show that no earlier
conflicting work can still arrive," typing effects as bufferable, reversible, or
irreversible ([arXiv:2608.16813](https://arxiv.org/abs/2608.16813);
[arXiv:2602.14849](https://arxiv.org/abs/2602.14849)). **[inference]** CoAgent's
premise does not hold for Horismos: a declared input closure makes the read set
statically known, so version-checked OCC is cheap here. Atomix's effect taxonomy is the
one thing `ProposedEffectSet` does not yet type (see A1).

**X4 — Tight closures and manifests carry a measurable utility tax.** **[measured]**
AID-Guard's "strict exact-manifest profile reduced benign utility by 35.4–43.8
percentage points," recovered partly by a typed-widening profile
([arXiv:2608.21159](https://arxiv.org/abs/2608.21159)); the agent-safety survey's
"verifier tax" finding is that "blocking 94% of unsafe actions can still result in less
than 5% safe task completion" ([arXiv:2608.14590](https://arxiv.org/abs/2608.14590));
PolicyGuide moves from guarding one action to steering the whole workflow, raising mean
Pass^4 from 0.42 to 0.62 ([arXiv:2608.19861](https://arxiv.org/abs/2608.19861)).
**[inference]** Horismos should expect, state, and measure this tax rather than
discover it in deployment (see A9).

**X5 — The two neural boundaries remain the weak link, and the narration boundary is
an attack surface.** **[measured]** Autoformalization studies show "scope laundering"
and "implicit constraint blindness" across all models
([arXiv:2606.16118](https://arxiv.org/abs/2606.16118)); models "mistranslate premises
during formalization, producing internally consistent outputs that evade detection
entirely" ([arXiv:2604.19459](https://arxiv.org/abs/2604.19459)). On narration, "the
certificate ties to a property of a formula, not to the sentence that [the] LLM model
outputs": under prompt injection five open models inverted the verified conclusion in
61% of 120 trials, a hardened prompt cut this to 14% but left the stealthy share nearly
unchanged (0.57 → 0.53), and "the only defense that drives their rate to zero is
enforcement, which reports ρ(v) in place of the model conclusion"
([arXiv:2606.19588](https://arxiv.org/abs/2606.19588)). **[inference]** The pivot
*helps* here: it promotes the structured-input bypass to the primary path
([`docs/framework/06-llm-boundaries.md`](https://github.com/billaram/horismos/blob/main/docs/framework/06-llm-boundaries.md)), moving the highest-risk boundary out of the
main flow for standards-hydrated deployments. It does not address narration; a
deterministic conclusion monitor (proof verdict vs narrated conclusion) is the missing
piece (see A8).

**X6 — EHR vendors are internalizing policy-customized agents without opening an
agent-protocol surface.** **[measured]** Epic's UGM 2026 introduced Agent Factory
("more than 120 pre-built AI agents, customize them to organizational policy … without
writing code," broader availability 2027) and Ergo (November 2026); no coverage
mentions third-party plug-ins, MCP, CDS Hooks, or new decision-service APIs
([HealthTech HotSpot](https://healthtechhotspot.com/epic-ugm-2026-recap-cosmos-curiosity-epicops-and-an-expanding-ai-footprint/);
[Advisory Board](https://www.advisory.com/daily-briefing/2026/08/26/epic-ugm-ab-oi-ec)).
Oracle Health's 2026-08-19 expansion says agents "use semantic reasoning" with no
FHIR or third-party surface mentioned
([oracle.com](https://www.oracle.com/news/announcement/oracle-health-expands-clinical-ai-agent-with-coding-dictation-chart-review-2026-08-19/)).
No major EHR exposes a first-party MCP server
([eureka.md](https://eureka.md/docs/ehr-mcp-servers)). **[inference]** This challenges
the thin-overlay business case at the margin and confirms its technical target: CDS
Hooks + SMART + FHIR REST is the only durable third-party decision surface, and it just
went to production scale (C3).

**X7 — Several PRP-008 D-items now have evidence pointing a specific way.**
**[measured]** StructureMap and the FHIR Mapping Language were moved out of R6 core to
an incubator IG ("not ready for normative"), while FHIR RDF/Turtle, ShEx and the OWL
ontology become Normative in R6 — with modifier extensions omitted "because modifier
extensions could introduce non-monotonicity"
([rdf.html](https://build.fhir.org/rdf.html); [ballot intro](https://hl7.org/fhir/6.0.0-ballot5/ballot-intro.html)).
CDS Hooks 3.0, both CQL engines, and the AI Transparency IG all bind to R4 (4.0.1).
Oracle Millennium appears to return 409 rather than 412 on version mismatch (search
snippet only → [unverified]). CDS Hooks 3.0 moved `fhirAuthorization` to STU and
switched to SMART App Launch STU 2. **[inference]** Bearing: D1 (baseline) is really
"R4 wire + R6-shaped PlanDefinition import"; D2 (mapping) now disfavours StructureMap
and favours an RDF bridge with modifier-extension rejection; D3 (source consistency)
must treat 409/412 and weak ETags as equivalent drift signals; D4/D5 favour
prefetch-declared closures over service-side fetch.

**X8 — Trace transparency is becoming a differentiator, not a mandate.** **[measured]**
HTI-5 (proposed 2025-12-29, comments closed 2026-02-27, no final rule as of
2026-09-02) would "fully remove the artificial intelligence (AI) 'model card'
requirements" from DSI certification §170.315(b)(11) and remove legacy CDS
§170.315(a)(9) ([fact sheet](https://www.healthit.gov/topic/laws-regulation-and-policy/hti-5-proposed-rule-fact-sheet)).
FDA's revised CDS guidance (January 2026) keeps Criterion 4 — the HCP must be able to
"independently review the basis for the recommendations" — and now exercises
enforcement discretion for singular recommendations, while remaining "silent on
AI-enabled tools" ([Covington analysis](https://www.cov.com/en/news-and-insights/insights/2026/01/5-key-takeaways-from-fdas-revised-clinical-decision-support-cds-software-guidance);
FDA pages returned 404 → [unverified] on exact issue date). **[inference]** A card
whose narration cites the proof is the Criterion-4 artifact; an authorized write with
no HCP review step would put a deployment on the device side of the line — relevant to
D5 (output posture), not decided here.

**X9 — The absent-vs-not-fetched distinction has no peer.** **[measured]** FORGE adopts
the opposite stance — "Any fact absent from ℰ(Gt,vt) may be treated as false during
evaluation" ([arXiv:2602.16708](https://arxiv.org/abs/2602.16708)) — and the sweep found
no literature treating the distinction as a verification concern for agents.
**[inference]** Horismos is alone here, which is both a differentiator to state in
PRP-005 and a sign the mechanism has no external validation yet.

**X10 — Naming and positioning collisions.** **[measured]** OaK calls its frozen
ontology "the kernel"; Palantir's August 2026 SuperRepo ships "Ontology definitions,
your functions, and your React application together … as one versioned artifact" and
Huang & Gupta describe its agent pattern as "Agents propose; they do not execute
directly," with "DecisionReceipt" objects recording inputs, constraints, approvals and
outcomes ([Palantir announcements](https://www.palantir.com/docs/foundry/announcements/2026-08);
[kenhuangus.substack.com](https://kenhuangus.substack.com/p/a-technical-follow-up-building-an)).
**[inference]** The differentiator must now be stated against a shipping product that
owns the write path: the same guarantees *without owning state*, and a proof rather
than a receipt.

## Adjustments worth considering

Each is **[inference]**, listed with the measured facts it rests on and the owner who
would decide it. None is applied by this wiki.

- **A1 — Type effects by reversibility in `ProposedEffectSet`.** Rests on: Atomix's
  bufferable/reversible/irreversible taxonomy; "Verified Tool Calls Under Non-Atomic
  Failures" (postcondition verification, verify-before-retry, idempotency keys,
  [arXiv:2608.02645](https://arxiv.org/abs/2608.02645)); AID-Guard's reservation-under-
  ambiguity. Owner: PRP-008 D10 (generic host SPI) and D3.
- **A2 — Probe and pin the server's version contract at adapter init.** Read
  `CapabilityStatement.rest.resource.versioning` and transaction support per resource
  type; require `versioned-update`; treat 409, 412 and weak ETags as drift; fail closed
  otherwise. Rests on: R6 HTTP/CapabilityStatement text; Oracle Millennium docs. Owner:
  PRP-008 D3.
- **A3 — Prefer prefetch-declared closures over service-side fetch.** Rests on: CDS
  Hooks 3.0 moving `fhirAuthorization` to STU while the request/response core goes
  normative. Owner: PRP-008 D4/D5.
- **A4 — Re-weigh D2 toward an RDF bridge and away from StructureMap**, with explicit
  modifier-extension rejection. Rests on: R6 ballot5 moving StructureMap/FML to an
  incubator IG and making FHIR RDF/OWL/ShEx Normative minus modifier extensions. Owner:
  PRP-008 D2.
- **A5 — State the baseline as R4 wire plus an R6-shaped PlanDefinition subset.** Rests
  on: CDS Hooks 3.0, cqframework, Firely and the AI Transparency IG all on 4.0.1; R6
  PlanDefinition Normative in ballot5. Owner: PRP-008 D1/D6.
- **A6 — Label every proposed resource and point Provenance at the proof.** Rests on:
  the AI Transparency on FHIR IG (`hl7.fhir.uv.aitransparency` 1.0.0-ballot) explicitly
  covering "non-AI algorithms" and Zulip sentiment "for making the label required"
  ([Health Samurai notes](https://www.health-samurai.io/events/agents-on-fhir/2026-08-06-meeting)).
  Owner: PRP-008 D5 and D9 (trace schema).
- **A7 — Hydrate terminology with `check-system-version` and record
  `expansion.parameter` in the trace.** Rests on: R6 `$expand` parameters; 2027 eCQM's
  pinned vector (SNOMED CT US 2025-09, LOINC 2.81, RxNorm 2026-01); VSAC labeled
  expansions; SNOMED's 255→4096 description-length change and monthly International
  cadence. Owner: PRP-008 D8. Detail: [`../terminology/00-overview.md`](../terminology/00-overview.md) (sweep #2 delta).
- **A8 — Add a deterministic conclusion monitor at the narration boundary.** Compare the
  narrated conclusion to the proof's verdict/effect set and report the kernel's verdict
  when they differ. Rests on: the Narration Gap paper's Theorem 3.1 (a conclusion
  monitor is sound and complete) and its 61% inversion rate. Owner: doc 06's
  NarrationStep contract (PRP-004 built it; PRP-007 renders authoring summaries) — a
  question for those owners, not decided here.
- **A9 — Add a utility-tax measurement to the PRP-008 vertical-slice DoD.** Report the
  benign-completion rate under the declared closure alongside the fail-closed cases.
  Rests on: AID-Guard's 35–44 pp strict-manifest cost; the survey's verifier-tax finding.
  Owner: PRP-008.
- **A10 — Extend the doc 15 §5 threat list and PRP-007's open items.** Add
  stealth-consistent axiom edits (93.3% success at 100% reasoner-consistency stealth),
  selection-integrity attacks on mixed-trust graphs
  ([arXiv:2606.12290](https://arxiv.org/abs/2606.12290)), and approval-verdict staleness.
  For PRP-007: a consistency check is not a security gate — SME semantic-diff review plus
  non-vacuity plus per-fact provenance. Owner: doc 15's security workstream / PRP-008;
  PRP-007 (see [`../ontology-authoring/prp-007-open-items.md`](../ontology-authoring/prp-007-open-items.md), item 4 added this sweep).
- **A11 — Name the fragment's export surfaces and widen its effect model.** Claude Code
  `SKILL.md` `hooks` (a `PreToolUse` command/http hook) with `metadata` carrying the
  fragment IRI; Gemini CLI `toolAnnotations` matching; MCP `tools/resolve` (draft
  SEP-1862) as a future preflight surface; DeepSeek Harness's Claude Code/Codex hook
  bridge as a single deployment path. Effect constraints must cover credentials and
  endpoints, not only file paths (PocketOS). Owner: PRP-007 capability-fragment compiler
  mode / PRP-008 coding-harness adapter. Detail: [`coding-agent-harnesses.md`](coding-agent-harnesses.md).
- **A12 — Wrap an existing patch format and put the base-hash check in the policy
  stage.** The envelope can carry Codex V4A verbatim plus base hash and declared checks;
  the hash check belongs outside the editing tool (OpenCode's tool-local check failed
  open). Owner: the joint PRP-007/PRP-008 envelope-vocabulary D-item.
- **A13 — Restate the differentiator against Palantir and OaK.** "Same guarantees
  without owning state; a proof, not a receipt." Owner: PRP-000.
- **A14 — State absent-vs-not-fetched as a differentiator, and test manifest
  assumptions rather than list them.** Rests on: FORGE's closed-world stance; "Agent
  Behavioral Contracts II" finding independence assumptions "routinely stated and rarely
  tested" ([arXiv:2608.12895](https://arxiv.org/abs/2608.12895)). Owner: PRP-005.

## Open questions (decision-shaped — flagged, not answered here)

- Which scenario (VTE / NEWS2 / sepsis-compliance) is PRP-008's first vertical slice —
  captain call per [`PRPs/BOARD.md`](https://github.com/billaram/horismos/blob/main/PRPs/BOARD.md). Nothing in this sweep bears on the choice; Epic's
  CRD launch is prior-authorization, which [`../scenarios/ruled-out.md`](../scenarios/ruled-out.md) deprioritizes as
  a first slice.
- PRP-008 D1–D5, D8, D9, D10 — evidence above (X7, A1–A7) narrows them; resolution stays
  in PRP-008 via the brainstorming protocol.
- Whether narration gains a conclusion monitor (A8) — doc 06 / PRP-004 / PRP-007 owners.
- Whether PRP-007's gate rationale is restated from accuracy to safety (X2, A10) —
  PRP-007.
- Whether "kernel" stays in the positioning given OaK's usage (X10) — PRP-000.

## Jev deep dive delta (2026-09-21): the verdict re-issued on a third axis

**[inference — the re-issued verdict line, scoped]** **aligned-with-adjustments —
scoped to the non-generative typed-decision-model axis**, extending (not re-litigating)
sweep #2's verdict above and the RL deep dive's verdict of the same date, which is
recorded on the [verdict history](../verdict-history.md) rather than in the table below.

On 2026-09-15 TypeSafe AI opened early access to **Jev**, a non-generative model that
answers **typed questions declared in the request** and returns calibrated
probabilities, with no text output. Full assessment:
[`typed-decision-models.md`](typed-decision-models.md) and the
[Jev deep dive report](../reports/2026-09-21-typesafe-jev-deep-dive.md) §6.

**[inference]** Commitment by commitment: **C1, C2, C3, C7 and C8 are untouched** — the
category holds no state, hydrates nothing, commits nothing, generates no candidate
artifact, and makes no verification claim. **C4 is confirmed from an adversarial
direction**: OrderBench measures 100% schema validity against ~80% semantic success and
concludes that structured output "serves as a necessary interface component rather than
a replacement for proper domain verification and fail-closed system architecture"
([arXiv:2607.18261](https://arxiv.org/abs/2607.18261)) — this page's own thesis, reached
independently by people benchmarking structured output rather than building a kernel.
**C5 is confirmed on the premise and sharpened on the mechanism**: **[measured]** the
vendor documents that answers to separate questions are not jointly constrained (a
proposition asked as a Noul returned 0.22 where the equivalent Choice returned
P(yes) = 0.01; a claim and its negation summed to 1.19 —
[jaggedness](https://docs.typesafe.ai/model-jaggedness/jev-1.13)), and a rule layer
evaluated to a fixed point is exactly that missing constraint, so "symbolic decides"
gains a mechanical justification alongside the safety one. **C6 is lightly confirmed**:
the category reached harness distribution within days (an agent skill for Claude
Code/Codex; Vercel AI Gateway), and its documented function-calling pattern — the model
picks from a closed set, your code dispatches — converges on the capability fragment's
shape while stopping short of declaring preconditions or effects.

**[inference]** Three shape-preserving items are owed, in the same idiom as A1–A14:

- **J1 — Measure the confidence the perception threshold compares against.** Rests on:
  `accepted = result.confidence >= p.confidence` over a number the model self-reports
  ([`crates/horismos-core/src/workflow/agent.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/agent.rs):492;
  [`crates/horismos-bridges/src/openrouter.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-bridges/src/openrouter.rs):322-331,
  **[measured — local repo]**);
  ConfidenceBench's finding that calibration and accuracy diverge across model families
  ([arXiv:2607.20526](https://arxiv.org/abs/2607.20526)); Kotte's measurement that
  heuristic abstention violates stated risk targets on 7.5–12.5% of settings
  ([arXiv:2606.29054](https://arxiv.org/abs/2606.29054)). Owner: **PRP-005**. A build
  item, not a decision.
- **J2 — Replace "typed decisions" with cross-answer consistency and *epistemic* (not
  merely structural) auditability in the positioning.** Rests on the vendor's own
  documented non-invariance and its statement that it generates no explanations. Owner:
  **PRP-000**; captain-held as Jev OD-6 in the report.
- **J3 — Name the inverse confidence threat in doc 15.** Doc 15 §1 names injected text
  that *clears* the threshold; it does not name an adversary who *depresses* confidence
  to force `horis:fallback` on every case — a denial-of-service on the human in the loop
  (Forced Deferral, [arXiv:2606.15308](https://arxiv.org/abs/2606.15308)). Owner: **doc
  15's security workstream / PRP-008**, the same owner A10 named.

**[measured — primary abstract, added by the dive's regulatory sub-sweep]** One piece of
external support for the pivot's core bet is stronger than anything the corpus previously
held, and it arrived from the clinical literature rather than the AI one. Gorenshtein et
al.'s systematic review of neuro-symbolic LLM integration in clinical medicine (Research
Square rs.3.rs-8902022, 2026-02-18; 3,166 records screened to 21 studies) sorts integration
patterns "by **increasing symbolic authority**" — structured output, rule-guided generation,
knowledge retrieval, iterative validation with symbolic veto — and reports median gains of
**9%, 16%, 26% and 40%** along that ordering. **[inference]** That is a dose-response curve
for C1 ("neural proposes, symbolic decides"), measured across a published body of work, with
a typed-decision model sitting at the lowest-authority rung. Its caveat is equally
load-bearing and is recorded with it: **all 21 studies are rated high risk of bias**, only 9
report external validation, and the highest-authority pattern costs "2–88 s latency and up
to 100× cost increases". It does not move the verdict — it is corroboration of a premise the
verdict already assumed, not evidence about the pivot's *scope* — but it is the first
external measurement the corpus has of the principle itself, and it belongs on this page.

**[inference]** Why not `misaligned`: nothing here is evidence against a commitment. The
only challenge is commercial — a cheap typed classifier absorbs the segment whose
requirement was "text in, typed value out" — and that segment is outside what the pivot
claims. Narrowing the addressable market is a positioning fact, not a misalignment. The
one place a reader should push back is C4's second half ("traces are output"), which
this axis does not test; the RL deep dive's finding that the *signal* half of that claim
is still spec-only stands unchanged.

## Verdict

**[inference]** **aligned-with-adjustments.** The pivot's shape — verify a declared package, hydrate a
versioned closure, reason to a fixed point, prove, propose typed effects, commit only
if the source is unchanged — is the shape the database community is now proving
theorems about for LLM agents, the shape HL7 has made normative-ballot-stable for the
host surface, and the shape the agent-safety literature measures as the thing that
actually works when models are unreliable at exactly these steps. The field's
divergences are toward putting models back at the approval, schema-change, and
conflict-adjudication steps; the same literature supplies the numbers for why
Horismos should not follow. The adjustments sharpen types, disclosures, and one
boundary; they do not move the line.


## Verdict history

| Sweep | Date | Verdict | One-line reason |
|---|---|---|---|
| #2 | 2026-09-02 | aligned-with-adjustments | Core commitments independently confirmed post-2026-08-24; divergences run neural-ward at the line Horismos draws; adjustments are shape-preserving (A1–A14). |
| — (Jev deep dive) | 2026-09-21 | aligned-with-adjustments (scoped to the non-generative typed-decision-model axis) | A commercial model concedes C5's premise — typed values in a pre-declared space — while documenting that its own answers are not jointly constrained; C1–C3, C7, C8 untouched, C4 confirmed adversarially, C6 lightly confirmed. Three items owed (J1–J3). |

## Source

Per-claim citations inline; every source is also logged in [`../reading-log.md`](../reading-log.md) (sweep
#2 rows, 2026-09-02, sections Q1, Q3, Q4, Q5, Q6 and the new Q7). Repo facts cite
`local repo` paths.

## Confidence

mixed — the cited findings, spec text, release facts and harness documentation are
[measured] (items marked [unverified] were relayed from search snippets where primary
fetch failed); every "bearing", adjustment, and the verdict are [inference].

## Last-verified

2026-09-21 (Jev deep dive delta appended and verdict re-issued, scoped; sweep #2 assessment 2026-09-02 unchanged)
