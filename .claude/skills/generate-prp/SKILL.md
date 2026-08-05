---
name: generate-prp
description: Use when drafting a new Horismos PRP, activating a Draft PRP into folder form, or splitting work out of an existing phase plan — covers the Draft/Active/Done lifecycle, the 01-objective + 02-tasks-and-gaps shape, D-item and U-item conventions, [DECIDE] marker ownership, prior-art citation, and the BOARD.md-in-the-same-commit rule. Also use when someone asks "where does this work go?" and no existing PRP owns it.
---

# Generating a PRP

Lifecycle authority: `PRPs/CONVENTIONS.md`. Board: `PRPs/BOARD.md`. Reading order and
track table: `PRPs/README.md`. When this skill and CONVENTIONS.md disagree, the doc
wins — then fix this skill.

## Form follows status

| Status | Form | Contents |
|---|---|---|
| **Draft** | one file `PRPs/PRP-NNN-slug.md` | intent, scope, non-goals, key decisions, DoD — no unit breakdown. Far-future: ≤ ~150 lines. **Next-up: teaching-grade** (CONVENTIONS.md § Teaching-grade PRPs) — background, worked triage examples, Python-parallel notes, glossary; teaching material doesn't trigger early folder conversion |
| **Active** | folder `PRPs/PRP-NNN-slug/` | `01-objective.md` (the one-pager, kept current) + `02-tasks-and-gaps.md` (working surface) + optional `03-*.md` (spikes, benchmarks, decision records) |
| **Done** | the folder stays as the record | DoD met and verified |
| **Standing** | one file, revised in place | 000, foundational-skeleton, CONVENTIONS, BOARD — never "Done" |

Conversion file → folder happens **at activation** (or earlier if the one-pager
outgrows a page). `01-objective.md` is the original one-pager moved verbatim, then
kept current. Numbering is flat and is dependency sequence; each PRP assumes its
predecessors are Done. Every PRP belongs to a track (Standing / Engine / Surface /
Runtime / Ship & Author) — the track is the narrative unit on BOARD.md.

## `01-objective.md` — required sections, in this order

```markdown
# PRP-NNN — <Title>

**Status**: Draft | **Active** (YYYY-MM-DD) | Done (YYYY-MM-DD)
**Depends on**: PRP-...

## Intent          # 2-4 sentences. What exists after this PRP that did not before.
## Scope           # bulleted, concrete, doc-anchored (cite docs/framework/NN)
## Non-goals       # what a reader would reasonably assume is here and is NOT, with the owning PRP
## Key decisions to make    # prose list; becomes the D-items at activation
## Definition of done       # expressed against the clinical triage example, always
```

The DoD is **executable prose**: "loads `triage.ttl` + a SNOMED subset, rejects
schema-violating instances with domain-term errors, round-trips Turtle, snapshots,
answers pattern queries. Property-tested. No LLM anywhere." Not "ontology core is
solid."

## `02-tasks-and-gaps.md` — the working surface

Three sections, exactly these headings (CONVENTIONS.md template):

```markdown
## Decisions to resolve (before implementation)
- [ ] D1. **<Short title>**: <the question, its options, where it's recorded once resolved>

## Units of work
- [ ] U1. <independently verifiable unit — has its own test target>

## Gaps discovered during implementation
(append dated notes here; scope changes go back into `01-objective.md` by explicit edit)
```

### D-items (decisions)

- One D-item per `[DECIDE]` marker this PRP owns, plus any decision the units block on.
- Resolve with `superpowers:brainstorming`'s repo variant → `.claude/skills/brainstorming`.
  Never decide in a code comment, a commit message, or a report.
- Resolved form is a checked box with a **dated resolution line** and the downstream
  pins, e.g.:
  `- [x] D2. **Store representation** (decided 2026-07-28): u32-newtype interned IRIs in
  an arena; instance data as adjacency maps… No generic SPO/POS/OSP indexes up front.`
- If a spike was skipped, say so in the line (D1: "human call, spike waived") and state
  the fallback, so a later session cannot re-litigate it by accident.
- **All D-items must be resolved before any U-item starts.** That is the activation gate.

### U-items (units of work)

- A unit is a **test target, not a vibe**. If you cannot name the assertion that turns
  green, it is not a unit.
- Size for TDD in one focused session, and for batching with its neighbours (the PRP-001
  run batched U3+U4, U6+U7, U8+U9, U10+U11). Adjacent units that share a data structure
  should be adjacent in numbering.
- Every unit ends with `make verify` green — no unit may leave the branch red.
- Order units so each is independently reviewable: types → parse → validate → store →
  gate → derived features → property tests → DoD run.
- The **last unit is always the DoD run** against the triage example (PRP-001's U11).

### Gaps

Appended, never silently absorbed. Dated note: what the plan missed and where it was
taken. A gap that changes scope goes back into `01-objective.md` by an explicit edit
in the same commit.

## [DECIDE] markers — the two-way link

Format in `docs/framework/`: `**[DECIDE: <question>; <lean, if any> — PRP-NNN]**`.
Every marker is owned by **exactly one** PRP, and every owned marker appears as a
D-item in that PRP. When authoring a PRP:

1. `grep -rn "DECIDE" docs/framework/` and claim the markers this phase must answer.
2. Any marker you claim gets a D-item. Any D-item that came from a doc names the doc.
3. Resolving a D-item includes **deleting the marker** from the doc and rewriting that
   sentence in shipped voice, in the same commit as the resolution.
4. Discovering a new open question mid-implementation? Add the marker to the doc *and*
   the D-item to the owning PRP — do not decide it inline.

## Prior art

`../sigil-lang` is the proving ground (~1,270 tests). Cite **what to port and where**,
not "see sigil-lang": `src/sigil_core/src/matcher.rs` (Rete-lite), `verifier/` (BDD
checks + witnesses), `solver.rs` (constraints), `trace.rs`, `vm/` (string interning,
chunk design), `src/sigil/neural/` (bridge), `tests/<area>/`. Horismos is not a
rewrite — it inverts sigil-lang (the engine is the product, the language surface is
gone), so every citation states what changes.

## Board discipline

Any session that changes a PRP's status updates, **in the same commit**:

1. the PRP's own `**Status**:` line,
2. `PRPs/BOARD.md` — Current state, Next goal, and the Board table row (+ `_Last
   updated:_`),
3. `planning/board.html`, republished to the existing artifact URL (see BOARD.md
   header — same URL, never a new one),
4. `PRPs/README.md`'s reading-order link if the PRP changed file → folder form.

## Checklist before you call a PRP drafted

- [ ] Non-goals name the PRP that owns each excluded thing.
- [ ] DoD is expressed against clinical triage and is mechanically checkable.
- [ ] Every claimed `[DECIDE]` marker has a D-item; every D-item names where it is
      recorded once resolved.
- [ ] Every U-item names its test target; the final U-item is the DoD run.
- [ ] Track assigned, dependency line correct, BOARD.md row added.
