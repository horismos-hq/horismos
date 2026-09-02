# Autonomous agents' failure modes: NELL, Wikidata governance, AutoSchemaKG, KARMA

## Claim

The field's longest-running real example of largely-autonomous knowledge extension
(NELL) shows concrete, documented schema drift; the largest real continuously-evolving
public knowledge graph (Wikidata) achieves reliability by the opposite mechanism —
mandatory human discussion before any schema-level change.

## Evidence

**[measured]** NELL (CMU, running continuously since 2010) needed human correction
"every few weeks," and Stuart Russell reports confidence in only 3% of its own
beliefs. NELL's own documented drift chain is the clearest concrete evidence of
unchecked schema drift found anywhere in this sweep: internet *cookies* miscategorized
as *baked goods*, then further deduced into *computer files* by pattern-matching
sentence structure
([en.wikipedia.org/wiki/Never-Ending_Language_Learning](https://en.wikipedia.org/wiki/Never-Ending_Language_Learning)).

**[measured]** Wikidata, the largest real, continuously-evolving public knowledge
graph, achieves reliability by the opposite mechanism: mandatory human community
discussion before any schema-level change, with only instance-level data left to bots
([en.wikipedia.org/wiki/Wikidata](https://en.wikipedia.org/wiki/Wikidata)).

**[measured]** AutoSchemaKG (2025, ACL 2026) performs fully autonomous web-scale
schema induction with no dedup mechanism, and admits residual inconsistency risk
([arXiv:2505.23628](https://arxiv.org/abs/2505.23628)). KARMA (2025) is a 9-agent
autonomous KG enrichment system that is confidence-gated but not systematically
human-reviewed ([arXiv:2502.06472](https://arxiv.org/abs/2502.06472)).

**[measured]** A genuine reasoner-in-loop precedent exists (a HermiT-in-loop
neuro-symbolic integration paper), but it was withdrawn by its own authors for
inaccuracies ([arXiv:2504.07640](https://arxiv.org/abs/2504.07640)). GLaMoR (2025)
replaces a symbolic OWL reasoner with a learned approximator — counter to horismos's
anti-wrapper stance ([arXiv:2504.19023](https://arxiv.org/abs/2504.19023)).

**[inference]** Horismos's existing design already has a structural answer to both
risks these examples raise: traces are one-directional output (never an input that
mutates the ontology), and no action ever fires off a raw prediction — only off
symbolically-checked preconditions. Where horismos is genuinely *behind* is the
learning axis: v1 has no path at all from accumulated execution traces back into
candidate ontology edits. See
`../intake/filed/2026-08-24-markovic-learnable-ontologies.md` for the fuller version
of this argument, worked through against a captain-supplied source.

## Sweep #2 delta (2026-09-02): the learnable-ontology thread, one sweep on

**[measured]** No new Markovic or Cognee material on learnable ontologies → world
models appeared after 2026-08-24. Cognee's in-window posts are memory-infrastructure
guides; the most relevant, "How to Build a Memory Graph for AI Agents" (2026-08-31),
advises "Automate merges only under a defined confidence policy … Leave ambiguous
references separate until more evidence or human review resolves them" and "Keep the
initial node and edge vocabulary small … Add schema elements only when a recurring
query requires them" ([cognee.ai](https://www.cognee.ai/blog/fundamentals/memory-graph-for-ai-agents)).
**[inference]** Cognee's own how-to guidance is gated schema growth — closer to sweep
#1's "proposal source into the gated loop" recommendation than to the auto-reinforcing
loop the Markovic piece sketched.

**[measured]** The counter-case is OaK ("ontology-as-a-kernel", 2026-08-24): a
task-oriented ontology and KG are built from task requirements and training data and
iteratively refined with an LLM judge (claude-sonnet-4.6) plus a HermiT consistency
check, with no human review mentioned; once frozen, "the kernel is the only channel
through which the agent reaches the data. It cannot name a concept or invoke a
computation the kernel does not declare." Reported: TravelPlanner 59.31% vs ReAct
44.29%, CRMArenaPro 78.38% vs MemP 66.44%, ToolQA 64.58% vs 55.02%
([arXiv:2608.22974](https://arxiv.org/abs/2608.22974)). ASKS lets a schema/hub layer
emerge from embedding geometry and controls drift by measuring churn, not by a gate
([arXiv:2608.29612](https://arxiv.org/abs/2608.29612); see
`kg-maintenance-and-agent-memory.md`). **[measured]** The security counterweight: on a
205-entity medical KG, targeted poisoning reaches break-even symbolic-integrity
violation at an injection budget of 5, and "single-axiom OWL edits achieve 93.3% SIV
success with 100% Pellet-consistency stealth," with STIX-based detection at 50% —
chance ([arXiv:2606.17223](https://arxiv.org/abs/2606.17223)).

**[measured]** Gated designs on the other side: GrOIL produces an OWL TBox "without any
unconstrained generation step" — LLM choices restricted to closed vocabularies by
guided decoding, a fully deterministic compilation stage, and "low-confidence
assignments … flagged automatically for human review and … not committed to the TBox
until inspected"; competency-question coverage 0.85 vs 0.63/0.62 for direct/multi-agent
LLM drafting ([arXiv:2608.22135](https://arxiv.org/abs/2608.22135)). PolicyKG translates
institutional policy to deontic FOL and SHACL with 86.9% deontic classification and
shape F1 .866 in-domain but macro F1 .370 on an out-of-domain lease benchmark
([arXiv:2608.09028](https://arxiv.org/abs/2608.09028)). Practitioner framing converges:
Packkildurai — new relationships are "hypotheses … separate from verified edges" with
provenance and lifecycle states
([dataengineeringweekly.com](https://www.dataengineeringweekly.com/p/an-ontology-for-ai-agents-is-a-system));
KGC 2026 — "The ontology stops being the output of the LLM and becomes the harness
around it," with a seven-step method ending in an explicit Release step
([Futia, Medium](https://medium.com/@giuseppefutia/notes-from-kgc-2026-c9b4ac8569e5)).
Palantir now ships ontology-as-code ("SuperRepo … one versioned artifact," 2026-08-04)
and its documented agent pattern is "Agents propose; they do not execute directly," with
"DecisionReceipt" objects as first-class records
([Palantir announcements](https://www.palantir.com/docs/foundry/announcements/2026-08);
[Huang & Gupta](https://kenhuangus.substack.com/p/a-technical-follow-up-building-an)).

**[inference]** Two consequences for PRP-007's gate, filed as item 4 in
`prp-007-open-items.md`: the gate must be argued on safety (stealth-consistent edits,
drift) rather than accuracy, since OaK and ASKS show accuracy gains without one; and the
verifier's consistency pass is necessary but not sufficient — a Pellet/HermiT-consistent
edit can flip downstream conclusions 93% of the time. The sweep #1 recommendation
(trace-mining as a proposal source, never a self-reinforcing loop) stands and is now
corroborated by Cognee's own guidance and by GrOIL.

**[measured — negative result of this sweep's queries]** Searched but not found:
"self-evolving ontology" / "self-defining ontology" / agent
schema evolution as explicit terms — no paper; nearest is OaK. No new Ken Huang
ontology piece in window. No Palantir positioning material after 2026-08-24 beyond
product notes.

## Source

See `../reading-log.md`, Q6 section.

## Confidence

mixed — case facts are [measured]; the "horismos already has a structural answer"
framing is [inference].

## Last-verified

2026-09-02 (sweep #2 delta appended; founding claims unchanged)
