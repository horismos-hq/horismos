# Q3: neuro-symbolic AI landscape — overview

## Claim

The academic field that currently owns the term "neurosymbolic AI" (Scallop, Logical
Neural Networks, DeepProbLog) is unified by making symbolic structure differentiable
and probabilistically weighted — the opposite of horismos's hard, non-differentiable
gate. Industry offers no ready-made competitor: no major clinical AI vendor publicly
documents a formal/symbolic verification layer, and "genuine pre-execution formal
gating of LLM output, applied uniformly, in a shipping product" is a real, currently
open gap horismos could fill.

## Evidence

**[measured]** Scallop (UPenn, [arXiv:2304.04812](https://arxiv.org/abs/2304.04812)),
Logical Neural Networks (MIT-IBM,
[arXiv:2006.13155](https://arxiv.org/abs/2006.13155), reference implementation at
[github.com/IBM/LNN](https://github.com/IBM/LNN)), and DeepProbLog/ProbLog (KU Leuven,
[Manhaeve et al. 2018](https://arxiv.org/abs/1805.10872), actively funded through an
ERC Advanced Grant awarded 2023) all make symbolic structure differentiable and
probabilistically weighted. A unifying survey is Marra, Dumančić, Manhaeve, De Raedt,
*Artificial Intelligence* 328 (2024,
https://www.sciencedirect.com/science/article/pii/S0004370223002084).

**[measured]** Horismos deliberately does the opposite: confidence gates *acceptance*,
never *weights inference*, and there is no fuzzy logic anywhere in the reasoner (per
[`docs/framework/06-llm-boundaries.md`](https://github.com/billaram/horismos/blob/main/docs/framework/06-llm-boundaries.md)). By the standard the field's own most-cited
reference systems set, a hard, non-differentiable wall between neural and symbolic
components is a **looser** claim to the "neurosymbolic" label than what
Scallop/LNN/DeepProbLog actually do. Wikipedia's own overview of neuro-symbolic AI
contains no mention of MYCIN, Arden Syntax, or classical expert systems at all
([en.wikipedia.org/wiki/Neuro-symbolic_AI](https://en.wikipedia.org/wiki/Neuro-symbolic_AI))
— the field does not see itself as continuous with that lineage. See
[`lineage-assessment.md`](lineage-assessment.md) for the full positioning analysis.

**[measured]** Concrete evidence for why the hard-gate design matters: a Mount Sinai
adversarial study planted a single fake lab value, sign, or diagnosis into 300
clinical vignettes; six leading LLMs repeated or elaborated the planted error in
50–83% of cases, and prompt engineering reduced but did not eliminate this
(*Communications Medicine*, Aug 2025,
[nature.com/articles/s43856-025-01021-3](https://www.nature.com/articles/s43856-025-01021-3)).
One genuine, deployed, quantified precedent for symbolic gating beating an ungated LLM
baseline: an oncology clinical-trial-matching system where LLM-extracted patient
features must pass explicit graph-enforced numeric/Boolean/temporal consistency checks
before a match surfaces, reporting F1 0.82 versus 0.47 (GPT-4 zero-shot) / 0.67 (GPT-4
chain-of-thought) on a prospective 3,804-patient evaluation
([PMC13091143](https://pmc.ncbi.nlm.nih.gov/articles/PMC13091143/)). **No head-to-head
"hard-gated ontology system vs. RAG-grounded LLM" clinical trial exists anywhere in the
literature surveyed** — horismos's safety argument is currently architectural, not yet
measured, for itself specifically.

**[inference]** Industry offers no ready-made competitor to imitate or fear here: none
of the major clinical AI scribe/agent vendors surveyed (Abridge, Ambience Healthcare,
Nuance DAX, Epic's own AI features) publicly document a formal or symbolic
verification layer — all rely on retrieval/traceability plus human sign-off as the
actual safety mechanism. No company surveyed brands itself "neurosymbolic clinical AI"
or "ontology-grounded clinical agent" as a core identity. Among generic LLM agent
frameworks (LangGraph, Google ADK, OpenAI Agents SDK, Microsoft Agent Framework,
CrewAI), only Google ADK draws a genuine hard line — "Workflow Agents" execute without
consulting a model — but only over fixed orchestration scaffolding, never over an LLM
node's semantic content; OpenAI's own Guardrails docs confirm guardrails are soft and
post-hoc (tools "may have already... executed" before a tripwire cancels anything,
[openai.github.io](https://openai.github.io/openai-agents-python/guardrails/)).
Apollo-1/AUI markets itself as "the first neuro-symbolic model" with GA "scheduled for
Q2 2026," but no published technical specification, benchmark, or proof of an actual
hard-gate mechanism was found for it ([aui.io](https://www.aui.io)) — a genuine
example, in a system horismos already names as a comparator, of "neurosymbolic" used
as positioning language without documentation. **This gap — genuine pre-execution
formal gating of LLM output, applied uniformly, in a shipping product — is real and
currently open.** Horismos is not competing against an incumbent here; it's racing
absence.

**[measured]** RAG (retrieval-augmented generation) is the actual mechanism most
industry players lean on: a JAMIA meta-analysis found pooled OR 1.35 favoring RAG in
biomedicine, though hallucination reduction is not consistently quantified
([academic.oup.com](https://academic.oup.com/jamia/article/32/4/605/7954485)); a 2026
medRxiv preprint found RAG raised unsupported-claim rates from 5.0% to 43.6% in one
setting
([medrxiv.org](https://www.medrxiv.org/content/10.64898/2026.02.13.26346256v1.full)) —
evidence RAG alone is not a safety mechanism equivalent to a hard gate.

## Sweep #2 delta (2026-09-02)

Two pages added this sweep: [`pivot-alignment.md`](pivot-alignment.md) — the standing assessment of the
bounded-projection pivot against the field (verdict: **aligned-with-adjustments**) —
and [`coding-agent-harnesses.md`](coding-agent-harnesses.md) — how Claude Code, Codex, Gemini CLI, Cursor, pi,
DeepSeek Harness and MCP structure capabilities and policies.

**[measured]** Industry motion since 2026-08-24 bearing on the "racing absence" framing
above: Microsoft GraphRAG is "largely in maintenance mode" (3.1.2, 2026-08-21,
[pypi](https://pypi.org/project/graphrag/)); Epic's UGM 2026 introduced Agent Factory
("more than 120 pre-built AI agents, customize them to organizational policy … without
writing code," broader availability 2027) and Ergo (November 2026), with no third-party
decision-service, MCP or CDS Hooks surface mentioned in any coverage
([HealthTech HotSpot](https://healthtechhotspot.com/epic-ugm-2026-recap-cosmos-curiosity-epicops-and-an-expanding-ai-footprint/);
[Advisory Board](https://www.advisory.com/daily-briefing/2026/08/26/epic-ugm-ab-oi-ec));
Oracle Health's 2026-08-19 expansion says its agents "use semantic reasoning to
understand clinical meaning" with no FHIR or third-party surface
([oracle.com](https://www.oracle.com/news/announcement/oracle-health-expands-clinical-ai-agent-with-coding-dictation-chart-review-2026-08-19/));
MEDITECH's last AI announcement is 2026-03-10
([meditech.com](https://ehr.meditech.com/news/meditech-s-latest-ai-innovations-support-smarter-care-and-stronger-performance));
no major EHR exposes a first-party MCP server ([eureka.md](https://eureka.md/docs/ehr-mcp-servers)).
The coding-harness approval step is moving to model classifiers (Codex `auto_review`,
Claude Code `auto` mode, Cursor Auto-review — [`coding-agent-harnesses.md`](coding-agent-harnesses.md)). Against
that, Epic put Da Vinci CRD hook-triggered third-party decision services into production
on 2026-08-17 ([`../formalization/cql-cds-hooks-fhir.md`](../formalization/cql-cds-hooks-fhir.md), sweep #2 delta). Naming
collision: OaK (2026-08-24) calls its frozen task ontology "the kernel" — "the only
channel through which the agent reaches the data"
([arXiv:2608.22974](https://arxiv.org/abs/2608.22974)).

**[inference]** The gap named above — pre-execution formal gating of LLM output in a
shipping product — is still open, and the incumbents are moving toward classifier-based
approval rather than filling it. The only durable third-party clinical decision surface
remains CDS Hooks + SMART + FHIR REST, which is what the pivot targets. See
[`pivot-alignment.md`](pivot-alignment.md) §X1, X6, X10.

## Source

See [`../reading-log.md`](../reading-log.md), Q3 section, for the full citation list.

## Confidence

mixed — the academic field's own definitional posture and the industry survey are
[measured]; the "racing absence" framing and lack-of-competitor conclusion are
[inference].

## Last-verified

2026-09-02 (sweep #2 delta appended; founding claims unchanged)
