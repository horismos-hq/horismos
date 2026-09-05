# KG maintenance, agent memory graphs, and write-path control (sweep #2)

## Claim

**[inference — summary of the tagged evidence below]** Between June and September 2026
the agent-memory and knowledge-graph communities
converged, from a different starting point, on three of Horismos's commitments: a
gate at admission rather than "accept now, clean later"; no language-model judge on
the write path; and versioned, audit-preserving invalidation instead of overwriting.
The same literature also names Horismos's differentiator by contrast — the leading
memory vendor explicitly chooses availability over consistency, which is the right
trade for memory and the wrong one for a decision kernel. KG-as-retrieval-index
(GraphRAG) is frozen in maintenance mode while activity has moved to memory and
decision substrates.

## Evidence

### Admission gates and no judge on the write path

**[measured]** TOKI types the four production contradiction-resolution heuristics as
bitemporal operators and finds "every baseline that keeps a language-model judge on the
write path admits at least one of three write-time anomalies (replay inconsistency,
belief-drift skew, audit erasure); a content-addressed engine-layer comparator avoids
them only by removing the judge" ([arXiv:2606.06240](https://arxiv.org/abs/2606.06240)).
Quipu argues that KG stores' human-curation defaults ("accept writes now and clean
later," one time axis, equal trust for all writers) are "jointly untenable under agent
workloads"; in Quipu "no fact enters except through a gate whose predicates evaluate the
pending post-state," and gated writes caught 6/6 planted defects vs 0/6 ungated
([arXiv:2608.16813](https://arxiv.org/abs/2608.16813)). MemStrata shows cosine
similarity distinguishes a contradicted fact from a duplicate at AUROC 0.59 (near
chance), while a deterministic (subject, relation, object) supersession rule "with no
similarity threshold and no LLM call" cuts served-stale-value rates from 15–40% to ~0%
at ~2.1s vs ~16–18s for LLM re-ranking ([arXiv:2606.26511](https://arxiv.org/abs/2606.26511)).
A KG-validation benchmark (EDBT'26) finds LLM validators "lack sufficient stability"
([arXiv:2602.10748](https://arxiv.org/abs/2602.10748)). "Control-Plane Placement Shapes
Forgetting" tests 13 configurations over 385 adversarial cases and finds deterministic
mutation-time hooks score best (91.7–93.2%) ([arXiv:2606.15903](https://arxiv.org/abs/2606.15903)).

**[measured]** lemmalog (2026-08-27; 256 GitHub stars within a week) is a Rust
Datalog engine positioned as "an agent's memory should be a deductive database": facts
"extracted at the LLM boundary," stratified negation, seminaive fixpoint, bitemporal
validity, provenance as semiring annotations, "scoped negative deltas for retraction —
only transitive dependents recompute," and an MCP server; its README lists regression
tests for retraction bugs ("swap_remove corruption", "predicate-granularity
recompute") ([github.com/JordyZomer/lemmalog](https://github.com/JordyZomer/lemmalog)).
**[inference]** This is the nearest prior art for the truth-maintenance ruling in
[`docs/framework/03-world.md`](https://github.com/billaram/horismos/blob/main/docs/framework/03-world.md): lemmalog chose incremental scoped retraction and its own
bug list documents the class Horismos's simpler reset-and-rechain avoids.

### Versioned, audit-preserving invalidation — and the consistency trade

**[measured]** Zep's Graphiti gives every edge four timestamps and, on contradiction,
marks "the old fact invalid using an `invalid_at` date instead of overwriting it, so
history is preserved for audit"; snapshots are checksummed. Its 2026-08-27 service
post states the consistency model plainly: "Konig favors availability over strong
consistency. It is eventually consistent across the fleet and relies on idempotency
instead of exactly-once semantics." Graphiti 0.29.x added "attribute-hallucination
guards" and a 250-character cap on LLM-extracted string attributes; no schema-
validation framework is mentioned ([blog.getzep.com](https://blog.getzep.com/why-we-built-a-graph-database-service-for-agent-memory/);
[releases](https://github.com/getzep/graphiti/releases)). Agent Zero Memory (2026-08-30)
carries "origin, timestamp, and evidence pointer" per learned item and reads under a
"citation lock" ([arXiv:2608.29606](https://arxiv.org/abs/2608.29606)). IncRML adds
explicit/implicit/tombstone change detection for RML materialization across versions
([Semantic Web Journal 2026, doi:10.1177/22104968251412270](https://doi.org/10.1177/22104968251412270)).
**[inference]** Availability-first is correct for memory and wrong for a decision
kernel; Horismos's fail-closed conditional commit is the differentiator, and Graphiti's
post-hoc string caps are what an ad hoc gate looks like next to a typed five-check gate.

### Concurrency: the failure class is now named

**[measured]** A position paper argues "many MAS failures are fundamentally concurrency
control problems … long LLM inference windows amplify the risk of stale reads, lost
updates, and inconsistent outcomes" ([arXiv:2608.18092](https://arxiv.org/abs/2608.18092)).
CoAgent counters that locks and OCC fit agents poorly ("read sets are broad and opaque
rather than statically inferable"; "OCC abort-and-retry discards minutes of work") and
lets the LLM judge whether a conflicting write invalidates its plan, with saga-style
inverses ([arXiv:2606.15376](https://arxiv.org/abs/2606.15376)). "Selection Integrity"
shows provenance checks are necessary but insufficient: an untrusted party who can add
graph structure changes "which authenticated facts are selected" without forging any
record (28 irreversible financial transfers across 499 actions); the fix recomputes
selection only over the authenticated subgraph
([arXiv:2606.12290](https://arxiv.org/abs/2606.12290)). **[inference]** CoAgent's
premise does not hold under a declared input closure — the read set is statically
known, so version-checked commit is cheap — and Selection Integrity's fix is the
declared-closure, all-or-nothing gate by another name; retrieval-ranked selection over a
mixed-trust graph is an attack surface a snapshot-scoped World does not have.

### Schema induction without a gate, and its lessons

**[measured]** ASKS compiles scientific papers into a KG where "an LLM produces a
readable Wiki view and machine-facing semantics. Deterministic checks convert the
latter into a document-local GraphDelta, and embedding geometry together with explicit
graph rules integrates the proposed changes into persistent state"; over 56 papers the
hub layer "remains stable and low-churn" with growth "predominantly additive"
([arXiv:2608.29612](https://arxiv.org/abs/2608.29612)). SymbolLKG treats "logical rules
and constraints as first-class topological nodes" extracted from text and routed to
symbolic solvers ([arXiv:2608.26836](https://arxiv.org/abs/2608.26836)). For QA over
policy documents, "an open, LLM-discovered schema matches or exceeds the formal
ontology" across 5 LLMs and 42 tasks ([arXiv:2604.27713](https://arxiv.org/abs/2604.27713)).
In industrial asset operations, LLM agents over flat documents scored 65%, LLM-generated
Cypher over a typed KG 82–83%, and "native graph and optimization primitives, with no
LLM" 99% — with LLM-derived facts admitted but tagged `source:LLM-derived`
([arXiv:2605.26874](https://arxiv.org/abs/2605.26874)). Practitioner guidance: KGC 2026 notes — "most production failures
attributed to 'model limitations' or 'prompt engineering' are actually representation
failures" and the ontology "becomes the harness around" the LLM
([Futia, Medium](https://medium.com/@giuseppefutia/notes-from-kgc-2026-c9b4ac8569e5)).

**[inference]** Two lessons for the gated loop in [`docs/framework/13-authoring.md`](https://github.com/billaram/horismos/blob/main/docs/framework/13-authoring.md): the
schema-level gains reported without a gate are QA gains, not rule-execution gains (the
industrial paper's 99% comes from the no-LLM path); and ASKS's churn/lineage metrics are
drift instrumentation Horismos should adopt regardless of keeping the human gate. The
industrial paper also exposes the missing control in its own design — LLM-derived facts
are tagged but admitted unconditionally, which is exactly the provenance-policy check
in Horismos's gate.

### KG-as-retrieval-index has frozen

**[measured]** Microsoft GraphRAG 3.1.2 (PyPI, 2026-08-21) ships under a README stating
the project is "largely in maintenance mode, and won't be accepting new PRs or
implementing new features"; 3.1.x changes are infrastructure only
([pypi.org/project/graphrag](https://pypi.org/project/graphrag/);
[github.com/microsoft/graphrag](https://github.com/microsoft/graphrag)). No HippoRAG
release since HippoRAG 2 (ICML 2025); no Mem0 paper since arXiv:2504.19413.
**[inference]** Supports not positioning Horismos against GraphRAG-style indexing;
the live comparison set is memory/decision substrates (Quipu, lemmalog, Graphiti, TOKI).

## Searched but not found

**[measured — negative results of this sweep's queries, 2026-09-02]**

- Any KG-specific paper on version-checked commit-back for agent write-back; nearest is
  generic MAS concurrency.
- A 2026 NELL-style drift retrospective, or longitudinal measurement of entity-
  resolution drift in LLM-built KGs.
- A study of an LLM-drafts / formal-verifier / SME-sign-off ontology pipeline as such.
- A Text2KGBench successor beyond Text2KGBench-LettrIA (ISWC 2025 workshop).
- ACE-GraphRAG (arXiv:2608.01269) was withdrawn 2026-08-04 — do not cite.

## Source

Per-claim citations inline; all rows in [`../reading-log.md`](../reading-log.md) Q6 (sweep #2, 2026-09-02).

## Confidence

mixed — paper findings, release facts and repository metadata are [measured]; the
fit-to-design conclusions are [inference].

## Last-verified

2026-09-02
