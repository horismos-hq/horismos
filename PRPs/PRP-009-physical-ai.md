# PRP-009 — Physical AI

**Status**: Draft (2026-09-23)
**Depends on**: PRP-003 and PRP-005 (Done). Tiers 1–2 need a release shipping
`horismos.examples`, which the live site already assumes (`main` has it since #34;
`0.1.0a3`, the only PyPI release, does not). Tier 3's extra needs PRP-006's landed release
path, not its completion. PRP-006 also owns the docs site; that overlap and activation
order are decision 4.

## Intent

The captain has made Physical AI the lead story for Horismos, on docs.horismos.ai and in
positioning (relayed 2026-09-23). After this PRP, the home page, the nav and the ten-minute
quickstart teach through the motion-command shield (`examples/gallery/safety-envelope.ttl`).
A MuJoCo demo shows the shield stopping a robot that the same policy, unshielded, drives
into a person. The two questions this invites, "is it a robot-safety product?" and "how
fast is it?", get words the captain approved and a measured number.

## Scope

**The strongest beat.** The overlay was merged in #12 and stopped loading when PRP-005's
fail-closed loads landed in #14. Completeness had found two regions with no verdict, and in
one of them a command arrives with no proximity reading. #16 closed that region with a rule
that refuses (the captain ruled the defaults): `:Blocked via noProximityReadingBlocks`,
proof leaf `nearestHumanDistance isEmpty (closed world)`. **We could not prove it was safe,
so it did not move.** Re-run on the published `0.1.0a3` (2026-09-23, no model): open floor
2.4 m, 1.8 m/s → `:Permitted`; shared cell 0.8 m, 1.2 m/s → `:Attenuated`; no reading →
`:Blocked`; a direct `verdict` assertion → refused (`RulesOnly`).

Three tiers, each shipping alone and leaving the site coherent. P1 and P2 gate tiers 1–2:
`docs-pages.yml` deploys every `main` push touching `docs/`.

- **Tier 1: reposition the docs.** Physical AI is fourth of five *Use cases* nav entries;
  promote it. The home page's "The whole idea in ten lines" teaches triage rule `r1` and
  the `acuity` refusal; lead with the shield instead. The hero line, "Neural proposes.
  Symbolic decides.", stays. Rule on every other clinical or LLM-framed surface: the
  tagline; both `heartRate 1280 … :Patient` refusals (in the diagram and in *What you
  get*); the Jev chip and the VTE section; *What breaks*; the hero buttons;
  `site_description`; *Where next*; and the non-claims list, which has no physical line.
  Amend PRP-000's pitch, its personas (none robotic) and its triage-narrative ruling, as far
  as decision 3 reaches.
