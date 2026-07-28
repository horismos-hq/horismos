# 13 · Authoring

SMEs do not write Turtle. They describe their domain in conversation with the
authoring agent (`sigil author`), which drafts the ontology and negotiates with the
verifier until the result is provably well-formed — then the SME signs off on a
rendered summary, never syntax. The runtime's principle applied at design time:
**neural proposes, symbolic decides.**

## The loop

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

Two hard gates, always: the **verifier** (no unverified draft survives the loop) and
the **SME sign-off** (no autonomous deployment, ever).

## The agent

A ReACT loop (Anthropic tool-use API) — a direct retarget of sigil-lang's proven
`sigil code` agent and NL-to-Sigil generator (validate → auto-retry refinement),
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
  review of its logic.

## The corpus side effect

Every authoring session logs `(NL description, Turtle draft, verification outcome,
revision, final)` tuples — exported through the trace exporter (doc 09) as the
**authoring corpus**: training data for an ontology-drafting model, distinct from the
runtime trajectory corpus. Both flywheels start turning on day one of real use.

## CLI

```
sigil author triage.ttl                 # start/continue a session
sigil author --review triage.ttl       # render summary + outstanding witnesses
sigil author --diff v1.1.0 v1.2.0      # domain-term changelog
```
