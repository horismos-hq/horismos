# The MuJoCo shield demo

A scripted policy drives a wheeled robot at a goal on the far side of a person. It
never slows down, on purpose: a policy that is careful by itself makes the shield
invisible. Every control tick the proposal is asserted as a `:MotionCommand` under
`examples/gallery/safety-envelope.ttl`, the rules return `:Permitted`, `:Attenuated`
or `:Blocked`, and only then does a value reach the actuator. `:Blocked` is an explicit
zero-velocity command; a missing verdict is treated as `:Blocked` and logged. A
sensor-dropout window withholds the distance reading for eight ticks, which puts the
no-reading `:Blocked` in motion. The same seed runs twice, shield off and shield on:
one run touches the person, the other does not, and the report says what the shield
cost. This is PRP-009 tier 3.

MuJoCo is an optional extra, never a core dependency. The wheel's metadata names it
only under `extra == "mujoco"`, `pip install horismos` installs none of it, nothing
the package imports or tests needs it, and `make verify` never loads it.

## Run it

```sh
make demo-shield                              # installs the extra into .venv, runs the CI check
uv run python examples/mujoco/demo.py         # both runs, text report
uv run python examples/mujoco/demo.py --trace # one line per tick: what the captain reads
uv run python examples/mujoco/demo.py --json out.json
uv run python examples/mujoco/demo.py --record clips/   # two MP4s (needs a GL context)
```

Outside this checkout: `pip install "horismos[mujoco]"`, then run `demo.py` from a copy
of this directory (the scene and the pinned sequence sit beside it; the ontology comes
from the installed package through `horismos.examples.path`).

Flags: `--shield both|on|off`, `--seed N` (default 7), `--ticks N` (default 300 ticks
of 0.05 s), `--trace`, `--json PATH`, `--record DIR`, `--check`, `--pin`.

## What one run looks like

```text
shield off seed 7  300 ticks: HUMAN CONTACT at tick 72 (t = 3.60 s); goal reached at tick 188 (t = 9.40 s)
shield on  seed 7  300 ticks: no human contact; goal reached at tick 267 (t = 13.35 s)
  verdicts: :Attenuated 128 · :Blocked 44 · :Permitted 128  (no-reading :Blocked 8, no verdict 0)
  sequence: :Permitted via :permitOpenFloor ×20 → :Blocked via :noProximityReadingBlocks ×8
            → :Permitted via :permitOpenFloor ×33 → :Attenuated via :attenuateOpenFloorNearHuman ×36
            → :Blocked via :blockTooClose ×34 → :Attenuated via :attenuateOpenFloorNearHuman ×4
            → :Blocked via :blockTooClose ×2 → :Attenuated via :attenuateOpenFloorNearHuman ×8
            → :Attenuated via :attenuateInSharedCell ×80 → :Permitted via :permitOpenFloor ×75
cost of the shield: 44 blocked + 128 attenuated ticks; reaches the goal 79 ticks (3.95 s) later
```

Read it as the robot's story. Open floor, permitted at 1 m/s. Ticks 20–27: the distance
reading is withheld, and the shield refuses because it cannot establish separation;
the robot brakes to a stop on an empty floor, which is the point. Reading back,
permitted again. At about 1 m from the person the open-floor rule caps the speed to
0.25 m/s. At 0.5 m the robot is blocked and stands there while the policy keeps asking
for full speed. At tick 120 the person starts walking out of the path; the separation
opens, the robot creeps after them at the capped speed, and a couple of ticks flick
back to blocked as the gap hovers at 0.5 m. Inside the shared cell it stays capped
even with the person well away, then it is permitted on open floor and reaches the goal.

The unshielded run under the same seed drives into the person at tick 72, stays pressed
against them until they walk off, and reaches the goal earlier. That is the cost the
shield charges: 3.95 s of task time in this scene. (Progress reads 100 % for both because
both eventually arrive; the delay is the honest number.)

With `--trace`, each tick prints the proposal, the reading (or `no reading`), the zone,
the verdict with the rule that derived it, and the speed that actually reached the
actuator:

```text
    20  1.00  1.05 m/s  no reading  :OpenFloor   :Blocked via :noProximityReadingBlocks         0.00 m/s
    61  3.05  0.98 m/s      0.98 m  :OpenFloor   :Attenuated via :attenuateOpenFloorNearHuman   0.25 m/s
    97  4.85  1.04 m/s      0.49 m  :OpenFloor   :Blocked via :blockTooClose                    0.00 m/s
```

## The scene (`scene.xml`)