- **Tier 2: make the first ten minutes physical.** `docs/quickstart.md` is clinical
  throughout: §0 VTE, §1–8 triage, §9 triage and VTE, §10 triage. It has no physical
  reference. Make the shield the primary worked example. Move the clinical walkthrough, the
  site's most battle-tested prose, intact to its own page, where `test_devdocs_snippets.py`
  still executes and pins it. The shield needs **no model at all**, which suits "zero API
  keys" better, but has nothing matching §6 (subsumption), §9 (agent loop), §10 (go-live)
  or §0 (`python -m horismos demo`, the release gate's smoke run): decision 6.
- **Tier 3: a MuJoCo demo.** Nothing in the repository mentions MuJoCo today. The loop is
  a small, readable scene in which a policy proposes a velocity each tick. The velocity is
  asserted as a `:MotionCommand`, the shield returns `:Permitted`, `:Attenuated` or
  `:Blocked`, and only then does anything reach the actuator. `:Blocked` becomes an
  explicit zero-velocity command, and a missing verdict counts as `:Blocked` and is logged.
  The policy is deliberately imperfect, deterministic and scripted, and it drives at the
  human, because a good policy makes the shield invisible. A sensor-dropout window puts
  the no-reading `:Blocked` in motion. The same seed runs twice, with the shield on and
  off, and produces two clips: one collides, one stops. The demo also reports the shield's
  cost (blocked or attenuated ticks, task progress). The wiki's RL deep-dive (§3.3) cites a
  proof that no shield keeps both strong safety and strong permissiveness (CAV 2026), and
  a measured task-success cost of blocking (*The Verifier Tax*). MuJoCo is an **optional
  extra, never a core dependency**: the wheel does not carry it and `make verify` never
  needs it. CI runs headless with a fixed seed and asserts the verdict sequence and the
  contact outcome, never pixels; physics needs no GL. The check is a `make` target, never
  a pytest that skips without MuJoCo. No PR test workflow exists yet; PRP-006 owns it.
- **P1: measure decision latency.** Today there are only internal Rust runs. PRP-002 has
  `triage_transaction` at 5.6 µs and the 60-rule `chain60_transaction` at 132 µs. PRP-005
  has load+verify under 1 ms and a one-second ceiling test. Doc 14 publishes *targets* at
  "~50 rules": gate+chain under 1 ms, load+verify under 1 s. It also says the benchmarks
  are "CI-tracked… regressions fail builds", but no workflow runs them (PRP-006 owns that
  gate). Nothing measures the shield, the tail, or the Python path, which `make develop`
  builds without `--release`. An outside timing, not reproduced here, put a 55-rule
  ontology at 21 s (candidate: `reason::backward`'s 55-rule depth-cap test), inside doc
  14's scale and over its target for either path. **Work:** a checked-in steady-state
  benchmark of one decision (assert → fixed point → read `:verdict`; `explain` apart),
  Rust and Python, release build, p50/p99/max on named hardware; a world-size sweep (every
  transaction re-chains all standing derivations); the 21 s attributed. An honest number
  is published before tiers 1–2 merge.
- **P2: the safety-boundary statement (owned here).** Disclaimers sit in the `.ttl` header
  ("not a safety-standard implementation"), the *Use cases* index and the Physical AI
  page's closing edges; none on the home page or quickstart. No doc or example names ISO
  10218, ISO/TS 15066 or IEC 61508, or says Horismos is unassessed. See decision 1.

## Non-goals

- **Not a safety-rated component.** No claim against ISO 10218, ISO/TS 15066, IEC 61508 or
  any other standard, and no assessment; only the wording is open (decision 1).
- No claim about control-loop rate beyond P1's number. No hardware, ROS or sensor models;
  the sim supplies ground-truth distance. Both are out of scope, with no owner.
- No learned policy or RL in the demo (PRP-000's research appendix). No new `horis:` terms
  (doc 02 owns the vocabulary).
- No clinical content is deleted. Triage stays the flagship; VTE stays PRP-008's slice.
- No release, bump, tag, workflow edit or PR CI (PRP-006). No authoring agent (PRP-007).
  No sim host-SPI adapter (PRP-008): an in-process loop needs none.

## Key decisions to make

**[DECIDE]** items are the captain's. The rest are engineering D-items, resolved with
`.claude/skills/brainstorming`. All of them move to `02-tasks-and-gaps.md` at activation.

1. **[DECIDE — captain] Safety-boundary statement: placement and wording.** Candidates:
   the home page's non-claims list, the *top* of the Physical AI page, the physical
   quickstart, doc 15 (deployer-owned), the demo's output. At minimum: no claim against ISO
   10218, ISO/TS 15066 or IEC 61508, and unassessed under them. No wording drafted here.
2. **[DECIDE — captain] Latency: the claim and the gate.** The claim's percentile,
   surface, hardware and world size; the loop rate pages may state; whether measurements
   replace doc 14's targets and CI-tracking sentence; and whether a poor number ships with
   the supervisory-layer framing or holds tiers 1–2.
3. **[DECIDE — captain] Reach.** Either the front door only (home, nav, quickstart, and
   PRP-000's pitch and personas), or also the framework reference's running example and
   CONVENTIONS.md's triage-DoD rule. Either one amends PRP-000's 2026-07-28 ruling. Lean:
   the front door only, because the shield uses no imports, perception, tools or narration.
4. **[DECIDE — captain] Sequencing and the PRP-006 overlap.** PRP-006 is Active and owns
   the docs site ("the triage walkthrough as the tutorial"; DoD: "a newcomer reproduces the
   triage quickstart"). Run in parallel (amend PRP-006, or let the clinical page satisfy
   it) or after it, and before or after PRP-007? Teaching grade follows once next-up.
5. **[DECIDE — captain] Demo clips vs the site's "no external assets (no webfonts, no
   CDN, no images)" rule.** Self-host them (amends the rule), draw an inline SVG
   trajectory from the recorded run, or keep them off-site.
6. Quickstart §0/§6/§9/§10: keep them clinical with a hand-off; give the shield a
   perception workflow (new example vocabulary, re-verified); or stop at §8.
7. Per-tick world shape: replace one `:MotionCommand`, a fresh subject per tick, or a fresh
   `World` per decision (trace readability against latency).
8. Attenuated speed: `:Attenuated` carries no number (0.25 m/s is only a rule threshold).
   A rule-derived `RulesOnly` speed (re-verified, both copies byte-identical), or an
   adapter constant, which is logic outside the ontology.
9. Packaging and determinism: the extra's name and exact MuJoCo pin (runs repeat only on
   one build and platform); whether the scene ships as wheel data, runnable by public
   readers of a private repo; confirming with the captain that the extra's `Requires-Dist`
   metadata line satisfies "must not carry it"; per-tick vs collapsed-sequence assertions.

## Definition of done

The DoD is expressed against the shield, with triage as the regression surface. This is
the exception recorded in `PRPs/CONVENTIONS.md`.

- **Tier 1.** `mkdocs build --strict` passes. Physical AI is top-level in the nav. The home
  page's first example loads `gallery/safety-envelope.ttl` through
  `horismos.examples.path` and prints the `:Permitted`, `:Attenuated` and no-reading
  `:Blocked` proofs and the `RulesOnly` refusal. All of it is executed and output-pinned by
  `test_devdocs_snippets.py`. The hero line is unchanged, the decision-1 statement is placed
  per decision 1, and PRP-000 is amended.
- **Tier 2.** The quickstart's primary worked example is the shield, with no model and no
  keys; its other sections follow decision 6. The clinical page still executes and pins
  today's triage and VTE blocks: the `:p7` proof, the four gate refusals, `sct:4557003`,
  and the 0.93/0.61 split. `examples/triage/` is unchanged.
- **Tier 3.** One command runs the scene headless. With a fixed seed, the shield-off run
  records a human contact and the shield-on run records none. The shield-on verdicts,
  including `:Attenuated` and the no-reading `:Blocked`, match their pinned sequence and
  repeat for the same seed. The wheel holds no MuJoCo, `pip install horismos` installs
  none, and `make verify` and `make no-model-gate` pass without it.
- **P1.** A checked-in benchmark measures per-decision latency, published per decision 2
  before tiers 1–2 merge; the 21 s figure is attributed or recorded as unreproducible.
- `make verify` is green throughout.
