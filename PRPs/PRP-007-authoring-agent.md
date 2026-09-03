# PRP-007 — Ontology Authoring Agent

**Status**: Draft — amended 2026-08-24 for the bounded-projection pivot
**Depends on**: PRP-003, PRP-005 (verifier is the gatekeeper); sequenced **after PRP-008**
per the dependency DAG (`PRPs/CONVENTIONS.md` § Tracks) — its default job below needs
PRP-008's mapping/package contracts to overlay against. Do not implement before then; this
amendment is planning-only (captain ruling, `decisions/captain-pivot-ruling.md` item 2).

## Pivot note (2026-08-24) — bounded projection kernel

Captain-adopted (`horismos-thin-pivot-s1/report.md`, PRP-007 row; `decisions/captain-pivot-
ruling.md` items 3–4). The **default job changes**: from "invent a whole ontology from a
blank-slate conversation" to **import, align, and generate a small verified overlay/delta
over existing standards and tool schemas** — FHIR profiles, terminology (SNOMED/LOINC/
RxNorm), published PlanDefinitions, or a coding harness's tool JSON Schemas/skill
manifests/repo conventions. Blank-slate authoring (the original Scope below) remains a
supported mode for domains with no existing standard to overlay, but it is no longer the
first acceptance case: per the recommended sequence, "its first acceptance case should
modify a standards-anchored VTE overlay, not regenerate triage from scratch."

The **candidate-artifact boundary** (captain-adopted design, item 3 of the ruling) is owned
jointly: **PRP-008 owns the host-adapter/runtime side** (policy validation, sandbox test
execution, symbolic authorization of a model-generated patch/file/command proposal);
**PRP-007 owns the authoring/vocabulary side** — the typed envelope's ontology vocabulary
(`horis:CandidateArtifact` and friends) is minted here, once, not ad hoc in either PRP. See
`docs/framework/06-llm-boundaries.md` and `docs/framework/07-tools.md` for the doc-level
declaration of this boundary as future work.

## Intent

The SME (or, for the capability-fragment mode, a repo/tool owner) experience: describe a
domain and its protocols in conversation, or point the agent at an existing standard
artifact or tool/skill definition to overlay; an agent (Claude) drafts a Turtle overlay;
the framework's own verifier gatekeeps every draft; the SME signs off on rendered rules —
never syntax. Applies the runtime principle at design time: neural proposes, symbolic
decides.

## Scope

- **The loop**: SME description, or an imported standard artifact/tool schema, → agent
  generates a Turtle overlay → parse → shape validation → BDD verification (PRP-005,
  including its non-vacuity pass) → counterexample witnesses in domain terms fed back →
  agent refines → converged ontology → SME review of a rendered summary → versioned,
  signed-off `.ttl`.
- **Import/align/overlay mode (default job, per the pivot)**: ingest a FHIR profile,
  PlanDefinition, terminology subset, or tool/skill schema without trusting it —
  deterministically lift explicit structure (names, types, allowed sets, cardinalities)
  first, no model needed for facts already machine-readable in the source; the model then
  proposes only the prose-derived mapping (rule conditions, effects, fallbacks) with every
  clause linked back to its source text.
