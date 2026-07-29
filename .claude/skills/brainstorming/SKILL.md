---
name: brainstorming
description: Use when resolving a Horismos design question — any [DECIDE] marker, any D-item in a PRP's 02-tasks-and-gaps.md, any "should we use crate X or write it ourselves", any architecture fork discovered mid-implementation. Runs a falsifiable-statement → evidence-sweep → options → adversarial-pass → ranked-criteria → recorded-resolution protocol, with a spike escape hatch. Use INSTEAD of generic brainstorming, and before writing code that would decide the question implicitly.
---

# Deciding, the Horismos way

A design question here is never "an idea we had". It is a **D-item owned by exactly
one PRP**, resolved once, dated, pinned downstream, and struck from the docs. This
skill is the procedure. Never decide in a code comment, a commit message, or a report
(CLAUDE.md).

Related: `.claude/skills/generate-prp` (where D-items live),
`.claude/skills/impl-prp` (when a run is blocked on one).

## Step 0 — Locate the owner

`grep -rn "DECIDE" docs/framework/` and open the owning PRP. If no PRP owns it, you
have found a gap: assign it to a PRP by dependency sequence (`PRPs/README.md`), add
the `[DECIDE: <question>; <lean> — PRP-NNN]` marker to the governing doc *and* the
D-item to that PRP, before proceeding. Two owners = a bug; zero owners = a bug.

## Step 1 — Falsifiable decision statement + blast radius

Write the decision as a sentence that could be **wrong**, in shipped voice, and name
what it touches. Not "how should we store triples" but:

> "Instance data is stored as subject→property→values adjacency maps over interned
> `u32` IRIs, with no generic SPO/POS/OSP indexes in v1."

**Blast radius** — list explicitly, because it sets who reviews and what gets pinned:

| Axis | Ask |
|---|---|
| Crates/modules | which of `horismos-core::{ontology,world,reason,workflow,verify,trace,bridge}`, `horismos-bridges`, `horismos-py`, `python/horismos` |
| Units | which U-items in the owning PRP change if the answer flips |
| Downstream PRPs | which later phases inherit the consequence |
| Docs | which `docs/framework/NN` sentences are rewritten when the marker is struck |
| Gates | does any option touch `check-boundary`, `no-model-gate`, wasm32, or determinism |

If the blast radius is one function in one unit, **stop** — it is an implementation
choice, not a D-item. Decide it in the brief and note it as a controller pin.

## Step 2 — Evidence sweep (BEFORE options)

Generating options first anchors you. Sweep in this order and take notes with
citations; you will quote them in the resolution line.

1. **The spec.** `docs/framework/` — what does the definition already commit to?
   Doc 02 (vocabulary), 03 (gate order and error wording), 04 (operator semantics),
   14 (**performance posture table** and the dependency invariant). The docs are
   shipped voice: if they answer it, the decision is already made and your job is to
   find that out cheaply.
2. **Prior art in `../sigil-lang`.** ~1,270 tests of proving ground. Cite specific
   modules and tests, not the repo: `src/sigil_core/src/matcher.rs` (Rete-lite),
   `verifier/` (BDD + witnesses), `solver.rs`, `trace.rs`, `vm/` (interning, chunk
   design), `src/sigil/neural/`, `tests/<area>/`. Ask what it *cost* there, not just
   what it did — sigil-lang's PyO3 marshaling lesson is a design input, not trivia.
3. **The literature.** RDF/SHACL/RDFS semantics, Datalog and semi-naive evaluation,
   Rete and its variants, provenance semirings (the Type-5 door doc 01 keeps open).
   Use WebSearch/WebFetch when they are available; a decision citing a real result
   beats one citing intuition.
4. **The ecosystem.** What would `oxigraph`, `sophia`, `datafrog`, `ascent`,
   `biodivine-lib-bdd`, `imbl` do? Read the actual crate's approach — and check its
   dependency tree against the anti-wrapper and wasm32 constraints before you fall in
   love with its API.

## Step 3 — 2–4 genuinely distinct options, each steelmanned

Distinct means *different in kind* (own it / adopt it / defer it), not three tunings
of one idea. For each: the strongest honest case, including who it makes happy
downstream. If you cannot steelman an option, you haven't understood it — go back to
step 2.

## Step 4 — Adversarial pass

For each option, write the strongest argument it is wrong **for this repo** — the
house constraints, in this order:

- **Determinism**: does it introduce wall-clock, RNG, or hash-iteration order into any
  output? (Transitive deps count: oxrdf drags `rand`/`getrandom`.)
- **Anti-wrapper**: does it pull anything networked or model-related toward
  `horismos-core`? Does it survive `--no-default-features --features verify`?