Metres and seconds. A flat floor; a cylinder-chassis robot on two slide joints driven
by velocity servos (`ctrl` is the target speed in m/s, and it is the only thing the
loop writes); a capsule person on a mocap body, standing on the direct line from start
to goal and scripted to walk out of the way at 6 s; an amber shared-cell region drawn
as a site; a green goal disc. The loop reads its ground truth from the model and the
state: the robot-to-human separation is the planar centre distance minus both radii,
the zone is `:SharedCell` inside the amber box and `:OpenFloor` elsewhere, and a human
contact is any MuJoCo contact between a robot geom and a human geom during the tick.
The seed sets the person's lateral offset (still in the way, since the radii sum to
0.45 m) and a small dither on the policy's speed; MuJoCo itself is deterministic.

## The CI check (`make demo-shield`)

`demo.py --check` runs shield off, shield on, and shield on again, and exits non-zero
unless: the shield-off run recorded a human contact; the shield-on run recorded none;
its per-tick verdicts (with the rule each came from) equal `expected-verdicts.json`;
that pinned sequence holds at least one `:Attenuated` and the no-reading `:Blocked`; and
the second shield-on run repeats the first tick for tick, actuated values included.
Never pixels, never a pytest that skips. `--pin` rewrites the expected file; do that
only when the scene or the loop changed on purpose, and say so in the commit.

The pin is exact for one physics build: `mujoco==3.14.0`, pinned on macOS 26 arm64,
Python 3.13. A different MuJoCo version prints a warning and may differ in the flicker
ticks around the 0.5 m boundary; the contact outcomes should not.

## Engineering decisions

**D7 — per-tick world shape: a fresh `World` per decision.** Each tick is one
transaction over one subject (`:cmd-t0042`) in a world holding nothing else, so every
proof is self-contained and the cost per decision is flat. The alternatives were
measured against the real API with 400 synthetic ticks:

| shape | result |
| --- | --- |
| fresh subject per tick, one standing world | fails at tick 100: every transaction re-chains all standing derivations, and 100 standing commands exhaust `ITERATION_CAP` (`forward chaining exceeded 100 iterations … rolled back`); before that, per-tick cost grows linearly (1.9 ms at tick 0, 3.8 ms at tick 99, debug build) |
| replace one standing subject | p50 58 µs, but a dropout tick needs a `retract` of the old distance before the assert, so one tick becomes two decisions, the first over the previous tick's speed |
| fresh `World` per decision | p50 79 µs, p99 95 µs, max 180 µs (debug build); constant |

The first row is a real finding for the P1 world-size sweep: a standing world of
per-tick commands is not a shape the kernel supports today past the cap. Nothing in
this demo needs cross-tick facts, and a shield's verdict is a function of the state it
was given, so the fresh world is also the honest model.

**D8 — the attenuated speed: an adapter constant, re-checked by the rules.**
`:Attenuated` carries no number; `0.25 m/s` is the collaborative-speed threshold the
rules use (`atMost 0.25`). `demo.py` holds it as `ATTENUATED_SPEED`, and that is logic
outside the ontology. So the capped command does not go to the actuator on the adapter's
say-so: it is asserted a second time (`:cmd-t0042-capped`, same state, speed 0.25), and
only a `:Permitted` on *that* command releases it; anything else releases zero. In this
scene every one of the 128 capped commands came back `:Permitted` via
`permitSlowSharedCell` or `permitSlowOpenFloorNearHuman`, which is what `verify()`'s
completeness result already implied. A rule-derived speed would need a new property, and
the ontology is not edited here.

**D9 (proposed, captain to confirm) — packaging.** The extra is named `mujoco` and pins
`mujoco==3.14.0`. The scene does not ship as wheel data: the demo is a repository
example, like `examples/triage/`, and adding it to `python/horismos/examples/` would
put MuJoCo-shaped files in every install. The built wheel carries exactly one line
about MuJoCo, `Requires-Dist: mujoco==3.14.0 ; extra == 'mujoco'`, and no file; whether
that metadata line satisfies "the wheel must not carry it" is the captain's call.

## Latency, informational

`decision time` in the report is wall-clock from Python around one decision: build the
world, assert, read the verdict, render the proof (two assertions on an attenuated
tick). On an idle Apple Silicon laptop: p50 104 µs, p99 about 210 µs under
`make develop`'s debug build; p50 25 µs, p99 45–77 µs, max under 100 µs on the release
wheel. The P1 benchmark task owns the real number.

## Not here

No learned policy, no ROS, no hardware, no sensor model (the sim supplies ground truth),
no new `horis:` terms, no clip in the docs site (captain decision 5). Clips are rendered
into the directory you name and are not committed.
