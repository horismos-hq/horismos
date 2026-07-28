# PRP-007 — Ontology Authoring Agent

**Status**: Draft
**Depends on**: PRP-003, PRP-005 (verifier is the gatekeeper)

## Intent

The SME experience: describe a domain and its protocols in conversation; an agent
(Claude) drafts Turtle; the framework's own verifier gatekeeps every draft; the SME
signs off on rendered rules — never syntax. Applies the runtime principle at design
time: neural proposes, symbolic decides.

## Scope

- **The loop**: SME description → agent generates Turtle → parse → shape validation →
  BDD verification (PRP-005) → counterexample witnesses in domain terms fed back →
  agent refines → converged ontology → SME review of a rendered summary → versioned,
  signed-off `.ttl`.
- **Agent implementation**: ReACT loop over the Anthropic API with tools
  (`validate_ttl`, `verify_ontology`, `render_summary`, `write_file`, `diff_ontology`) —
  a direct retarget of sigil-lang's `sigil code` agent (7 tools, 58 tests) and
  NL-to-Sigil generator (validate → auto-retry refinement), emitting `.ttl` instead
  of `.sigil`.
- **Rendered review surface**: rules and workflows displayed in domain language
  ("Immediate when heart rate exceeds 120 with chest pain — because…"), diffs between
  ontology versions in the same terms.
- **Import assistance**: agent proposes anchors into external vocabularies
  ("ChestPain → SNOMED 29857009?") for SME confirmation.
- **Corpus side effect**: every session logs NL→Turtle pairs with verification outcomes
  — the second training corpus (authoring model) per the Vision tab.

## Non-goals

- No ontology GUI/editor (rendered summaries + conversation only, v1).
- No autonomous ontology deployment: SME sign-off is a hard gate, always.
- No multi-SME collaborative editing workflows (git is the collaboration layer, v1).

## Key decisions to make

1. Delivery form: CLI (`sigil author`) vs. library API vs. both (sigil-lang precedent:
   CLI-first worked).
2. Refinement budget and convergence policy when verification keeps failing (surface
   partial ontology + witnesses rather than loop forever).
3. Prompt strategy: full RDFS-plus vocabulary reference in system prompt (sigil-lang's
   syntax-reference-in-prompt approach) vs. constrained generation against a grammar.

## Definition of done

A clinician-roleplay transcript produces a verified `triage.ttl` from scratch through
conversation alone; seeded incompleteness in the SME's description is caught by the
verifier and resolved in-loop via a domain-term question back to the SME; the session
corpus exports NL→Turtle pairs with outcome labels.
