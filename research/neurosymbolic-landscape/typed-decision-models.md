# Non-generative typed-decision models ("System One models")

<!-- Founded by the Jev deep dive (task horismos-research-jev-s1, 2026-09-21). New topic
     page for an axis the corpus had no home for: commercial, non-generative models that
     answer typed questions declared in advance. Deltas append; founding text is not
     rewritten. -->

## Claim

A commercial category appeared on 2026-09-15 that concedes Horismos's first principle —
decisions should be **typed values in a space declared before the data arrives**, not
free-generated text — and stops short of everything downstream of it. The category has
no new science in it: every load-bearing component has a citation between 1950 and July
2025, and its own vendor documents the two properties that matter most to Horismos as
limitations. Its arrival is **validation of the premise, a complement in the
architecture, and a threat only to the part of Horismos's positioning that rested on
"typed decisions" being differentiating**. The durable differentiators the corpus should
carry instead are **cross-answer consistency** and **epistemic (not merely structural)
auditability**.

## Evidence

### What the category is

**[measured]** TypeSafe AI (founded 2024, San Francisco; three co-founders — Diogo
Almeida CEO, Erik Gafni CTO, Sasha Sheng COO; $40M seed led by DCVC announced
2026-09-15) opened early access to **Jev** on 2026-09-15 after "two years in stealth",
marketing it as the first "System One model"
([typesafe.ai/team](https://typesafe.ai/team);
[launch blog](https://typesafe.ai/blog/introducing-system-one-models-and-jev);
[DCVC](https://www.dcvc.com/news-insights/typesafe-emerges-from-stealth-with-a-new-way-of-doing-ai/)).

**[measured]** The whole interface is one endpoint taking a `state` string plus a
`questions` map, both in the same request body — there is no schema registry and no
stored template. Three primitives: **Choice** (categorical, max 255 options, returns a
probability map plus a `confidence`), **Score** (an *ordered rubric of 2–10 level
descriptions*, returning a probability-weighted value that can land between levels), and
**Noul** (yes/no, returning a single 0–1 number and **no `confidence` field**). Context
is 64k tokens per request; input is **text only**; there is **no tool-calling surface**
([docs/api](https://docs.typesafe.ai/api),
[docs/primitives](https://docs.typesafe.ai/primitives)). Pricing is **$0.042/MTok input,
output counted but unbilled** ([docs/models](https://docs.typesafe.ai/models)).

**[measured]** Training is "RLCD — Reinforcement Learning for Calibrated Decisions",
pitched against RLHF and RLVR as optimizing *"Calibrated decisions: answers with
epistemically honest probabilities"*, with the caveat that calibration describes *"groups
of predictions, not a guarantee about any single answer"*
([ML primer](https://docs.typesafe.ai/introduction/machine-learning-primer)). There is no
paper, no technical report, no scientific model card, and no disclosed parameter count or
architecture.

### The literature already owns every component

**[measured]** Multi-task hard parameter sharing — one encoder, many heads — is Caruana
(1997) **[unverified — secondary]** and MT-DNN (ACL 2019,
[arXiv:1901.11504](https://arxiv.org/abs/1901.11504)) **[unverified]**. Schema-declared
typed classification is Yin, Hay & Roth (EMNLP 2019,
[arXiv:1909.00161](https://arxiv.org/abs/1909.00161)). Prompt-free small-model
classification is SetFit ([arXiv:2209.11055](https://arxiv.org/abs/2209.11055))
**[unverified]**. Proper scoring rules are Brier (1950) and Gneiting & Raftery (2007)
**[unverified]**; the calibration fix is temperature scaling, Guo et al., ICML 2017
([arXiv:1706.04599](https://arxiv.org/abs/1706.04599)). Confidence-driven deferral is
FrugalGPT ([arXiv:2305.05176](https://arxiv.org/abs/2305.05176)) and AutoMix
([arXiv:2310.12963](https://arxiv.org/abs/2310.12963)); *training the small tier so its
probabilities route well* is Cascade-Aware Training
([arXiv:2406.00060](https://arxiv.org/abs/2406.00060), Google, 2024).

**[measured]** Calibration-rewarded RL has a canonical published method one letter away:
**RLCR — "Reinforcement Learning with Calibration Rewards"** (Damani et al.,
[arXiv:2507.16806](https://arxiv.org/abs/2507.16806), ICLR 2026), which augments the RL
reward with a Brier score and **outperforms both ordinary RL and post-hoc confidence
classifiers**. A separate 2023 paper already used the acronym RLCD for "RL from
Contrastive Distillation" ([arXiv:2307.12950](https://arxiv.org/abs/2307.12950), ICLR
2024). Findings of ACL 2026 carries a third close relative, reporting **ECE reduced by up
to 9 points** ([aclanthology 2026.findings-acl.610](https://aclanthology.org/2026.findings-acl.610/)).

**[measured]** The encoder-beats-decoder-on-classification result is settled with matched
recipes: Ettin ([arXiv:2507.11412](https://arxiv.org/abs/2507.11412), ICLR 2026) — "a
400M encoder outperforms a 1B decoder on MNLI"; encoders at "one to two orders of
magnitude lower cost and latency" ([arXiv:2602.06370](https://arxiv.org/abs/2602.06370));
and Ettin-150M at **3.60 ms median latency** against LlamaGuard-4-12B's 717.63 ms, with
better out-of-distribution F1 ([arXiv:2606.25782](https://arxiv.org/abs/2606.25782)).

**[inference]** Measured against the correct baseline — a fine-tuned encoder, not a
reasoning LLM — the hosted option is roughly **20–140× slower** and **~32× more
expensive per case** ($0.0004 against DistilBERT's $0.00001244). What it genuinely sells
is the removal of the fine-tuning pipeline.

### The two properties that matter to Horismos, both vendor-documented

**[measured] Answers are not jointly constrained.** TypeSafe's "jaggedness" page lists
"common-sense structural invariants" as a failure mode with two examples: the same
proposition asked as a Noul returned **0.22** while the equivalent Choice returned
**P(yes) = 0.01**, and a claim and its negation asked as two Nouls **summed to 1.19**.
The guidance is explicit: *"don't hold the model to arithmetic identities between
separate questions"*
([docs/model-jaggedness/jev-1.13](https://docs.typesafe.ai/model-jaggedness/jev-1.13)).
This is the predicted consequence of answering N questions in one conditionally
independent pass — the same property ParallelBench measures for token-level parallel
decoding ([arXiv:2510.04767](https://arxiv.org/abs/2510.04767)).

**[measured] There is no explanation, and the vendor says so.** *"System One models do
not write replies, produce code, or generate explanations of their reasoning"*
([docs/concepts/system-one](https://docs.typesafe.ai/concepts/system-one)). The scalar
`confidence` is *"a statistic computed from the probability distribution"* whose formula
is unpublished ([docs/confidence](https://docs.typesafe.ai/confidence)).

**[measured]** And the headline guarantee is explicitly non-empirical. Under the "0%
structured-output error" chart: *"Our number is not empirical. Schema matching is
guaranteed, thus we can confidently add 0% into the plots."* The FAQ concedes the rest:
*"Jev guarantees the shape of its answers, not that every decision is correct … it can
choose the wrong one."*

### What Horismos already specifies, for comparison

**[measured — local repo]** at `adfc27a`:

- The neural surface is one trait with three working methods — `perceive`, `narrate`,
  `rank` ([`crates/horismos-core/src/bridge/mod.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/bridge/mod.rs):23-41) —
  routed by model-string prefix through a `BTreeMap<String, Arc<dyn Bridge>>`
  ([`workflow/agent.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/agent.rs):165, 207), with
  `hf:` and `onnx:` already reserved for "classification-style perception without a
  frontier model" ([`docs/framework/08-bridge.md`](https://github.com/billaram/horismos/blob/adfc27a/docs/framework/08-bridge.md)).
- `rank`'s signature is a categorical decision over a closed, runtime-computed candidate
  set returning `chosen` plus a confidence — the same shape as a Choice question — but
  **`ProposalStep` does not execute in v1**: `StepBody::Proposal(_) => Err(…)`
  ([`workflow/agent.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/agent.rs):454)
  and "nothing in this PRP calls `Bridge::rank`"
  ([`workflow/deferred.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/deferred.rs):5-7).
- Acceptance is `accepted = result.confidence >= p.confidence`
  ([`workflow/agent.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/agent.rs):492)
  over a number the model **self-reports** in a tool-call field
  ([`crates/horismos-bridges/src/openrouter.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-bridges/src/openrouter.rs):322-331);
  doc 08 states this is "one calibration strategy for every provider" with logprob-derived confidence "a future
  per-backend enhancement".
- Narration falls back to the mechanical `proof.render()` on any bridge error
  ([`workflow/agent.rs`](https://github.com/billaram/horismos/blob/adfc27a/crates/horismos-core/src/workflow/agent.rs):814-822, D7).
- Doc 06 states the invariant the whole comparison turns on: confidence is "metadata, not
  truth: it gates acceptance … it never weights inference. There is no fuzzy logic in
  Horismos."

**[inference]** Three consequences. First, the mechanical cost of adding a typed-decision
bridge is near zero, and the honest slot is `Bridge`, not the PRP-008 `HostAdapter` —
doc 08 already rules that an Adapter "is *not* a Bridge". Second, the primitives are
**strictly less expressive** than what doc 06 derives: Choice covers closed
`horis:oneOf`/subclass enums and Noul covers `xsd:boolean`, but **Score is an ordinal
rubric, not a numeric extractor**, so `xsd:decimal` with `horis:min`/`max`/`unit`,
`horis:pattern`, and every string-valued property have no encoding at all. Third, the
gate's five checks — class, domain/range, constraints, cardinality/immutability,
provenance policy — include joint and stateful constraints that no typed-question schema
expresses; the decomposition makes visible how much of the boundary was never the
proposer's job.

### How the category is actually doing (six days in)

**[measured]** Vercel's own engineering post (2026-09-18) reports Jev "reached more than
twice as many paid teams as any previous model launch, making it the fastest-adopted
model in gateway history" — ~13% of paid teams by hour 24 — while flagging that "the next
test is whether that early adoption lasts"
([vercel.com](https://vercel.com/blog/ai-gateway-jev-model-launch)). On TypeSafe's own
four-workflow dashboard Jev is **fourth of nine on accuracy** (67.8%, behind 74.1%, 73.1%
and 67.9%), at $0.0004/case and 0.4 s ([evals.typesafe.ai](https://evals.typesafe.ai/)) — and
"accuracy" there means agreement with a consensus of two frontier models, not ground
truth. A third-party analysis of 12,759 launch posts puts user-reported medians at **7×
faster and 30× cheaper** against vendor claims of 193.6× and 444.6×
([openchamber.dev](https://openchamber.dev/blog/jev-typesafe-ai/)). The launch itself was
a documented paid distribution campaign — 37.2M post views, 80–100 accounts seeded in
hour one ([doomers.ai case study](https://doomers.ai/work/typesafe-ai-case-study)).

**[measured]** The most rigorous independent test in the first week — pre-registered,
with published raw data — found **0 of 30 out-of-scope inputs flagged** and **16.7%
accuracy, below the 25% random floor, when the question descriptions were wrong**
([priorbench/jev](https://github.com/priorbench/jev)). A separate hands-on run found ECE
rising **0.040 → 0.116** as questions became longer and more carefully qualified
([lindfors.no](https://lindfors.no/blog/a-first-look-at-typesafes-jev/)).

**[inference]** Too early to tell commercially; not too early on three points. The
latency and cost advantages are real and independently reproduced. The
frontier-intelligence framing is falsified by the vendor's own dashboard. And the failure
modes found in six days — confidently wrong type-valid answers, no refusal capability,
below-chance behaviour on a mis-specified question — are the failure class "neural
proposes, symbolic decides" exists to contain.

### The clinical literature is already building the layer above this category

**[measured — primary abstract]** Gorenshtein A., Omar M., Barash Y., Nadkarni G. N. &
Klang E., "Neuro-symbolic LLM Integration in Clinical Medicine: A Systematic Review and
Taxonomy" (Research Square rs.3.rs-8902022, 2026-02-18, PMID 41756413) screened 3,166
records to 21 studies and sorted the integration patterns "by **increasing symbolic
authority**: structured output, rule-guided generation, knowledge retrieval, and iterative
validation (**symbolic veto or regeneration**)". Reported improvements "ranged from +3.1%
to +125.6% (median +21.9%) and **increased with symbolic authority (median gains: 9%, 16%,
26%, 40%)**".

**[inference]** That is the closest thing in the published literature to an empirical
statement of Horismos's first principle — *the more authority the symbolic layer has over
the neural layer's output, the better the system performs* — and it places a typed-decision
model at the **lowest-authority rung** of its own taxonomy ("structured output"), with the
largest measured gains at the rung Horismos occupies. The caveat belongs in the same
breath: the review rates **all 21 studies at high risk of bias**, only 9 of 21 report
external validation, and the highest-authority pattern carries "2–88 s latency and up to
100× cost increases".

**[measured — primary abstracts]** Two individual results in the same direction, both with
authors employed by or founding the commercialising vendor, which is worth saying out loud:
Prenosil et al., *Communications Medicine* 2025;5:491 (206 PET/CT reports, 5,356 data
points; neuro-symbolic F1 1.00/1.00 against GPT-4's 0.63/0.95, and it "delivers always an
auditable chain of reasoning"); and Loaiza-Bonilla et al., *ESMO Real World Data and Digital
Oncology* 2026;12:100706 — **prospective**, 3,804 consecutive patients, dual-oncologist gold
standard κ = 0.92, neuro-symbolic F1 **0.82** against GPT-4 zero-shot 0.47 and
chain-of-thought 0.67, with "ontology-grounded, deterministic eligibility reasoning … while
preserving clinician oversight". A third, Xu et al.
([arXiv:2606.16337](https://arxiv.org/abs/2606.16337)), builds "an executable rule-based
expert system" from LLM-proposed heuristics because "the decision logic is explicit,
interpretable, and auditable".

**[inference]** The reading for this page: the neural-proposes/symbolic-decides shape is
not a Horismos idiosyncrasy and it is not unvalidated — it is a pattern with a published
taxonomy, a dose-response curve, and three 2025–26 clinical instances. What none of them
has is a *declarative* substrate (they hand-build the symbolic layer per study), formal
verification of the rule set, or a versioned trace. That is the gap, and it is a narrower
and more credible gap than "nobody does this".

### What this means for the corpus's positioning

**[inference]** Three statements the corpus can now make that it could not before:

1. **"Typed decisions" is no longer a differentiator.** It is a market-validated premise,
   defended by a funded company. Horismos should stop spending positioning budget there.
2. **Cross-answer consistency is a differentiator, and the evidence is the competitor's
   own documentation.** A rule layer evaluated to a fixed point, with priority-ordered
   conflict resolution and joint cardinality/immutability constraints, is precisely the
   guarantee a conditionally independent parallel pass documents itself as lacking.
3. **The auditability distinction to use is structural versus epistemic.** A typed
   decision model gives structural auditability — which question, which state, which
   threshold, which branch, all in code you can diff. A proof tree gives epistemic
   auditability — *why this answer*, as the actual chain that produced it rather than a
   post-hoc rationalization. Only the second survives the question "on what basis?".

**[inference]** One caution the corpus must apply to itself: the complaint that "you
cannot audit a threshold whose definition you cannot see" lands on Horismos too.
`horis:confidence` is thresholded against a self-reported, unmeasured number. That is a
defect in something shipped, and the deep dive ranks fixing it first.

## Source

Full citations inline. The complete evidence base, sub-agent methodology, worked
examples and the ranked moves are in the
[Jev deep dive report](../reports/2026-09-21-typesafe-jev-deep-dive.md); its sources are
logged under Q9 in the [reading log](../reading-log.md). Repo facts cite `local repo`
paths at `adfc27a`.

## Confidence

mixed — TypeSafe's interface, pricing, documented limitations and benchmark methodology
are [measured] from primary pages; the literature citations are [measured] where the
abstract was opened and [unverified] where only a snippet was seen; the adoption and
skeptical evidence is [measured] from primary posts and threads; every "means for
Horismos" statement, the three positioning claims and the verdict are [inference].

## Last-verified

2026-09-21
