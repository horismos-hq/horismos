# Horismos × TypeSafe's Jev — the non-generative typed-decision-model axis

> **Provenance.** Produced by captain-directed deep-research task
> `horismos-research-jev-s1` on **2026-09-21**, six days after TypeSafe AI opened early
> access to Jev. It takes sweep #2's pivot verdict and the
> [RL-axis deep dive](2026-09-02-rl-deep-dive.md) as given and extends them along a
> third axis: commercial, non-generative models that answer typed questions declared in
> advance. Grounded read-only against
> [`adfc27a`](https://github.com/billaram/horismos/blob/adfc27a) (`main`); repo-file
> references and `file:line` numbers are true at that commit only. **Where its findings
> went:** the new topic page
> [`neurosymbolic-landscape/typed-decision-models.md`](../neurosymbolic-landscape/typed-decision-models.md),
> dated "Jev deep dive delta (2026-09-21)" sections on the
> [Q3 overview](../neurosymbolic-landscape/00-overview.md),
> [lineage assessment](../neurosymbolic-landscape/lineage-assessment.md),
> [coding-agent harnesses](../neurosymbolic-landscape/coding-agent-harnesses.md),
> [validation ladder](../validation/00-ladder.md) and
> [regulatory shape](../validation/regulatory.md) pages, ~145 reading-log rows under a new
> Q9 section, and a re-issued standing verdict on the
> [pivot-alignment page](../neurosymbolic-landscape/pivot-alignment.md) mirrored on the
> [verdict history](../verdict-history.md). Its eight open decisions (§9) are
> captain-held and unacted-on as of writing.

**Task**: `horismos-research-jev-s1` (captain-directed deep research round, 2026-09-21)
**Repo state grounded against**: `horismos` `main` = `adfc27a`, read-only — `crates/`,
`docs/framework/`, `PRPs/` and `examples/`, plus the existing corpus.
**Deliverable**: this report plus the wiki deltas listed above. No `crates/` change, no
PRP text edited, nothing implemented. Evidence convention per
[`research/README.md`](../index.md): every claim is **[measured]** (with a URL or
`local repo` path) or **[inference]**; **[unverified]** marks a fact seen only in a
search snippet or a page that could not be opened. A tag at the start of a heading or
lead sentence applies to the bullets beneath it until the next tag.

**A naming note, once.** The model is **Jev**, from **TypeSafe** (also styled TypeSafe
AI). The captain's intake referred to it as "Dev"; that is a mishearing, and nothing in
this report concerns a product called Dev.

---

## TL;DR

**[inference]** Nine findings, then the evidence.

1. **Jev is real, narrow, and not a competitor.** It answers typed questions declared in
   the request — Choice (≤255 options), Score (a 2–10 level ordinal rubric) and Noul
   (0–1) — in one parallel pass, and returns a probability per answer. It does not write
   text, call tools, chain, or decide anything. It is a **proposer**, which is the slot
   Horismos calls `Bridge` (§3, §6.1).
2. **"Hallucination is impossible by construction" is true about the *type* and says
   nothing about the *answer*.** A Choice always returns one of the declared options; it
   can return the wrong one. Separating type correctness from decision correctness from
   auditability is the whole analysis (§3.6).
3. **The category is a rebrand of good engineering, not a new science** — Chow's reject
   option (1970), calibrated discriminative classification, selective prediction,
   learning-to-defer. What is genuinely new is the *product shape*: declare the whole
   question set up front and answer it in one pass (§4).
4. **Four of the fourteen intake facts were wrong** and are corrected in a table at §3.1,
   including the access model, the Score primitive's nature, and the determinism posture
   — TypeSafe explicitly declines determinism in favour of "consistency" (§3.1, §6.2.4).
5. **The strongest positioning finding is the vendor's own.** TypeSafe publishes that its
   answers are **not jointly constrained** — a Noul at 0.22 and a Choice at 0.01 on the
   same proposition; a claim and its negation summing to 1.19. A rule layer over a fixed
   point is exactly that missing constraint. "Typed decisions" stopped being Horismos's
   differentiator on 2026-09-15; **cross-answer consistency** and **epistemic
   auditability** are what replace it (§6.1, §7 R3).
6. **The sharpest finding about Horismos is a defect in something already shipped.**
   `accepted = result.confidence >= horis:confidence` compares against a number the model
   self-reports and nobody has ever measured. That is a mis-set decision boundary, and the
   whole calibration literature says so (§2.2, §6.4, §7 R1–R2).
7. **The auditability wedge holds, but it is narrower and aimed at a different buyer than
   assumed.** Mandated *system-level* transparency is being dismantled — ASTP reports "no
   publicly available evidence indicating that a single doctor, nurse, or administrator has
   accessed, recorded, or modified a single source attribute". *Per-decision* reason-giving
   is holding or strengthening, and it holds specifically where it sits in older statutes
   with private enforcement. The defensible claim is **"non-reviewability is a cost
   multiplier"**, addressed to general counsel, not "clinicians want explanations" (§6.3).
8. **And that section had to be corrected mid-dive.** Ghassemi et al. attacks *inherently*
   explainable models too, not only post-hoc ones — the full body text says so. A proof
   defeats the fidelity argument completely and the human-factors and validation arguments
   not at all, and the human-factors failure has now been measured in clinicians for
   intrinsic explanations (§6.3.2(v), §10).
9. **Nothing here says build a `typesafe:` bridge.** Four independent blockers, any one
   sufficient (§7, "What NOT to do"). Nine ranked moves (§7) and eight open decisions (§9)
   are what this dive actually produced; the standing pivot verdict is re-issued unchanged
   in direction and sharpened in wording (§6.6).

**If you read one section**: §6.3 for the positioning argument, §9 for what firstmate
should file.

---

## 1. What I did

**Grounding first (local, read-only).** Before any web search, the repository at
`adfc27a`: the Bridge contract (`crates/horismos-core/src/bridge/mod.rs`), the agent
loop's confidence/threshold/fallback arithmetic (`workflow/agent.rs`), the five-check
gate (`world/gate.rs`), the trace schema (`trace/schema.rs`), the PRP-008 host SPI
(`crates/horismos-host/src/{spi,trace,assumptions}.rs`), the only non-mock Bridge
(`crates/horismos-bridges/src/openrouter.rs`) and the four stub modules beside it,
`docs/framework/{02,06,07,08,09,10,15}.md`, `PRPs/{BOARD.md,PRP-000,PRP-005,PRP-008}`,
and the canonical triage overlay (`examples/triage/triage.ttl`). Then the corpus this
report extends: [`research/README.md`](../index.md), the
[standing pivot-alignment page](../neurosymbolic-landscape/pivot-alignment.md),
[the RL deep dive](2026-09-02-rl-deep-dive.md), the
[verdict history](../verdict-history.md), and the
[validation](../validation/00-ladder.md) and [regulatory](../validation/regulatory.md)
pages. §2 is the result, and it is where every "Horismos does X" claim in this report
is sourced from.

**Live web sweep (2026-09-21), five parallel sub-agents.** One per question, with
explicit instructions to treat every fact in the task brief as `[unverified]` until
confirmed against a primary source or two independent secondary ones:

| Sub-agent | Mandate | Question |
|---|---|---|
| A | TypeSafe primary sources: identity, interface/schema, training story, benchmark methodology, pricing, stated limitations, the Doom demo | Q-A |
| B | Calibration, proper scoring rules, uncertainty quantification, conformal prediction, calibration of LLMs vs small discriminative models, calibration-objective RL | Q-B (half) |
| C | Routing/cascades, constrained decoding and structured output, non-autoregressive and parallel decoding, the "System 1 / System 2" framing's lineage | Q-B (half) |
| D | Adoption evidence, independent benchmarks, the skeptical case in critics' own words, marketing momentum vs demonstrated capability | Q-C |
| E | Post-2026-05 regulatory and clinical-AI literature on explanation-vs-calibration; HTI-5 status; EU AI Act; the steelman of "auditability is overrated" | Q-D3 |

**Naming, once, for the record.** The model is **Jev**, from TypeSafe (also styled
TypeSafe AI). The captain's intake referred to it as "Dev"; that is a mishearing, and
nothing in this report concerns a product called Dev.

**Scope discipline.** This is a research and documentation change. No `crates/` file is
touched, no PRP's decided text is edited, nothing is implemented. Where a finding would
imply a change to decided PRP text, it is filed as an Open Decision in §9, not applied.
Sweep #2's verdict and the RL deep dive's four open decisions are taken as given and
cited, not re-litigated; §6.6 re-issues the standing verdict per the wiki's
[re-verdict convention](../sweeps.md#what-a-sweep-produces).

---

## 2. Baseline: the Horismos surface Jev would touch

Everything in this section is **[measured — local repo]** at `adfc27a` unless tagged
otherwise. It is deliberately narrow: not "what is Horismos", but "which parts of
Horismos would change if a typed-decision model existed", read out of the code and the
shipped-voice docs rather than remembered. Sections 4 and 5 stand on it.

### 2.1 The neural surface is three methods wide

Horismos's *entire* connection to any model is one Rust trait with four methods, of
which three do work:

```rust
// crates/horismos-core/src/bridge/mod.rs:23-41
#[async_trait]
pub trait Bridge: Send + Sync {
    async fn perceive(&self, req: PerceiveRequest) -> Result<PerceiveResult, BridgeError>;
    async fn narrate(&self, req: NarrateRequest) -> Result<NarrateResult, BridgeError>;
    async fn rank(&self, req: RankRequest)   -> Result<RankResult,   BridgeError>;
    fn id(&self) -> &str;
}
```

> **Python parallel.** A Rust `trait` is the same idea as a `typing.Protocol` (or an
> `abc.ABC`): a named set of method signatures that a type can satisfy. Horismos's own
> docs make the correspondence explicit — [`docs/framework/08-bridge.md`](https://github.com/billaram/horismos/blob/adfc27a/docs/framework/08-bridge.md) shows the
> Python side as `class Bridge(Protocol)` with the same three capabilities. The
> difference that matters here is *when* it is checked: `Protocol` conformance is
> checked by mypy (or not at all, at runtime); `trait` conformance is checked by the
> compiler, so a `JevBridge` that failed to implement `narrate` would not build.
>
> `Result<PerceiveResult, BridgeError>` is Rust's way of saying "returns a
> `PerceiveResult`, or fails with a `BridgeError`". In Python you would write
> `def perceive(...) -> PerceiveResult` and `raise BridgeError`. The practical
> difference is that Rust makes the caller *name* the failure branch — which is why
> [`workflow/agent.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/agent.rs) has a visible `Err(_) => return Ok(self.fallback_outcome(fallback))`
> arm rather than a `try/except` somewhere up the stack.

Three facts about this trait decide most of what follows.

**(a) It is a client, not an orchestrator.** The module doc is explicit: "`Bridge` is a
client, not another orchestrator (doc 08): three methods, nothing else. Thresholds,
retries, fallbacks, and timeout enforcement are the runtime's job … never a `Bridge`
implementation's" ([`bridge/mod.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/bridge/mod.rs):8-13). A Jev adapter therefore cannot bring its
own control flow with it; it gets to answer three questions and nothing else.

**(b) `rank` is already a Choice question.** `RankRequest { context: String, candidates:
Vec<serde_json::Value> }` → `RankResult { chosen: usize, confidence: f64 }`
([`bridge/mod.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/bridge/mod.rs):102-122). The doc comment states the constraint that makes it
symbolic: "candidates come from a graph query (doc 07) — every candidate already
satisfies its preconditions. The model ranks; it cannot add options." That is a
categorical decision over a closed, runtime-computed option set, returning a scalar
confidence. Section 4 argues this is the same object as a Jev **Choice** question, with
the option set supplied by the kernel instead of the caller.

**(c) There is no Score question and no distribution.** `RankResult::confidence` is one
`f64` about the *call*, not a probability per candidate. `PerceiveResult` carries
`confidence: f64` plus an optional `fact_confidence: BTreeMap<usize, f64>` keyed by
index into `instances` ([`bridge/mod.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/bridge/mod.rs):63-80). Nothing in the trait can express
"the probability that the answer is A is 0.62 and B is 0.31".

**(d) — and `rank` is never called.** This is the fact that most changes the shape of
§6.2, and it is easy to miss. `ProposalStep` parses, has an ontology vocabulary
(`horis:candidates`/`horis:rankBy`/`horis:confidence`/`horis:fallback`, doc 07), and has
a real trace emission builder (`trace::emit::proposal_step`, `Step::Proposal` in
[`trace/schema.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/trace/schema.rs):230-257) — but the runtime refuses it:

```rust
// crates/horismos-core/src/workflow/agent.rs:454
StepBody::Proposal(_) => Err(deferred::proposal_deferred(world, step.iri)),
```

and [`workflow/deferred.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/deferred.rs):5-7 says so in as many words: "`ProposalStep`'s
candidate ranking is deferred the same way — trace/2 gives it a real emission builder …
for when ranking does land, but **nothing in this PRP calls `Bridge::rank`**."
`ChoiceStep`, `WaitStep` and `SubflowStep` are deferred identically; reaching one is a
domain-term `WorkflowError::StepDeferred`, "never a panic, never a silent no-op".

**[inference]** So the slot whose shape matches a typed-decision model most exactly is
*reserved but unbuilt*. That is the most useful possible state for this report to find
it in: the `rank` contract can still be designed around what a calibrated proposer can
actually supply, instead of being retrofitted. §6.4 and OD-2 turn on this.

### 2.2 The confidence Horismos gates on today is self-reported

This is the single most load-bearing measured fact in the report, so it is stated with
its evidence in full.

[`docs/framework/08-bridge.md`](https://github.com/billaram/horismos/blob/adfc27a/docs/framework/08-bridge.md) § "Provider mechanics (perception)":

> v1 ships one calibration strategy for every provider: the forced tool call's schema
> carries a `confidence` number field the model must self-report, and a result that
> omits it is treated as `0.0` — failing safe into the fallback edge rather than
> false-accepting. **Logprob-derived confidence, where a provider exposes it, is a
> future per-backend enhancement layered on the same field**; the mock backend's
> scripted confidence makes every test independent of either.

The only non-mock Bridge that exists implements exactly that:

```rust
// crates/horismos-bridges/src/openrouter.rs:322-331
// D9: a self-reported confidence the wire omitted normalizes to
// 0.0 — fails safe into the runtime's fallback edge.
let confidence = args.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
```

`anthropic.rs`, `openai.rs` and `onnx.rs` are one-line module stubs behind
`bridges-*` feature flags; `mock.rs` scripts confidence for tests
(`wc -l crates/horismos-bridges/src/*.rs` → 1, 1, 1, 296, 749).

And the runtime gates on it with a bare comparison:

```rust
// crates/horismos-core/src/workflow/agent.rs:492
let accepted = result.confidence >= p.confidence;
```

where `p.confidence` is the ontology's declared `horis:confidence` threshold for that
step. On `false`, the step follows `horis:fallback` and the trace records
`confidence`/`threshold`/`accepted` ([`trace/schema.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/trace/schema.rs):151-167).

**[inference]** So Horismos's acceptance decision is a threshold over a number the
model wrote about itself. Doc 06 is careful that this number is "metadata, not truth:
it gates acceptance and is recorded in trace and proof provenance; it never weights
inference. There is no fuzzy logic in Horismos." That is exactly right as a statement
about *inference*, and it is precisely why the number's quality has been easy to leave
alone. But it does not follow for *acceptance*: the threshold is a real decision
boundary, and an uncalibrated input to it produces a mis-set boundary. This is the hole
§4.4 argues Jev's whole product category is pointed at.

### 2.3 An Adapter is explicitly not a Bridge

[`docs/framework/08-bridge.md`](https://github.com/billaram/horismos/blob/adfc27a/docs/framework/08-bridge.md) § "Not a Bridge":

> an Adapter that hydrates an `ExternalSnapshot` from a FHIR server or repository is
> *not* a Bridge, and gains no `Bridge` implementation — a Bridge supplies a neural
> *proposal* that must clear the gate; an Adapter supplies authoritative structured
> state that's admitted the same way any other `api`-provenance assertion is.

The PRP-008 host SPI is on the Adapter side of that line:

```rust
// crates/horismos-host/src/spi.rs:501-508
pub trait HostAdapter {
    type HydrationRequest;
    fn hydrate(&self, request: &Self::HydrationRequest) -> Result<ExternalSnapshot, HostError>;
    fn commit(&mut self, effects: &ProposedEffectSet) -> Result<CommitReceipt, HostError>;
}
```

> **Python parallel.** `type HydrationRequest;` is an *associated type* — read it as a
> `Protocol` with a `TypeVar` bound per implementation, i.e. roughly
> `class HostAdapter(Protocol[ReqT])` where `hydrate(self, request: ReqT)`. Each
> concrete adapter (FHIR, coding-harness) fixes `ReqT` to its own request type, and the
> compiler stops you passing a FHIR request to the repo adapter.

**[inference]** This matters for Q-D2: the honest place for Jev is the `Bridge` side,
not the `HostAdapter` side. Jev does not supply authoritative structured state; it
supplies a model's opinion, which is exactly what the gate exists to filter. Putting a
model behind `HostAdapter::hydrate` would give model output `api` provenance and route
it around the confidence threshold — the one thing doc 06's structured-input bypass
section forbids ("An LLM never infers what a structured standard resource means; that
would smuggle judgment into what is supposed to be a mechanical conversion").

### 2.4 Model routing is already prefix-based and pluggable

Doc 08's registry table routes ontology-declared model strings by prefix: `claude:`,
`openai:`, `hf:` ("local HuggingFace pipeline — **classification-style perception
without a frontier model**"), `onnx:` ("edge/embedded"), `python:`, `mock:`. Deployment
maps names to configured Bridge instances; "swapping providers is a config change; the
ontology is provider-agnostic", and every trace step's `model` field records
`Bridge::id()` — the model *actually invoked* — so a deployment override stays visible
after the fact.

**[inference]** The mechanical cost of adding a `typesafe:` prefix is therefore near
zero. The interesting question is never "can it plug in"; it is what the gate is
entitled to demand once it does (§4.4) and what the trace has to carry (§4.5).

### 2.5 What the gate checks, and what it does not

[`crates/horismos-core/src/world/gate.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/world/gate.rs):12-28 fixes the five checks and their
order — class exists; property domains/ranges; constraints (`horis:min`/`max`/
`pattern`/`oneOf`); cardinality (`horis:cardinality`, `horis:immutable`); provenance
policy (`horis:assignedBy`). A refusal returns a `Witness` carrying both domain text and
a machine-readable `ConstraintKind`.

> **Python parallel.** `ConstraintKind` is a Rust `enum` with data in its variants —
> `Max { bound: String, unit: Option<String> }`, `CardinalityMax { … }`,
> `ProvenanceRefused { … }` and sixteen others. The closest Python is a tagged union: a
> set of frozen dataclasses under a `Union` alias, matched with `match`/`case`. The
> property Rust adds is exhaustiveness: if a new variant is added, every `match` that
> does not handle it fails to compile. That is why the retry loop can branch on the
> *kind* of refusal without a silent `else: pass`.

**[inference]** Note what this list does *not* contain: any check on how confident the
proposer was, beyond the earlier threshold, and any check that the proposed fact is
*true*. The gate proves **well-formedness against the declared ontology** — a strictly
type-level guarantee — plus provenance policy. It is the same class of guarantee as
Jev's "answer is always a member of the declared type" (§3.6), applied to a richer type
system (ranges, units, cardinality, enumerations, mutability, provenance) than a JSON
schema expresses.

### 2.6 Where the decision correctness actually comes from

Horismos's decision is not the model's. The chain is: gate-admitted facts →
forward-chaining to a fixed point over declared rules ([`reason/forward.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/reason/forward.rs)) →
every derived fact carries a proof tree ([`reason/proof.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/reason/proof.rs)) → truth maintenance
re-chains on every external transaction, so a world built through the public API is
always at its fixed point (CLAUDE.md; [`docs/framework/03-world.md`](https://github.com/billaram/horismos/blob/adfc27a/docs/framework/03-world.md) § Truth
maintenance).

**[inference]** This is the structural difference from every "typed decision model"
framing, Jev's included: Horismos's typed answer is *computed from* declared rules over
gate-admitted facts, so the reason for the answer exists as a data structure before
anyone asks for it. A model that emits a typed answer directly has no such structure to
emit, whatever its calibration.

### 2.7 The honest limit Horismos already writes down

[`docs/framework/15-security.md`](https://github.com/billaram/horismos/blob/adfc27a/docs/framework/15-security.md) § 1, on prompt injection at the perception
boundary, after listing what an attacker cannot do:

> The residual risk is narrower and different in kind: injected text can still get a
> **wrong-but-schema-valid** fact accepted (claim a heart rate the patient doesn't
> have) if it's confident-sounding enough to clear `horis:confidence`. That's a
> data-integrity risk bounded by the ontology author's threshold and fallback choice,
> not an RCE/tool-firing risk.

**[inference]** Horismos has already written down, in shipped voice, the exact limit
that §3.6 argues Jev's "hallucination is impossible by construction" marketing elides.
Schema forcing buys type correctness; it does not buy decision correctness. Horismos
says so about its own perception boundary. That candour is a positioning asset and
should be used as one (§5, move R7).

---

## 3. Q-A — what Jev actually is

The task brief supplied fourteen intake facts and instructed that each be treated as
`[unverified]` until confirmed. Most survived. Four did not, and the four that failed
are the ones that matter.

### 3.1 Corrections to the intake

| # | Intake said | What the primary sources say | Source |
|---|---|---|---|
| 1 | Founder: Diogo Almeida (singular) | **Three co-founders**: Almeida (CEO), **Erik Gafni (CTO)**, **Sasha Sheng (COO)** | [typesafe.ai/team](https://typesafe.ai/team) |
| 2 | "former OpenAI researcher / ChatGPT contributor" | TypeSafe's own claim is stronger: *"Diogo co-invented RLHF and InstructGPT"*; previously Google Brain | [team page](https://typesafe.ai/team) |
| 3 | Boolean type "Noul" — confirm, may be garbled | **"Noul" is correct and official** — a `type` enum value, a response field, a Python class, a TypeScript interface, and its own docs page | [docs/primitives/noul](https://docs.typesafe.ai/primitives/noul) |
| 4 | Benchmark reports "agreement" (implying human labels) | **No human labels anywhere.** Reference answers are *"the average of the responses of **GPT-6 Astra and Claude Fable 5.1, both at high thinking**"*; graded models ran at provider **default** reasoning | [evals.typesafe.ai](https://evals.typesafe.ai/) |
| 5 | 67.8% "parity" with the baselines | Numbers exact, framing wrong: Jev is **4th of nine**, and ranges **61.7%–76.0%** per workflow with a **~17-point deficit** on Invoice Processing | [evals.typesafe.ai](https://evals.typesafe.ai/) |
| 6 | "0% structured-output error and 0% tool-call error" as measured results | TypeSafe states under the chart: ***"Our number is not empirical."*** The LLM comparators come from **OpenRouter production telemetry** — a different population from their own eval. And **Jev has no tool-calling surface at all** | [launch blog](https://typesafe.ai/blog/introducing-system-one-models-and-jev) |
| 7 | "no interpretability — called out as a problem for audits in regulated domains" | **Not TypeSafe's framing.** A grep of the full ~896 KB docs corpus returns **zero hits** for interpretability/audit-risk language in any limitations context; TypeSafe actively *markets* legal and compliance use cases. The critique is **DataCamp's** | [DataCamp](https://www.datacamp.com/blog/system-one-models-jev); [llms-full.txt](https://docs.typesafe.ai/llms-full.txt) |
| 8 | "non-LLM", stated flatly | TypeSafe says *"Jev is neither small nor an LLM"* — but their own ML primer diagrams RLCD branching off **"Pretrained language models"**, and Almeida told DCVC *"LLMs have an incredible amount of intelligence locked inside."* **Internally inconsistent; architecture undisclosed** | [blog FAQ](https://typesafe.ai/blog/introducing-system-one-models-and-jev); [ML primer](https://docs.typesafe.ai/introduction/machine-learning-primer); [DCVC](https://www.dcvc.com/news-insights/typesafe-emerges-from-stealth-with-a-new-way-of-doing-ai/) |
| 9 | latency 70–500 ms | Correct as quoted, but **three different framings across three primary sources**: the blog says 70–500 ms, the press release says "less than 100 milliseconds", the eval site measures 0.3–0.5 s per (multi-call) case | blog; [press release](https://www.financialcontent.com/article/bizwire-2026-9-15-typesafe-ai-emerges-from-stealth-with-40m-in-funding-with-new-model-for-composable-ai); evals |
| 10 | "Score (numeric/rubric)" | Precisely: **an ordered rubric of 2–10 level descriptions**, returning a probability-weighted value that can land between levels. **Not a numeric extractor** (§6.2.3). Choice caps at **255 options** | [docs/api](https://docs.typesafe.ai/api) |
| 11 | "RLCD", unqualified | **Three-way name collision** — §4.8 | — |
| 12 | Elixir SDK interest an open question | **Official SDKs are Python and JS/TS only.** Elixir clients exist, are community-built, and are unmentioned by TypeSafe | [docs/sdk](https://docs.typesafe.ai/sdk) |
| 13 | Doom demo URL 404s | **Live URL recovered** — §3.8 | verified |
| 14 | *(not in intake)* | **Jev is explicitly NOT deterministic**, and TypeSafe argues determinism is the wrong goal | [typesafe.ai](https://typesafe.ai/) FAQ |

Confirmed unchanged: TypeSafe AI, founded 2024, San Francisco; **$40M seed led by DCVC**,
announced 2026-09-15; **"After two years in stealth"** in Almeida's own words; early
access from **2026-09-15**, waitlisted; input **$0.042/MTok** ($42/Btok) with output
tokens counted but unbilled; the four-workflow benchmark numbers exactly as quoted.

**[measured] One unresolved discrepancy, recorded rather than resolved.** Sub-agent A,
reading typesafe.ai's own FAQ on 2026-09-21, found the access answer still *"Join the
waitlist!"*. Sub-agent D recorded general availability on **2026-09-20** ("Jev is now
available to everyone. No waitlist", with $5 of free credit). Both readings are
first-hand; the most likely explanation is a stale FAQ, but this report does not assert
which is current. **[inference]** Either way the access model went from waitlist to open
in five days, which suggests the scarcity was a launch device rather than a capacity
constraint.

### 3.2 The interface, which is smaller than the marketing implies

**[measured]** One endpoint, `POST https://api.typesafe.ai/v1/systemone`. The entire API
is a `state` string plus a `questions` map, sent **in the same request body** — there is
no schema registry, no pre-registration step, no server-side stored template, and no
schema version. The only versioned artifact is the model (`jev-1.13.0`, with
`jev-latest`/`jev-preview` aliases), echoed back in the response so you can log which
one answered ([docs/api](https://docs.typesafe.ai/api),
[docs/models](https://docs.typesafe.ai/models)).

```json
{
  "state": "I've been trying to connect my Stripe account for 3 days and the integration keeps failing.",
  "model": "jev-latest",
  "questions": {
    "department":  {"type": "choice", "instructions": "Which team should handle this",
                    "criteria": {"billing": "Payment or subscription issues",
                                 "technical": "Bugs or integration problems",
                                 "sales": "Pricing or account questions"}},
    "frustration": {"type": "score",  "instructions": "How frustrated the customer appears",
                    "criteria": ["Calm, just stating facts", "Frustrated but civil", "Very angry"]},
    "is_urgent":   {"type": "noul",   "instructions": "The message conveys urgency"}
  }
}
```

and the response:

```json
{"model": "jev-1.13.0",
 "answers": {
   "department":  {"type":"choice","choice":"technical","confidence":0.78,
                   "probabilities":{"technical":0.85,"sales":0.0,"billing":0.15}},
   "frustration": {"type":"score","score":1.0,"confidence":1.0,"legend":{…},"probabilities":{…}},
   "is_urgent":   {"type":"noul","noul":1.0}},
 "usage": {"input_tokens": 392, "output_tokens": 65}}
```

**[measured]** Shape facts that matter downstream: **Choice** caps at 255 options and
returns a probability map summing to 1 plus a `confidence`; **Score** takes an ordered
array of **2 to 10** level descriptions and returns a probability-weighted value that can
land *between* levels; **Noul** returns a single 0–1 number and **no `confidence` field**.
Question keys are client-side only — *"The key is not sent to the underlying model and is
not used in inference."* Questions are *"evaluated independently, so adding more questions
does not create context-rot."* Context is **64k tokens per request, 32k for `state` plus
the single longest question**. Input is **text only** — no image, audio or video. Rate
limits are **250,000 tokens/sec and 1,200 requests/min**, explicitly subject to change
without notice. There is **no fine-tuning or LoRA** — "the same weights serve every
account". English is "the primary training language and where accuracy is currently best."

**[measured] There is no tool-calling surface.** Jev cannot emit a tool call. What
TypeSafe ships instead is a *pattern*: reflect over your own typed function signatures,
turn every closed-set argument into a Choice, and dispatch in your own code — *"whatever
reaches the function is a value the function accepts. You leave the functions alone"*
([function-calling cookbook](https://docs.typesafe.ai/cookbooks/function_calling)).

**[inference]** That pattern is, structurally, the same move Horismos makes with
`horis:takes` and a graph-derived candidate set: the model never constructs the call, it
only picks a member of a set the surrounding code already fixed. Two independent designs
arriving at "the model picks from a closed set, the code dispatches" is a meaningful
convergence, and it is worth recording in the corpus as such.

### 3.3 The training story, and what is actually disclosed

**[measured]** **RLCD = "Reinforcement Learning for Calibrated Decisions"**, positioned
as a third post-training path after RLHF and RLVR. The blog's comparison table, on the
objective: existing LLMs optimize "human preference … or outputs that can be
programmatically verified"; System One optimizes *"Calibrated decisions: answers with
epistemically honest probabilities on System One tasks."* The operational definition:
*"Higher probability should correspond to a greater chance that the answer is correct …
Outcomes assigned a probability of `0.2` should occur about 20% of the time … **These
rates describe groups of predictions, not a guarantee about any single answer.**"* The
motivation is framed as *"the bitterest lesson: optimizing for the right task matters
more than data, compute, or algorithms."*

**[measured] There is no paper, no arXiv preprint, no technical report, and no scientific
model card.** The only artifact called a "ModelCard" is an SDK type with three fields
(`name`, `description`, `release_date`). No parameter count, layer type,
encoder-vs-decoder, tokenizer or base-model provenance is disclosed anywhere. There are
no weights and no self-host option. TypeSafe declines public benchmarks by policy: *"We
deliberately chose not to publish performance against public benchmarks"*, with stated
best practices including *"Put no weight on public benchmarks"* and *"De-emphasizing
benchmarks even when you're ahead."*

**[measured] One non-marketing architectural signal.** The `typesafe-ai` GitHub
organization holds two forks made during the stealth period: **`ML-GSAI/LLaDA`** — the
masked *diffusion* language model (§4.7) — forked **2025-07-13**, and
**`vllm-project/vllm`**, forked **2025-05-17**. **[inference]** LLaDA decodes
non-autoregressively and in parallel, which is exactly the shape of "a parallel sampler
that generates all outputs in a single query". A fork proves nothing about what shipped,
but it is the only architectural evidence outside marketing copy, and it points at
diffusion/masked-prediction rather than a shrunken autoregressive transformer. Treat as
a hypothesis.

**[inference]** The public statements are in tension with each other. "Jev is neither
small nor an LLM" sits awkwardly against a docs diagram that branches RLCD off
*pretrained language models* and against the CEO's own "LLMs have an incredible amount of
intelligence locked inside". The most defensible reading is a pretrained-LM-derived
backbone with a non-autoregressive parallel head over enumerated answer spaces — which
is also what the open reproductions built (§5.1).

### 3.4 The benchmark, methodologically

**[measured]** Four workflows, ~711 cases total: **Security Incidents** (240), **Customer
Service** (204), **Invoice Processing** (150), **Agent Trace Observability** (117). The
graded unit is the model's emitted *set of workflow actions* against a consensus action
set, averaged over the four workflows with equal weight.

**[measured] The reference answers are two frontier models, not humans**: *"the reference
labels are generated via an average of the responses of GPT-6 Astra and Claude Fable 5.1,
both at high thinking … All other models are evaluated using the provider's default
reasoning settings."* TypeSafe flags the bias itself — *"which biases answers towards
OpenAI and Anthropic's models. We likely underestimate the relative performance of our
model and DeepSeek's models"* — and separately concedes that the workflows "were made by
individuals on our model capabilities team, so some bias could exist."

**[inference]** Three consequences. The score is **inter-model agreement, not
correctness**: a perfect 100% would mean "indistinguishable from Astra + Fable", not
"right". The two reference models ran at *high thinking* while every graded model ran at
*provider default*, so the answer key is also structurally advantaged. And the exact
set-comparison formula — exact match? per-action F1? Jaccard? — **is not published**;
sub-agent A searched the site and the raw data files and could not find it.

**[measured] But the apples-to-apples question comes out better than expected, and this
is the strongest part of the methodology.** *"Every model gets the same workflow"*, and
the LLM baselines ran through TypeSafe's **open-source** wrapper
([`typesafe-ai/system-one-adapter-python`](https://github.com/typesafe-ai/system-one-adapter-python),
MIT). Reading its README directly: baselines used **provider-native structured output**
(OpenAI via the Responses API with strict JSON Schema; Anthropic native), an
`llm_answer_mode="probabilities"` setting so they returned **per-label distributions**
matching Jev's contract rather than a bare label, and **corrective retries** on schema
validation failure, with per-call telemetry recording retry counts and latency. The
baselines were not handicapped. (Few-shot content and the exact prompts the adapter
builds were not verified — **[unverified]**.)

**[measured] Reproducibility: no.** The ~711 cases, the harness code and the reference
answers are not published; the site shows five illustrative cases per workflow. No
independent reproduction of TypeSafe's benchmark existed as of 2026-09-21.

**[measured] The 0% charts are a separate exhibit**, and TypeSafe's own disclosure under
them reads: *"The numbers for LLMs are from **OpenRouter** i.e., there almost certainly is
bias here: more complex queries might be routed to better models. **Our number is not
empirical. Schema matching is guaranteed, thus we can confidently add 0% into the
plots.**"*

**[inference]** That decomposes into four distinct problems, and they are worth
separating because only the first is about honesty: (1) Jev's 0% is a deduction plotted
beside measurements, with no visual distinction between an axiom and an observation —
though TypeSafe says so in text, to their credit; (2) the LLM numbers come from arbitrary
third-party production traffic with arbitrary schemas and often no structured-output
setting, *not* from the same eval where those same models had strict schemas and retries
and would have scored near zero too; (3) "tool call error rate" is category-incoherent
for a model with no tool-calling surface — 0% is an absence, not a result; (4) Haiku
4.5's 45.5% structured-output error is so far off every other model that it almost
certainly reflects OpenRouter traffic composition rather than model capability.

### 3.5 Pricing, availability and trust posture

**[measured]** There is no standalone pricing page (`typesafe.ai/pricing` 404s); pricing
lives in the docs: **$42/Btok, $0.042/Mtok, charged per input token; "Output tokens are
free."** Output is still *counted* — `usage.output_tokens` appears in every response —
just not billed. On sustainability, the homepage FAQ says *"We can serve Jev profitably
at our current prices"*, while the blog hedges: *"We can't prove it isn't subsidized;
we'll need the long-term to prove the sustainability of our pricing."*

**[measured]** The trust posture, read from the legal pages directly: **no SOC 2
mention, no HIPAA/BAA mention, no SLA** (the MCA, effective 2026-09-19: "TYPESAFE DOES
NOT WARRANT THAT CUSTOMER'S USE OF THE SERVICES WILL BE UNINTERRUPTED OR ERROR-FREE"),
**US-hosted only** with no residency options, **no self-host** (the MCA scopes the
service to a "TypeSafe-hosted" interface and API), and **zero data retention available
only to enterprise customers on request**, with default retention having no stated
period. The one strong item: training on customer input is contractually excluded —
*"We will not train or fine tune any artificial intelligence or machine learning models
on your prompts or other Input"* — stated twice.

### 3.6 "Hallucination is impossible by construction": three guarantees, separated

There is no single TypeSafe sentence matching the intake's phrasing. The nearest
equivalents, in descending order of strength **[measured]**: *"it's optimized for
structured outputs and **can't hallucinate**"* (blog); *"**No type errors**: This would be
an easy thing to falsify with just a single counter-example, but **it is mathematically
impossible**"* (blog); *"**Zero Hallucinations**"* (homepage chart label); *"System One is
**type-safe by construction**"* (docs).

**[measured] And TypeSafe walks it back, clearly, in its own FAQ**: *"**Can Jev still get
things wrong?** Yes. **Jev guarantees the shape of its answers, not that every decision
is correct.** If you provide a list of categories, it can't invent a category outside
that list, **but it can choose the wrong one.** Uncertainty is a feature!"*

**[inference]** The rigorous claim is in the FAQ and the footnotes; the overclaim is on
the homepage. The gap between the two sentences is the entire critique, and it is the
same critique HN converged on in a day (§5.3). Separating the three guarantees:

**(a) Type / schema correctness — genuinely guaranteed, and the guarantee is real but
narrow.** The decoder's support *is* the declared answer set, so `answer ∈ declared_type`
is a theorem about the decoder, not a measured rate. It is stronger than
grammar-constrained LLM decoding in one specific respect: constrained decoding forces a
well-formed *string that parses*, and the parsed value can still fall outside the intended
domain — an enum member that is schema-legal but not what you meant, a number outside a
range, a truncation at `max_tokens`. Jev's answer space is the output alphabet, so there
is no string layer to go wrong and no length at which it can be cut off.

**(b) Decision correctness — not guaranteed, and conceded.** Empirically ~68% agreement
with a two-model consensus on TypeSafe's own favourable benchmark; roughly one case in
three diverges from the reference. What the construction adds is a machine-readable
uncertainty channel intended to be thresholded — which is the substantive engineering
claim, and is exactly Chow's reject option (§6.4.1).

**(c) Auditability — the weakest of the three, and largely unaddressed.** What you get:
the full distribution rather than just the argmax; question decomposition, so a wrong
end-to-end decision can be localized to the atomic question that went wrong; and control
flow that lives in *your* code, reviewable and diffable. TypeSafe leans on exactly this —
*"The most important thing for humans to review is the questions and any threshold
constants … defined in a single code file."* What you cannot get: any account of *why* a
given question got its answer. No reasoning trace, no evidence spans, no attribution, no
rationale — stated flatly as a capability boundary: *"System One models do not write
replies, produce code, or generate explanations of their reasoning."*

**[inference] The honest characterization, and the sentence this report will reuse:
Jev provides structural auditability — which question, which state, which threshold,
which branch — and zero epistemic auditability: why this answer.** Two documented details
sharpen it. `confidence` is an undisclosed statistic (§6.4.3a), so the threshold you
would defend to a regulator has an input you cannot define. And the Noul-vs-Choice
non-invariance means two framings of the same proposition can disagree by an order of
magnitude, so "which question you asked" is itself load-bearing in a way that is hard to
justify after the fact.

**[inference] The steelman, stated fairly:** the defensible version of TypeSafe's claim
is *"Jev cannot produce a value outside a set you declared, and it always tells you how
spread its belief was."* That is a real, checkable property and a genuine improvement
over parsing prose. The unsupportable version is *"Zero Hallucinations."*

### 3.7 Stated limitations — unusually candid, and directly relevant

**[measured]** The ["Jev 1.13 jaggedness"](https://docs.typesafe.ai/model-jaggedness/jev-1.13)
page (reviewed 2026-09-17) opens: *"`jev-1.13` is fast, calibrated, and good at
common-sense judgment **but it is not perfect** … **It can be quite literal in its
understanding. It struggles with tasks that require numeric precision.**"* Its nine
failure modes:

1. **Literal reading** — "answers the question you wrote, not the one you meant."
2. **Math and numbers** — *"Jev is not a calculator"*; *"does not count reliably … recognizes the shape of an answer rather than tallying"*; and on Score, *"score levels are weak in numerical calibration."*
3. **Date and time comparison** — *"reads dates as text, not as ordered quantities."*
4. **Indirection** — "a property of a property … costs accuracy."
5. **Large state full of irrelevant detail** — *"Jev suffers from context rot."*
6. **Adversarial content** — *"State is data, and `jev-1.13` does not treat it as hostile by default."*
7. **Contradictory instructions and criteria.**
8. **Common-sense structural invariants** — the Noul-vs-Choice and claim-plus-negation examples (§4.7).
9. **Generation** — "not trained to generate text."

Its closing advice names the boundary directly: avoid *"asking the model something code
can compute exactly"*, *"hiding several judgments inside one question"*, and **"System Two
tasks: more layers of indirections."**

**[inference]** Items 2, 3 and 8 are the ones that bear on Horismos, and they bear hard
(§6.2.3). Item 9's advice — *use code for what code can compute exactly* — is, in this
corpus's vocabulary, "neural proposes, symbolic decides", written by the vendor of the
neural half.

### 3.8 The Doom demo

**[measured]** The live URL is
[theregister.com/ai-and-ml/2026/09/16/typesafe-ai-debuts-model-for-machines-that-plays-doom/5296711](https://www.theregister.com/ai-and-ml/2026/09/16/typesafe-ai-debuts-model-for-machines-that-plays-doom/5296711)
— Thomas Claburn, 2026-09-16 02:35 UTC. The intake's 404 was a URL missing its
article-ID suffix. The original source is TypeSafe's own launch blog, "Fun Demos",
embedding Vimeo 1227495732.

**[measured]** TypeSafe's own nuance, verbatim: *"The demo is on structured state as a
data structure with text, not on images (yet…)"* and *"**A non-AI doom bot could play
better**, but we wanted a bot that was reactive to different representations of game
state."* The engineer's concern was cost at ~10 queries/second — about **$7/hour**. The
Register adds its own scepticism: the hallucination-free claim *"really isn't a fair
comparison as its output is not natural language … and that does not preclude the
possibility of being incorrect."*

**[inference]** Technically the demo shows one thing and one thing only: **Choice over
enumerated actions at interactive frame rates**, ~10 decisions/second sustained at
~$0.0002/call, with semantic robustness to how the harness phrases the world. It shows
nothing about capability — TypeSafe concedes a scripted bot plays better — and the
promised in-depth walkthrough had not shipped as of 2026-09-21 (no Doom repository exists
in the `typesafe-ai` GitHub organization). It is a latency and cost demo wearing a
capability costume, and the critics who called it "cooked" (§5.3) are reading the same
caveats TypeSafe printed.

---

## 4. Q-B — the literature landscape for this class of thing

The question the captain asked is whether "System One model" is a genuinely new
category or a rebrand. The answer is that it is a **product category built on a
well-populated research square**, and that the square has an unusually complete set of
owners. This section names them, then splits the claims three ways.

### 4.1 The structural reduction

**[inference]** Jev's described architecture decomposes without residue into four parts:
(1) a shared text encoder over unstructured state; (2) task-specific heads, one per
declared question, evaluated in one forward pass; (3) a schema fixing the output space
in advance; (4) a probability per head trained against a proper scoring rule.

**[measured]** The standard name for (1)+(2) is **multi-task learning with hard
parameter sharing** — Caruana, *Multitask Learning*, Machine Learning 28:41–75 (1997)
**[unverified — secondary sources only; the PDF was not opened]** — whose canonical NLP
instantiation is **MT-DNN** (Liu, He, Chen, Gao, ACL 2019,
[arXiv:1901.11504](https://arxiv.org/abs/1901.11504)) **[unverified]**. Note which fork
this is: MT-DNN's literal multi-head design, not the T5/UnifiedQA route that casts every
task as generation into a shared token space. Jev is on the older and cheaper side of
that fork.

**[inference]** The one component with no exact published precedent is that the heads
are **declared at request time** rather than fixed at training time — a schema-conditioned
multi-task model rather than a fixed one. That is a real but incremental extension, and
it is the honest core of the novelty claim.

### 4.2 "System One" is borrowed vocabulary, and it is borrowed pointing the other way

**[measured]** The dual-process frame was already model-operating-mode vocabulary in ML
before TypeSafe used it. *System-1.x: Learning to Balance Fast and Slow Planning with
Language Models* (Saha, Prasad, Chen, Hase, Stengel-Eskin, Bansal,
[arXiv:2407.14414](https://arxiv.org/abs/2407.14414), ICLR 2025) explicitly defines
System-1 as "directly generate plans without any explicit search or backtracking" and
System-2 as "planning step-by-step by explicitly searching over possible actions", with
a controller, a hybridization factor, and **a neuro-symbolic variant with a neural
System-1 and a symbolic System-2**. *From System 1 to System 2: A Survey of Reasoning
Large Language Models* ([arXiv:2502.17419](https://arxiv.org/abs/2502.17419), Feb 2025)
uses System 1 for foundational LLMs that "excel at fast decision-making but lack the
depth for complex reasoning" and System 2 for the o1/R1 generation. *Reasoning on a
Spectrum* ([arXiv:2502.12470](https://arxiv.org/abs/2502.12470)) is a third
**[unverified — snippet]**.

**[measured]** A precision worth recording, because the corpus will otherwise repeat a
common attribution error: Bengio's two canonical "System 2" papers — *The Consciousness
Prior* ([arXiv:1709.08568](https://arxiv.org/abs/1709.08568)) and *Inductive Biases for
Deep Learning of Higher-Level Cognition*
([arXiv:2011.15091](https://arxiv.org/abs/2011.15091)) — **do not use the terms in their
abstracts**. The vocabulary attaches to Bengio through his NeurIPS 2019 keynote
**[unverified]**.

**[inference]** Two observations follow. First, "System One model" is a *nominalization*
of a phrase already in circulation, not a coinage. Second, and more interesting: in
every prior ML use, System 1 is the **deficient** mode — the whole arc of the 2502.17419
survey is System 1 → System 2 as *progress*, and System-1.x's neuro-symbolic variant
puts the *symbolic* engine in the System 2 slot. TypeSafe is selling, as a frontier
category, the mode the field spent two years trying to escape. That is defensible on
task-fit grounds; it means the branding borrows authority from a literature pointed the
other way.

**[inference — and this matters for §6]** Under System-1.x's own vocabulary, **Horismos
is a System 2**: an explicit search over declared rules with backtracking
(`reason/backward.rs`) and a symbolic decision procedure. If the category takes hold, it
gives Horismos a ready-made, literature-sanctioned position — "the System 2 that the
System 1 proposes into" — rather than a competing one.

### 4.3 Calibration: a solved, textbook stack

**[measured]** The theory Jev's pitch describes without citing:

- **Brier (1950)**, "Verification of Forecasts Expressed in Terms of Probability,"
  *Monthly Weather Review* 78(1):1–3 **[unverified — snippet]**.
- **Dawid (1982)**, "The Well-Calibrated Bayesian," *JASA* 77(379) — the origin of the
  calibration criterion **[unverified — snippet]**.
- **Gneiting & Raftery (2007)**, "Strictly Proper Scoring Rules, Prediction, and
  Estimation," *JASA* 102:359–378 — a rule is *proper* if the forecaster maximizes
  expected score by reporting their true belief **[unverified — snippet]**. This is the
  theorem that makes "train on Brier" principled rather than heuristic, and it is
  nineteen years old.
- **Gneiting, Balabdaoui & Raftery (2007)**, *JRSS-B* 69(2):243–268 — the **"maximize
  sharpness subject to calibration"** paradigm **[unverified — snippet]**.
- **Guo, Pleiss, Sun & Weinberger (2017)**, "On Calibration of Modern Neural Networks,"
  ICML 2017 ([arXiv:1706.04599](https://arxiv.org/abs/1706.04599)) **[measured]** —
  "modern neural networks, unlike those from a decade ago, are poorly calibrated", and
  "temperature scaling — a single-parameter variant of Platt Scaling — is surprisingly
  effective at calibrating predictions."
- **Minderer et al. (2021)** ([arXiv:2106.07998](https://arxiv.org/abs/2106.07998))
  partially reverses Guo: the most recent non-convolutional models are among the
  best-calibrated **[unverified — snippet]**.

**[measured]** Prompt-free typed classification at small scale is equally settled: Yin,
Hay & Roth, "Benchmarking Zero-shot Text Classification" (EMNLP 2019,
[arXiv:1909.00161](https://arxiv.org/abs/1909.00161)) unifies heterogeneous
classification into a single entailment formulation with the label set declared in
advance; SetFit ([arXiv:2209.11055](https://arxiv.org/abs/2209.11055), 2022) sells "no
prompts or verbalizers", orders of magnitude fewer parameters, and explicit freedom from
"the instability and inconvenience of prompting" **[unverified — snippet]**.

**[inference]** SetFit's selling points are, line for line, Jev's, published in September
2022. The one thing SetFit did not claim is calibration.

### 4.4 The crux: do small discriminative models beat frontier LLMs on typed decisions?

This is the question that decides whether Jev is a new category or good engineering.

**[measured] On accuracy and cost: yes, decisively, three times independently.**
Bucher & Martini ([arXiv:2406.08660](https://arxiv.org/abs/2406.08660)) find fine-tuning
wins "in all cases" against GPT-3.5/GPT-4/Claude Opus. Valdes Gonzalez, "Cost-Aware
Model Selection for Text Classification"
([arXiv:2602.06370](https://arxiv.org/abs/2602.06370), Feb 2026): BERT-family encoders
are "competitive, and often superior … at **one to two orders of magnitude lower cost
and latency**" versus GPT-4o and Claude Sonnet 4.5, with p50 latency on SST-2 of 98 ms
(DistilBERT) against 1,394 ms (Claude zero-shot), and cost per 1M IMDB requests of
**$12.44 (DistilBERT) against $1,174.95 (Claude zero-shot)**. Christou & Tsoumakas,
"Sub-Billion, Super-Frontier" ([arXiv:2606.22606](https://arxiv.org/abs/2606.22606), Jun
2026): a fine-tuned Qwen2.5-0.5B reaches 0.83 positive-class micro-F1 against GPT-5.4's
0.69 and Claude Sonnet 4.6's 0.66 — and, decisively, "an in-domain RoBERTa baseline also
exceeds both frontier models," so "the gain stems from task adaptation rather than
generative decoding."

**[measured]** The encoder-vs-decoder question now has a matched-recipe answer:
**Ettin / "Seq vs Seq"** (Weller, Ricci, Marone, Chaffin, Lawrie, Van Durme,
[arXiv:2507.11412](https://arxiv.org/abs/2507.11412), ICLR 2026) trains paired
encoders and decoders 17M–1B on identical data and finds "a 400M encoder outperforms a
1B decoder on MNLI, and vice versa for generative tasks."

**[measured]** And the sharpest single datapoint the dive found: "Do Encoders Suffice? A
Systematic Comparison of Encoder and Decoder Safety Judges"
([arXiv:2606.25782](https://arxiv.org/abs/2606.25782), Jun 2026) measures, at batch
size 8, **Ettin-150M at 244.76 completions/sec and 3.60 ms median latency** against
**LlamaGuard-4-12B at 1.31 completions/sec and 717.63 ms median**, with better
out-of-distribution F1 (0.547 vs 0.385) — while concluding that "encoder classifiers are
best viewed as efficient first-line filters, while higher-cost decoder-based judges or
trajectory-aware systems may still be necessary."

**[measured] On calibration specifically: the head-to-head benchmark does not exist.**
Sub-agent B ran four differently-phrased searches for a study measuring ECE or Brier for
a fine-tuned encoder and a frontier LLM on the same task, and found none — and noted that
the paper that most obviously should contain it (2602.06370) conspicuously reports no
calibration metric. **[inference]** This is a genuine hole in the literature, not a
search failure.

**[measured]** The *mechanism*, however, is established and more general than a benchmark
would be. Desai & Durrett, "Calibration of Pre-trained Transformers" (EMNLP 2020,
[arXiv:2003.07892](https://arxiv.org/abs/2003.07892)): BERT and RoBERTa are solidly
calibrated in-domain, with out-of-domain calibration error "as much as 3.5× lower" than
non-pretrained baselines. And SECL
([arXiv:2604.09624](https://arxiv.org/abs/2604.09624)) names a **"calibration gap"** —
LLMs produce better-calibrated signals through *discriminative* tasks than through
generative verbalized confidence, "theoretically grounded" in generative error being
lower-bounded by roughly twice the corresponding discriminative error.

**[inference] The crux therefore resolves as: Jev is not refuted by prior work — it is
anticipated by it.** "A well-calibrated small discriminative model beats a big
generative one on a fixed label space at one to two orders of magnitude less cost" is a
position the field already occupies and already expects to win. Nobody had bothered to
run the ECE column. That is a commercial opportunity, not a scientific discovery — and
TypeSafe has not run the ECE column either (§5.2).

### 4.5 Routing and cascades: Jev is the first tier, unbundled

**[measured]** The lineage is complete. FrugalGPT (Chen, Zaharia, Zou,
[arXiv:2305.05176](https://arxiv.org/abs/2305.05176), 2023) — prompt adaptation, LLM
approximation, LLM cascade; "up to 98% cost reduction" matching GPT-4. Hybrid LLM
([arXiv:2404.14618](https://arxiv.org/abs/2404.14618), ICLR 2024) — routing by predicted
difficulty and a desired quality level; "up to 40% fewer calls to the large model, with
no drop in response quality". AutoMix
([arXiv:2310.12963](https://arxiv.org/abs/2310.12963), NeurIPS 2024) — a POMDP router
selecting model size "based on answer confidence". RouteLLM
([arXiv:2406.18665](https://arxiv.org/abs/2406.18665)) — preference-trained routers,
>2× cost reduction.

**[measured]** And the one that is nearly Jev's thesis statement: **Cascade-Aware
Training** (Google, [arXiv:2406.00060](https://arxiv.org/abs/2406.00060), 2024) trains
"the small LM with awareness of its place in a cascade and downstream capabilities",
validated over 60 tasks. **[inference]** Its entire mechanism is making the small
model's probabilities a trustworthy deferral signal — the same sentence as the RLCD
pitch, written in 2024, with the product part removed.

**[measured]** The strongest argument *for* Jev is one TypeSafe does not make. Bouchard,
"Is Escalation Worth It? A Decision-Theoretic Characterization of LLM Cascades"
([arXiv:2605.06350](https://arxiv.org/abs/2605.06350), 7 May 2026): across MATH, MMLU,
TriviaQA, SimpleQA and LiveCodeBench, "a lightweight pre-generation router exceeded the
best cascade policy on four of five datasets," because "cascade performance is limited
primarily by structural cost, since cascades pay the cheap model before any escalation
decision." **[inference]** A model that emits no tokens has driven that structural cost
to its floor. Jev is the pre-generation router *with a typed answer attached* — the
configuration 2026 decision theory endorses — and it takes the interesting side of the
open question by reusing the router's verdict as the answer instead of discarding it.

**[measured]** The strongest argument *against* is also from May 2026: **UCCI**
([arXiv:2605.18796](https://arxiv.org/abs/2605.18796)) maps token-margin uncertainty to
per-query error probability with isotonic regression and reports **ECE 0.12 → 0.03 and a
31% inference-cost cut (95% CI [27%, 35%]) at micro-F1 0.91 on 75,000 production NER
queries**. **[inference]** Calibration that is good enough to threshold is obtainable as
a *post-hoc transform on models you already run*. It is not a model class.

**[measured]** Two further 2026 results a report written today must carry: **Conformal
Cascade** ([arXiv:2607.25018](https://arxiv.org/abs/2607.25018), Jul 2026) replaces
confidence thresholds with prediction-set size — "accept when the calibrated set
collapses to a single answer, defer otherwise" — with distribution-free coverage over 18
datasets; and **Forced Deferral**
([arXiv:2606.15308](https://arxiv.org/abs/2606.15308), Jun 2026) shows that because "the
weak model's confidence directly controls compute allocation", a universal adversarial
trigger can depress first-tier confidence and force escalation — an attack on *compute
allocation* rather than correctness. **[inference]** Horismos inherits this threat the
moment a confidence threshold decides whether a human is paged: an attacker who can
depress confidence can force `horis:fallback` at will, which is a denial-of-service on
the human in the loop. Doc 15 does not name it (§6.3, and OD-5).

### 4.6 Constrained decoding: why "0% structured-output error" is not a result

**[measured] TypeSafe says it themselves.** Their launch post sources the LLM
hallucination rates from OpenRouter data while stating that **Jev's rate is "not
empirical" but "0%" by schema guarantee**
([typesafe.ai](https://typesafe.ai/blog/introducing-system-one-models-and-jev)). It is an
architectural tautology.

**[measured]** The comparison class has been at or near 100% for three years. Outlines
(Willard & Louf, [arXiv:2307.09702](https://arxiv.org/abs/2307.09702), 2023) reframes
generation as FSM transitions, guaranteeing structure "with little overhead"; XGrammar
([arXiv:2411.15100](https://arxiv.org/abs/2411.15100), MLSys '25) reports "up to 100×
speedup" and "near-zero overhead structure generation". OpenAI's live Structured Outputs
documentation claims the feature "ensures the model will always generate responses that
adhere to your supplied JSON Schema" while **stating no percentage** and enumerating its
failure modes (safety refusals, `max_tokens` truncation, unsupported schema features)
([developers.openai.com](https://developers.openai.com/api/docs/guides/structured-outputs)).
The widely-quoted "100% vs <40%" figures are from OpenAI's 2024 blog post, which
returned HTTP 403 to every fetch — **[unverified]**, and should be cited as such.

**[measured]** The real nuance is coverage, not compliance. **JSONSchemaBench**
([arXiv:2501.10868](https://arxiv.org/abs/2501.10868), 10,000 real-world schemas, six
frameworks) measures empirical coverage collapsing as schemas get complex: on "GitHub
Hard", Guidance 41%, llama.cpp 39%, XGrammar 28%, **OpenAI 9%, Outlines 3%, Gemini <1%**.

**[inference]** So "100% schema compliance" means *100% compliance with the schemas the
engine accepts*. Jev's schema — three primitives — is so small that acceptance never
binds, which makes its 100% trivially true rather than impressive.

**[measured]** And the field moved past the metric eighteen months ago. **OrderBench**
([arXiv:2607.18261](https://arxiv.org/abs/2607.18261), May 2026, 2,400 test calls): the
strongest model achieved **100% schema validity in both modes but only ~80% semantic
success**, with weaker models showing "double-digit rates of schema-valid unsafe
acceptances", concluding that structured output "serves as a necessary interface
component rather than a replacement for proper domain verification and fail-closed
system architecture". The **Structured Output Benchmark**
([arXiv:2604.25359](https://arxiv.org/abs/2604.25359), 21 models, 25,091 records):
schema compliance near-perfect; **exact leaf-value accuracy 83.0% on text, 67.2% on
images, 23.7% on audio**.

**[inference]** OrderBench's conclusion is, almost word for word, Horismos's design
thesis — arrived at independently, from the other direction, by people benchmarking
structured output rather than building a kernel. §6 uses it.

**[measured]** On whether constraints hurt quality, both sides, resolved: "Let Me Speak
Freely?" (Tam et al., EMNLP 2024 Industry,
[arXiv:2408.02442](https://arxiv.org/abs/2408.02442)) reports GSM8K text→JSON-mode drops
of −27.3 pp (GPT-3.5-turbo) and **−63.1 pp (Claude-3-Haiku)** — but *improvements* on
classification (DDXPlus, Gemini-1.5-Flash +18.7 pp, GPT-3.5-turbo +11.4 pp). The .txt
rebuttal ([blog.dottxt.ai](https://blog.dottxt.ai/say-what-you-mean.html)) shows the
prompts differed across conditions, that JSON-mode was conflated with constrained
generation, and that strict regex parsing beat the paper's LLM "perfect parser" 61% to
57%. JSONSchemaBench's Table 8 refereed it: constrained decoding *improved* Last Letter
(50.7→54.0), Shuffle Objects (52.6→55.9) and GSM8K (80.1→83.8). A weaker version
survives at ≤8B scale (DCCD, [arXiv:2603.03305](https://arxiv.org/abs/2603.03305), lifts
strict structured accuracy 15.2%→39.0% on GSM8K at 1B).

**[inference]** For classification — Jev's entire domain — both sides of that debate
agree constraints help. The relevance to Horismos is narrower and positive: doc 06's
schema derivation is a constrained-output mechanism applied to a classification-shaped
task, which is the case the literature supports.

### 4.7 Non-autoregressive and parallel decoding, and the consistency problem

**[measured]** The lineage: Gu et al., "Non-Autoregressive Neural Machine Translation"
([arXiv:1711.02281](https://arxiv.org/abs/1711.02281), 2017) — "an order of magnitude
lower latency", ~2.0 BLEU below the autoregressive teacher; Mask-Predict
([arXiv:1904.09324](https://arxiv.org/abs/1904.09324), EMNLP 2019) — within ~1 BLEU; and
the 2024 reality check ([arXiv:2405.12788](https://arxiv.org/abs/2405.12788), ACL
Findings) — "state-of-the-art NAT still underperforms AT under more reliable evaluation
metrics", with "explicitly modeling dependencies … crucial for generating natural
language and generalizing to out-of-distribution sequences."

**[measured]** The live 2026 thread: LLaDA
([arXiv:2502.09992](https://arxiv.org/abs/2502.09992)); Mercury
([arXiv:2506.17298](https://arxiv.org/abs/2506.17298)) at 1,109 tok/s on an H100;
**DiffusionGemma** (Google DeepMind, released **2026-06-10**) — 26B MoE / 3.8B active,
">1,000 tokens per second on a single NVIDIA H100", **"256 tokens in parallel per
forward pass"**
([deepmind.google](https://deepmind.google/models/gemma/diffusiongemma/)); and
Nemotron-Labs-Diffusion ([arXiv:2607.05722](https://arxiv.org/abs/2607.05722), Jul 2026),
whose finding is that "AR and diffusion objectives are complementary". Parallel
generation went mainstream *and open* three months before Jev's early access.

**[measured] The counterweight, and the most transferable finding in this section:**
**ParallelBench** ([arXiv:2510.04767](https://arxiv.org/abs/2510.04767), v2 23 Jun 2026)
finds that "the conditional independence assumption in dLLMs causes parallel decoding to
ignore token dependencies, inevitably degrading generation quality when these
dependencies are strong", that current strategies "struggle to adapt their degree of
parallelism based on task difficulty", and that "standard benchmarks (e.g., math and
coding) are not sufficient to capture the quality degradation caused by parallel
decoding."

**[inference]** Transfer that to N typed questions answered in one pass and it predicts
something precise: **the answers are conditionally independent given the input, so
cross-question logical consistency is not guaranteed by the architecture.**

**[measured] — and TypeSafe documents exactly this, in its own words.** The "Jev 1.13
jaggedness" page (reviewed 2026-09-17,
[docs.typesafe.ai](https://docs.typesafe.ai/model-jaggedness/jev-1.13)) lists
"common-sense structural invariants" as failure mode 8, with two worked examples: the
same proposition asked as a **Noul returned 0.22 while the equivalent Choice returned
P(yes) = 0.01**, and **a claim and its negation asked as two Nouls summed to 1.19**. The
guidance is explicit: *"don't rely on expected structural invariance … don't hold the
model to arithmetic identities between separate questions."*

**[inference]** This is the single most important structural fact in the dive, and it
did not have to be inferred — the vendor published it. §6 argues it is also the most
useful, because **a rule layer evaluated to a fixed point is exactly a cross-answer
consistency mechanism**, and the gate's cardinality and immutability checks are exactly
joint constraints across an instance. The property Jev documents itself as lacking is
the property Horismos's kernel supplies by construction.

**[measured]** Finally, "multiple answers from one pass" has a closer sibling than
diffusion: **Medusa** ([arXiv:2401.10774](https://arxiv.org/abs/2401.10774)) adds extra
decoding heads to a shared backbone with tree attention, for 2.2–3.6×; speculative
decoding ([arXiv:2211.17192](https://arxiv.org/abs/2211.17192)) gets 2–3× with
*identical* output distribution.

### 4.8 RLCD: a three-way name collision, and the direct predecessor

**[measured]** **Collision 1.** RLCD already exists: Yang, Klein, Celikyilmaz, Peng &
Tian, "RLCD: Reinforcement Learning from Contrastive Distillation for Language Model
Alignment" ([arXiv:2307.12950](https://arxiv.org/abs/2307.12950), **ICLR 2024**). It is
an alignment method and has nothing to do with calibration.

**[measured]** **Collision 2, and the one that matters.** Damani, Puri, Slocum, Shenfeld,
Choshen, Kim & Andreas, "Beyond Binary Rewards: Training LMs to Reason About Their
Uncertainty" ([arXiv:2507.16806](https://arxiv.org/abs/2507.16806), Jul 2025, **ICLR
2026**) names its method **RLCR — "Reinforcement Learning with Calibration Rewards"**.
One letter from TypeSafe's acronym, and semantically the same idea: augment the RL
reward with a **Brier score** so the objective incentivizes calibrated confidence rather
than mere correctness. Its framing of the problem is Jev's pitch verbatim — binary
rewards "do not penalize guessing or low-confidence outputs" and so "often have the
unintended side-effect of degrading calibration and increasing the rate at which LMs
generate incorrect responses (or 'hallucinate')". Its result: substantially improved
calibration with **no loss in accuracy**, in- and out-of-domain, **outperforming both
ordinary RL and post-hoc confidence classifiers**.

**[measured]** **Collision 3.** An open-source reproduction of Jev's interface expands
RLCD as "Reinforcement Learning with Calibration Distribution"
([github.com/wfzyx/von](https://github.com/wfzyx/von)).

**[measured]** And the subfield is crowded, not empty. Yaldiz, Spiliopoulou, Qi, Varia,
Doss & Pappas, "Balancing Classification and Calibration Performance in Decision-Making
LLMs via Calibration Aware Reinforcement Learning" (Findings of ACL 2026, pp.
12537–12553, [aclanthology](https://aclanthology.org/2026.findings-acl.610/)) opens with
what is effectively Jev's value proposition — "Well-calibrated confidence enables
downstream systems to decide when to trust a model and when to defer to fallback
mechanisms" — diagnoses why RLVR produces "extremely overconfident models", and reports
**ECE reduced by up to 9 points**. CAPO
([arXiv:2604.12632](https://arxiv.org/abs/2604.12632), Apr 2026) reports up to **15%**
calibration improvement at comparable or better accuracy. At least five further 2025–26
papers sit in the same square **[unverified — snippets]**.

**[inference]** "RL that optimizes epistemically honest probabilities instead of
human-preferred text" is not a frontier TypeSafe opened. It has an ICLR/ACL publication
record, a named canonical method, a diagnosed mechanism and quantified results — and the
closest one beat post-hoc confidence classifiers, which is the comparison class Jev sits
in.

### 4.9 Verdict: novel / good engineering / claims outrunning evidence

**Genuinely novel — three items, two of them packaging. [inference]**

1. **Selling the calibrated first tier as a product.** Cascade-Aware Training built the
   object in 2024; nobody sold one. Removing the fine-tuning pipeline is the real
   product, and it is the only thing a customer cannot get from an open checkpoint
   today.
2. **Removing generation from the first tier is the configuration 2026 decision theory
   endorses** (Bouchard, 2605.06350) — and Jev takes the interesting side by keeping the
   router's verdict as the answer.
3. **Request-time schema-conditioned heads** rather than training-time-fixed heads.

**Good engineering over known ideas.** Every remaining component has an owner:
multi-head classification (Caruana 1997 / MT-DNN 2019), schema-declared typed output
(Yin et al. 2019), prompt-free small-model classification (SetFit 2022), proper scoring
rules (Brier 1950 / Gneiting & Raftery 2007), temperature scaling (Guo et al. 2017),
constrained decoding (Outlines 2023 / XGrammar 2024), multi-head parallel output
(Medusa 2024), encoder-beats-decoder-on-classification (Ettin, ICLR 2026;
arXiv:2602.06370; arXiv:2606.25782), confidence-driven deferral (FrugalGPT 2023 /
AutoMix 2024), training the tier so its probabilities route well (CAT 2024), and
calibration-rewarded RL (RLCR, ICLR 2026).

**Where the claims outrun the evidence. [inference] unless tagged**

1. **"0% structured-output error" is a definition, and TypeSafe says so** — "not
   empirical" **[measured]**. Over a three-primitive schema every constrained-decoding
   stack since 2023 also scores 100%.
2. **The metric was retired in 2026.** OrderBench: 100% schema validity, ~80% semantic
   success **[measured]**.
3. **"0% tool-call error" is a category error** — Jev makes no tool calls.
4. **"Single parallel pass" is multi-head classification** with an unaudited
   conditional-independence problem (ParallelBench) **[measured]**.
5. **The latency and cost headlines use the wrong baseline.** Against a fine-tuned
   encoder — the correct comparator for a classification task — Jev's 70–500 ms is
   ~20–140× *slower* than Ettin-150M's 3.60 ms median, and its $0.0004/case is ~12–32×
   *more expensive* than DistilBERT's $0.0000124/case **[measured]**.
6. **Calibration carries a silent qualifier**: TypeSafe's own docs state it "is measured
   across groups of predictions", not per instance **[measured]** — exactly what a
   threshold rule needs and no more.
7. **"First System One model" is unsupportable on both words** — the term predates the
   product with the opposite referent, and an open reproduction on ModernBERT-Large
   (395M) exposes the identical Choice/Noul/Score surface **[measured]**.
8. **"RLCD" collides with two published methods**, one of which is its direct
   predecessor **[measured]**.

---

## 5. Q-C — how it is actually doing

Six days is not an adoption curve, and the honest headline is **too early to tell**. But
"too early" is not "no signal": several things are already settled, and the corpus
should record which.

### 5.1 The adoption claim: verified, and narrower than it sounds

**[measured]** The "fastest-adopted model in AI Gateway history" line is real, and it is
**Vercel's own words in Vercel's own engineering post**, not TypeSafe marketing —
[vercel.com/blog/ai-gateway-jev-model-launch](https://vercel.com/blog/ai-gateway-jev-model-launch),
2026-09-18, bylined Amelia Charles, Harpreet Arora and Eric Dodds:

> Within 24 hours of launching on AI Gateway, Jev from TypeSafe AI reached more than
> twice as many paid teams as any previous model launch, making it the fastest-adopted
> model in gateway history.

**[measured]** The metric is **the percentage of Vercel paid teams that used the model
at least once** — breadth of trial, not request volume, not tokens, not retention:
~10% by hour 18, ~13% by hour 24; 2× the GPT-5.6 family, >6× Fable 5.1. Vercel states
the caveat itself in the same post: "Its first-day adoption was unmatched among recent
launches; the next test is whether that early adoption lasts." Trying it was a one-line
change — `experimental_evaluate` in AI SDK 7.0.105+
([Vercel changelog, 2026-09-16](https://vercel.com/changelog/typesafe-ai-jev-now-available-on-ai-gateway)).

**[measured]** Package registries, pulled directly: PyPI `typesafe-sdk` went 15 (09-14)
→ 162 (09-15) → 1,329 → 12,925 → 26,445 → 28,284 → 29,602 (09-20) daily downloads,
plateauing at ~28–30k/day; npm `@typesafe-ai/sdk` 58,007 (09-18) and 51,347 (09-19).
Hex.pm, by contrast: `typesafe_sdk` **307 downloads all-time**, `typesafe_api` 98, `jev`
82. GitHub: the **official** Python SDK has 164 stars; `browser-use/jev-ultrafast` has
13,091; an open-model clone (`TheoLeeCJ/SemIf`, "on a 3090 at home") has 2,621; a
reverse-engineered clone (`vinnylarouge/jevlike`) 1,105. 1,530 repositories matching
`jev typesafe` created since 2026-09-14. One sub-agent recorded general availability, waitlist
removed, on **2026-09-20** with $5 of free credit; another, reading TypeSafe's own FAQ on
2026-09-21, found the access answer still "Join the waitlist!". The discrepancy is
recorded rather than resolved in §3.1.

**[inference]** Two readings of the same numbers, and the report should carry both. The
generous one: a genuinely novel developer ergonomic got tried by one in eight paying
teams on a major platform in a day, and five Elixir clients appeared within 24 hours
because the shape is that easy to bind. The deflationary one: first-day breadth-of-trial
on a platform where trial is one line, during a 37-million-view launch (§5.4), is a
**curiosity metric**; and the community's dominant response was not to adopt but to
*clone* — open reproductions out-star the official SDK roughly 16:1, and the Elixir
download counts say actual Elixir usage is approximately zero despite five SDKs.

### 5.2 Independent verification exists, is unusually good for six days, and is mixed

**[measured]** There is **no Artificial Analysis listing, no LMArena entry, no paper, no
model card, no disclosed architecture, parameter count, training data or reward
function.** TypeSafe declined public benchmarks in the launch post — "We deliberately
chose not to publish performance against public benchmarks. In fact, we plan to only
have one-off evals when we make product updates" — with a supporting post at
`typesafe.ai/blog/antibenchmaxxing`.

**[measured]** TypeSafe's own dashboard ([evals.typesafe.ai](https://evals.typesafe.ai/))
confirms the intake numbers and then some. Aggregate across the four workflows:

| Model | Accuracy | Cost/case | Latency |
|---|---|---|---|
| Sol | **74.1%** | $0.0836 | 23.3 s |
| Opus 5 | **73.1%** | $0.1761 | 37.8 s |
| Terra | 67.9% | $0.0304 | 10.1 s |
| **Jev** | **67.8%** | **$0.0004** | **0.4 s** |
| Sonnet 5 | 67.8% | $0.1174 | 78.1 s |
| Luna | 66.8% | $0.0033 | 12.9 s |
| DS v4 pro | 65.5% | $0.0413 | 86.5 s |
| DS v4 flash | 64.4% | $0.0059 | 51.9 s |
| Haiku 4.5 | 53.6% | $0.0195 | 12.5 s |

**[measured]** And the "accuracy" is agreement with a consensus label built by averaging
two frontier models, which one HN commenter (`zmmmmm`) reduced to: "the outcome is 'how
much of a Fable am I getting'."

**[inference]** On the vendor's own scoreboard Jev is **fourth of nine on accuracy**,
beaten outright by two models, edged by a third, and tied with a fourth — while being
two to three orders of magnitude cheaper and one to two orders faster. That is a
perfectly respectable result for a purpose-built model. It is not a frontier-intelligence
result, and the marketing frame does not survive its own benchmark.

**[measured]** Five independent efforts in six days, which is genuinely fast for this
kind of scrutiny:

- **[priorbench/jev](https://github.com/priorbench/jev)** (2026-09-20) — the only
  *pre-registered* evaluation: 50 predictions timestamped before collection, 26
  confirmed / 21 falsified; 5,721 calls; raw JSONL published. 95.9% zero-shot on their
  400-item benchmark vs 77.2% hand-written keywords and 66.0% supervised TF-IDF+LR; a
  ~430 ms latency floor; **800 typed judgments in one 985 ms call for $0.00075**. Three
  findings matter here more than the headline: *"Accuracy above threshold is flat from
  0.50 to 0.95, then jumps to 100% at 0.99. Gate at 0.99 or not at all."* — **"0 of 30
  out-of-scope messages were flagged"** — and with deliberately wrong criteria
  descriptions the model scored **16.7%, below the 25% random floor**.
- **[primeline.cc](https://primeline.cc/blog/typesafe-jev-pre-registered-test)**
  (2026-09-18), 9,750 calls, pre-registered: Jev **first** on 800 commit messages
  (65.8% vs Opus 5 63.5%), **third** on 450 knowledge-base notes (90.7% vs Haiku 4.5
  97.8%). Verdict: "you can get the same answer out of any LLM. You cannot get the same
  number." Rankings fully reverse between tasks.
- **[xenospectrum](https://xenospectrum.com/en/jev-typesafe-bert-classifier-decomposition/)**
  (2026-09-20), 2,000 phishing emails: single question, **Jev 62.6% vs Claude Haiku 4.5
  81.3%**; decomposed into five atomic questions with a logistic regression fitted on
  1,000 labeled examples, Jev 95.0% vs Haiku composite 93.2% — *p = 0.063, not
  significant*. **A two-line regex scored 91.8%.**
- **[Arize](https://arize.com/blog/typesafe-jev-llm-judge/)**: on spam detection,
  **Jev 98.3% zero-shot vs TF-IDF logistic regression 98.4%** on 14,800 labeled emails,
  a statistically insignificant difference.
- **[southbridge.ai](https://www.southbridge.ai/blog/jev-entity-resolution)** (2026-09-20),
  the strongest positive: Ohio campaign-finance entity resolution at 3.37¢/56 s (1
  mistake, 4 declined) against a frontier baseline at $9.83/845 s — **226× cheaper,
  7.35× throughput, within 0.5 pp accuracy** — but only in a *hybrid* architecture with
  an LLM review layer.

**[measured]** Two early-access reports carry the best calibration evidence in either
direction. [Emil Lindfors](https://lindfors.no/blog/a-first-look-at-typesafes-jev/)
(2026-09-18), 24 Norwegian salmon-tax hearing documents × 11 structured questions:
97–98% agreement at 0.9+ confidence — but **ECE rose 0.040 → 0.116 when questions were
made longer and more carefully qualified**. [Good Start
Labs](https://goodstartlabs.com/research/verification-is-the-bottleneck) (early access,
disclosed), 6,003 rubric checks: Jev at $160/M graded vs DeepSeek V4.1 Flash at $260/M
for *slightly better* agreement — a point the author conceded in the HN thread when
`kevmo314` put it to them directly.

**[measured]** Two hands-on failures are worth naming because of their *shape*:
[backnotprop](https://backnotprop.com/blog/jev-poker/) gave it exact opponent cards and
full game state and got solver agreement on 63% of 30 spots — including a trivial spot
holding the nuts where the solver checks 100% of the time and "Jev shoved in sixteen
runs out of sixteen." [Bartosz
Mikulski](https://mikulskibartosz.name/typesafe-jev-guess-what-i-drew) fed 400
Quick,Draw! sketches as SVG coordinates: Jev 35% vs Sonnet 5 on the *same* SVG 57%; and
**"Jev answered *airplane* for 209 of the 400 drawings"** — 85% on airplanes, 0% on
cats, a total per-class collapse invisible in the aggregate.

### 5.3 The skeptical case, in critics' own words

The HN launch thread ran to **1,930 points / 509 comments**
([49717558](https://news.ycombinator.com/item?id=49717558)). The recurring themes, not
the one-offs:

**"It's a zero-shot classifier with a launch video."** **[measured]** The single
most-repeated reaction was `gok`'s three words: *"So... a classifier model?"*.
`petesergeant`: *"This is basically a zero-shot classifier that can accept raw text… as
accurately (they claim) as a frontier-level LLM"* — to which the CEO replied **"exactly
right!"**. `prometheus1992` named a substitute (`deberta-v3-large-zeroshot`) and called
the blog comparison "definitely shady"; `mary776` guessed GLiNER2. Nathan Flurry's
*"jev is just a really smart switch statement"* reached ~590,000 views; Theo Browne's
video landed on *"This model is fast for dumb things, it's not smart for hard things"*
and the sharper question *"I don't get why everybody is recommending using a model that
doesn't reason to decide between the results of models that do reason."*

**"Can't hallucinate" is a word game.** **[measured]** The largest sub-thread. The
sharpest formulation, from `thduabmd`: *"Your launch post puts '0%' on a hallucination
chart, then explains that the number comes from guaranteed schema matching. You've
already agreed that this doesn't establish correctness. **An approve for an unauthorized
action still meets the schema guarantee.**"* `WhitneyLand`: **"Type safety is not
factual correctness"** — to which the CEO replied *"I very much agree with this."*
The Register made the same point editorially.

**The benchmark is self-run against itself.** **[measured]** `zmmmmm` (above);
`jceg`: *"'We deliberately chose not to publish performance against public benchmarks.'
lol, I bet they would publish them if their score on those benchmarks were good."*;
`ramon156` on the latency figures being *"apples-to-oranges unless the LLM baseline is
doing comparable work"*; and `NitpickLawyer` on the structured-output comparison: *"You
can setup constrained gen so that only valid options can be outputted. Their own
inference lib likely does that. So comparing to one that doesn't is a bit cheeky."*

**The Doom demo is cooked.** **[measured]** `bigglebear`: *"They've severely cooked this
… it can cheat because the structured state it is using is not bound by obstructions …
You could re-create this whole demo without using AI."* `hdjrudni`: *"They're not
feeding it video, they're feeding it a text description of what's going on."*
[The Register's coverage](https://www.theregister.com/ai-and-ml/2026/09/16/typesafe-ai-debuts-model-for-machines-that-plays-doom/5296711)
is the live URL the intake could not find — the 404 was a missing article-ID suffix.

**Prior art.** **[measured]** A separate 88-point HN thread
([49736660](https://news.ycombinator.com/item?id=49736660)) from Nandakishor Mukkunnoth
claims priority. The specific arXiv IDs he cites do **not** support the claim — sub-agent
B opened both: [2503.23303](https://arxiv.org/abs/2503.23303) is a sales-conversion RL
agent and [2510.01237](https://arxiv.org/abs/2510.01237) is confidence-aware routing;
neither is a non-autoregressive typed-decision model. **[inference]** The prior-art
objection is nevertheless correct — it just has the wrong citations. §4 supplies the
right ones.

**The strongest defences**, which the report should not bury: `janalsncm` — *"Frontier
LLMs are expensive jack of all trades. You can absolutely compare them to purpose-built
tools on any domain they touch."*; `lubujackson` — *"After much fumbling around with
prompts and evals, this is exactly how I am using LLMs in production, to narrowly make
choices and return structured data."*; and Sean Goedecke, who replicated the core trick
on `Qwen2.5-1.5B-Instruct` for a 2–3× speedup and still concluded that *"fast structured
output could be a genuinely new computational primitive"* while judging the moat thin.

### 5.4 Momentum versus capability, with the receipts

**[measured]** The launch was a purchased, documented distribution campaign. The agency
published its own case study:
[doomers.ai/work/typesafe-ai-case-study](https://doomers.ai/work/typesafe-ai-case-study)
— 37,196,137 post views; **80–100 accounts seeded in hour one**; 150,000+ waitlist
signups in 24 hours; founder account 1,500 → 124,021 followers; company account 200 →
99,887; ranked 4th of 653 launches the agency has run. Founder post at 18:17 UTC on
2026-09-15, company account within the hour, the $40M Business Wire release the same
day.

**[measured]** [openchamber.dev](https://openchamber.dev/blog/jev-typesafe-ai/) analysed
12,759 of 26,896 collected posts and separated vendor claims from user-reported
measurements:

| | TypeSafe claim | User-reported median |
|---|---|---|
| Speed | 193.6× | **7×** (n = 215) |
| Cost | 444.6× | **30×** (n = 180) |
| Latency | 70–500 ms | 76 ms (n = 333) |

**[inference]** This is the cleanest momentum-versus-capability number available. The
real advantage is roughly an order of magnitude smaller than the headline — and still
large. 7× faster and 30× cheaper on typed decisions is a good product. "193.6× / 444.6×"
is a marketing artifact.

**[measured]** Company staff worked the threads directly: the CEO posted ~20+ HN
replies, and a staff account offered faster waitlist access *"in exchange for good
memes."* **[inference]** Effective and not dishonest, but it means a meaningful share of
the visible positive signal was cultivated. The only disclosed investor anywhere is
**DCVC** (lead, $40M seed —
[DCVC, James Hardiman, 2026-09-15](https://www.dcvc.com/news-insights/typesafe-emerges-from-stealth-with-a-new-way-of-doing-ai/));
no financial tie between Vercel and TypeSafe was found, which is why the Vercel post
carries the weight it does here.

**[inference]** Press ratio: roughly **8 genuinely hands-on pieces against 25+
press-release restatements** in six days, and the hands-on ones are overwhelmingly
individual blogs rather than outlets. The most rigorous single artifact — priorbench's
pre-registered evaluation with published raw data — had **zero GitHub stars** when
sub-agent D read it.

### 5.5 Bottom line, and what would settle it

**[inference] Already settled:**

1. Latency and cost are **directionally real and independently reproduced** by at least
   four unaffiliated parties. Sub-500 ms typed decisions at ~$0.0004/case is not vapour.
2. The **intelligence claim is falsified by TypeSafe's own dashboard** (4th of nine).
3. The **launch was bought**, and documented as such by the agency that ran it.
4. The **headline multipliers are ~10× inflated** against user-reported medians.
5. **"Can't hallucinate" is definitional**, and the CEO conceded the point in public.
6. The **architecture is not defensible as a moat**: a Qwen-2.5-1B reproduction appeared
   within hours, open clones out-star the official SDK, and a 2003-vintage TF-IDF
   logistic regression ties it on spam.

**[inference] Genuinely unresolved:** whether first-day breadth converts to sustained
traffic; whether the calibration is real (the only defensible moat claim, and the one
with the least published evidence — nobody has published a reliability curve, TypeSafe
included); whether the pricing is sustainable ("we can't prove it isn't subsidized");
and whether the hybrid pattern that actually works in practice (Jev plus an LLM review
layer) leaves enough value for a standalone model company.

| Evidence that would settle it | What it settles | Expected |
|---|---|---|
| Vercel's day-30 / day-90 cut of the same metric | Whether the adoption number meant anything | ~mid-Oct 2026 (Vercel promised the follow-up) |
| A published ECE / reliability curve on ground truth | The calibration moat | Any time; continued absence past ~30 days is itself the answer |
| Head-to-head against DeBERTa-zeroshot / GLiNER / a fine-tuned 1B on identical tasks | The "just a classifier" charge | Days — the clones exist now |
| A technical report | Novelty and the prior-art dispute | Unknown; treat silence as evidence |
| A named customer with disclosed volume | Whether it is a business | Q4 2026 |

**[inference] The finding Horismos should carry forward** is not about Jev's business.
It is that the failure modes independent testers found in six days — confidently wrong
type-valid outputs, **zero refusal capability (0 of 30 out-of-scope inputs flagged)**,
**below-random accuracy when the question descriptions were wrong**, per-class collapse
hidden by an aggregate, and calibration that degrades as questions get more carefully
qualified — are precisely the failure class that "neural proposes, symbolic decides"
exists to contain. A faster, cheaper, better-shaped transducer does not remove the need
for the gate; it makes the gate cheaper to run.

---

## 6. Q-D — positioned against Horismos

This is the heart of the task. Everything below stands on §2 (what the repository
actually specifies) and §§3–5 (what Jev actually is, where it sits in the literature,
and how it is actually doing).

### 6.1 Threat, complement, or validation?

**[inference] Validation on the premise, complement in the architecture, and a real but
narrow threat to one piece of the positioning — not to the kernel.**

**Validation, and it is not a small one.** Horismos's first principle is "neural
proposes, symbolic decides — no rule firing, workflow transition, or tool call is ever
made by a model" (CLAUDE.md). The structural half of that — that a model's output should
be a *typed value in a space declared before the data arrives*, not free text — is
exactly what a $40M-seed company built a product around and what one in eight paying
Vercel teams tried within a day (§5.1). Horismos derives its typed space from an
ontology (doc 06's schema-derivation table); TypeSafe has the caller declare it in the
request body. Same premise, different source of the schema.

**[inference]** The premise is now *market-validated*, which changes what Horismos has
to argue. Before 2026-09-15, "decisions should be typed, not generated" was a position
that needed defending. It no longer is. The corpus should stop spending its positioning
budget there and spend it on what remains contested.

**Complement, mechanically — and the mechanism is documented by the vendor, not
inferred.** §4.7 records TypeSafe's own jaggedness page: a proposition asked as a Noul
returned 0.22 while the equivalent Choice returned P(yes) = 0.01; a claim and its
negation summed to 1.19; *"don't hold the model to arithmetic identities between
separate questions"* **[measured]**. That is a vendor-published statement that **the N
answers are not jointly constrained**.

**[inference]** A rule layer evaluated to a fixed point is, structurally, the missing
constraint. Horismos derives answers from other answers ([`reason/forward.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/reason/forward.rs)),
resolves conflicts by declared priority and records the loser as `Suppressed` or
`Overridden`, and the gate's cardinality and immutability checks
([`world/gate.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/world/gate.rs), check 4) are joint constraints across a whole instance rather
than per-field ones. Horismos supplies precisely the guarantee Jev documents itself as
not having. This is a complementarity claim with a mechanism and a citation behind it,
not a slogan — and it is the sharpest single sentence this dive produces for the
positioning.

**The threat, stated honestly.** It is not technical and it is not to the kernel. It is
to a *market segment*. If a calibrated typed classifier costs $0.0004 a case and answers
in 400 ms, then every deployment whose real requirement was "turn this text into a typed
value, then branch on it" will buy the classifier and write the `if` itself — and will
be right to. That segment was never going to buy a decision kernel. What Jev's existence
does is **narrow Horismos's addressable market to deployments that need the clause**:
regulated, audited, contested or multi-rule domains where the question is not "what is
the answer" but "on what basis, and can you show me". §6.3 asks whether that segment is
real in 2026.

**[inference]** A second-order threat worth naming because it is the kind that actually
kills positioning: **vocabulary capture**. If "System One model" becomes the accepted
name for "the typed-decision layer", Horismos risks being read as a slower, harder
System One model rather than as the System 2 it is. §4.2 shows the literature's own
usage makes the opposite framing available for free — System-1.x's neuro-symbolic
variant puts a *symbolic* engine in the System 2 slot. Taking it is a positioning
decision (OD-6), not a research finding.

**What is *not* a threat, and the corpus should say so plainly. [inference]** Nothing in
§§3–5 touches the gate, the proof tree, truth maintenance, determinism, the versioned
snapshot, the conditional commit, or verification. Jev has no state, no rules, no proof,
no commit path, no tool-calling surface, and — by its own documentation — no
explanation. A faster proposer makes a kernel that consumes proposals *cheaper to run*,
not less necessary. The six-day independent record makes this concrete in the least
flattering way for the "you don't need a gate" reading: **[measured]** priorbench found
**0 of 30 out-of-scope inputs flagged** and **16.7% accuracy — below the 25% random
floor — when the question descriptions were wrong**; TypeSafe's own jaggedness page says
*"State is data, and `jev-1.13` does not treat it as hostile by default. Content written
to adversarially steer the model … can move the answer."* A proposer that cannot refuse,
degrades below chance on a mis-specified question, and is not injection-hardened is
precisely a proposer that needs something downstream of it that can decide.

### 6.2 Jev as the proposer: what changes, and what breaks

#### 6.2.1 Where it goes: the Bridge slot, and only the Bridge slot

**[measured — local repo]** Doc 08 draws the line explicitly: an Adapter "is *not* a
Bridge, and gains no `Bridge` implementation — a Bridge supplies a neural *proposal*
that must clear the gate; an Adapter supplies authoritative structured state." A typed
decision from a model is a proposal. It goes behind `Bridge`.

**[inference]** The consequence is that **PRP-008's host SPI needs nothing at all for a
Jev adapter** — the opposite of what the task brief anticipated, and worth stating
plainly. `HostAdapter::hydrate`/`commit` ([`spi.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-host/src/spi.rs):501-508) is the
system-of-record boundary; `IntegrationTrace` already embeds the core `Trace`
([`host/trace.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-host/src/trace.rs):36-49), so a perception step performed by any bridge is already
carried inside the integration envelope with no schema change. The only host-adjacent
question a hosted typed-decision model raises is **data egress** (§6.2.6).

Registration is one line, because the registry is already prefix-keyed:

```rust
// crates/horismos-core/src/workflow/agent.rs:165, 207 — the registry
bridges: BTreeMap<String, Arc<dyn Bridge>>,
// …
self.bridges.insert(prefix.into(), Arc::new(bridge));
```

> **Python parallel.** `BTreeMap<String, Arc<dyn Bridge>>` is a `dict[str, Bridge]`
> whose values are any object satisfying the protocol. `Arc` is a reference-counted
> shared pointer — CPython refcounts every object already, so the Python reading is
> simply "a dict of prefix → bridge object". `dyn Bridge` is the part with no Python
> analogue *because Python needs none*: it tells Rust to do method lookup at runtime,
> which is Python's only mode.

#### 6.2.2 Method by method

| `Bridge` method | Jev fit | Consequence |
|---|---|---|
| `rank(RankRequest{context, candidates}) → RankResult{chosen, confidence}` | **Exact.** A Jev **Choice** over a closed option set with a probability is this signature — and Jev's Choice caps at 255 options **[measured]**, comfortably above any graph-derived candidate set | The best fit — but `rank` is **never called** in this build ([`deferred.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/deferred.rs):5-7). See §6.2.4 |
| `perceive(PerceiveRequest{text, schema}) → PerceiveResult{instances, confidence, …}` | **Partial, and more partial than it first looks** — see §6.2.3 | Workable for a schema of enums and booleans; **not workable for numeric or free-string properties at all** |
| `narrate(NarrateRequest{proof_text, audience}) → NarrateResult{text}` | **None.** Jev "does not write replies, produce code, or generate explanations of their reasoning" ([docs.typesafe.ai](https://docs.typesafe.ai/concepts/system-one)) **[measured]** | Already handled, and handled *well* — see §6.2.5 |

#### 6.2.3 Worked example: the triage `:Intake` step, and where it stops working

The canonical overlay declares ([`examples/triage/triage.ttl`](https://github.com/billaram/horismos/blob/adfc27a/examples/triage/triage.ttl):136-141):

```turtle
:Intake a horis:PerceptionStep ;
    horis:extracts :Patient, :Symptom ;
    horis:model "claude:claude-fable-5" ;
    horis:confidence 0.85 ;
    horis:fallback :EscalateToNurse ;
    horis:next :Assess .
```

**Today.** One call to a frontier model with a JSON Schema derived from `:Patient` and
`:Symptom` and a forced tool call; the model returns `{"instances": […], "confidence":
0.94}`; the runtime computes `0.94 >= 0.85`; each instance goes through the five-check
gate; accepted facts land with `Provenance::Perception`.

**Under a `typesafe:` bridge.** The ontology changes by exactly one string —
`horis:model "typesafe:jev"` — because the registry is prefix-routed and "the ontology
is provider-agnostic" (doc 08). Inside the bridge, the derived schema is decomposed into
Jev questions, and this is where the analysis gets interesting, because **the
decomposition does not cover the schema.**

**[measured]** Jev's three primitives are exactly:

- **Choice** — `criteria` is a map of option key → description, **maximum 255 options**;
  returns `choice`, a `probabilities` map summing to 1, and `confidence`.
- **Score** — `criteria` is an *ordered array of 2 to 10 level descriptions*; returns a
  probability-weighted `score` that can land between levels, a `legend`, `probabilities`
  and `confidence`.
- **Noul** — a yes/no proposition; returns **a single 0–1 number and no `confidence`
  field at all**.

([docs.typesafe.ai/api](https://docs.typesafe.ai/api),
[/primitives](https://docs.typesafe.ai/primitives),
[/primitives/noul](https://docs.typesafe.ai/primitives/noul).) The name "Noul" is
official, not a garbled transcription — it is a `type` enum value, a response field, a
Python class and a TypeScript interface.

Now map doc 06's schema-derivation table onto those three:

| Ontology construct (doc 06) | Derived JSON Schema | Jev primitive | Verdict |
|---|---|---|---|
| Subclass tree under an extract target | `enum` of local names | **Choice** | **Clean fit.** The closed symptom vocabulary is exactly a Choice's `criteria` map |
| `horis:oneOf ( … )` | `enum` of IRIs | **Choice** | **Clean fit** |
| `xsd:boolean` property | `boolean` | **Noul** | **Fits — but the answer arrives with no `confidence` field**, so a per-fact confidence has to be derived from the Noul value itself |
| `xsd:decimal` + `horis:min`/`horis:max` (+ `horis:unit`) | `number` with `minimum`/`maximum` | **— none —** | **Does not fit.** See below |
| `horis:pattern` | `pattern` | **— none —** | **Does not fit.** No free-string output exists |
| Any `xsd:string` / identifier / IRI-valued property | `string` | **— none —** | **Does not fit** |
| `required` from cardinality `1..` | `required` | **— none —** | Joint constraint; stays at the gate |
| `horis:assignedBy horis:RulesOnly` | omitted entirely | n/a | Unaffected — omission is the mechanism |

**[measured]** The numeric row is the important one, and it is not a nuance. A Score is
**not a numeric extractor**: it is an ordinal rubric over at most ten labelled levels.
TypeSafe's jaggedness page is unambiguous — *"Jev is not a calculator"*; *"`jev-1.13`
does not count reliably … The model recognizes the shape of an answer rather than
tallying"*; and, directly on Score, *"do not use score outputs … to compute the exact
magnitude of a number between two levels … score levels are weak in numerical
calibration."* Date and time comparison is separately listed as unreliable: *"reads
dates as text, not as ordered quantities."*

**[inference]** So the triage example's central fact — `:heartRate 128` (`xsd:decimal`,
`horis:min 0`, `horis:max 300`, `horis:unit "bpm"`) — **cannot be produced by Jev at
all.** The best available encoding is a Score over buckets ("normal", "elevated",
"tachycardic"), which throws away the value the rules actually compare against
(`horis:max 300`, and every threshold in the rule set). Horismos's own ontology
vocabulary is numeric-first: `horis:min`, `horis:max`, `horis:unit`, and decimal
comparison operators are central to doc 02's profile. That is a structural mismatch with
this product, not a configuration gap.

**[inference] The honest conclusion for Q-D2 is therefore narrower and more useful than
"yes, Jev can be the proposer":** a typed-decision model of this shape can occupy the
proposer slot **for the classification-shaped part of a perception step** — closed
vocabularies, enumerations, booleans, coarse ordinal judgments — and cannot occupy it
for measurement. A realistic deployment is *hybrid at the bridge*: structured values
arrive through the PRP-008 `ExternalSnapshot` path with `api` provenance (which doc 06
already promotes to "the **primary** structured-input path"), and a typed-decision model
handles the residual categorical judgments over free text. That is a better architecture
than either alone, and it is the one the independent evidence also favours
(southbridge's hybrid: 226× cheaper, within 0.5 pp; §5.2).

**What the gate sees is unchanged, which is the point.** For whatever Jev *can* supply,
`:Symptom` still has to be a declared subclass, cardinality still has to hold,
`horis:RulesOnly` still has no slot. **[inference]** Jev's type guarantee ("the answer is
a member of the declared set") is a *strict subset* of what the gate enforces: the gate's
type system carries units, ranges, cardinality, immutability, cross-property constraints
and provenance policy; a Jev question carries a label set or a ten-level rubric.

#### 6.2.4 What changes, item by item

**Latency and cost.** **[measured]** Independently reproduced: a ~430 ms floor
(priorbench), 76 ms median in the wild across n = 333 user reports (openchamber),
$0.0004/case on TypeSafe's dashboard, $0.042/MTok input with output counted but unbilled,
and user-reported medians of 7× faster / 30× cheaper against frontier LLMs.
**[inference]** For a triage-shaped workflow this converts the perception boundary from
the run's dominant cost into a rounding error, and it makes *retries* affordable in a way
they currently are not: at $0.0004 a call, `horis:retries` stops being a cost decision
and becomes purely a latency one.

**Determinism — and here the answer is measured, not open, and it is a clash.**
**[measured]** TypeSafe's homepage FAQ: *"Determinism means returning the same result for
an identical input. **This is less valuable than consistency.** We define consistency as
making similar decisions when the meaning stays similar, even if the wording changes.
Jev is designed for consistency."* ([typesafe.ai](https://typesafe.ai/)).

**[inference]** This is a direct collision with one of Horismos's four hard rules — "no
wall-clock, randomness, or iteration-order dependence anywhere in inference … Trace
equality is a valid test assertion" (CLAUDE.md). The collision is *containable*, because
Horismos's rule scopes to inference and the Bridge has always been outside it (which is
why CI runs on `MockBridge` with "zero API keys"). But two consequences follow that the
corpus should record:

1. **The mock stays mandatory.** Any hope that a cheap, fast, typed proposer would let
   `make test` exercise the full loop against a live model and still assert byte-identical
   traces is dead on the vendor's own statement. A `typesafe:` bridge changes nothing
   about the testing story.
2. **Replay is bounded the same way it already is.** A trace records what the proposer
   said, not a procedure for reproducing it. That is already true of every bridge; it is
   worth noting that the fastest, cheapest option on the market chose *not* to close it,
   and argued the choice. **[inference]** Horismos's counter-position is stronger than it
   looks: it does not need a deterministic proposer, because the decision — the part
   anyone audits — is deterministic given the admitted facts, and the admitted facts are
   in the trace.

**What the gate can demand as evidence.** **[inference]** Nothing changes about what the
gate *checks*, which is the correct answer — the gate has no back door. What changes is
what the deployment can *know about the proposer*. And here a documented detail matters:
**[measured]** Jev's `confidence` is *"a statistic computed from the probability
distribution"* whose formula TypeSafe does not publish, and the docs suggest computing
your own from the raw `probabilities` instead ([docs/confidence](https://docs.typesafe.ai/confidence)).
**[inference]** For Horismos that settles a design question: a bridge should carry the
**`probabilities` map**, from which a per-fact confidence can be derived by a *declared,
auditable* rule, and should not thread through an undisclosed vendor statistic into a
threshold the ontology author is supposed to be able to reason about. "You cannot audit a
threshold whose definition you cannot see" applies to Horismos exactly as it applies to
TypeSafe.

**What the trace and snapshot would have to carry.** **[measured — local repo]**
`Step::Perception` already carries `confidence`, `threshold`, `accepted`, `model` and
`latency_ms`, all `Option`-typed with `skip_serializing_if`
([`trace/schema.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/trace/schema.rs):151-167), so a Jev run is representable *today* with no schema
change. Doc 09's design rule 1 ("schema versioning is sacred") plus the `allow_incomplete`
precedent ("additive-optional under the versioning policy, not a reinterpretation of an
existing field") give the mechanism for anything new. **[inference]** Two fields would be
worth having and neither exists: **how the confidence was produced** (self-report /
logprob / calibrated head, with a named method) and, optionally, the **full distribution**
rather than its argmax. Both are additive-optional; both are OD-1. The snapshot binary
form ([`world/snapshot.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/world/snapshot.rs)) needs nothing — a Jev-proposed fact is an ordinary
`Provenance::Perception` fact.

**What the PRP-008 host SPI would need: nothing** (§6.2.1).

#### 6.2.5 The narration hole is already plugged, and the result is interesting

**[measured — local repo]** `execute_narration` falls back to the mechanical render on
*any* bridge error, by design (D7):

```rust
// crates/horismos-core/src/workflow/agent.rs:812-824
Err(e) if n.required => { return Err(WorkflowError::NarrationFailed { … }); }
// D7: non-fatal by default — the mechanical proof.render() fallback stands in
Err(_) => proof_text,
```

**[inference]** A `JevBridge` whose `narrate` returns
`BridgeError::Refused { reason: "jev generates no text" }` therefore works *today*,
unmodified: the narration step emits `proof.render()` and records it as that step's
output. And that yields an architectural possibility worth naming rather than treating as
a workaround: **a Horismos deployment with no generative model anywhere in it.**
Perception is a typed classifier, decision is symbolic, narration is a mechanical render
of the proof tree. For a regulated deployment that is not a degraded mode, it is the
preferred one — and it is a configuration Horismos can already describe honestly, because
doc 06 already states that the proof "remains the authoritative explanation; narration is
its presentation."

#### 6.2.6 What breaks

**[inference]** Five things, in descending order of seriousness.

1. **Numeric and string extraction, entirely** (§6.2.3). This is the one that decides the
   shape of any real integration, and it is not fixable by configuration.
2. **A new data-egress path into a vendor with a thin trust posture.** Doc 15 §4 governs
   "Provider APIs, credentials, and traces". **[measured]** TypeSafe's legal pages, read
   directly: **no SOC 2 and no HIPAA/BAA mention anywhere** in docs, privacy policy or
   the Master Customer Agreement; **no SLA** — the MCA (effective 2026-09-19) states
   "TYPESAFE DOES NOT WARRANT THAT CUSTOMER'S USE OF THE SERVICES WILL BE UNINTERRUPTED
   OR ERROR-FREE"; **US-hosted only**, "The Services are hosted in the United States",
   with no residency options; **no self-host** — the MCA scopes the service to a
   "TypeSafe-hosted" interface and API; zero data retention exists but is
   **enterprise-only, on request**. Training on customer input is contractually excluded,
   which is the one strong item. Rate limits "can change without notice".
   **[inference]** For the clinical scenarios this corpus is aimed at, that is a blocker
   today, and it is a deployment question rather than a design one. It is also exactly
   the posture one expects six days after emerging from stealth.
3. **`ProposalStep` does not execute.** The cleanest fit is to a slot that returns
   `WorkflowError::StepDeferred`. Any "Jev ranks the legal options" story is a *proposal
   about future work*, not something demonstrable on this build.
4. **A confidence-controlled boundary is an attack surface, and doc 15 does not name it.**
   **[measured]** Forced Deferral
   ([arXiv:2606.15308](https://arxiv.org/abs/2606.15308)) shows a universal adversarial
   trigger can depress a first-tier model's confidence to force escalation; and TypeSafe
   states Jev "does not treat [state] as hostile by default". **[inference]** In Horismos
   that is a denial-of-service on the human in the loop: depress confidence below
   `horis:confidence` and every case routes to `:EscalateToNurse`. Doc 15 §1 names the
   *opposite* risk — confident-sounding injected text clearing the threshold — and not
   this one. The gap exists today with any bridge; it becomes pressing when confidence is
   the product's headline feature. See OD-5.
5. **The calibration is not yet trustworthy enough to lean on.** **[measured]**
   priorbench: "Accuracy above threshold is flat from 0.50 to 0.95, then jumps to 100% at
   0.99. Gate at 0.99 or not at all." Lindfors: ECE 0.040 → 0.116 as questions became
   longer and more carefully qualified. **[inference]** An ontology author writing
   `horis:confidence 0.85` against a proposer whose accuracy-above-threshold is flat
   across 0.50–0.95 has written a threshold that does nothing. If Horismos adopts a
   calibrated proposer it must measure the calibration itself rather than inherit the
   claim — which is §7 move R2, and the same measurement it already owes for its existing
   bridges.

---

### 6.3 The auditability wedge

The task asks three things: is this Horismos's wedge; does the post-2026-05 regulatory
and clinical-AI literature back it; and has the ground shifted, given that sweep #2
already recorded the 2022 FDA CDS guidance as superseded and HTI-5 as proposing to
remove the AI model-card requirements outright.

#### 6.3.1 First, a distinction the whole question turns on

**[inference]** There are three different things called "explanation" and they have
different fates in the literature and the regulation. Keeping them apart is the
difference between a defensible claim and a marketing one.

| | What it is | Example | What it is worth |
|---|---|---|---|
| **Post-hoc explanation** | A second model or method, run after the decision, that produces a story about it | SHAP, LIME, saliency, an LLM asked "why did you say that?" | Contested. It is not guaranteed to be faithful to the computation it describes |
| **Structural auditability** | The surrounding control flow is inspectable, even though the model's contribution is not | Jev: which question was asked, over what state, against what threshold, and which branch the code took | Real, and underrated. But it stops at the model boundary |
| **Epistemic auditability (a proof)** | The reason *is* the computation. The decision was produced by a chain of declared rules over admitted facts, and the chain is emitted as a data structure | Horismos's proof tree (`reason/proof.rs`), rendered by `proof.render()` | The strongest of the three — and the only one where "why" and "how" are the same object. Faithful by construction; *not* thereby minimal, correct, or safe to put in front of a tired clinician (§6.3.2(v)) |

**[measured — local repo]** Horismos is in the third row by construction, not by
addition. Derived facts are produced by forward chaining over declared rules
(`reason/forward.rs`), each firing records "the per-condition proof recorded at fire
time", and suppressed and overridden firings are recorded too — "the trace never claims a
replaced value is still standing". Narration is explicitly *not* the explanation: doc 06
says "The proof remains the authoritative explanation; narration is its presentation.
Auditors read the proof; humans in the loop read the narration; they must agree because
one is generated from the other — and *only* from the other."

**[measured — primary, full text]** The best-known sceptical argument in clinical AI —
Ghassemi, Oakden-Rayner & Beam, "The false hope of current approaches to explainable AI in
health care" (*Lancet Digital Health*, 2021;3(11):e745–e750) — is **not** confined to the
first row, and an earlier draft of this report said it was. The body text attacks both
rows: "[t]he intuitive simplicity of inherently explainable models is appealing, but even
these explanations are hampered by the presence of unrecognised confounders", and
"increased transparency can hamper users' ability to detect sizable model errors and
correct for them, 'seemingly due to information overload', **even for clear-box or
inherently explainable models**." **[inference]** What the distinction above *does* defeat
is the paper's **fidelity** argument — "not only can the model be right or wrong, but so
can the explanation" — which is the argument a second, approximating artifact is
vulnerable to and a derivation is not. Its **human-factors** and **validation** arguments
survive against a proof tree untouched, and both are now backed by measurements in
clinicians. §6.3.2(v) decomposes the paper argument by argument and gives the evidence;
§10 records the correction. The short version, which should travel with any use of this
table: **a proof buys faithfulness. It does not buy validity, and it does not buy safe
human use.**

**[inference]** The same distinction cuts against Jev cleanly, and using the vendor's own
words rather than a critic's: *"System One models do not write replies, produce code, or
generate explanations of their reasoning."* That places it in row two. TypeSafe leans on
row two honestly — *"The most important thing for humans to review is the questions and
any threshold constants … defined in a single code file"* — and that is a genuine
improvement over parsing prose. It is not a reason.

#### 6.3.2 Second, what the post-2026-05 evidence actually says

A dedicated sub-sweep ran the regulatory and clinical-AI question to primary sources on
2026-09-21. Its single most useful structural finding is that **"auditability" is not one
market, it is three, and they are moving in different directions at the same time.** The
three line up almost exactly with the three rows of the table above. Its single most
*uncomfortable* finding is in (v), and it required correcting a paragraph this report had
already drafted.

##### (i) System-level transparency — model cards, source attributes — is losing, and it has an evidentiary obituary

Sweep #2 recorded HTI-5 as *proposed*. Checked again today: **[measured — primary,
reginfo.gov]** RIN 0955-AA09 is no longer at the proposal stage. The OIRA "Executive Order
Submissions Under Review" page, dated 2026-09-21, carries the row `AGENCY: HHS-ONC | RIN:
0955-AA09 | Status: Pending Review | STAGE: Final Rule | Economically Significant: Yes |
RECEIVED DATE: 07/28/2026`. The Unified Agenda's projected `Final Action 08/00/2026` has
slipped, and the Federal Register API returns **zero** 2026 ASTP/ONC rules. **[inference]**
The final rule cleared agency drafting, has been at OIRA about eight weeks, and Executive
Order 12866 review nominally runs 90 days — so publication is plausible in October–November
2026. The corpus's "no final rule as of 2026-09-02" remains literally true and is now
*misleading* if read as "stalled". It is imminent.

What it would remove is broader than the corpus recorded. **[measured — primary, the NPRM
preamble at 90 FR 60970]** ASTP proposes to strike **all** of §170.315(b)(11)(iv) (source
attributes), (v)(A)–(B) (their access and modification) and (vi) (intervention risk
management), *and* to remove and reserve the legacy CDS criterion §170.315(a)(9), whose
2012 requirements "included … the display of source attribute or bibliographic citation of
CDS". The preamble is explicit that (b)(11)(iv) covered both kinds of decision support:
"most clinical users lack the time or technical background to engage with source attribute
information **for either evidence-based DSIs or predictive DSIs**." **[inference]** If it
finalises as proposed, the United States will have **no certification requirement that a
decision-support tool cite its source at all** — not a model card, not a bibliographic
citation — and no replacement mechanism is proposed.

The rationale is the strongest single piece of evidence against the wedge found anywhere
in this dive, and it deserves quoting rather than paraphrasing. **[measured — primary]**:

> "Despite having access to source attribute information since the beginning of 2025 … we
> have no publicly available evidence indicating that **a single doctor, nurse, or
> administrator has accessed, recorded, or modified a single source attribute.** … We
> believe that the underlying assumption that source attribute information would be
> valuable to health care delivery organizations and health care professionals when
> deciding whether to implement or use a DSI … **was incorrect.**"

HTI-1 had projected up to **$3 billion** in ten-year savings from transparency-driven model
selection in the sepsis use case alone; ASTP now writes "the benefits articulated in the
HTI-1 Final Rule impact analysis are not being realized." **[measured — primary]** This is
a regulator reporting that it ran the experiment and the experiment failed, and it should
be read at full strength.

It should also be read for what it is about. **[measured — secondary; AHA comment letter
2026-02-27, TechTarget 2026-03-03 reporting 6,400+ comments]** The buy side fought it — AHA
asked that the DSI criteria "be retained in their current form", WEDI opposed, MGMA warned
removal would "leave practices to independently assess vendor AI tools and assume
inappropriate risk" — while the party ASTP cites in support, footnoted three times in the
preamble, is the EHR Association, i.e. the vendors. **[inference]** And the artifact ASTP
measured was a *static metadata panel sitting beside a product*, not a reason attached to a
decision. "Nobody clicked the model card" is a finding about artifact type and placement.
It is decisive evidence against shipping a model card. It is not, on its own, evidence
about whether a clinician engages with the sentence explaining the alert in front of them
— though (v) below supplies evidence that bears on exactly that, and it is not comfortable
either.

##### (ii) Per-decision reason-giving to the professional in the loop is *strengthening* at FDA

**[measured — primary, the 27-page PDF read directly]** FDA's revised **Clinical Decision
Support Software** final guidance carries the cover date **JANUARY 2026**, docket
FDA-2017-D-6569. Sweep #2 could only reach it through a law-firm analysis and marked the
date `[unverified]`; the document itself confirms the month. Its term profile is striking:
`artificial intelligence` **0**, `machine learning` **0**, `generative` **0**, `calibrat`
**0**, `uncertaint` **0** — against `independently review` **17**, `basis for` **19**,
`rely primarily` **11**, `time-critical` **16**. The single AI reference is a parenthetical
list of evidence sources: "(e.g., meta-analysis of clinical studies, expert panel,
statistical modeling, AI/ML techniques)". The corpus's recorded finding that the guidance
"remains silent on AI-enabled tools" is now confirmed **at token level**.

Criterion 4's operative language reads, on its face, like a specification for a proof tree.
Verbatim:

> "(d) The software output provides the HCP user with relevant patient-specific information
> and **other knowns/unknowns for consideration** (e.g., missing, corrupted, or unexpected
> input data values) that will enable the HCP to independently review the basis…
> In order to describe the basis … **regardless of the complexity of the software and
> whether or not it is proprietary**, the software or associated labeling should provide
> adequate background information about underlying sources in plain language …
> presented **in a manner that promotes usability and avoids information overload,
> including prioritizing the most decision-relevant information**…
> Beyond the input medical information relied upon, **being able to clearly understand how
> the logic was applied for the patient (e.g., matching of patient-level data to criteria
> in practice guidelines, comparison to a reference/normal database, development and use of
> a prioritized schema for recommendation generation) will aid an HCP in understanding the
> basis of the recommendations.**"

**[inference]** Read that last clause next to what Horismos already emits. "Matching of
patient-level data to criteria in practice guidelines" is forward chaining over declared
rules. "Development and use of a prioritized schema for recommendation generation" is
`horis:priority` and the `Suppressed`/`Overridden` firing outcomes. "Knowns/unknowns …
missing, corrupted, or unexpected input data values" is the gate's `Witness` and its
`domain_text()`. FDA's own compliant worked example (§V.B.1(d)) asks for "a description of
how the patient-specific information matches the criteria for treatment options in the
practice guidelines" and an output that "indicates whether any expected input medical
information from the medical record was missing" — which is, nearly line for line, a
rendered proof plus a gate witness. And the "avoids information overload, including
prioritizing the most decision-relevant information" clause is precisely the failure mode
ASTP documented in (i): FDA is asking for the opposite artifact.

Two structural qualifications keep this honest. **[measured — primary]** Criterion 4 still
excludes time-critical use: "FDA does not consider software functions intended for a
critical, time-sensitive task or decision to meet Criterion 4, because an HCP is unlikely
to have sufficient time to independently review the basis" — so the triage example, as the
founding claim on the [regulatory page](../validation/regulatory.md) already says, likely
fails it on timing regardless of how good the proof is. The enforcement-discretion example
is sharper than sweep #2 recorded: a cardiovascular-risk predictor qualifies, but "a
software function with the same functionality, but predicts risk of a cardiovascular event
**in the next 24 hours**, would not fall under this example. Such a software function is a
device." And **[inference]** Criterion 4 is the *non-device exclusion*: failing it does not
make a product unlawful, it makes it a regulated device requiring a premarket submission.
Reviewability is a **regulatory-pathway lever**, not a prohibition. That is a weaker claim
than "auditability is required", and a much more defensible one.

Then the genuinely new document, published after sweep #2 closed. **[measured — primary,
31-page PDF read directly]** FDA CDRH, **"Considerations for the Regulation of Generative
AI-Enabled Medical Devices: Discussion Paper and Request for Feedback"**, page current as
of **2026-08-18**, docket **FDA-2026-N-7874**, comments due **2026-10-19**, expressly "does
not represent draft or final guidance". Term counts across the full 31 pages:
`explain`/`explainab` **0**; `interpretab` **0**; `transparen` **1**; `calibrat` **2**;
`uncertaint` **9**; `traceab` **2**; `automation bias` **1**.

Taken alone that is the sharpest "the ground shifted" data point in the dive: FDA's newest
thinking on generative-AI devices does not use the word "explainability" once, while naming
calibration and uncertainty repeatedly. Calibration is proposed as a premarket benchmarking
element, verbatim:

> "**S.3. Calibration, uncertainty communication, and clinical deferral.** Whether the
> device accurately represents the confidence and limits of its outputs, expressing
> appropriate uncertainty when evidence is contested, evolving, or beyond its reliable
> knowledge. **Presenting uncertain, outdated, or contested information with false
> confidence is treated as a safety failure.**"

But the same paper promotes reviewability from a checkbox to a **risk driver**, verbatim:

> "CDRH is considering that although these functions yield non-directive information,
> **they may nonetheless be of higher risk because the user typically cannot independently
> evaluate the basis for the output (and therefore the function may migrate higher on the
> consequences axis).**"

— with footnote 13 tying it back: "This consideration seems consistent with the
independent-review criterion of the clinical decision support exclusion under section
520(o)(1)(E)." Discussion Question 1 asks whether "the traceability of the output (i.e., to
primary source materials)" should be a risk dimension at all. **[measured — primary]**

**[inference]** So FDA's position, stated precisely, is: **calibration is a safety floor;
inability to show the basis is a risk multiplier. They stack; they do not compete.** That
is the most consequential regulatory finding of this sub-sweep, and it is the correct frame
for §6.4's question about what Horismos should adopt — a calibrated proposer does not buy
you out of a reason, and a reason does not buy you out of calibration.

Two FDA corrections, since the [regulatory page](../validation/regulatory.md) is being
edited anyway. **[measured — primary, fda.gov]** The PCCP guidance that page cites as
"finalized December 2024" is dated **AUGUST 2025**, current as of 2025-08-18; the founding
text's reasoning is unaffected, the date is wrong, and it is corrected in the page's delta.
And the **"AI-Enabled Device Software Functions: Lifecycle Management and Marketing
Submission Recommendations"** guidance is **still a draft** ("Draft. Not for
implementation", content current as of 2025-01-07, docket FDA-2024-D-4488), sitting on
CDRH's FY2026 **B-list** ("as resources permit") **[measured — primary for the draft
status, secondary for the B-list]**. Its term profile is the mirror image of the GenAI
paper — `transparen` **47**, `model card` **40** — and it draws this report's exact
distinction in its own words: "some models can be explained using a simple decision tree
which is generally easy for a user to follow and understand the basis of a model's
recommendations. Other models use complex, deep neural networks, where it may not be
feasible to explain in a way that allows a user to completely understand the basis." It
also warns against explanation theatre: "It is also important to consider when additional
information may detract from understanding, rather than add to it."

##### (iii) Per-decision reason-giving to the *affected person* is holding — and it holds where it is old

The strongest legal authority for this project is not in healthcare regulation at all, and
it is not new AI regulation. It is a 2025 judgment interpreting a 2016 statute.

**[measured — primary, EUR-Lex CELEX 62022CJ0203, independently verified by the sub-sweep]**
CJEU, **C-203/22, *CK v Dun & Bradstreet Austria*, 27 February 2025.** Operative ruling
point 1 holds that under GDPR Art. 15(1)(h) the data subject may require the controller, as
"meaningful information about the logic involved", "to explain, by means of relevant
information and in a concise, transparent, intelligible and easily accessible form, **the
procedure and principles actually applied** in order to use, by automated means, the
personal data concerning that person with a view to obtaining a specific result, such as a
credit profile." And ¶ 59 — the sentence that decides the question this section is asking:

> "Those requirements **cannot be satisfied either by the mere communication of a complex
> mathematical formula, such as an algorithm, or by the detailed description of all the
> steps in automated decision-making**, since none of those would constitute a sufficiently
> concise and intelligible explanation."

¶ 57 recognises "a genuine right to an explanation as to the functioning of the mechanism";
¶ 61 says the subject must "understand which of his or her personal data have been used in
what way", and that "the complexity of the operations … [is] not … capable of relieving the
controller of the duty"; ¶ 62 blesses counterfactuals. On the facts, a score plus a
statement that socio-demographic data were "given equal weighting" was held
**insufficient**. Trade secrets do not license refusal; they go to the supervisory
authority or the court for balancing (ruling point 2). And under **C-634/21 *SCHUFA***
(7 December 2023) **[measured — primary]**, the score *itself* is the Art. 22 decision where
a third party "draws strongly" on it — so handing the number downstream does not escape the
article, it triggers it.

**[inference]** Read together those two judgments are the most directly on-point authority
in the world for a product whose output is a calibrated probability: a bare probability with
no justification fails GDPR outright. But note the *second* half of ¶ 59 carefully, because
it cuts at Horismos too: "the detailed description of all the steps in automated
decision-making" is **also** held insufficient. A raw proof tree dumped at a data subject
does not discharge the duty either. What discharges it is a *concise, intelligible* account
of the procedure and principles actually applied — which is exactly doc 06's split between
the proof (the authoritative object) and the narration (its presentation), and exactly why
that split must hold. This is the best external justification the corpus has found for a
design decision Horismos already made.

Three more sets of teeth, briefly. **[measured — primary]** US consumer credit: 12 CFR
§ 1002.9(b)(2) says in terms that "[s]tatements that the adverse action was based on the
creditor's internal standards or policies **or that the applicant … failed to achieve a
qualifying score on the creditor's credit scoring system are insufficient**" — a regulation
that names a calibrated score as a non-answer — with a private right of action under 15
U.S.C. § 1691e that survives any change in agency enforcement posture. (The CFPB's
supporting circulars, including 2022-03's "a creditor's lack of understanding of its own
methods is therefore not a cognizable defense against liability", were withdrawn 2025-05-12
**[measured — secondary]**; the statute and the regulation were not. Note the ceiling the
Official Interpretations set: "[a] creditor need not describe how or why a factor adversely
affected an applicant" — the duty is to *name the principal factors actually scored*, not to
narrate causation.) **[measured — primary, leginfo.ca.gov]** California SB 1120, in force
since 2025-01-01, requires a utilization-review tool to base its determination on "[a]n
enrollee's medical or other clinical history" and "[i]ndividual clinical circumstances as
presented by the requesting provider", provides that it "does not supplant health care
provider decisionmaking", bars determinations based "solely on a group dataset", and
reserves medical necessity to a licensed clinician — **[inference]** a direct statutory hit
on calibrated-probability-only decisioning, since a population risk model's output *is* a
group-dataset inference. Copycats are spreading (Maryland 2025-10-01; 2026 enactments in
several states, one of which — Washington SB 5395 — provides that "[a]rtificial intelligence
shall not be the sole means used to deny, delay, or modify health care services"), and
CMS's WISeR model requires that "non-affirmation decisions require the review of a licensed
clinician prior to issuance" **[measured — secondary; the state count itself is
[unverified], snippet-level]**.

And the EU AI Act, whose final shape sweep #2 did not have. **[measured — primary, OJ text
extracted and searched]** **Regulation (EU) 2026/1744** of 8 July 2026 (the "Digital Omnibus
on AI") entered into force 2026-07-27 and replaced Art. 113's timetable: Chapter III
Sections 1–3 now apply from **2 December 2027** (Annex III high-risk) and **2 August 2028**
(Annex I). Searching the full enacting terms, "Article 86" and "Article 13" appear **zero**
times. **Arts. 13, 14, 26 and 86 are untouched in substance** — the transparency and
human-oversight duties are *deferred*, and Art. 86, the right to explanation, sits in
Chapter IX, which was not deferred at all.

| AI Act provision | Chapter | Applies from |
|---|---|---|
| Art. 13 — transparency to deployers | III §2 | 2027-12-02 (Annex III) / 2028-08-02 (Annex I) |
| Art. 14 — human oversight | III §2 | same |
| Art. 26 — deployer obligations | III §3 | same |
| **Art. 86 — right to explanation of individual decision-making** | **IX §4** | **not deferred — 2026-08-02** |

Art. 86(1) entitles an affected person to "clear and meaningful explanations of the role of
the AI system in the decision-making procedure and **the main elements of the decision
taken**" **[measured — primary]**, and Annex III point 5(d) expressly lists "emergency
healthcare patient triage systems" **[measured — secondary, Annex III mirror]** — i.e. the
corpus's own running example. **[inference, and explicitly contestable]** Whether Art. 86
*bites* in the interim is genuinely unsettled: it is keyed to systems "listed in Annex III",
while the classification article (Art. 6(2)) is itself deferred to December 2027. The
sub-sweep found **no practitioner source addressing the question**, which is itself a
signal. Treat "enforceable now" as arguable, not settled. **[unverified]** Note also that an
MDR-regulated clinical device is Annex I, deferred to 2028-08-02 and **outside Art. 86
entirely** — the right to explanation does not reach medical-device AI. Arts. 13 and 14,
when they arrive, address the *deployer* through the instructions for use and hedge the
explainability hook twice with "where applicable"; **[inference]** a calibrated score with
documented accuracy metrics and limits plausibly discharges them, since there is no
per-output duty anywhere in Art. 13 or 14. The one strain is Art. 14(4)(d)'s requirement
that the overseer be able "to decide, **in any particular situation**, … to disregard,
override or reverse the output" — a bare probability gives no per-case handle on *which*
situations are the wrong ones. That is an argument, not a mandate.

##### (iv) And where a calibrated number is genuinely enough

Recording the negative space matters as much as the positive, because it bounds the claim.
**[measured — primary for SR 26-2, secondary for the rest]** US bank model-risk governance
no longer even reaches this class of system: **Fed SR 26-2** and **OCC Bulletin 2026-13**
(both 2026-04-17) supersede SR 11-7 and OCC 2011-12, and OCC 2026-13 states "[g]enerative AI
and agentic AI models are novel and rapidly evolving. As such, they are not within the scope
of this guidance"; the duty ran to the supervisor, never the customer, and there was never a
private right of action. **NYC Local Law 144** is an *audit* regime — independent bias
audit, public summary, ten-business-day notice of the qualifications used — with **zero**
individual-explanation duty: a calibrated black box plus an audit report fully complies.
**Texas TRAIGA**, **Illinois HB 3773**, **Utah AIPA** and California's **FEHA
automated-decision-system regulations** are anti-discrimination, notice-of-use and
recordkeeping regimes; none requires a reason. The **NAIC Model Bulletin** treats
"[t]ransparency and explainability of outcomes to the impacted consumer" as a risk factor to
weigh, not a disclosure duty. And **Colorado's** SB 24-205 duty to disclose "the principal
reason or reasons for the consequential decision" was delayed, then enjoined (*X.AI LLC v.
Weiser*, D. Colo., stay entered 2026-04-27, with DOJ intervening), then legislated away by
**SB 26-189** (signed 2026-05-14, effective 2027-01-01), which "repeals and reenacts those
provisions with new requirements" — a plain-language description of the decision and the
role the tool played, and human review "to the extent commercially reasonable". "Principal
reason or reasons" does not survive. **[measured — primary for the Colorado bill status;
secondary for the replacement's contents]**

##### (v) The clinical human-factors literature gives Horismos much less than §6.3.1 first claimed — a correction

An earlier draft of §6.3.1 asserted that Ghassemi, Oakden-Rayner & Beam is "an argument
against the first row" and that "any use of that paper against Horismos is a category
error". **That was wrong, and it was wrong because the draft was written from the abstract.**
The sub-sweep obtained the full body text (the paper is CC-BY; thelancet.com blocks
automated access, so it took a headed browser session) and the body is explicit
**[measured — primary, full text]**:

> "Attempts to produce human-comprehensible explanations … have typically been divided into
> two categories: **inherent explainability and post-hoc explainability.** … **The intuitive
> simplicity of inherently explainable models is appealing, but even these explanations are
> hampered by the presence of unrecognised confounders.** Work in the human–computer
> interaction community has identified that **increased transparency can hamper users'
> ability to detect sizable model errors and correct for them, 'seemingly due to information
> overload', even for clear-box or inherently explainable models.**"

So the paper attacks both rows, and it names prototype models — Rudin's own family — as
"generally considered to be a form of inherent explainability" that "still requires human
interpretation". **Ghassemi cannot be cited as endorsing interpretable-by-design systems.**
The corrected §6.3.1 above says so, and the error is recorded in §10.

Decomposing the paper into its distinct arguments is the honest way to use it, because they
have genuinely different fates against a derivation **[inference, on measured premises]**:

| # | Ghassemi's argument | Does it survive against a proof tree? |
|---|---|---|
| 1 | **Fidelity** — "not only can the model be right or wrong, but so can the explanation" | **No.** A derivation *is* the computation; there is no second artifact to be wrong. Rudin's positive form applies directly: inherently interpretable models "provide their own explanations, which are faithful to what the model actually computes" |
| 2 | **Incompleteness** — a saliency map "does not reveal exactly what it was in that area that the model considered useful" | **No.** A rule chain states which facts, which rule, which conclusion |
| 3 | **Descriptive vs normative** (after Selbst & Barocas) — techniques give "descriptive accounts of how the model behaved", while what is wanted is "whether that behaviour was justified", and intuition "serves as the unacknowledged bridge" | **Yes, but transformed.** The descriptive account becomes exact, so the clinician stops guessing what was used. The normative question remains — but it now attaches to an explicit, versioned, pre-registered rule that can be validated **once, offline**, rather than being re-adjudicated by intuition at every bedside |
| 4 | **Human factors** — transparency "can hamper users' ability to detect sizable model errors … even for clear-box or inherently explainable models" | **Yes, in full — and it is now measured in clinicians** (below) |
| 5 | **Validation is the only justification** — "the only effective way to justify the decisions of AI systems is thorough, careful, meticulous safety and validation efforts" | **Yes, in full.** A faithful derivation is guaranteed faithful, not correct |

Arguments 1 and 2 are the ones a proof defeats, and defeating them is not nothing — it is
the whole fidelity literature. Arguments 4 and 5 are untouched, and 4 now has direct
measurement behind it:

- **[measured — primary abstract]** Gomez C. et al., "The explainable AI dilemma under
  knowledge imbalance in specialist AI for glaucoma referrals in primary care", *npj Digital
  Medicine* 2025;8:706 (PMID 41266539) — the one study that isolates **intrinsic** from
  **post-hoc** explanation in clinicians. 87 optometrists. Human–AI teams beat humans alone
  (60% vs 51%), but "explanations did not enhance performance. Instead, they introduced
  uncertainty about when to trust AI. **Post-hoc explanations increased over-reliance on
  incorrect AI recommendations, and both explanation types contributed to anchoring bias**".
  And the sting: "human–AI teams still underperformed the AI alone (**80% accuracy**)".
  **[inference]** Post-hoc came off worse, which partially vindicates the distinction. But
  intrinsic explanations still produced anchoring bias. The distinction is real and it is
  *not* a free pass.
- **[measured — primary abstract]** Nicolson A. et al., *npj Digital Medicine* 2025;8:658 —
  prototype-based (ante-hoc) explanations with 10 sonographers: model predictions cut MAE
  from 23.5 to 15.7 days, explanations gave a further **non-significant** reduction to 14.3,
  "with some performing worse with explanations than without", and no significant effect on
  trust or reliance.
- **[measured — primary]** Jabbour S. et al., *JAMA* 2023;330(23):2275–2284 (n = 457
  clinicians), the canonical automation-bias result: baseline accuracy 73.0%, +4.4 pp with a
  standard model and explanations, **−11.3 pp** with a biased model and **−9.1 pp** with a
  biased model *plus* explanations — a 2.3 pp rescue, **not significant** (95% CI −2.7 to
  7.2). **[inference]** The explanation tested was a saliency map over a chest radiograph
  against an injected spurious *input* dependency, which a heatmap structurally cannot
  surface; so it is strong evidence against post-hoc explanation and weak evidence about
  reason-giving. But read the numbers the other way too: explanations bought +1.5 pp when the
  model was right and an indistinguishable +2.3 pp when it was systematically biased. **They
  helped slightly more when help was least needed.**
- **[measured — primary]** Bansal G. et al., CHI 2021: "explanations increased the chance
  that humans will accept the AI's recommendation, **regardless of its correctness**", and a
  **confidence-only** baseline matched explanations on all three datasets (p = .24/.22/.64).
  Buçinca Z., Malaya M. & Gajos K., CSCW 2021 (N = 199): on trials where the AI was wrong,
  the **no-AI control beat both explainable conditions**; and "people assigned the **least
  favorable subjective ratings to the designs that reduced the overreliance the most**".
  Nagendran M. et al., *npj Digital Medicine* 2024;7:202: eye-tracking on 19 ICU physicians
  found "no appreciably higher level of attention placed onto any of the four types of
  explanation during unsafe AI scenarios (i.e. XAI did not appear to 'rescue'
  decision-makers)".
- **[inference]** Buçinca's rating inversion has a direct methodological consequence for this
  project: **any evaluation of a proof-rendering UI based on clinician satisfaction is
  measuring the wrong thing, and probably measuring it in the wrong direction.**

There is one positive mechanism result, and it is worth more than its size suggests.
**[measured — primary, PMC full text]** Han Y. et al., "Eye-Tracking Evidence That Verifiable
Explanations Support Visual Evidence Checking in AI-Assisted Chest Radiograph
Interpretation", *J Eye Mov Res* 2026;19(3):55, defines a "verifiable explanation"
functionally as "brief verification cues linked to the current AI recommendation",
explicitly **not** a heatmap, saliency map, confidence score or exemplar — "an interface
affordance that supports evidence checking, rather than … a repackaging of the AI output
itself". On **incorrect-AI trials**, AI↔image gaze switches rose from **6.04 to 8.52** per
trial (β = 0.311, p < 0.001; interaction β = −0.155, p = 0.004) and image dwell from 4.26 s
to 5.12 s. **[inference]** This is the only result the sub-sweep found where an
explanation-like artifact measurably increased scrutiny *specifically when the AI was wrong*
— and it worked because it was a pointer back to checkable primary evidence rather than a
rationalisation. That is structurally what a derivation over declared, gate-admitted facts
is. It is one small study, and it is the shape of the experiment §7's move **R9** proposes.

Finally, the sharpest *technical* objection to "the proof is the explanation", which is not
a human-factors argument at all. **[measured — primary abstract]** Marques-Silva J. &
Ignatiev A., "No silver bullet: interpretable ML models must be explained", *Frontiers in
Artificial Intelligence* 2023;6:1128212: a decision-tree path or rule chain is faithful, but
it is **not a minimal sufficient reason**, and answers obtained "by human inspection of the
interpretable ML model representation" "can exhibit **arbitrary redundancy**". The proper
object, on this view, is an *abductive explanation* — a subset-minimal set of features
sufficient for the prediction, computed with formal guarantees. **[inference]** A Horismos
proof tree is exactly the object they are criticising: it records every rule that fired and
every condition that matched, including conditions that were not load-bearing. Whether
Horismos should compute subset-minimal sufficient reasons rather than emit the whole
derivation is a real design question with a real cost, and it is not one this report can
settle — see **OD-8**.

#### 6.3.3 So: is it Horismos's wedge, and has the ground shifted?

**Yes to both, and the two answers are compatible — but the wedge is narrower, differently
shaped, and aimed at a different buyer than "auditability sells".**

**[inference]** The ground shifted decisively along **one** axis: *mandated system-level
transparency* is being dismantled, with an unusually damning empirical finding attached. If
the wedge had been "we ship a better model card", it would be gone. It was not, and the
distinction in §6.3.1 is what saves it: a model card is a **row-two** artifact — metadata
about the system, adjacent to the decision. Horismos's claim is **row three** — the reason
*is* the computation, attached to the individual decision. Every duty found in (ii) and
(iii) above is a per-decision duty, and every one of them is either holding or
strengthening.

The pattern in the evidence is sharper than "some things went up and some went down", and it
is worth stating as a rule, because it predicts the next two years better than a sentiment
reading does:

> **[inference]** Every reason-giving duty that depended on *new AI-specific regulation or
> agency guidance* was withdrawn, enjoined, deferred or repealed in 2025–26 — HTI-5, the
> CFPB circulars, Colorado, SR 11-7, AI Act Chapter III. Every reason-giving duty that sits
> in an *older statute with private or judicial enforcement* survived untouched — GDPR
> Art. 15(1)(h) as read in C-203/22, ECOA § 1691e and 12 CFR § 1002.9(b)(2), FDA's § 520(o)
> device/non-device line, California HSC § 1367.01(k). The variable is not subject matter.
> It is **whether the duty needs an agency to want to enforce it.**

And the reframing the evidence forces, which is the practically useful output of this
section:

> **[inference]** The defensible form of the claim is **not** "buyers want auditability". It
> is **"non-reviewability is a cost multiplier."** It pushes a product from non-device to
> device at FDA; up the consequences axis in FDA's August 2026 generative-AI framework; into
> Art. 86 exposure in the EU for exactly the Annex III categories this corpus uses as
> examples; and into per-se insufficiency under 12 CFR § 1002.9(b)(2) and California
> § 1367.01(k). A proof tree is not a feature a clinician will rank in a survey. It is what
> keeps a product out of a 510(k), out of an Art. 86 complaint and out of an ECOA class
> action. The audience for the wedge is **general counsel and regulatory affairs**, not the
> physician at the bedside.

That last sentence is load-bearing, because the physician evidence does not support the
other framing. **[measured — primary, the report PDF read directly]** The AMA's 2026
Physician Survey on Augmented Intelligence (n = **1,692** US physicians) finds the top
adoption factors are **validation of safety and efficacy (88%)** and **data privacy (86%)**;
the top-ranked regulatory priority (n = 1,419, 1 = most important) is **clear liability
frameworks at 3.0**, ranked #1 by 31%, ahead of post-market surveillance (3.4) and FDA
oversight (3.6). The words "explain" and "black box" appear **zero times in the report** —
explainability is not even an enumerated option. And **[measured — primary, ama-assn.org,
2025-06-11]** the AMA's own transparency policy, while calling for explainable clinical AI,
states that "explainability should not be used as a substitute for other means of
establishing safety and efficacy of AI tools, such as randomized clinical trials."

##### The falsifiers, ordered by likelihood

**[inference]** In the corpus's convention of naming the falsifier rather than only the
finding:

1. **Counterfactual sufficiency.** C-203/22 ¶ 62 blesses counterfactual explanations as one
   acceptable form. A calibrated discriminative model can emit per-feature counterfactual
   sensitivities with no interpretability whatsoever. **If a court reads ¶ 62's option as
   sufficient on its own, the GDPR leg of the wedge closes, and a typed-decision model could
   ship that feature in a week.** Most likely falsifier; watch hardest.
2. **A Gomez-style replication showing no advantage for intrinsic over post-hoc.** Gomez
   already found anchoring bias in both arms; a replication showing parity would remove the
   last measured human-factors support for the distinction this whole section rests on.
3. **FDA choosing calibration over reviewability.** Docket FDA-2026-N-7874 closes
   2026-10-19. If FDA's response drops "traceability of the output" and the "independently
   evaluate the basis" consequences-axis language and keeps only S.3 calibration, that is
   FDA stating a preference. Concrete date, concrete document — a later sweep should read it.
4. **Formal-XAI minimality becoming an expectation.** If Marques-Silva's abductive-explanation
   standard enters a regulation or a standards document, "here is the derivation" stops being
   sufficient and Horismos owes subset-minimal sufficient reasons (OD-8).
5. **A second Digital Omnibus reaching Art. 86**, or a Commission/EDPB instrument reading
   "the main elements of the decision taken" down to notice-of-use.
6. **HTI-5 finalising with ASTP's reasoning generalised.** The removals themselves are only a
   partial falsifier (they are row-one). If the final rule imports the "no evidence anyone
   used it" reasoning into *per-decision* explanation, that is a strong one.

##### The discipline this section owes in return

**[measured — primary, the 220-page PDF read directly]** The most authoritative
general-purpose assessment available, the **International AI Safety Report 2026** (February
2026, chaired by Yoshua Bengio, 100+ authors, panel nominated by 30+ countries,
[arXiv:2602.21012](https://arxiv.org/abs/2602.21012)), is in this report's favour on the
fidelity distinction and against it on complacency. In its favour: "'[i]nterpretability'
techniques that aim to explain models' internal workings require major simplifying
assumptions and can be misleading if used incorrectly", with a citation list that is the
interpretability-illusion canon, and, on generated rationales, Arcuschin et al.,
*Chain-of-Thought Reasoning in the Wild Is Not Always Faithful* — corroborated by Turpin et
al. (NeurIPS 2023: chain-of-thought explanations "systematically misrepresent the true reason
for a model's prediction", accuracy dropping "by as much as 36%") and Anthropic's own April
2025 measurement of hint verbalisation at 25% (Claude 3.7 Sonnet) and 39% (DeepSeek R1)
**[measured — primary]**. Against it, on formal methods specifically: "these proofs assume
that the testing context matches the deployment context, and are untested against many types
of adversaries. They also cannot currently be scaled to large models. Overall, there is
significant debate among experts over the promise of interpretability and formal verification
methods," and "model interpretability and formal verification techniques have seen little use
to date in production systems."

**[inference]** Horismos's verification story is not the kind that report is criticising — it
verifies an authored rule set, not a learned model, and [doc 10](https://github.com/billaram/horismos/blob/adfc27a/docs/framework/10-verification.md)
already states the limit in the same terms the report uses. But the underlying caution applies
without modification, and the [validation ladder](../validation/00-ladder.md) has already
written it down: **a perfect derivation from a wrong rule is a perfectly auditable wrong
answer.** The 2026 academic work closest to Horismos concedes the same thing structurally —
arXiv:2603.10027's governance framework for deterministic rule-based CDS validates "behavioral
alignment with specified rules **rather than clinical effectiveness, predictive accuracy, or
outcome optimization**" and states plainly "[n]o empirical clinical validation is claimed"
**[measured — primary abstract]**. And the empirical warning is concrete and current: the Epic
Sepsis Model scored **AUROC 0.63 (95% CI 0.62–0.64)** in external validation against a
vendor-reported 0.76–0.83 (Wong et al., *JAMA Intern Med* 2021), and its **v2**, evaluated
prospectively across four health systems and 227,091 encounters, scored **0.82 to 0.92
depending on the site** with **PPV 0.13 to 0.26** (Wong et al., *JAMA Network Open* 2026)
**[measured — primary abstracts]**. A number carries no evidence of its own validity; neither
does a proof.

##### The one place the evidence runs unambiguously Horismos's way

**[measured — primary abstract]** Gorenshtein A., Omar M., Barash Y., Nadkarni G. N. & Klang
E., "Neuro-symbolic LLM Integration in Clinical Medicine: A Systematic Review and Taxonomy"
(Research Square rs.3.rs-8902022, 2026-02-18, PMID 41756413) screened 3,166 records to 21
studies and sorted the integration patterns "by **increasing symbolic authority**: structured
output, rule-guided generation, knowledge retrieval, and iterative validation (**symbolic veto
or regeneration**)". Improvements "ranged from +3.1% to +125.6% (median +21.9%) and
**increased with symbolic authority (median gains: 9%, 16%, 26%, 40%)**".

**[inference]** That is close to a direct empirical statement of Horismos's first principle —
the more authority the symbolic layer has over the neural layer's output, the better the
system performs — measured across a published body of clinical work. It deserves to be in the
corpus, and it deserves its caveat stated in the same breath: the review rates **all 21 studies
at high risk of bias**, only 9 of 21 report external validation, and the highest-authority
pattern carries "2–88 s latency and up to 100× cost increases". Two individual results in the
same direction: Prenosil et al., *Communications Medicine* 2025;5:491 (206 PET/CT reports,
5,356 data points; neuro-symbolic F1 1.00/1.00 against GPT-4's 0.63/0.95, and it "delivers
always an auditable chain of reasoning") and Loaiza-Bonilla et al., *ESMO Real World Data and
Digital Oncology* 2026;12:100706 (**prospective**, 3,804 consecutive patients, dual-oncologist
gold standard κ = 0.92; neuro-symbolic F1 **0.82** vs GPT-4 zero-shot 0.47 and
chain-of-thought 0.67, with "ontology-grounded, deterministic eligibility reasoning … while
preserving clinician oversight") — both **[measured — primary abstracts]**, and both with
authors employed by or founding the commercialising vendor, which is worth saying out loud.

Finally, the observation that closes the loop back to the subject of this report. **[measured
— primary, vendor]** TypeSafe's own launch post makes **no interpretability or audit claim at
all**. It claims typed outputs and calibrated probabilities. The wedge described here is not
one TypeSafe is contesting — consistent with §6.1's finding that this is a complement rather
than a competitor, and worth remembering before sharpening an argument against a claim the
other party never made.


### 6.4 Q-D4 — calibration vs proof: what, if anything, Horismos should adopt

This is the question with the largest yield, and the answer is not the obvious one. The
obvious answer would be "Horismos's gate is boolean, Jev's output is probabilistic,
therefore they are philosophically opposed." That is wrong on the facts. Horismos
already has a probability in a load-bearing position, it is already the weakest link in
the perception boundary, and the literature this dive surveyed says precisely what is
wrong with it.

#### 6.4.1 Horismos's perception boundary already *is* a selective-prediction system

**[measured — local repo]** The contract is: one Bridge call; `accepted =
result.confidence >= p.confidence` ([`workflow/agent.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/agent.rs):492); on `false`, follow
`horis:fallback`, "typically to a human (`WaitStep`)" (doc 07). Doc 06 states the rule
as "The LLM never guesses."

**[measured]** That is, exactly, **Chow's reject option**: C. K. Chow, "On Optimum
Recognition Error and Reject Tradeoff," *IEEE Trans. Inf. Theory* IT-16(1):41–46 (1970)
— the Bayes rejection rule and the error–reject tradeoff. The modern deep-learning line
is Geifman & El-Yaniv, "Selective Classification for Deep Neural Networks" (NeurIPS
2017), which constructs a selective classifier so that "the user sets a desired risk
level and the classifier rejects as needed to guarantee it with high probability", and
Mozannar & Sontag, "Consistent Estimators for Learning to Defer to an Expert" (ICML
2020, PMLR v119) for the learning-to-defer formulation. **[unverified — these three
were confirmed bibliographically by sub-agent B from search results; the primary PDFs
were not opened.]**

**[inference]** Horismos has therefore been shipping a selective-prediction system
without naming it, and the corpus has never cited the 56-year-old literature that owns
the mechanism. Naming it is free and it buys something concrete: that literature says
what the *threshold* is for. `horis:confidence 0.85` in
[`examples/triage/triage.ttl`](https://github.com/billaram/horismos/blob/adfc27a/examples/triage/triage.ttl):139 is a number an ontology author picked. In Chow's
framing a threshold is chosen to hit a *target risk*, and there is a defined tradeoff
curve between the risk you accept and the fraction you reject. Horismos publishes
neither the curve nor the target.

#### 6.4.2 The number being thresholded is unmeasured, and the field says that matters

**[measured — local repo]** `PerceiveResult::confidence` is "the `assert_facts`
schema's self-reported `confidence` field, normalized by the `Bridge` implementation"
([`bridge/mod.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/bridge/mod.rs):64-70); the only real Bridge reads it straight out of the tool
call ([`openrouter.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-bridges/src/openrouter.rs):324-328); doc 08 says logprob-derived confidence "is a future
per-backend enhancement."

**[measured]** The GPT-4 Technical Report's Figure 8 caption states, verbatim: "The
post-training hurts calibration significantly", with body text "the pre-trained model is
highly calibrated … However, after the post-training process, the calibration is
reduced" ([arXiv:2303.08774](https://arxiv.org/html/2303.08774v4), §5). Note carefully —
because it is widely misquoted — that **the report gives no numeric ECE before or
after**; the famous result is a figure, not a number.

**[measured]** But the picture in 2026 is more favourable to LLMs than that 2023 figure
suggests, and the report should say so rather than lean on a convenient result:
Tian et al., "Just Ask for Calibration" (EMNLP 2023, pp. 5433–5442) find that for
RLHF'd models the *verbalized* confidence is better calibrated than the model's own
conditional probabilities **[unverified — bibliographic]**; and ConfidenceBench
([arXiv:2607.20526](https://arxiv.org/abs/2607.20526), 10 Jul 2026) measures Brier 0.103
for the best frontier models on prompted verbalized confidence against a
calibrated-random baseline of 0.1875, while finding that "accuracy and calibration
diverge substantially across model families: the most accurate model is not the
best-calibrated," with several models *below* the random baseline.

**[inference]** Three consequences for Horismos, in order of importance:

1. **Self-reported confidence is not worthless, but its quality is model-specific and
   unknown to the deployment.** ConfidenceBench's own finding — the best-calibrated
   model is not the most accurate one, and several models are worse than random — means
   a deployment that swaps `claude:` for a cheaper `claude:` model (which doc 08
   explicitly permits as "a config change") may silently move its acceptance boundary.
   Nothing in the trace records that. `Bridge::id()` records *which* model ran; nothing
   records *how well calibrated* it was.
2. **The comparison is protocol-fragile, so any measurement Horismos makes must fix its
   protocol.** Kim & Kang, "Same Answer, Different Confidence: Protocol Sensitivity in
   LLM Confidence Calibration" ([arXiv:2605.27752](https://arxiv.org/abs/2605.27752), 26
   May 2026) audit twelve operational variants and find the protocol "changes which
   signal performs better … in 4 of 12 settings under ECE and 9 of 12 under AUROC" and
   "leave[s] the sign of the ECE comparison undetermined in 6 of 12 settings"; five of
   twelve audited studies never state the choice. **[measured]**
3. **This is a defect in something already shipped, not a missing feature.** That
   distinction decides the §7 ranking: fixing it is ordinary engineering under an
   existing contract, whereas adding a probabilistic decision path would be a change to
   the design's shape.

#### 6.4.2a Abstention is not free, and the guarantee is not per-case: two results that bound R2

The regulatory sub-sweep turned up two measured results that bear directly on the
recommendation this report is about to make, and both of them make it narrower. They are
recorded here rather than buried in a caveat, because either one would embarrass a
deployment that adopted R2 without reading them.

**First: abstention has an asymmetric cost, and it has now been measured in clinicians.**
**[measured — primary abstract]** Jabbour S. et al., "Selective Prediction Reduces the
Negative Effects of Automation Bias Overall but Increases False Negatives"
([arXiv:2508.07617](https://arxiv.org/abs/2508.07617), v2, 2026-08-11), 259 clinicians, is
the follow-up to the *JAMA* 2023 study cited in §6.3.2(v). Its finding, verbatim:

> "clinician accuracy declined when shown inaccurate AI predictions (**66%** … vs **56%**),
> but recovered under selective prediction (**64%**). However … **when informed the AI
> abstains, clinicians underdiagnose (18% increase in missed diagnoses) and undertreat (35%
> increase in missed treatments) compared to no AI input at all.**"

**[inference]** Read what that says about Horismos's design, not about Jev's. The
perception boundary's failure path is, verbatim from doc 07, `horis:fallback` "typically to
a human (`WaitStep`)". §6.4.1 establishes that this is a reject option, which is the
correct mechanism. This result says the reject option is not a null action: *telling a
clinician the machine declined to answer measurably biases them toward inaction*, and the
bias is larger on treatment (35%) than on diagnosis (18%). Aggregate accuracy recovers
while the error profile rotates into false negatives — which in the triage example is
exactly the direction that kills people.

Two consequences, neither of which changes the mechanism:

1. **The fallback branch is a clinical design surface, not a safety valve.** What the human
   is *told* on abstention matters as much as the fact of abstention. An ontology that
   routes to `:EscalateToNurse` with a message reading "the model was not confident" is
   materially different from one whose message states which extraction was uncertain and
   what the standing facts already are. Horismos can emit the second — the gate witness and
   the partial proof exist — and nothing in the vocabulary currently encourages it.
2. **Raising `horis:confidence` is not a conservative move.** The intuition that a higher
   threshold is "safer" is wrong in the direction that matters: it buys fewer wrong
   accepted extractions at the cost of more abstentions, and abstentions are now measured
   to carry their own harm. That is the error–reject tradeoff of §6.4.1 with a *clinical*
   cost on both axes rather than an abstract one, and it is precisely the argument for
   choosing the number against a declared target rather than by hand.

**Python parallel.** The Rust is a boolean branch on a float; the shape in Python is
familiar, and so is the trap:

```python
# what the mechanism is
if result.confidence >= step.confidence_threshold:
    world.assert_(result.instances, provenance=Provenance.PERCEPTION)
else:
    raise Escalate(step.fallback)          # <- NOT a no-op. It has its own measured cost.
```

An engineer reading that reaches for `except Escalate: pass` and a higher threshold. The
measured result says the `except` branch is where the harm moved to.

**Second: a conformal guarantee is marginal, not conditional — it does not hold per case or
per subgroup.** **[measured — primary, full PDF]** Mehrtens, Bucher & Brinker (DKFZ),
[arXiv:2506.18162](https://arxiv.org/abs/2506.18162) (2025-06-22), state it flatly:
"conformal predictions are unreliable under distributional shifts … [they] should not be
used for selecting predictions to improve accuracy and are **not reliable for subsets of the
data**." The mechanism, verbatim: "**Marginal coverage is a property over the distribution
and does not give guarantees for singular prediction sets**" — with a worked example in
which a 90% marginal guarantee delivers 92% coverage to white patients and **62% to black
patients**.

**[inference]** This is the single most important qualification on §7's move **R2**. Split
conformal turns `horis:confidence 0.85` from an author's constant into a checkable
statement — but the statement it produces is *"across the calibration distribution, the
error rate is at most ε"*, and it is **not** *"for this patient, the extraction is right
with probability 1 − ε"*. An overlay that published the first sentence while a deployment
read it as the second would have manufactured exactly the false assurance doc 06's
"confidence is metadata, not truth" rule exists to prevent. Three practical riders, all of
which belong in OD-3 rather than in a design doc today: calibrate per deployment, not once
at the vendor; report coverage by clinically meaningful subgroup, not only in aggregate;
and state the calibration distribution in the same artifact as the number, because a
conformal guarantee against a distribution the deployment does not match is not a guarantee
at all.

**And a monitoring corollary, from the same sub-sweep.** **[measured — primary abstract]**
Davis S. E. et al., "Calibration drift in regression and machine learning models for acute
kidney injury", *JAMIA* 2017;24(6):1052–61, followed seven models over nine years and 2,003
VA admissions: "**[d]iscrimination was maintained for all models. Calibration declined as
all models increasingly overpredicted risk.**" **[inference]** AUROC held steady for nine
years while calibration rotted underneath it — so a monitoring regime built on
discrimination would never have fired. If Horismos publishes a reliability diagram under
R1, the thing that must be *re-measured on a schedule* is the calibration, not the accuracy.
The same paper's recommendation — that updating be "tailored … rather than be limited to
regularly scheduled annual or biannual intervals" — is the drift-detection framing, and it
is one more instrument the [validation ladder](../validation/00-ladder.md) does not yet
carry.

#### 6.4.3 Calibration is a diagnostic; the auditable object is something else

**[inference — this is the load-bearing judgment of the section]** A calibrated
probability is a *marginal, average-case* statement: among all the cases scored 0.30,
about 30% are positive. It says nothing checkable about *this* case, and its finite-sample
estimator (ECE) is binning-dependent. That is the wrong shape for an audit, and it is
the shape Jev sells.

**[measured]** The distribution-free literature offers the right shape. Angelopoulos &
Bates, "A Gentle Introduction to Conformal Prediction and Distribution-Free Uncertainty
Quantification" ([arXiv:2107.07511](https://arxiv.org/abs/2107.07511)): the sets are
"valid in a distribution-free sense: they possess explicit, non-asymptotic guarantees
even without distributional assumptions or model assumptions", usable with "any
pre-trained model" to produce sets "guaranteed to contain the ground truth with a
user-specified probability, such as 90%." **Conformal risk control**
([arXiv:2208.02814](https://arxiv.org/abs/2208.02814)) generalizes this to controlling
the expected value of any monotone loss — e.g. a false-negative rate on one question
type — rather than only miscoverage **[unverified — snippet]**.

**[measured]** And the 2026 literature is blunt about the limits, which the report must
carry or it is doing the same thing it criticizes Jev for. Exact finite-sample
distribution-free *conditional* coverage is impossible (Vovk 2012; Lei & Wasserman 2014;
Foygel Barber et al. 2021) **[unverified — relayed]**; Min, Peng & Zou
([arXiv:2605.11602](https://arxiv.org/abs/2605.11602), 12 May 2026) decompose conditional
miscoverage into score-estimation, finite-sample-calibration and intrinsic-mismatch
error; and Kotte, "When Can Conformal Risk Control Certify LLM Outputs?"
([arXiv:2606.29054](https://arxiv.org/abs/2606.29054), 27 Jun 2026) — which covers
schema-validated structured generation and classification, i.e. exactly this boundary —
finds heuristic abstention policies "violate user-specified risk targets on 7.5–12.5% of
settings, with no per-deployment guarantee", proves an abstention floor of
(μ−α)/(M−α) when base risk exceeds the target, and reports that "under cross-dataset
shift the target is violated on 14 of 16 transfers by static CRC and every tested
adaptive-conformal-inference step size."

**[inference]** Read that last result against Horismos's own design and it lands
squarely: *"heuristic abstention policy with no per-deployment guarantee"* is a precise
description of `horis:confidence 0.85` + `horis:fallback`. Kotte's paper is, in effect,
a measurement of how badly Horismos's current perception contract can miss a stated risk
target — and simultaneously the recipe for fixing it. The fix is not "become
probabilistic". It is: calibrate the score, then choose the threshold by split-conformal
calibration against a declared risk target, and publish the abstention rate that target
implies. The gate stays boolean; the *threshold* acquires a guarantee.

**[inference]** Note what this does to the Horismos/Jev comparison. Jev ships the
diagnostic (a probability) and markets it as the auditable object. Horismos already
ships an auditable object of a different and stronger kind — a proof that the decision
follows from admitted facts and declared rules — and is weakest exactly where Jev is
strongest: the quality of the number that decides whether a fact gets admitted at all.
The two guarantees are not competing; they are stacked, and Horismos currently has a
soft bottom layer under a hard top one.

#### 6.4.3a A caution Horismos must apply to itself: an undisclosed statistic is not auditable

**[measured]** Jev returns both a full `probabilities` map and a scalar `confidence`,
and TypeSafe describes the latter only as *"a statistic computed from the probability
distribution"* — the formula is not published, and the docs suggest computing your own
from `probabilities` instead ([docs/confidence](https://docs.typesafe.ai/confidence)).
A Noul returns **no `confidence` field at all**, only a single 0–1 number
([docs/primitives/noul](https://docs.typesafe.ai/primitives/noul)).

**[inference]** Two things follow, and the second is uncomfortable. First, a Horismos
bridge for a model of this shape should carry the **distribution** and derive its
per-fact confidence by a rule *declared in the deployment*, never thread an undisclosed
vendor statistic into `horis:confidence`; a threshold whose input you cannot define is
not auditable, which is the whole complaint §6.3 makes about black-box decisions.
Second, and this is the uncomfortable part: **Horismos's current confidence is also
undefined in exactly this sense.** `PerceiveResult::confidence` is whatever number the
model wrote in a tool-call field, with no stated semantics and no measurement. The
corpus should not make an argument against TypeSafe that it cannot survive itself. §7
move R1 is the remedy, and its first beneficiary is Horismos's own honesty.

#### 6.4.4 Item by item: what would have to be true for each adoption to be a good idea

| Candidate adoption | What it would mean concretely | What would have to be true | Verdict |
|---|---|---|---|
| **Carry a calibrated confidence alongside the proof** | Keep `horis:confidence`; require a Bridge to declare how its `confidence` was produced (self-report / logprob / calibrated head + method), record that in the trace, and publish a reliability diagram per deployment | A Bridge can actually supply it (logprobs, or a calibrated classifier); the trace can carry it additively (doc 09 rule 1 + the `allow_incomplete` precedent show it can); someone owns measuring it | **Yes — but as provenance, not as a second decision input.** The confidence must stay "metadata, not truth" (doc 06). See OD-1. |
| **Choose the threshold by conformal calibration against a declared risk target** | Replace "the author picked 0.85" with "0.85 is what split-conformal calibration on N held-out cases yields for a 5% target error rate, at a 12% abstention rate" | An exchangeable calibration set exists for the deployment; the target is declared in the ontology or the assumption manifest; drift is monitored (Kotte: 14/16 transfers violated the target under shift) | **Yes, and this is the strongest single idea in the dive** — it converts a hand-picked constant into a stated, checkable guarantee, without touching the gate. See §7 move R2 and OD-3. |
| **A full probability distribution in `RankResult`** | `rank` returns `Vec<f64>` over candidates instead of `chosen: usize` + one `confidence` | `ProposalStep` actually executes (it does not — §2.1(d)); a consumer exists for the distribution beyond argmax + threshold | **Only if `rank` lands, and only if a consumer exists.** Right now argmax + threshold is all the runtime uses, so a distribution would be recorded and ignored. See OD-2. |
| **RLCD-style honesty objectives for a learned component** | Train something with a proper-scoring-rule reward | Horismos has a learned component. It does not. PRP-007's authoring agent is the only candidate and it is Draft, sequenced after PRP-008 | **Not now.** And when it is time, the citation is **RLCR**, not RLCD — see §4.7. |
| **Adopt Choice/Score/Noul as the proposer contract** | Restate `horis:extracts`-derived schemas in Jev's three primitives | The three primitives were more expressive than what doc 06 already derives. They are **strictly less** expressive: §6.2.3 shows Choice ≈ closed `oneOf`/subclass enums and Noul ≈ `xsd:boolean`, while **Score is a 2–10 level ordinal rubric, not a numeric extractor** — so `xsd:decimal` + `horis:min`/`max`/`unit`, `horis:pattern`, and every string-valued property have no encoding at all | **No — it would be a downgrade, and a lossy one.** But the *vocabulary* is worth borrowing for positioning (§7 move R6). |

### 6.5 Q-D5 — is "one parallel pass" an architectural idea worth borrowing?

**Short answer: for the proposer, partly; for the kernel, no — and the reason is
load-bearing, not conservative.**

**[measured — local repo]** Horismos's forward chainer is deliberately one-at-a-time.
[`crates/horismos-core/src/reason/forward.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/reason/forward.rs):1-30 reproduces doc 04's loop
verbatim:

```text
loop (max N iterations, default 100):
    candidates = rules whose conditions hold in the current graph
                 minus rules already fired on the same bindings
    if empty: fixed point reached → done
    fire highest-priority candidate (ties: rule IRI lexicographic)
    assert its conclusions through the gate (provenance: rule)
```

and the module doc states why the order is observable: "priority orders candidate
*selection*, not rule truth"; two rules concluding different values for the same
single-cardinality property — "higher priority wins, the loser is recorded as
suppressed, never silently dropped"; and a higher-priority rule that becomes fireable
late "re-marks the earlier firing `Overridden` — the trace never claims a replaced
value is still standing."

> **Python parallel.** `FiringOutcome` is an enum whose variants are outcomes
> (`Fired`, `Suppressed`, `Overridden`, `Rejected`) — in Python, a `StrEnum` or a
> tagged union of dataclasses recorded per firing. The point is that the *sequence* of
> these is the audit record; it is not an implementation detail you could reorder.

**[inference]** Firing is therefore not parallelizable without changing the semantics
the proof and the trace are built from. If N candidate rules fired simultaneously, the
"suppressed" and "overridden" records — the ones that let a reviewer see the rule that
*almost* fired — would have no defined content, and two runs could disagree about which
value stood. Determinism ("trace equality is a valid test assertion", CLAUDE.md) is
bought with the sequential loop. This is the single clearest case in the report where
Jev's headline architectural idea is genuinely *not* transferable.

**What *is* transferable, in two places:**

1. **Matching, not firing.** Computing `candidates` is a pure function of the graph; so
   is each of the five gate checks ([`world/gate.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/world/gate.rs):1-12 — "*pure* functions of the
   current store plus the proposed mutation"). Those could be evaluated in parallel
   inside an iteration without changing a single observable. That is an ordinary
   performance question, not an architectural one, and load+verification already
   benchmark "below 1 ms here" ([`PRPs/BOARD.md`](https://github.com/billaram/horismos/blob/adfc27a/PRPs/BOARD.md)), so there is nothing to buy yet.
2. **The proposer's question set.** Jev's real idea is not "evaluate in parallel"; it is
   **"declare the whole question set before you look at the data."** Horismos already
   does exactly that, one step at a time: `horis:extracts` fixes the target classes, the
   JSON Schema is "derived fresh from the loaded ontology at load time and cached; it is
   never hand-written and cannot drift from the ontology" (doc 06). What Horismos does
   *not* do is hoist independent perception steps into one call — today it is one Bridge
   call per `PerceptionStep`, executed in workflow order.

**[inference]** Hoisting is only sound for perception steps whose inputs do not depend
on an earlier step's decision — and in a state machine that is a static property a
verifier could compute, which makes "batch the independent perception questions into one
call" a real, bounded optimization with a verification story rather than a vague idea. It
does **not** appear in §7's ranked moves, and the reason is worth stating so it is not
mistaken for an oversight: **[measured — local repo]** the canonical triage overlay
declares exactly one `PerceptionStep` and the shipped PRP-008 standards slice
(`examples/vte/vte.ttl`) declares **none at all** — it is fed entirely through the
structured-input path doc 06 promotes to primary. There is nothing to batch. Revisit if
a shipped overlay ever declares two independent perception steps.

**[measured]** For scale, the batching win is real where it applies: an independent test
put **800 typed judgments in a single 985 ms call for $0.00075**
([priorbench/jev](https://github.com/priorbench/jev)). That is the number that would
justify the work, in a world where Horismos had eighty questions to ask at once.

### 6.6 Q-D6 — the standing verdict, re-issued

The wiki's convention for a standing page is that its verdict is **re-issued, not
silently edited**, with a row added to the page's own verdict-history table and mirrored
on the [verdict history](../verdict-history.md)
([how sweeps work](../sweeps.md#what-a-sweep-produces)). The task requires this dive to
either re-issue or state explicitly, with reasons, that it does not move the verdict.
It re-issues — scoped, and unchanged in direction.

#### 6.6.1 Verdict

**[inference]** **aligned-with-adjustments — scoped to the non-generative
typed-decision-model axis, extending (not re-litigating) sweep #2's verdict on the pivot
itself and the RL deep dive's verdict on its axis.**

The arrival of a funded, widely-tried commercial model that answers typed questions
declared in advance **confirms the premise of the pivot's C5 commitment** ("neural
proposes, symbolic decides") from an unexpected direction — a vendor conceding that a
model's output should be a typed value in a pre-declared space — and **leaves every
other commitment untouched**. It challenges nothing about the kernel's shape. It
challenges two things that are not the kernel: a piece of the *positioning* that rested
on typed decisions being differentiating, and the *quality* of one shipped mechanism,
the self-reported confidence the perception threshold compares against. Both are
addressable within the current design, and the second is a defect rather than a design
question.

#### 6.6.2 Commitment by commitment

| Commitment (sweep #2's C1–C8) | This axis's bearing |
|---|---|
| **C1** snapshot-scoped guarantee | **Untouched.** A typed-decision model holds no state and hydrates nothing |
| **C2** conditional commit, fail closed on drift | **Untouched.** No commit path exists in the category |
| **C3** thin overlay on CDS Hooks / FHIR / CQL | **Untouched.** Text-only input, US-hosted, no clinical-standards surface |
| **C4** gate as proof-emitter; traces are output | **Confirmed, from an adversarial direction.** OrderBench measures 100% schema validity against ~80% semantic success and concludes that structured output "serves as a necessary interface component rather than a replacement for proper domain verification and fail-closed system architecture" ([arXiv:2607.18261](https://arxiv.org/abs/2607.18261)) — Horismos's thesis, reached independently by people benchmarking structured output. The independent Jev evidence adds the failure modes: 0 of 30 out-of-scope inputs flagged, 16.7% (below the random floor) under mis-specified questions |
| **C5** neural proposes, symbolic decides | **Confirmed on the premise; sharpened on the mechanism.** The vendor's own jaggedness page documents that answers to separate questions are not jointly constrained. A rule layer over a fixed point is exactly that missing constraint — so "symbolic decides" acquires a second, mechanical justification alongside the safety one |
| **C6** capability fragments | **Lightly confirmed.** The category reached harness distribution immediately (an agent skill for Claude Code/Codex; Vercel AI Gateway), and its function-calling pattern — model picks from a closed set, your code dispatches — converges on the fragment's shape while stopping short of declaring preconditions or effects |
| **C7** candidate-artifact boundary | **Untouched.** No generation, therefore no candidate artifact |
| **C8** assumption manifest and the gated ontology loop | **Untouched.** OD-7 asks whether the manifest gains a sixth field, and recommends not |

#### 6.6.3 What the dive owes, and to whom

**[inference]** Three items, all shape-preserving, in the same idiom as sweep #2's A1–A14:

- **J1 — Measure the confidence the gate's upstream threshold compares against.**
  Rests on: `accepted = result.confidence >= p.confidence` over a self-reported number
  (`workflow/agent.rs`:492, `openrouter.rs`:322-331); ConfidenceBench's finding that
  calibration and accuracy diverge across model families; Kotte's 7.5–12.5% target
  violation for heuristic abstention. Owner: **PRP-005**. This is §7's R1 and it is a
  build item.
- **J2 — Replace "typed decisions" with cross-answer consistency and epistemic
  auditability in the positioning.** Rests on: the vendor's own documented structural
  non-invariance; its own statement that it generates no explanations; and the
  literature's own System-1/System-2 definitions placing Horismos in the System 2 slot.
  Owner: **PRP-000**. Captain-held (OD-6).
- **J3 — Name the inverse confidence threat in doc 15.** Rests on: Forced Deferral
  ([arXiv:2606.15308](https://arxiv.org/abs/2606.15308)); the vendor's "does not treat
  [state] as hostile by default". Owner: **doc 15's security workstream / PRP-008** — the
  same owner sweep #2's A10 named. Captain-held only as to posture (OD-5).

#### 6.6.4 Why this is not a `misaligned` finding, stated explicitly

**[inference]** The corpus's vocabulary is deliberate: *misaligned* would mean a
load-bearing commitment has independent evidence against it and the pivot's shape is in
question. Nothing in this dive is evidence against a commitment. The closest thing to a
challenge is commercial rather than technical — that a cheap typed classifier absorbs the
segment whose requirement was only "text in, typed value out" — and that segment is
explicitly outside what the pivot claims: PRP-000's honest claim is a *snapshot-scoped
guarantee over a versioned projection with a proof and a conditional commit*, none of
which that segment was buying. Narrowing the addressable market is a positioning fact,
not a misalignment.

**[inference]** The one place a reader could reasonably push back is C4's second half —
"traces are output". This dive does not test it and does not claim to; the RL deep dive's
finding that the *signal* half of that claim is still spec-only stands unchanged, and
nothing here closes it. Readers auditing C4 should read both verdicts together.

---

## 7. Q-E — ranked concrete moves

Ranked by expected usefulness. Effort S/M/L on the RL deep dive's scale. The last column
is the distinction the brief asked to be kept sharp: **Build** means "Horismos should do
this, and it needs no decision above the implementer"; **Decide** means "someone should
decide whether Horismos does this", and the corresponding OD in §9 is self-contained.
**[inference]** throughout.

| # | Move | Why it is worth doing | Effort | Owner | Build or Decide |
|---|---|---|---|---|---|
| **R1** | **Measure the calibration of the confidence Horismos already gates on.** Publish a reliability diagram and an ECE for the `openrouter:` bridge on the triage and VTE fixtures, with the measurement protocol stated. | `accepted = result.confidence >= p.confidence` ([`agent.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/agent.rs):492) is a real decision boundary over a number nobody has ever measured (§2.2). ConfidenceBench shows the best-calibrated frontier model is not the most accurate one, and several are worse than a calibrated-random baseline — so the boundary moves silently when a deployment swaps models, which doc 08 explicitly permits as "a config change". This is a defect in something shipped, not a new feature. | M | **PRP-005** — it already owes the gate's ρ₀/ρ₁ under the RL dive's move 4; this is the same instrument pointed one step earlier | **Build** |
| **R2** | **Give `horis:confidence` a declared target risk, and choose the number by split-conformal calibration rather than by hand.** Publish the implied abstention rate beside it. | This is the strongest single idea in the dive. `horis:confidence 0.85` is currently a constant an author picked; Kotte ([arXiv:2606.29054](https://arxiv.org/abs/2606.29054)) measures heuristic abstention policies over exactly this class of task violating user-specified risk targets on **7.5–12.5% of settings, with no per-deployment guarantee**. Conformal calibration turns the constant into a statement an auditor can check ("5% target error, 12% abstention, calibrated on N held-out cases") **without making the gate probabilistic** — the gate stays boolean; only the threshold acquires a guarantee. Ships with two riders from §6.4.2a: the guarantee is **marginal, not per-case or per-subgroup** (Mehrtens et al., [arXiv:2506.18162](https://arxiv.org/abs/2506.18162): a 90% marginal guarantee delivering 92% coverage to one subgroup and **62%** to another), and abstention has its own measured clinical cost (Jabbour et al. 2026: **+18% missed diagnoses, +35% missed treatments**), so a higher threshold is not automatically the safer one. | M | **PRP-005** (procedure and acceptance item) + doc 06 text | **Decide** → OD-3 |
| **R3** | **Restate the positioning around the two things this dive found are actually differentiating: cross-answer consistency, and structural-vs-epistemic auditability.** Add a typed-decision-model row to PRP-000's comparison table. | "Decisions should be typed" stopped being a differentiator on 2026-09-15 (§6.1). Two replacements are now evidence-backed and sharp: (a) TypeSafe *publishes* that its answers are not jointly constrained — Noul 0.22 vs Choice 0.01 on the same proposition, a claim and its negation summing to 1.19 — and a rule layer over a fixed point is exactly that missing constraint; (b) Jev gives structural auditability (which question, which threshold, which branch) and zero epistemic auditability (why this answer), while a proof tree is precisely the second. | S | **PRP-000** | **Decide** → OD-6 |
| **R4** | **Record how a proposer's confidence was produced, in the trace.** An additive-optional field naming the method (self-report / logprob / calibrated head + named calibration), and optionally the full distribution rather than its argmax. | `Bridge::id()` records *which* model ran; nothing records *how trustworthy its number was*. Doc 09's design rule 5 and the `allow_incomplete` precedent give the exact mechanism ("additive-optional under the versioning policy"). Without it, R1's measurement is not attributable after the fact, and a trace cannot answer "was this threshold meaningful?". | S | **doc 09 / PRP-002** (trace schema is versioned forever) | **Decide** → OD-1 |
| **R5** | **Name the perception boundary as what it is — selective prediction — in doc 06, and cite the literature that owns it.** | Horismos has shipped Chow's reject option (1970) without calling it that (§6.4.1). Naming it costs a paragraph and buys two things: ontology authors learn what a threshold is *for* (a target risk on an error–reject curve, not a vibe), and the corpus stops re-deriving a 56-year-old mechanism. It is also the prerequisite framing for R2. | S | **doc 06** (shipped-voice text; adds a citation, decides nothing) | **Build** |
| **R6** | **If a fast typed proposer is wanted, prototype the *local encoder* bridge behind the `hf:`/`onnx:` prefixes that doc 08 already reserves — not a hosted third-party one.** | Doc 08's registry already lists `hf:` as "classification-style perception without a frontier model" and `onnx:` as "edge/embedded", and both modules are one-line stubs. The measured case is overwhelming: Ettin-150M at **3.60 ms median / 244 completions per second** ([arXiv:2606.25782](https://arxiv.org/abs/2606.25782)) against Jev's 70–500 ms; DistilBERT at **$0.0000124/case** ([arXiv:2602.06370](https://arxiv.org/abs/2602.06370)) against Jev's $0.0004; open reproductions of Jev's exact Choice/Noul/Score surface already exist on ModernBERT-Large. And it has **no data-egress problem, no vendor trust posture to clear, and no non-determinism imposed from outside** (§6.2.6). | M | **No PRP owns this.** Bridges were PRP-004's (Done); a new backend has no home | **Decide** → OD-4 |
| **R7** | **Add confidence-suppression to doc 15's threat list.** | Doc 15 §1 names the risk that injected text *clears* the threshold; it does not name the inverse — that an adversary who can *depress* confidence forces `horis:fallback` on every case, a denial-of-service on the human in the loop. Forced Deferral ([arXiv:2606.15308](https://arxiv.org/abs/2606.15308)) demonstrates a universal trigger doing exactly this to a cascade's first tier. The threat exists today with any bridge. | S | **doc 15 security workstream / PRP-008** (the same owner sweep #2's A10 named) | **Decide** → OD-5 (whether v1 claims a mitigation or declares it deployer-owned) |
| **R8** | **Run the joint-consistency experiment, and publish it.** Ask a typed-decision model a set of logically entailed questions over the triage fixtures, measure the inconsistency rate, then show the same questions answered through a Horismos overlay where the rule layer makes the entailment hold by construction. | This converts §6.1's strongest positioning claim from an argument into a number, using a property the vendor has already documented as a weakness. It is the cheapest credible piece of external evidence the project could produce, and it does not require building a production bridge — a throwaway harness suffices. | M | **No PRP owns this.** Positioning evidence for PRP-000; nearest home is the research corpus itself | **Decide** → OD-4 (same sequencing question) |
| **R9** | **Fund or run the experiment nobody has run: a Jabbour-style clinician study where the artifact under test is a *derivation over declared facts*, not a saliency map.** Same design, same biased-model arm; measure whether scrutiny rises specifically on the wrong-AI trials. | §6.3 rests on a distinction between post-hoc explanation and reason-giving that is argued from first principles and measured almost nowhere. The one positive signal — Han et al. 2026's eye-tracking result, where "verifiable explanations" raised AI↔image gaze switches from **6.04 to 8.52** on incorrect-AI trials — is a single small study of an artifact defined as "brief verification cues linked to the current AI recommendation", which is structurally what a proof tree is. Everything else in the human-factors literature (Gomez 2025, Nicolson 2025, Buçinca 2021, Bansal 2021, Nagendran 2024) measures the *other* artifact and finds it does not help. Until someone runs this, the wedge in §6.3 is a legal argument with no human-factors evidence behind it — and Ghassemi's human-factors argument stands unanswered. | L | **No PRP owns this.** Validation evidence; nearest home is the [validation ladder](../validation/00-ladder.md) as an instrument on rung 1 | **Decide** — it is a funding call, not a build |

### What NOT to do

**[inference]**, each with the reason:

- **Do not build a `typesafe:` production bridge now.** Four independent blockers, any one
  of which is sufficient: numeric and string properties cannot be expressed in its
  primitives at all (§6.2.3), and Horismos's ontology vocabulary is numeric-first; the
  vendor explicitly declines determinism; there is no SOC 2, BAA, SLA, self-host or
  residency option (§3.5); and the slot it fits best (`ProposalStep`/`rank`) does not
  execute. Revisit if a published reliability curve appears, a BAA becomes available, and
  `ProposalStep` lands.
- **Do not adopt Choice/Score/Noul as the proposer schema contract.** It is strictly less
  expressive than what doc 06 already derives (§6.4.4). Borrow the vocabulary for
  positioning if the category takes hold; do not borrow the type system.
- **Do not make the gate probabilistic, or give it a second confidence input.** Doc 06's
  "confidence is metadata, not truth … it never weights inference. There is no fuzzy
  logic in Horismos" is correct and this dive found nothing against it. Calibration
  belongs to the *threshold* and the *trace*, not to inference.
- **Do not chase "System One" as a self-description.** In the literature's own usage
  Horismos is the System 2 (§4.2); adopting the other label would be both inaccurate and
  self-diminishing.
- **Do not treat TypeSafe's benchmark as a baseline to beat.** It grades agreement with a
  two-frontier-model consensus, not correctness; the reference models ran at high thinking
  while graded models ran at provider default; the cases and harness are unpublished; and
  no independent reproduction exists (§3.4).
- **Do not cite the "0% structured-output error" chart as a competitor claim to rebut.**
  TypeSafe already states the number is not empirical. Rebutting a claim its own author
  qualifies is the kind of argument that makes a corpus less trustworthy, not more.

---

## 8. Searched but not found

Per the corpus's convention that negatives are first-class and a sweep records its own
misses. Each line states what was looked for, by whom, and what the absence means.

**About Jev specifically**

- **No paper, no arXiv preprint, no technical report, and no scientific model card.**
  Sub-agent A read the full ~896 KB docs corpus (`llms-full.txt`, 91 pages), the blog,
  the homepage and the legal pages; the only artifact called a "ModelCard" is an SDK type
  with three fields. **[measured]** No parameter count, layer type, encoder-vs-decoder
  distinction, tokenizer, or base-model provenance exists anywhere public.
- **No published calibration measurement of any kind** — no ECE, no reliability diagram,
  no Brier score, from TypeSafe or anyone else. **[inference]** This is the most
  significant absence in the dive, because calibration is the *only* claim that would
  make Jev a category rather than a product, and §5.5 records it as the thing whose
  continued absence past ~30 days is itself the answer.
- **No definition of the two formulas the product's claims rest on.** The eval's
  "accuracy" is a set comparison whose formula (exact match? per-action F1? Jaccard?) is
  unpublished; `confidence` is described only as "a statistic computed from the
  probability distribution". Both were searched for directly.
- **No independent reproduction of TypeSafe's four-workflow benchmark**, as of
  2026-09-21. The cases, harness and reference answers are unpublished; the
  LLM-baseline adapter is the only open piece.
- **No Artificial Analysis listing, no LMArena entry**, and no equivalent third-party
  leaderboard placement — by TypeSafe's stated policy of declining public benchmarks.
- **No SOC 2, HIPAA/BAA, SLA, self-host, or data-residency statement.** Searched across
  docs, privacy policy and the Master Customer Agreement.
- **No published walkthrough or repository for the Doom demo**, despite the blog
  promising one; no Doom repository exists in the `typesafe-ai` GitHub organization.
- **No TypeSafe response to the prior-art claim** raised on Hacker News **[unverified —
  absence observed via search only]**.
- **No participating investors named beyond DCVC**, and no team headcount disclosed.

**In the literature**

- **No head-to-head calibration benchmark of a fine-tuned discriminative model against a
  frontier LLM on the same task.** Sub-agent B ran four differently-phrased searches and
  found none, and noted that the paper that most obviously should contain it
  ([arXiv:2602.06370](https://arxiv.org/abs/2602.06370), which measures F1, latency and
  cost across exactly that comparison) **conspicuously reports no calibration metric**.
  **[inference]** A genuine hole, not a search failure — and the hole is precisely where
  Jev's differentiating claim would have to be settled.
- **No published measurement of cross-question joint consistency** for any
  parallel/multi-head typed-decision system. ParallelBench measures the analogous
  property for token-level parallel decoding; TypeSafe documents two anecdotal examples
  of its own inconsistency. Nobody has put a rate on it. **[inference]** This is why §7's
  R8 is cheap and valuable: the experiment is unrun and the vendor has already conceded
  the direction of the result.
- **No use of an RDFS/OWL/Datalog kernel downstream of a calibrated typed-decision
  model** anywhere sub-agents B, C or D surveyed. The nearest thing is the *pattern*
  TypeSafe documents in its function-calling cookbook — model picks from a closed set,
  your code dispatches — which is the same move Horismos makes with `horis:takes` and a
  graph-derived candidate set, arrived at independently. **[inference]** The square is
  open, exactly as the RL deep dive found for its own axis.
- **No determinism or seeding guarantee** offered by any of the commercial
  typed-decision offerings surveyed; the one that addresses it argues explicitly against
  it.

**In the regulatory and clinical-AI record**

- **No authority, anywhere, on whether AI Act Art. 86 bites while Annex III
  classification is deferred.** Art. 86 was not deferred by the Digital Omnibus; the
  classification article it depends on was. The sub-sweep searched practitioner
  commentary and found **not one source that addresses the question** — including FPF's
  post-omnibus timeline, which discusses Chapter III and does not mention Arts. 85–86.
  **[inference]** The silence is itself the finding: this is unsettled, and anyone
  asserting "Art. 86 is enforceable now" is asserting past the evidence.
- **No rigorous buyer-side willingness-to-pay evidence for explainability, in either
  direction.** Searches returned almost entirely SEO content from XAI vendors, at least
  one of which asserts an "EU AI Act June 2026 enforcement deadline" that the Official
  Journal contradicts. **[inference]** Recorded as an *incomplete search* rather than a
  confirmed negative — the sub-sweep's 200-call web-search budget was exhausted before
  the negative could be established. The AMA 2026 survey (§6.3.3) is the closest thing to
  buyer-side evidence found, and explainability is not an enumerated option in it.
- **The adopted CPPA CCPA-ADMT regulation text could not be read.** Every route was
  blocked. The reported access-right language ("a description of the ADMT's logic that
  clearly explains how the ADMT processed personal information to generate the output",
  with a trade-secret carve-out and a possible aggregate-answer allowance) is
  **[unverified]** and should be read before anyone relies on it — compliance begins
  2027-01-01, so there is time.
- **Whether the April 2026 Reg B rewrite touched 12 CFR § 1002.9 could not be confirmed**
  against the final rule text; the inference that it did not rests on the rule's stated
  three-issue scope.
- **State adverse-underwriting-decision notice statutes were not researched at all** —
  the NAIC Privacy Protection Model Act lineage is the likeliest place a real insurance
  reason-giving duty lives, and this dive did not look.
- **No 2026 FDA Digital Health Advisory Committee meeting was found**, but the negative
  could not be established before the search budget ran out. The last confirmed meeting
  is 2025-11-06.
- **Reddy's 2022 *Lancet Digital Health* comment on Ghassemi could not be read** (the
  same Cloudflare block); only its PubMed registration as a "Comment on" and its declared
  competing interest are recorded.

**Environment limits of this dive, recorded rather than hidden**

- **Reddit (r/LocalLLaMA, r/MachineLearning) and Lobste.rs were unreachable** from this
  environment — the Reddit JSON API and direct fetches were blocked, and Lobste.rs
  returned an interstitial. Sub-agent D compensated with the full Hacker News threads
  (509 + 88 comments, read via the Algolia items API), the Elixir Forum thread, and a
  third-party analysis of 12,759 posts, but the community picture has a hole in it.
- **X/Twitter was reachable only as snippets** (HTTP 402 on direct fetch); the
  openchamber 12,759-post analysis substantially compensates but is itself a secondary
  source.
- **OpenAI's 2024 Structured Outputs blog post returned HTTP 403** to every attempt, so
  the widely-quoted "100% vs <40%" schema-compliance figures are `[unverified]` here and
  are cited as such in §4.6; the live API documentation, which *was* read, states no
  percentage at all.
- **Two hands-on reports (Every, NearHere) were reachable only second-hand** and their
  numbers are attributed to the intermediary.
- The **"GPT-5.6 Sol"** model name carried in the intake **could not be confirmed**; the
  eval site labels the point bare (`sol`).
- **thelancet.com hard-blocks automated access** (Cloudflare) and Europe PMC reports
  "Subscription required" for the Ghassemi Viewpoint. The body text was obtained in the
  end — the paper is CC-BY and a headed browser session worked — but only on the second
  attempt, and the first attempt's failure is why §6.3.1 was drafted from the abstract and
  had to be corrected (§10).
- **Fierce Healthcare is Cloudflare-blocked to both fetch and browser**, so the report of
  CHAI's assurance labs having "never materialized" is `[unverified]`, snippet-only, and
  is not load-bearing anywhere in this report.

---

## 9. Open decisions

None of the following is decided by this report. Each is written to stand alone —
firstmate files them as captain-held backlog items, so each restates the context it
needs. PRP-owned D-items stay in their PRPs; the RL deep dive's OD-1–OD-4 and sweep #2's
A1–A14 / X1–X10 are untouched.

---

### OD-1 — Does the trace record *how* a proposer's confidence was produced?

**The decision.** Horismos's trace records which model ran (`Bridge::id()` →
`Step::Perception.model`) and what number it returned (`confidence`), but nothing about
how that number was derived. This asks whether to add an additive-optional field naming
the derivation method — self-report, logprob-derived, calibrated head with a named
calibration procedure — and, optionally, whether to carry the full probability
distribution rather than only the accepted scalar.

**Why evidence cannot settle it.** The trace schema is one of two wire contracts the
project has committed to versioning forever (doc 09 design rule 1: "`horismos.trace/1`
never changes meaning"; CLAUDE.md: "Two wire contracts are versioned forever"). Adding a
field is cheap; committing to its meaning for the life of the project is a product
decision about what a Horismos trace *is for*. The research says the field would be
useful — ConfidenceBench measures frontier models' calibration varying widely and not
tracking accuracy, and doc 08 already permits a deployment to swap the model behind a
prefix — but nothing in the literature can tell you whether Horismos wants to promise
this forever.

**Options.** (a) Add `confidence_method` (and optionally `distribution`) as
additive-optional fields to `Step::Perception` and `Step::Proposal`, under the existing
`allow_incomplete` precedent for additive-optional evolution. (b) Carry it only in
`CallAccounting::raw_metadata`, which already exists as "an audit escape hatch for
provider-specific fields", and promote later if it proves load-bearing. (c) Do nothing;
the deployment's configuration record is the place to look this up.

**Recommendation.** (b) now, (a) when a second bridge with a genuinely different
confidence derivation exists. `raw_metadata` was built for exactly this and costs
nothing; promoting a field to the permanent schema before there is a second derivation
to distinguish would be committing to a distinction that has only one value.

**Owner.** doc 09 / **PRP-002** (trace schema). Touches decided text.

---

### OD-2 — When `ProposalStep` lands, does `rank` return a distribution or an argmax?

**The decision.** `Bridge::rank` is specified as
`RankRequest{context, candidates} → RankResult{chosen: usize, confidence: f64}` — one
selected index and one scalar. `ProposalStep` parses, has ontology vocabulary, and has a
real trace emission builder, but the runtime refuses it
(`StepBody::Proposal(_) => Err(deferred::proposal_deferred(…))`;
`workflow/deferred.rs`: "nothing in this PRP calls `Bridge::rank`"). This asks whether,
when ranking is implemented, the contract should instead return a probability over the
candidate set.

**Why evidence cannot settle it.** Both shapes are implementable and both have working
precedent. The question is whether Horismos wants a consumer for a distribution. Today
the runtime would use only `argmax` plus a threshold, so a distribution would be recorded
and ignored — but the slot is unbuilt, which is the cheapest possible moment to widen the
contract, and re-narrowing a shipped trait is far more expensive than widening an unbuilt
one. That is a judgment about future intent, not a fact.

**Options.** (a) Keep `chosen + confidence`; add a distribution later if a consumer
appears. (b) Widen to `Vec<f64>` over candidates now, with `chosen` derived by a declared
rule in the runtime rather than by the bridge. (c) Widen the *result type* now but leave
the runtime consuming only argmax, so the trace carries what the model actually believed.

**Recommendation.** (c). It costs one field, it is decided before anything depends on the
narrow form, it keeps the *decision rule* in the runtime where doc 08 says thresholds
belong ("the runtime — not the Bridge — owns thresholds"), and a recorded distribution is
exactly what R1's calibration measurement needs as input.

**Owner.** doc 07 / doc 08 contract text; implementation belongs to whichever PRP lands
`ProposalStep` execution (**PRP-004** is Done, so this currently has no active owner).

---

### OD-3 — Does `horis:confidence` acquire a declared target risk, or stay an author-chosen constant?

**The decision.** Today an ontology author writes `horis:confidence 0.85` and the runtime
computes `accepted = result.confidence >= 0.85`. This asks whether the vocabulary and the
verification story should additionally carry a *target* — a stated acceptable error rate
— with the threshold derived from it by a calibration procedure on held-out cases, and
the resulting abstention rate published alongside.

**Why evidence cannot settle it.** The literature is clear that a hand-picked threshold
over an unmeasured score carries no guarantee (Kotte, arXiv:2606.29054: heuristic
abstention violates user-specified risk targets on 7.5–12.5% of settings "with no
per-deployment guarantee"; and under cross-dataset shift "the target is violated on 14 of
16 transfers"). What it cannot settle is whether Horismos wants to *require* a calibration
set. Requiring one raises the bar to author an overlay and may be impossible for a
deployment with no labelled history; not requiring one means the project keeps shipping a
boundary it cannot defend. That trade is a product call.

**Options.** (a) Status quo — an author-chosen constant, documented as such. (b) Optional:
add vocabulary for a declared target risk and a calibration artifact; verification warns
when a threshold has no calibration evidence. (c) Required for any deployment claiming
the guarantee — the assumption manifest carries calibration evidence or the claim is not
made.

**Recommendation.** (b), staged toward (c) for regulated deployments. It preserves the
current authoring experience, makes the stronger claim available to deployments that can
support it, and fits the existing "fail closed, but `allow_incomplete` is visible in every
trace" pattern — the same shape of escape hatch, applied to a different guarantee.

**Three riders, whichever option is chosen** (§6.4.2a, added after the regulatory sub-sweep
returned): the guarantee a conformal procedure produces is **marginal over the calibration
distribution**, not per-case and not per-subgroup — Mehrtens et al.
([arXiv:2506.18162](https://arxiv.org/abs/2506.18162)) give a worked example of a 90%
marginal guarantee delivering 92% coverage to one patient subgroup and **62%** to another —
so (1) calibrate per deployment rather than once at the vendor, (2) report coverage by
clinically meaningful subgroup and not only in aggregate, and (3) state the calibration
distribution in the same artifact as the number. Publishing the marginal sentence while a
deployment reads it as the per-case one would manufacture exactly the false assurance doc
06's "confidence is metadata, not truth" rule exists to prevent. And note that raising the
threshold is **not** the conservative default: Jabbour et al. 2026 measure abstention itself
costing **+18% missed diagnoses and +35% missed treatments** relative to no AI at all.

**Owner.** **PRP-005** (verification and the assumption manifest) with doc 06 text.

---

### OD-4 — Is a typed-decision proposer in scope at all, and if so, hosted or local?

**The decision.** Doc 08's registry already reserves `hf:` ("classification-style
perception without a frontier model") and `onnx:` ("edge/embedded"); both modules are
one-line stubs. This asks (i) whether Horismos builds a classification-style bridge at
all before PRP-006, and (ii) if it does, whether it is a local encoder or a hosted
third-party typed-decision API.

**Why evidence cannot settle it.** The technical comparison is one-sided and *is*
settled: a local encoder is roughly 20–140× faster (Ettin-150M at 3.60 ms median vs
70–500 ms) and 12–32× cheaper per case (DistilBERT at $0.0000124 vs $0.0004) than the
hosted option, with no data egress, no vendor trust posture to clear, and no imposed
non-determinism. What evidence cannot settle is **sequencing**: PRP-006 (packaging) is the
next unlocked phase per the board, no PRP owns bridge backends now that PRP-004 is Done,
and adding a backend competes for the same capacity. This is a roadmap call.

**Options.** (a) Nothing before PRP-006. (b) A minimal `hf:`-prefixed local encoder bridge
as a demonstration, not a supported backend, sized to make R8's joint-consistency
experiment runnable. (c) A supported local classification backend as a new PRP. (d) A
hosted typed-decision bridge.

**Recommendation.** (b) if and only if R8 is wanted; otherwise (a). Explicitly not (d) —
see §7's "what not to do". Revisit (d) only if a published reliability curve appears, a
BAA becomes available, and `ProposalStep` lands.

**Owner.** **No PRP owns this.** Nearest candidates: a PRP-006 packaging deliverable, or a
new PRP. The sequencing question belongs on `PRPs/BOARD.md`.

---

### OD-5 — Does doc 15 name confidence suppression, and does v1 claim any mitigation?

**The decision.** Doc 15 §1 names one side of the confidence boundary's attack surface:
injected text that is "confident-sounding enough to clear `horis:confidence`", yielding a
wrong-but-schema-valid fact. It does not name the inverse — an adversary who *depresses*
confidence to force `horis:fallback` on every case, which is a denial-of-service against
the human in the loop. This asks whether to add the threat, and whether v1 claims a
mitigation or declares it deployer-owned.

**Why evidence cannot settle it.** That the attack works is measured: Forced Deferral
(arXiv:2606.15308) demonstrates a universal adversarial trigger depressing a first-tier
model's confidence to force escalation, precisely because "the weak model's confidence
directly controls compute allocation"; and the one commercial typed-decision model states
it "does not treat [state] as hostile by default". What evidence cannot settle is
Horismos's *posture*. Doc 15 already has a "What v1 does not defend against (stated, not
silent)" section and a considered position that model-level injection robustness is not
Horismos's to guarantee. Whether this belongs there, or in the mitigated list with a
fallback-rate monitor, is a security-product judgment.

**Options.** (a) Add to §1's threat list with the mitigation stated as deployer-owned
(monitor the fallback rate). (b) Add, plus a runtime signal — the trace already records
`fallback_taken`, so a per-run or per-window fallback-rate anomaly is computable without
new mechanism. (c) Add to "what v1 does not defend against". (d) Do not add.

**Recommendation.** (a) plus the observation in (b) that the signal already exists in
`Outcome.labels`, without committing v1 to shipping a monitor. Naming the threat is free
and the corpus's own convention is that threats are "named here as the reason that design
exists, not assumed obvious".

**Owner.** doc 15's security workstream / **PRP-008** — the same owner sweep #2's A10
named for extending the doc 15 threat list.

---

### OD-6 — Does Horismos adopt, reject, or ignore the "System One / System Two" vocabulary?

**The decision.** TypeSafe has nominalized "System One model" into a product category with
a large launch behind it (37M post views, a documented distribution campaign, 13% of
Vercel's paid teams trying it in a day). In the ML literature's own prior usage, "System
1" means fast direct answering and "System 2" means explicit search with backtracking —
and System-1.x's neuro-symbolic variant puts a *symbolic* engine in the System 2 slot. By
that vocabulary, Horismos is a System 2. This asks whether PRP-000's positioning adopts
the frame ("the System 2 that a System One proposes into"), explicitly rejects it, or
stays silent.

**Why evidence cannot settle it.** The literature supports the mapping; nothing about
whether a young project should tie its positioning to a six-day-old commercial category
that may not survive. Adopting it borrows reach and risks being dated by a vendor's
fortunes; ignoring it risks being read *as* a slower System One model, which is the worse
error. This is the same class of question as sweep #2's X10 ("does 'kernel' survive OaK's
usage of the word") and belongs with it.

**Options.** (a) Adopt, with the literature's definitions cited rather than the vendor's.
(b) Reject explicitly, and keep "neural proposes, symbolic decides" as the only frame.
(c) Stay silent; revisit if the category survives ninety days.

**Recommendation.** (c) with a prepared (a). The §5.5 evidence table names a concrete
check — Vercel's day-30 cut of the same adoption metric, due around mid-October 2026 —
which is a cheap, dated trigger for revisiting. Meanwhile PRP-000 should carry the
*substance* (R3's cross-answer-consistency and structural-vs-epistemic-auditability
arguments) whether or not it carries the vocabulary; the substance does not depend on the
category surviving.

**Owner.** **PRP-000**. Pairs with X10 and A13.

---

### OD-7 — Does the assumption manifest's five fixed fields gain a sixth for proposer evidence?

**The decision.** Every `VerificationReport` carries a five-field assumptions manifest —
`input_closure`, `external_profile_versions`, `mapping_version`, `supported_subset`,
`unverified_external_effects` — and doc 10 states "the five fields are fixed". This asks
whether a deployment's *proposer* evidence (which model, what confidence derivation, what
calibration was demonstrated, on what set) belongs there as a sixth field.

**Why evidence cannot settle it.** The manifest exists "precisely so the 'does not claim'
list … is enforced structurally, not just stated in prose" (doc 10). Whether the
perception boundary's trustworthiness is in scope for a *verification* report is a
boundary question about what verification means in this project: PRP-005's verifier is a
pure, WASM-safe, deterministic analysis of a loaded ontology, and calibration evidence is
a property of a deployment's model configuration, not of the ontology. Both readings are
defensible. The fixity of the five fields is a commitment that makes this a decision
rather than an edit.

**Options.** (a) Leave the manifest at five fields; carry proposer evidence in the
deployment record and the trace (OD-1) instead. (b) Add a sixth field, populated or
explicitly empty like the others. (c) Add it only to the *host*-side manifest in
`horismos-host::assumptions`, which PRP-008's gap note already flags for convergence with
the core one.

**Recommendation.** (a). Verification is about the artifact; the proposer is about the
deployment, and conflating them would make a verification report's `ok` depend on a model
configuration the verifier cannot inspect. If the claim needs to travel, the trace
(OD-1) is where it belongs. Note that (c) would be the worst option, since it deepens a
divergence PRP-008 has already gap-noted for convergence.

**Owner.** **PRP-005**, with a note to PRP-008's manifest-convergence gap.

---

### OD-8 — Does a proof have to be *minimal*, or is the full derivation the deliverable?

**The decision.** `proof.render()` emits the derivation as it happened: every rule that
fired, every condition that matched. The formal-XAI line argues that is the wrong object
to hand a human. Marques-Silva & Ignatiev, "No silver bullet: interpretable ML models must
be explained" (*Frontiers in Artificial Intelligence* 2023;6:1128212) hold that a
decision-tree path or rule chain is faithful but **not a minimal sufficient reason**, and
that answers read off "by human inspection of the interpretable ML model representation"
"can exhibit **arbitrary redundancy**"; the object they argue for is an *abductive
explanation* — a subset-minimal set of facts sufficient for the conclusion, computed with
formal guarantees. This asks whether Horismos should compute that, and if so where it
lives. **[measured — primary abstract]**

**Why evidence cannot settle it.** Both properties are real and they are not the same
property. Faithfulness is what makes a proof an audit record: it must say what *actually*
happened, redundancy included, or it stops being a record of the computation. Minimality is
what makes it a usable *reason*: the smallest set of facts whose truth would have forced the
same conclusion. The evidence says both matter and says nothing about which Horismos should
privilege, because that depends on who the artifact is for — and §6.3.2 found that the two
audiences (a regulator reconstructing a decision; a clinician deciding whether to override
one) genuinely want different things. GDPR C-203/22 ¶ 59 points the same way from the legal
side: "the detailed description of all the steps in automated decision-making" was held
**insufficient**, which is a holding against the full derivation as a delivered explanation.
Computing minimality is also not free — it is a search over subsets of the support set, and
Horismos's determinism rule means any such search must itself be deterministic and
order-independent.

**Options.** (a) Status quo — the full derivation is the artifact; minimality is the
narrator's problem, and doc 06 already says narration is "its presentation". (b) Add a
derived, additive-optional *minimal support set* alongside the full proof — the proof stays
the record, the minimal set becomes what narration and any UI render by default. (c) Make
minimality the contract: `proof.render()` returns the minimal sufficient reason and the full
derivation moves behind a verbose flag.

**Recommendation.** (b), and not soon. It preserves the audit record exactly as it is —
which is the property the trace schema's versioning policy protects — while giving narration
and any future review UI the object the formal-XAI literature and ¶ 59 both ask for. (c)
would break the "traces are output, not logging" principle by making the emitted object a
summary rather than a record. (a) is defensible today and becomes indefensible if
subset-minimality ever enters a standard; falsifier 4 in §6.3.3 is the trigger to revisit.

**Owner.** **PRP-002** (reasoning substrate — `reason/proof.rs` is its surface), with doc 06
text. Note this is the one open decision in this report that is *not* about the neural
boundary at all; it arrived through the auditability question and belongs to the symbolic
core.

---

**Not preempted, referenced only.** PRP-008 D1–D10; the RL deep dive's OD-1–OD-4
(environment/grader ownership, external publication, doc 09 rule 2, staged ontology
admission); sweep #2's A8 (narration conclusion monitor), A9 (utility tax), A10 (doc 15
threat list), A13/X10 (positioning vocabulary); and PRP-007's five open items.

---

## 10. Source log

**[measured]** Unlike the RL-axis deep dive — whose sources
[remain undistributed](index.md) and live only in its own §6 — this report's sources were
filed into the wiki the day it landed. The full log is
[`reading-log.md`](../reading-log.md) **§ "Q9 — Non-generative typed-decision models,
calibration and parallel decoding (Jev deep dive, 2026-09-21)"**: ~145 rows in five
groups (TypeSafe primary sources; adoption, independent evaluation and commentary;
calibration, uncertainty quantification and selective prediction; routing, cascades,
constrained decoding and parallel generation; and regulation, auditability and clinical
human factors), each with its date, the takeaway this report drew from it, its URL, and
the wiki page it was filed under. Q8 remains reserved
for the RL dive's own sources.

Repository facts cite `local repo` paths at [`adfc27a`](https://github.com/billaram/horismos/blob/adfc27a);
`file:line` references are only true at that commit.

### Tag counts

| | Count | Note |
|---|---|---|
| Distinct sources cited | ~145 in the Q9 reading-log section, plus the repository files listed in §1 | |
| `[measured]` / `[inference]` / `[unverified]` | the three tags appear on every substantive claim | `[measured]` is further split in §6.3 into *primary* (the document itself was read) and *secondary* (a reliable intermediary read it), because the regulatory material makes that distinction load-bearing |
| `[unverified]` items | 22 | Ten bibliographic-only confirmations (Brier 1950, Dawid 1982, both Gneiting papers, Chow 1970, Geifman & El-Yaniv 2017, Mozannar & Sontag 2020, Caruana 1997, MT-DNN, SetFit, Tian et al. 2023); the OpenAI Structured Outputs blog percentages (HTTP 403); the Bengio System-2 keynote attribution; the "Noul" etymology; TypeSafe's non-response to the prior-art claim; and eight from the regulatory sub-sweep (AI Act Art. 86's interim effect, the adopted CPPA ADMT text, the April 2026 Reg B scope, the Dutch DPA guidance's final status, the 2026 state health-AI law count, the NYC LL 144 enforcement numbers, the CHAI assurance-lab report, and Siala et al. on rule-set redundancy). None is upgraded anywhere in this report |
| Sub-agents | 5 | One each for Q-A, Q-C and Q-D3; two splitting Q-B. The Q-D3 agent ran ~41 minutes and exhausted a 200-call web-search budget; its incomplete searches are recorded as such in §8 |

### Methodology notes, including what went wrong

- **The intake facts were treated as adversarial input.** Each of the five sub-agents was
  given the brief's facts explicitly labelled `[unverified]` with instructions to confirm
  against a primary source or two independent secondary ones, and to report corrections.
  Fourteen were checked; four were wrong or materially misleading (§3.1). One — "Noul" —
  the brief flagged as probably garbled, and it turned out to be exactly right.
- **Two sub-agents independently read the same primary pages** (the launch blog and the
  eval dashboard) without coordination, which is how the access-model discrepancy in §3.1
  surfaced. It is recorded rather than resolved.
- **A sub-agent opened two arXiv papers cited on Hacker News as prior art and found that
  neither supports the claim made for it** (§5.3). The prior-art objection is nevertheless
  correct; §4 supplies the citations that do support it. This is the kind of check that
  only happens when abstracts are opened rather than titles trusted.
- **Environment limits are recorded in §8**, not hidden: Reddit and Lobste.rs were
  unreachable, X/Twitter was snippet-only, and OpenAI's 2024 Structured Outputs blog
  returned HTTP 403 to every attempt.
- **One claim in an early draft of this report was wrong and is corrected here rather than
  quietly fixed.** Before the primary-source sub-agent returned, §6.2.3 mapped Horismos's
  `xsd:decimal` properties onto Jev's **Score** primitive on the reasonable assumption
  that "Score (numeric/rubric)" meant numeric extraction. It does not: a Score is an
  ordered rubric of two to ten labelled levels, and TypeSafe's own documentation says
  "score levels are weak in numerical calibration" and "Jev is not a calculator". The
  corrected analysis — that numeric and string properties have **no** encoding in the
  primitives at all — is materially more negative, and it is the finding that most shapes
  §6.2 and §7.
- **A second claim was wrong, and it was the load-bearing one in §6.3.** An early draft of
  §6.3.1 stated that Ghassemi, Oakden-Rayner & Beam "is an argument against **the first
  row**" and that "any use of that paper against Horismos is a category error". That was
  written from the paper's abstract, because thelancet.com blocked every automated fetch
  and Europe PMC reported "Subscription required". The sub-sweep later obtained the full
  body text — the paper is CC-BY, and a headed browser session worked — and the body
  explicitly attacks inherently explainable models too, on information-overload and
  unrecognised-confounder grounds, naming prototype models as "generally considered to be
  a form of inherent explainability" that "still requires human interpretation". The claim
  is corrected in place, and §6.3.2(v) replaces it with an argument-by-argument
  decomposition: the paper's **fidelity** and **incompleteness** arguments do not survive
  against a derivation; its **human-factors** and **validation** arguments survive intact,
  and both now have clinician measurements behind them (Gomez et al. 2025 in particular,
  which isolates intrinsic from post-hoc explanation and finds anchoring bias in *both*
  arms). This is the single biggest change the sub-sweep forced, and it makes §6.3 a
  weaker and more defensible claim than the draft it replaces.
- **The lesson generalises and is worth stating as a rule for this corpus.** Twice in one
  report, a claim drafted from an abstract was materially wrong once the primary source was
  read, and in both cases the correction ran in the same direction — *less* favourable to
  Horismos. A paywall or a block is not a reason to reason from the abstract; it is a
  reason to mark the claim `[unverified]` until the document is read, or not to make it.
