# 13 · Authoring

SMEs do not write Turtle. They describe their domain in conversation with the
authoring agent (`horismos author`), which drafts an ontology **overlay** and negotiates
with the verifier until the result is provably well-formed — then the SME signs off on a
rendered summary, never syntax. The runtime's principle applied at design time:
**neural proposes, symbolic decides.**

**Default job, amended 2026-08-24** (PRP-007, under the adopted bounded-projection
pivot): the primary authoring mode is no longer blank-slate drafting
from conversation alone — it is **import, align, and overlay**: ingest an existing FHIR
profile, terminology subset, PlanDefinition, or a tool/skill's own schema/manifest without
trusting it, deterministically lift whatever structure is already machine-readable (names,
types, cardinalities, allowed sets — no model needed for facts already explicit in
metadata), and let the model propose only the prose-derived mapping (rule conditions,
effects, fallbacks), with every drafted clause linked back to its source text. Blank-slate
drafting (the loop below) remains fully supported for domains with nothing to overlay.

## The loop (blank-slate mode)

```
SME: "Patients with heart rate over 120 and chest pain go straight to Immediate."
  ↓
agent drafts Turtle           (classes, rule r1, constraints it can infer)
  ↓
parse → gate shapes → VERIFY  (docs 02, 10 — the same machinery as runtime)
  ↓
witnesses come back in domain terms:
  "No acuity assigned when heartRate ∈ (100, 120] and no Symptom present."
  ↓
agent asks the SME a domain question — never shows a triple:
  "What should happen for heart rates between 100 and 120 without other symptoms?"
  ↓
SME answers → agent revises → re-verify → … until report.ok
  ↓
SME reviews the rendered summary:
  "IMMEDIATE when heart rate exceeds 120 bpm AND any chest pain is present
     — 'Tachycardia with chest pain — immediate track'
   URGENT when …"
  ↓
sign-off → versioned triage.ttl (+ domain-term changelog entry)
```

## The loop (import/align/overlay mode — declared-future, PRP-008 coordination)

```
Owner points the agent at a source: a FHIR profile, a PlanDefinition, a tool JSON
Schema, a repo's SKILL.md/AGENTS.md — every source is hashed, never trusted blindly
  ↓
deterministic lift: names, types, cardinalities, allowed sets, exit codes, timeouts →
  candidate classes, typed arguments, constraints        (no model needed here)
  ↓
agent proposes the prose-derived mapping: workflow steps, horis:requires conditions,
  declared effects, fallbacks — each clause source-anchored to the line it came from
  ↓
parse → gate shapes → VERIFY (+ non-vacuity, PRP-005)   (docs 02, 10)
  ↓
compiler emits a formalization coverage report: formalized / machine-checkable /
  test-backed-opaque / prose-only / contradictions — holes are named, never guessed
  ↓
Owner reviews the rendered summary + source-anchor view + coverage report — never a triple
  ↓
sign-off → versioned overlay .ttl (+ domain-term changelog + coverage report)
```

Two hard gates, always, in both modes: the **verifier** (no unverified draft survives the
loop) and the **human sign-off** (no autonomous deployment, ever).

## The agent

A ReACT loop (Anthropic tool-use API) — a direct retarget of sigil-lang's proven
`horismos code` agent and NL-to-Horismos generator (validate → auto-retry refinement),
emitting standard Turtle instead of a bespoke DSL. Its tools:

| Tool | Purpose |
|------|---------|
| `validate_ttl` | Parse + profile check + gate-shape check |
| `verify_ontology` | Full BDD verification; returns structured witnesses |
| `render_summary` | Ontology → domain-language rules/workflow rendering |
| `diff_ontology` | Two versions → domain-term diff ("r1 threshold changed 120→110") |
| `search_vocab` | Query imported vocabularies ("chest pain" → SNOMED candidates) |
| `write_draft` | Persist a draft version (never the signed-off path) |

Convergence policy: bounded refinement rounds per SME turn; if verification still
fails, the agent surfaces the remaining witnesses as questions rather than looping —
unresolvable gaps are *the SME's information*, not a failure to hide.
**[DECIDE: round budget — PRP-007]**

## Vocabulary anchoring

When the SME names a concept, the agent searches imported vocabularies and proposes
anchors for confirmation:

> "I'll add **ChestPain** as a symptom. SNOMED CT has *Chest pain (29857009)* — anchor
> to it? That gives you its 40+ subtypes (crushing, pleuritic, …) automatically."

Confirmed anchors become `owl:sameAs`; declined ones stay local. The agent never
invents IRIs in external namespaces.

## Review surfaces

- **Rendered rules** — the summary above; the only thing the SME approves.
- **Rendered diffs** — every revision, in domain terms, suitable for change-control
  records in regulated environments.
- **Witness reports** — outstanding verification findings, phrased as questions.
- **Scenario probes** — the SME can ask "what would happen for a 45-year-old with
  HR 115 and dizziness?"; the agent runs the draft ontology symbolically (structured
  facts, no perception) and shows the proof tree. Dry-running the ontology *is* the
  review of its logic. **Amended (2026-08-24)**: the DoD extends this to include a seeded
  subtly-wrong-but-internally-consistent axiom, measuring — not assuming — whether an SME
  reviewing only the rendered summary and scenario probes catches it. This is in
  PRP-007's definition of done because review-by-summary is exactly where formalized
  guidelines have historically passed review while encoding the wrong thing.
- **(Import/align mode, declared-future) Source-anchor view, mapping-loss report, and
  assumption diff** — which drafted clause traces to which line of the imported standard;
  what was lost or approximated in the mapping (never silently); and what changed in the
  overlay's own assumption manifest (`docs/framework/10-verification.md`) between
  revisions — so a reviewer sees not just "what changed" but "what this overlay now
  assumes that it didn't before."

## The corpus side effect

Every authoring session logs `(NL description, Turtle draft, verification outcome,
revision, final)` tuples — exported through the trace exporter (doc 09) as the
**authoring corpus**: training data for an ontology-drafting model, distinct from the
runtime trajectory corpus. Both flywheels start turning on day one of real use.

## Skills-to-fragment loop and trace-mining (declared-future, PRP-007/PRP-008)

Two extensions, captain-adopted alongside the pivot, neither built in v1:

- **Capability-fragment compiling**: the same ingest → deterministic-lift → model-proposes
  → verify → sign-off loop above, pointed at an existing coding-agent skill/tool instead
  of a clinical standard, producing a **Capability Fragment** — a small overlay attached
  to, not replacing, the original package, with the formalization coverage report from the
  overlay loop above. PRP-007 owns the compiler mode's full specification; the
  positioning against existing harnesses is on
  [Coding agents](../governing-a-coding-agent.md).
- **Trace-mining as a proposal source**: accumulated execution traces (doc 09) may suggest
  candidate ontology edits, but only by entering this exact gated loop as a new kind of
  draft proposal — never by mutating a signed-off ontology directly. This is a deliberate
  rejection of a self-reinforcing ingest-learn-reingest loop — the documented schema-drift
  failure mode of never-ending-learning systems, where a system that re-ingests its own
  output degrades its schema over time:
  Horismos's principle 2 ("traces are output, not logging") stays one-directional at the
  authoring layer exactly as it is at the runtime layer.

## CLI

```
horismos author triage.ttl                 # start/continue a session
horismos author --review triage.ttl       # render summary + outstanding witnesses
horismos author --diff v1.1.0 v1.2.0      # domain-term changelog
```