- **wasm32**: does core still build for `wasm32-unknown-unknown`?
- **PyO3 boundary**: does it force chatty cross-language calls inside a transaction,
  or logic into `horismos-py`?
- **Performance posture** (doc 14): gate+chain < 1 ms, `prove`/`explain` < 1 ms, load
  + verify < 1 s at triage scale (~50 rules, ~10² facts). Does the option optimize a
  scale we do not have?
- **Gate integrity**: does it create any mutation path that could bypass the gate?

An option that fails determinism or anti-wrapper is dead, not "scored lower".

## Step 5 — Rank criteria, then score

**Write the criteria and their order before you look at the options again.** Ranking
after scoring is criteria-fitting and this repo has a lot of tempting crates. Typical
ranking: house invariants (pass/fail) → the hot path this touches → what it forecloses
downstream → code we own vs. code we adopt → implementation cost. State the ranking in
the resolution line so a later session can argue with the *criteria*, not just the
answer.

## Step 6 — Record (all four, same commit)

1. **D-item resolution line** in the owning PRP's `02-tasks-and-gaps.md`: checked box,
   bolded title, `(decided YYYY-MM-DD)`, the decision in one or two sentences, the
   reason, and **the fallback if it proves wrong** — D1's "if oxigraph proves heavy
   there, the fallback is its parser crates alone, not a re-litigation".
2. **Downstream pins** — the concrete obligations this puts on units, copied into the
   ledger's *Controller resolutions* block and into the affected unit briefs.
3. **Strike the `[DECIDE]` marker** from `docs/framework/NN` and rewrite the sentence
   in shipped voice. (If the question came from a PRP's "Key decisions" section rather
   than a doc marker, there is nothing to strike — say so.)
4. **BOARD.md** if the resolution changes a status or the Next-goal narrative.

## Step 7 — Spike protocol (only when evidence is genuinely insufficient)

A spike is the **smallest** benchmark or prototype that discriminates between the
surviving options. Define, before writing it: the single question, the metric, the
**kill criterion** ("if X > Y ms at 10³ facts, option B is out"), the time box, and
where the result is recorded (`03-*.md` in the PRP folder). Spike results are
evidence, not decisions — you still run steps 5–6.

**Escape hatch**: a human may waive the spike and call it. Record that verbatim —
D1's `Decided 2026-07-28 (human call, spike waived)` — with the fallback, so the
waiver is auditable rather than invisible.

## Worked example — PRP-001 D2, store representation

| Step | What it looked like |
|---|---|
| **Statement** | "Instance data is subject→property→values adjacency maps over `u32`-newtype interned IRIs in an arena; no generic SPO/POS/OSP indexes in v1." |
| **Blast radius** | `core::ontology` (interning), `core::world` (store, gate); U1, U6, U8, U9; PRP-002's matcher inherits the index question; PRP-003 inherits whole-value marshaling. Gates: wasm32 + no-model if a store crate is adopted. |
| **Evidence** | Doc 03's gate order is subject-major (class → domain/range → constraints → cardinality → provenance) ⇒ the hot path is point lookup by subject. Doc 14 targets < 1 ms at ~10² facts. sigil-lang `vm/` proved u32 string interning end-to-end. oxigraph's *store* is SPO/POS/OSP over RocksDB — heavy, and D1 had already scoped us to `oxttl`/`oxrdf` for parsing only. datafrog/ascent want relation-shaped tuples, relevant later for PRP-002, not now. |
| **Options** | (A) adopt oxigraph's Store; (B) own it: interned + subject-major adjacency; (C) own it: interned triple table + three sorted indexes up front. |
| **Adversarial** | A: drags disk/`rand` into core, risks no-model + wasm gates, and duplicates the gate's job. B: property-major and object-major queries (U9, PRP-002 joins) degrade to scans. C: triples write cost and memory to serve a scale we have not measured — premature at 10² facts. |
| **Criteria (ranked first)** | 1 house invariants pass/fail · 2 gate hot path · 3 O(1) snapshot compatibility (D4 pins persistent maps) · 4 doesn't foreclose PRP-002 indexes · 5 own vs. adopt. |
| **Resolution** | B. Recorded as D2, dated 2026-07-28, with "secondary indexes arrive benchmark-driven with PRP-002's matcher" as the explicit escape valve. |
| **Pins** | U1: `Iri` = `Copy` u32 newtype + `Interner` arena. U6: adjacency over `imbl::OrdMap` from day one so U8's `snapshot()` is O(1), not a retrofit. U8: a snapshot must not deep-copy the interner. |
