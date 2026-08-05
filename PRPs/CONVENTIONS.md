# PRP Conventions

How phase plans are written, tracked, and grown in this repo.

## The lifecycle

```
Draft ──activate──▶ Active ──DoD met──▶ Done
                      │
Standing (000, foundational-skeleton, this file, BOARD): never "done", revised in place
```

- **Draft**: a single `.md` file — intent, scope, non-goals, key decisions,
  definition of done. Far-future Drafts stay one page; the next-up Draft is
  teaching-grade (see below). Sprint detail is produced at activation either way.
- **Active**: the PRP being implemented. **A PRP converts from single file to folder
  at activation** (see below). Only one PRP is Active at a time unless explicitly
  parallelized.
- **Done**: definition of done met and verified (`make verify` + the PRP's own DoD).
  The folder stays as the record.

## Teaching-grade PRPs (PRP-002 onward)

From PRP-002 onward, the **Active PRP and the next-up Draft are teaching-grade**: in
addition to the five spec sections (intent, scope, non-goals, key decisions,
definition of done — meaning unchanged), the document carries:

- a **background section** explaining the concepts the phase assumes, written for a
  Python developer new to the domain;
- **worked examples** from the clinical-triage running example
  (`examples/triage/`), hand-checkable step by step and consistent with shipped
  semantics (illustrative rules/properties are allowed but must be flagged as not
  part of the fixture, and must use only declared vocabulary);
- **"Python parallel" notes** wherever the spec touches Rust-specific design
  (traits, enums, ownership, error types), anchored in sigil-lang's Python side
  where it is the prior art;
- a **glossary** of terms in plain English.

Rules: teaching sections cite `docs/framework/` rather than restating it at length;
they never resolve open **[DECIDE]** questions — decisions stay open until resolved
in the owning PRP. The one-page summary form remains correct for far-future Drafts;
a PRP is expanded to teaching grade when it becomes next-up. Teaching material does
not count toward the ~150-line early-folder-conversion trigger below — conversion
still happens at activation, when the teaching-grade document moves to
`01-objective.md` and is kept current as usual.

## The folder convention (large / active PRPs)

A PRP must become a folder when it is activated, or earlier if it outgrows one page
(~150 lines). Single `.md` → folder with the same name:

```
PRPs/PRP-001-ontology-core/
├── 01-objective.md       # the original one-pager, kept current (status, scope, DoD)
├── 02-tasks-and-gaps.md  # unit-of-work tracking (see template below)
└── 03-*.md               # optional: benchmarks, decision records, spike results
```

`01-objective.md` stays the stable statement of intent. `02-tasks-and-gaps.md` is the
working surface:

```markdown
# PRP-NNN — Tasks & Gaps

## Decisions to resolve (before implementation)
- [ ] D1. <the [DECIDE] marker, its options, where it's recorded once resolved>

## Units of work
- [ ] U1. <independently verifiable unit — has its own test target>

## Gaps discovered during implementation
- <dated note: what the plan missed, and where it was taken>
```

Rules: every unit of work is independently verifiable (a test target, not a vibe);
every `[DECIDE]` marker owned by the PRP appears under *Decisions* and is resolved
**before** the units it blocks; gaps are appended, never silently absorbed — a gap
that changes scope goes back into `01-objective.md` by explicit edit.

## Tracks (grouping)

PRPs keep flat numbering (dependency sequence) but belong to tracks — the unit of
narrative on BOARD.md:

| Track | PRPs | Character |
|-------|------|-----------|
| **Standing** | 000, foundational-skeleton | Positioning & substrate; revised, never finished |
| **Engine** | 001, 002, 005 | The zero-LLM symbolic core; 005 deliberately needs only 001+002 |
| **Surface** | 003 | The Python front door |
| **Runtime** | 004 | Workflows, boundaries, tools, bridges |
| **Ship & Author** | 006, 007 | Distribution, docs site, the SME authoring agent |

## Other rules (carried over from README, now normative)

- Numbering is sequence; each PRP assumes the previous ones are done.
- Every **[DECIDE]** marker in `docs/framework/` is owned by exactly one PRP;
  activating a PRP begins by resolving its markers.
- `BOARD.md` is the at-a-glance state (current / next / end goal). Any session that
  changes a PRP's status updates BOARD.md in the same commit.
- The running example is clinical triage; every PRP's DoD is expressed against it.
