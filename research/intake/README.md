# Intake: the URL-drop convention

A person (most often the captain) can drop a URL into [`../intake/queue.md`](queue.md) at
any time, or hand it to an agent directly in conversation. Either way, the next agent
session that notices an `unfiled` row — or receives the URL directly — does:

1. **Fetch and read** the source in full (not just a search snippet).
2. **Summarize** its argument, name the author/affiliation, and — where the source
   cites or obviously implies further sources (references, the author's other work, a
   company/lab behind it) — do one shallow follow-up pass to place it in context,
   rather than filing it as an isolated fact.
3. **Position** it against horismos's actual design (not a generic take) — this is the
   step that makes an intake entry worth more than a bookmark.
4. **File**:
   - write `research/intake/filed/YYYY-MM-DD-<slug>.md` (the raw note),
   - update or create the relevant `research/<topic>/*.md` page with a short
     paragraph incorporating the finding,
   - append one row to [`../reading-log.md`](../reading-log.md),
   - mark the [`queue.md`](queue.md) row `filed` with a pointer to the filed note.
5. This is a small PR by construction — one URL in, a handful of file touches out —
   matching the delta-PR convention in [`../README.md`](../index.md).

## Worked example

The captain dropped
[the Markovic "learnable ontologies to world models" LinkedIn piece](https://www.linkedin.com/pulse/how-do-you-go-from-learnable-ontologies-world-models-markovic-flqtf/)
mid-sweep during `horismos-research-s1`. It was fetched and read in full; the author
(Vasilije Markovic, founder of Cognee, an open-source AI agent memory platform) and the
piece's argument were identified; one follow-up pass traced Markovic's own related
academic work and a thematically adjacent piece; it was positioned against horismos's
design; and it was filed. See:

- The filed raw note: [`../intake/filed/2026-08-24-markovic-learnable-ontologies.md`](filed/2026-08-24-markovic-learnable-ontologies.md)
- The updated topic page: [`../ontology-authoring/autonomous-agents-failure-modes.md`](../ontology-authoring/autonomous-agents-failure-modes.md)
- The reading-log row: [`../reading-log.md`](../reading-log.md), under "Captain intake"

This is exactly the shape a real intake PR takes.
