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
  a direct retarget of sigil-lang's `horismos code` agent (7 tools, 58 tests) and
  NL-to-Horismos generator (validate → auto-retry refinement), emitting `.ttl` instead
  of `.horismos`.
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

1. Delivery form: CLI (`horismos author`) vs. library API vs. both (sigil-lang precedent:
   CLI-first worked).
2. Refinement budget and convergence policy when verification keeps failing (surface
   partial ontology + witnesses rather than loop forever).
3. Prompt strategy: full RDFS-plus vocabulary reference in system prompt (sigil-lang's
   syntax-reference-in-prompt approach) vs. constrained generation against a grammar.
4. **[DECIDE, doc 15 §3]** Exact resource limits on drafted/ingested Turtle (max file
   size, max triple count, load timeout) before it reaches parse/profile validation.

## Definition of done

A clinician-roleplay transcript produces a verified `triage.ttl` from scratch through
conversation alone; seeded incompleteness in the SME's description is caught by the
verifier and resolved in-loop via a domain-term question back to the SME; the session
corpus exports NL→Turtle pairs with outcome labels.

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