- **Capability-fragment compiler mode**: given an existing tool/skill/plugin (a
  `SKILL.md`, a tool JSON Schema, a repo's `AGENTS.md`), compile a small ontology overlay
  — a **Capability Fragment** (working name; see PRP-000 §"Comparison" and
  `horismos-thin-pivot-s1/report.md` §"Coding-agent generalization") — attached to, not
  replacing, that original package. The compiler emits a **formalization coverage
  report**: which clauses became formalized/machine-checkable constraints, which are
  test-backed opaque claims, which remain prose-only guidance, and any detected
  contradictions. Missing effects or unbounded shell behavior block an "enforced" label
  but do not block using the original skill as advisory text — the report never claims
  more formalization than it achieved.
- **Agent implementation**: ReACT loop over the Anthropic API with tools
  (`validate_ttl`, `verify_ontology`, `render_summary`, `write_file`, `diff_ontology`) —
  a direct retarget of sigil-lang's `horismos code` agent (7 tools, 58 tests) and
  NL-to-Horismos generator (validate → auto-retry refinement), emitting `.ttl` instead
  of `.horismos`.
- **Rendered review surface**: rules and workflows displayed in domain language
  ("Immediate when heart rate exceeds 120 with chest pain — because…"), diffs between
  ontology versions in the same terms, and — for import/align mode — a source-anchor view
  showing which drafted clause traces to which line of the imported standard/schema.
- **Import assistance**: agent proposes anchors into external vocabularies
  ("ChestPain → SNOMED 29857009?") for SME confirmation.
- **Trace-mining as proposal source**: accumulated execution traces (doc 09) may propose
  candidate ontology edits, but only as input to this same gated loop — verifier + SME
  sign-off remains the only path any change takes. This is deliberately **not** a
  self-reinforcing ingest-learn-reingest loop (the NELL/Cognee failure mode surveyed in
  `horismos-research-s1/report.md` §1.7): traces stay one-directional output feeding a
  proposal, never a direct mutation.
- **Corpus side effect**: every session logs NL→Turtle pairs with verification outcomes
  — the second training corpus (authoring model) per the Vision tab.

## Non-goals

- No ontology GUI/editor (rendered summaries + conversation only, v1).
- No autonomous ontology deployment: SME sign-off is a hard gate, always.
- No multi-SME collaborative editing workflows (git is the collaboration layer, v1).
- No self-reinforcing trace-ingestion loop (see "Trace-mining as proposal source" above) —
  every trace-derived proposal re-enters the same human-gated authoring loop as any other
  draft; there is no path from trace to schema change that skips verification or sign-off.
- No claim that a compiled Capability Fragment fully formalizes its source skill — the
  coverage report's opaque/prose-only categories are the honest boundary, not a temporary
  gap to hide.

## Key decisions to make

1. Delivery form: CLI (`horismos author`) vs. library API vs. both (sigil-lang precedent:
   CLI-first worked).
2. Refinement budget and convergence policy when verification keeps failing (surface
   partial ontology + witnesses rather than loop forever).
3. Prompt strategy: full RDFS-plus vocabulary reference in system prompt (sigil-lang's
   syntax-reference-in-prompt approach) vs. constrained generation against a grammar.
4. **[DECIDE, doc 15 §3]** Exact resource limits on drafted/ingested Turtle (max file
   size, max triple count, load timeout) before it reaches parse/profile validation.
5. **[DECIDE, joint with PRP-008]** The candidate-artifact envelope vocabulary: exact
   `horis:` terms for a typed patch/file/command proposal, and where the PRP-007 (mint
   vocabulary) / PRP-008 (validate, sandbox, authorize) boundary sits mechanically —
   owned jointly per the captain ruling. PRP-008 pinned the runtime half of the
   boundary at its activation (`PRP-008-standards-host-integration/03-decisions.md`);
   the authoring-side vocabulary remains open here.
6. **[DECIDE]** Coverage-report format and the exact threshold at which a fragment may
   carry an "enforced" label vs. "advisory only."

## Definition of done

A clinician-roleplay transcript produces a verified `triage.ttl` overlay from scratch
through conversation alone (the blank-slate mode remains exercised, not removed); a
separate import/align transcript takes a standards-anchored fragment (per the recommended
sequence, a VTE-prophylaxis overlay) and produces a verified overlay whose formalized
clauses are source-anchored; seeded incompleteness in the SME's description is caught by
the verifier and resolved in-loop via a domain-term question back to the SME; **a seeded
subtly-wrong-but-internally-consistent axiom** (per `horismos-research-s1/report.md` §1.7's
recommended DoD extension) is drafted into a test transcript, and the DoD measures — not
assumes — whether an SME reviewing only the rendered summary and scenario probes actually
catches it; the non-vacuity checks (PRP-005) run as part of every verification pass in this
loop, not just at PRP-005's own gate; the capability-fragment compiler mode runs against at
least one real existing skill (this repo's own `brainstorming` skill is the worked example
in the source report) and emits a formalization coverage report with all four categories
populated; the session corpus exports NL→Turtle pairs with outcome labels.

## Security acceptance surface (doc 15)

PRP-007 owns the authoring-file-write trust boundary. Its DoD is not met until it
also proves:

- **Path jail**: `write_draft` rejects any target path that canonicalizes outside the
  configured drafts root (`../` traversal, absolute paths, symlink escapes), recorded
  as a rejected tool call in the session trace — never a write to the resolved
  out-of-root location.
- **Atomic writes**: a fixture that interrupts a draft write mid-flight and asserts no
  partial file is ever observable at the target path.
- **The signed-off path is never agent-written**: a test asserting no tool in the
  agent's surface can write to the path a deployment would load as trusted — only the
  distinct, human-gated sign-off step can.
- **Resource limits on adversarial Turtle** (per D4 above): an oversized or
  cyclic-import draft is rejected before it reaches downstream verification, bounded
  by an explicit timeout — a fixture proving the authoring loop can't be hung by a
  malicious or degenerate draft.
- **Prompt-injection resistance in the SME conversation**: an adversarial transcript
  fixture (SME-role text attempting to steer `write_draft`/`write_file` outside
  declared drafts, or to skip verification) proving the agent's tool surface — not the
  model's judgment — is what prevents it.
