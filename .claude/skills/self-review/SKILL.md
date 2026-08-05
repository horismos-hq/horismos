---
name: self-review
description: Use before reporting any horismos code change as done — after implementation, before writing the report, before requesting review, before committing. A repo-specific pre-report audit covering brief completeness, determinism, the anti-wrapper boundary, domain-term errors, gate integrity, test quality, Rust (clippy/edition/doc-comments) and Python (mypy strict/.pyi/marshal-only) specifics, and YAGNI.
---

# Self-review before reporting

Run this **after** `make verify` is green, not instead of it. Green is necessary and
nowhere near sufficient — every item below has slipped past a green build in this repo.
The reviewer (`.claude/agents/senior-reviewer.md`) checks the same axes; anything you
find here you fix, anything you decline to fix you **name as a risk** in your report.

## 1. Completeness vs. the brief

Walk the brief's numbered requirements one at a time and point at the code and the
test for each. Deviations are fine; **silent** deviations are not — state the
divergence and why the substitute is at least as strong (PRP-001's U6 enforced
min-cardinality against the *candidate* instance rather than the assertion, and said
so). Anything you skipped is a line in the report, not an omission.

## 2. Determinism audit (CLAUDE.md hard rule)

- No wall-clock, no `SystemTime`, no RNG anywhere in load, gate, or inference.
  `horis:now` is transaction-stamped, never read from the OS at use site.
- No `HashMap`/`HashSet` iteration order reaching an output, an error message, a
  serialized form, or a test assertion. Prefer `imbl::OrdMap`/`BTreeMap`; if you use a
  hash map internally, sort before emitting.
- Generated identifiers (skolem IRIs, ids, counters) are functions of the operation
  sequence only — same ops, same output, across processes. Rejected operations must
  not advance counters.
- Third-party nondeterminism counts: oxrdf mints random blank-node ids per parse —
  compare canonicalized, never raw.
- Ask: **is trace/state equality a valid test assertion for what I wrote?** If not, why
  not, in the report.

## 3. Boundary audit (any new dependency)

For each dep added: is it networked? a provider SDK? does it pull `reqwest`/`rand`/
`getrandom` transitively? is it pure enough for `wasm32-unknown-unknown`? is it
declared in root `[workspace.dependencies]` with `workspace = true` in the crate?
`horismos-core` gains nothing model-related, ever. `make check-boundary` and
`make no-model-gate` must be green **and unmodified**. Placement per
`.claude/skills/cargo-workspace-map`.

## 4. Error audit

Every failure path: a `thiserror`-derived `HorismosError` variant (not a bare
`String`, not `unwrap`/`expect` on user input), carrying a **machine-readable
structure** (subject, class, property, value, constraint kind + bound, provenance) and
a `domain_text()` rendered in ontology vocabulary — prefixed class names, local
property names, the `horis:` term that was violated:

```
heartRate 1280 exceeds horis:max 300 (bpm) on :Patient
```

Every rejection **names its constraint**. Templates come from the spec doc (03 for the
gate); if you invented wording, say so in the report.

## 5. Gate audit

Any mutation path that reaches world state without passing the validation gate is a
bug regardless of provenance (perception | rule | tool | api). Check: are store fields
private with no public mutators? is every public mutating method routed through the
gate? is the gate a pure function of (state, proposed mutation) so rejection leaves
state untouched? is atomicity tested by comparing against a pre-mutation clone rather
than by counting?

## 6. Test audit

- Tests assert real behaviour, not that the code ran. Rejection tests assert **both**
  the machine structure and the **exact** domain sentence.
- Fixtures: use the real `examples/triage/triage.ttl` wherever it supports the case;
  inline ontologies only for constraints triage doesn't exercise. Never fork or edit
  triage.ttl to make a test pass.
- Invariants get property tests — proptest (Rust) / Hypothesis (Python), parity in
  strategy shape.
- Output is pristine: no stray `println!`, no `dbg!`, no warnings in the test run.
- Test counts quoted in your report are counts you actually ran (a PRP-001 report
  mis-stated its count and it became a review finding).

## 7. Rust specifics

- `cargo clippy --workspace --all-targets -- -D warnings` clean; `unsafe_code` denied;
  edition 2024; no `unwrap` in library paths.
- Large `Err` payloads get boxed (`result_large_err` will tell you).
- Any new module's top doc-comment states **its spec doc and its owning PRP**, matching
  the existing `mod.rs` style.
- Public API shapes match `docs/framework/12-rust-api.md` — check the signatures, not
  the vibes.

## 8. Python specifics

- `horismos-py` marshals only: an `if` about domain behaviour there belongs in core.
- `python/horismos/_horismos.pyi` updated in the **same change** as the native surface;
  `mypy --strict` clean; `py.typed` intact.
- Errors surface as the `HorismosError` hierarchy with `domain_text()` as `str(exc)` —
  never bare `RuntimeError`/`ValueError`. GIL released around long core calls.
- Full rules: `.claude/skills/pyo3-boundary`.

## 9. YAGNI

Did you build anything the brief didn't ask for — a snapshot API in the gate unit, a
config knob nobody set, an abstraction with one implementer? Delete it. Slots for
future stages are a **comment marking the slot**, not scaffolding. Stubs for a later
unit must be genuinely inert (a default-allow check), not that unit's code smuggled in
early.

## Then write the report

State: what was implemented per requirement, RED/GREEN evidence pasted, `make verify`
output, deviations, and a **named-risks section** — the places you are least sure of,
in your own words. The reviewer will check each one specifically, so vagueness costs
you.
