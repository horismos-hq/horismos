"""The MuJoCo shield demo (PRP-009 tier 3).

A scripted policy drives a wheeled robot at a goal, straight through the person
standing in the way, because a policy that slows down by itself makes the shield
invisible. Each control tick the proposal is asserted as a ``:MotionCommand`` in
``examples/gallery/safety-envelope.ttl``; the rules return ``:Permitted``,
``:Attenuated`` or ``:Blocked``, and only the value the shield released reaches
the actuator. ``:Blocked`` is an explicit zero-velocity command, and a missing
verdict is treated as ``:Blocked`` and logged. A sensor-dropout window withholds
the distance reading for a few ticks, which puts the no-reading ``:Blocked`` in
motion. The same seed runs with the shield on and off: one run collides, one does
not, and the report says what the shield cost.

    python examples/mujoco/demo.py            # both runs, text report
    python examples/mujoco/demo.py --trace    # one line per tick
    python examples/mujoco/demo.py --check    # the CI assertion (make demo-shield)
    python examples/mujoco/demo.py --record out/   # two clips, if a GL context exists

MuJoCo is the ``mujoco`` extra (``uv sync --extra mujoco``), never a core
dependency; nothing here is imported by the package or its tests. Physics runs
headless; no GL is needed unless ``--record`` is given.
"""

# The report writes the multiplication sign and the arrow on purpose.
# ruff: noqa: RUF001, RUF002
from __future__ import annotations

import argparse
import json
import math
import random
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any

import horismos
import mujoco  # type: ignore[import-untyped]
import numpy as np
from horismos import Ontology, World
from horismos.examples import path as example_path

HERE = Path(__file__).resolve().parent
SCENE = HERE / "scene.xml"
EXPECTED = HERE / "expected-verdicts.json"
ONTOLOGY = "gallery/safety-envelope.ttl"

TICK_SECONDS = 0.05  # the control tick (20 Hz); physics substeps inside it
CRUISE_SPEED = 1.0  # m/s: what the policy wants, always (horis:max is 2.0)
# D8 — :Attenuated carries no number. This is the adapter's speed for an attenuated
# command: the collaborative-speed threshold the rules use (atMost 0.25). It is logic
# outside the ontology, so every capped command is asserted a second time and only
# reaches the actuator when the rules mark *it* :Permitted (see README.md).
ATTENUATED_SPEED = 0.25
DROPOUT_TICKS = range(20, 28)  # the distance reading is withheld on these ticks
HUMAN_WALK_TICK = 120  # the person starts walking out of the path here
HUMAN_WALK_SPEED = 0.5  # m/s, along +y, until they are 2 m off the path
HUMAN_WALK_LIMIT = 2.0
GOAL_RADIUS = 0.2  # the run counts the goal as reached inside this radius

SHIELD_OFF = "— (shield off)"
NO_VERDICT = "no verdict"


# --- the MuJoCo side: ground truth in, actuator command out ---------------------


class Scene:
    """The scene and its physics. Ground truth is read from the model (the cell's
    extent, the goal) and the state (positions); the only thing written is
    ``ctrl``, and only after the shield has spoken."""

    def __init__(self, seed: int) -> None:
        self.model = mujoco.MjModel.from_xml_path(str(SCENE))
        self.data = mujoco.MjData(self.model)
        self.substeps = round(TICK_SECONDS / self.model.opt.timestep)
        self.robot_body = self.model.body("robot").id
        self.human_body = self.model.body("human").id
        self.chassis_geom = self.model.geom("chassis").id
        self.robot_radius = float(self.model.geom("chassis").size[0])
        self.human_radius = float(self.model.geom("torso").size[0])
        self.goal = np.array(self.model.site("goal").pos[:2])
        cell = self.model.site("shared_cell")
        self.cell_lo = np.array(cell.pos[:2] - cell.size[:2])
        self.cell_hi = np.array(cell.pos[:2] + cell.size[:2])
        # The seed places the person slightly off the centre line: still in the
        # way (the radii sum to 0.45 m), never in the same spot twice.
        self.human_y0 = random.Random(seed).uniform(-0.15, 0.15)
        self.data.mocap_pos[0][1] = self.human_y0
        mujoco.mj_forward(self.model, self.data)

    @property
    def time(self) -> float:
        return float(self.data.time)

    @property
    def robot_xy(self) -> np.ndarray:
        return np.array(self.data.xpos[self.robot_body][:2])

    @property
    def human_xy(self) -> np.ndarray:
        return np.array(self.data.mocap_pos[0][:2])

    def separation(self) -> float:
        """The sim's ground-truth robot-to-human separation: planar centre distance
        minus both radii (a cylinder and a capsule seen from above are discs)."""
        centres = float(np.linalg.norm(self.robot_xy - self.human_xy))
        return max(0.0, centres - self.robot_radius - self.human_radius)

    def zone(self) -> str:
        inside = bool(
            np.all(self.robot_xy >= self.cell_lo) and np.all(self.robot_xy <= self.cell_hi)
        )
        return ":SharedCell" if inside else ":OpenFloor"

    def distance_to_goal(self) -> float:
        return float(np.linalg.norm(self.goal - self.robot_xy))

    def step(self, velocity: tuple[float, float]) -> bool:
        """Applies one actuator command for a whole control tick and returns
        whether the robot touched the human at any physics substep of it."""
        self.data.ctrl[:] = velocity
        touched = False
        walk_start = HUMAN_WALK_TICK * TICK_SECONDS
        for _ in range(self.substeps):
            walked = min(HUMAN_WALK_LIMIT, HUMAN_WALK_SPEED * max(0.0, self.data.time - walk_start))
            self.data.mocap_pos[0][1] = self.human_y0 + walked
            mujoco.mj_step(self.model, self.data)
            touched = touched or self.in_contact()
        return touched

    def in_contact(self) -> bool:
        for contact in self.data.contact[: self.data.ncon]:
            bodies = {
                int(self.model.geom_bodyid[contact.geom1]),
                int(self.model.geom_bodyid[contact.geom2]),
            }
            if bodies == {self.robot_body, self.human_body}:
                return True
        return False


class ScriptedPolicy:
    """Deliberately imperfect: heads straight for the goal at cruise speed with a
    small seeded dither, and never slows down for anyone."""

    def __init__(self, seed: int) -> None:
        self._rng = random.Random(seed)

    def propose(self, robot_xy: np.ndarray, goal_xy: np.ndarray) -> tuple[float, float]:
        to_goal = goal_xy - robot_xy
        distance = float(np.linalg.norm(to_goal))
        if distance < GOAL_RADIUS:
            return (0.0, 0.0)
        speed = CRUISE_SPEED + self._rng.uniform(-0.05, 0.05)
        heading = to_goal / distance
        return (float(heading[0] * speed), float(heading[1] * speed))


# --- the Horismos side: one decision per tick -----------------------------------


@dataclass
class Decision:
    verdict: str  # ":Permitted" | ":Attenuated" | ":Blocked" | "no verdict"
    rule: str | None  # the rule that derived it, e.g. ":noProximityReadingBlocks"
    released: float  # m/s that may reach the actuator
    capped_verdict: str | None = None  # D8: the rules' word on the capped command
    proof: str = ""

    @property
    def label(self) -> str:
        return f"{self.verdict} via {self.rule}" if self.rule else self.verdict


class Shield:
    """The shield: the overlay's rules over one ``:MotionCommand`` per tick.

    D7 — a fresh ``World`` per decision. Each tick is one transaction over one
    subject (``:cmd-t0042``), so every proof is self-contained and the cost per
    decision is constant. A standing world with a fresh subject per tick re-chains
    every earlier command on each transaction and trips the forward-chaining
    iteration cap at tick 100; replacing one subject needs a retract before every
    dropout tick, which makes two decisions of one. See README.md."""

    def __init__(self) -> None:
        self.onto = Ontology.load(example_path(ONTOLOGY))

    def decide(self, tick: int, speed: float, separation: float | None, zone: str) -> Decision:
        world = World(self.onto)
        state: dict[str, Any] = {"requested_speed": speed, "e_stop_engaged": False, "zone": zone}
        if separation is not None:
            state["nearest_human_distance"] = separation
        subject = f":cmd-t{tick:04d}"
        ref = world.assert_(":MotionCommand", subject=subject, **state)
        verdict = world.value(ref, "verdict")
        if verdict is None:
            # Cannot happen with this overlay (verify() proves every region has a
            # verdict), but the adapter still fails closed and says so.
            print(f"tick {tick}: no verdict for {subject} — treated as :Blocked", file=sys.stderr)
            return Decision(NO_VERDICT, None, 0.0)
        proof = world.explain(ref, "verdict")
        name = self.onto.compact(verdict.iri)
        rule = proof.rule if proof is not None else None
        rendered = proof.render() if proof is not None else ""
        if name == ":Permitted":
            return Decision(name, rule, speed, proof=rendered)
        if name == ":Attenuated":
            capped = min(speed, ATTENUATED_SPEED)
            state["requested_speed"] = capped
            ref = world.assert_(":MotionCommand", subject=f"{subject}-capped", **state)
            recheck = world.value(ref, "verdict")
            recheck_name = self.onto.compact(recheck.iri) if recheck is not None else NO_VERDICT
            released = capped if recheck_name == ":Permitted" else 0.0
            return Decision(name, rule, released, recheck_name, rendered)
        return Decision(name, rule, 0.0, proof=rendered)


# --- one run ----------------------------------------------------------------------


@dataclass
class Tick:
    tick: int
    time: float
    requested: float
    separation: float | None
    zone: str
    verdict: str  # Decision.label, or SHIELD_OFF
    actuated: float
    contact: bool
    proof: str = ""  # the rendered proof tree, in the JSON result (traces are output)


@dataclass
class RunResult:
    shield: str
    seed: int
    ticks: int
    contact_tick: int | None
    goal_tick: int | None
    progress: float  # fraction of the initial goal distance closed (1.0 once at the goal)
    final_separation: float
    counts: dict[str, int]
    sequence: list[str]  # one Decision.label per tick (empty with the shield off)
    capped_refused: int = 0  # D8: attenuated ticks whose capped command was not :Permitted
    decision_seconds: list[float] = field(default_factory=list)
    trace: list[Tick] = field(default_factory=list)

    @property
    def summary(self) -> str:
        return run_length(self.sequence)

    def latency(self) -> dict[str, float] | None:
        if not self.decision_seconds:
            return None
        ordered = sorted(self.decision_seconds)
        return {
            "p50_us": statistics.median(ordered) * 1e6,
            "p99_us": ordered[min(len(ordered) - 1, int(len(ordered) * 0.99))] * 1e6,
            "max_us": ordered[-1] * 1e6,
        }


def run_length(sequence: list[str]) -> str:
    """``Permitted ×20 → Blocked via ... ×8``: the readable form of a sequence."""
    parts: list[str] = []
    for label in sequence:
        if parts and parts[-1].startswith(label + " ×"):
            head, count = parts[-1].rsplit(" ×", 1)
            parts[-1] = f"{head} ×{int(count) + 1}"
        else:
            parts.append(f"{label} ×1")
    return " → ".join(parts)


def run(
    shield_on: bool,
    seed: int,
    ticks: int,
    *,
    trace: bool = False,
    record: Path | None = None,
) -> RunResult:
    scene = Scene(seed)
    policy = ScriptedPolicy(seed)
    shield = Shield() if shield_on else None
    recorder = make_recorder(scene.model, record, "shield-on" if shield_on else "shield-off")
    initial = scene.distance_to_goal()
    result = RunResult(
        shield="on" if shield_on else "off",
        seed=seed,
        ticks=ticks,
        contact_tick=None,
        goal_tick=None,
        progress=0.0,
        final_separation=0.0,
        counts={},
        sequence=[],
    )
    if trace:
        print(
            f"\n[shield {result.shield}] tick   t/s  proposed   separation  zone         "
            f"verdict                                        actuated"
        )
    for tick in range(ticks):
        now = scene.time  # the decision is stamped with the state it was made in
        vx, vy = policy.propose(scene.robot_xy, scene.goal)
        requested = round(math.hypot(vx, vy), 3)
        separation = None if tick in DROPOUT_TICKS else round(scene.separation(), 3)
        zone = scene.zone()
        proof = ""
        if shield is None:
            label, released = SHIELD_OFF, requested
        else:
            started = time.perf_counter()
            decision = shield.decide(tick, requested, separation, zone)
            result.decision_seconds.append(time.perf_counter() - started)
            label, released, proof = decision.label, decision.released, decision.proof
            result.sequence.append(label)
            result.counts[decision.verdict] = result.counts.get(decision.verdict, 0) + 1
            if decision.capped_verdict not in (None, ":Permitted"):
                result.capped_refused += 1
        scale = released / requested if requested > 0 else 0.0
        actuated = (vx * scale, vy * scale)  # same heading, the released speed
        if recorder is not None:
            recorder.frame(scene, label)
        touched = scene.step(actuated)
        if touched and result.contact_tick is None:
            result.contact_tick = tick
        if result.goal_tick is None and scene.distance_to_goal() < GOAL_RADIUS:
            result.goal_tick = tick
        entry = Tick(
            tick, now, requested, separation, zone, label, round(released, 3), touched, proof
        )
        result.trace.append(entry)
        if trace:
            reading = "no reading" if separation is None else f"{separation:5.2f} m"
            flag = "  CONTACT" if touched else ""
            print(
                f"  {tick:4d} {now:5.2f}  {requested:4.2f} m/s  {reading:>10}  {zone:<12} "
                f"{label:<46} {released:4.2f} m/s{flag}"
            )
    if recorder is not None:
        recorder.close()
    closed = (initial - scene.distance_to_goal()) / initial
    result.progress = 1.0 if result.goal_tick is not None else max(0.0, min(1.0, closed))
    result.final_separation = round(scene.separation(), 3)
    return result


# --- optional clips ---------------------------------------------------------------

VERDICT_COLOURS = {
    ":Permitted": (0.2, 0.4, 0.85, 1.0),  # blue, as drawn in the scene
    ":Attenuated": (0.95, 0.65, 0.15, 1.0),  # amber
    ":Blocked": (0.8, 0.15, 0.15, 1.0),  # red
    NO_VERDICT: (0.8, 0.15, 0.15, 1.0),
    SHIELD_OFF: (0.45, 0.45, 0.5, 1.0),  # grey: nobody is looking
}


class Recorder:
    """Renders one frame per tick with MuJoCo's offscreen renderer, colouring the
    chassis by the tick's verdict (blue permitted, amber attenuated, red blocked,
    grey with the shield off), and writes an MP4 through ffmpeg when it is on the
    PATH, or a folder of PPM frames otherwise. Rendering never touches the physics
    state, so a recorded run repeats an unrecorded one exactly."""

    def __init__(self, model: mujoco.MjModel, target: Path, name: str) -> None:
        self.renderer = mujoco.Renderer(model, height=360, width=640)
        self.chassis = model.geom("chassis").id
        target.mkdir(parents=True, exist_ok=True)
        self.frames = 0
        self.ffmpeg: subprocess.Popen[bytes] | None = None
        self.folder: Path | None = None
        if shutil.which("ffmpeg"):
            self.output = target / f"{name}.mp4"
            self.ffmpeg = subprocess.Popen(
                [
                    "ffmpeg", "-loglevel", "error", "-y",
                    "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "640x360",
                    "-r", str(round(1 / TICK_SECONDS)), "-i", "-",
                    "-c:v", "libx264", "-pix_fmt", "yuv420p", str(self.output),
                ],
                stdin=subprocess.PIPE,
            )  # fmt: skip
        else:
            self.folder = target / name
            self.folder.mkdir(exist_ok=True)
            self.output = self.folder

    def frame(self, scene: Scene, label: str) -> None:
        verdict = label.split(" via ")[0]
        scene.model.geom_rgba[self.chassis] = VERDICT_COLOURS.get(
            verdict, VERDICT_COLOURS[SHIELD_OFF]
        )
        self.renderer.update_scene(scene.data, camera="side")
        pixels = self.renderer.render()
        if self.ffmpeg is not None and self.ffmpeg.stdin is not None:
            self.ffmpeg.stdin.write(pixels.tobytes())
        elif self.folder is not None:
            (self.folder / f"{self.frames:05d}.ppm").write_bytes(
                b"P6 640 360 255\n" + pixels.tobytes()
            )
        self.frames += 1

    def close(self) -> None:
        if self.ffmpeg is not None and self.ffmpeg.stdin is not None:
            self.ffmpeg.stdin.close()
            self.ffmpeg.wait()
        self.renderer.close()
        how = "MP4 via ffmpeg" if self.ffmpeg is not None else "PPM frames (no ffmpeg on PATH)"
        print(f"recorded {self.frames} frames → {self.output} ({how})")


def make_recorder(model: mujoco.MjModel, target: Path | None, name: str) -> Recorder | None:
    if target is None:
        return None
    try:
        return Recorder(model, target, name)
    except Exception as error:  # no GL context (headless CI, SSH without a display)
        print(f"not recording {name}: MuJoCo could not open an offscreen GL context ({error})")
        return None


# --- the report -------------------------------------------------------------------


def at(tick: int) -> str:
    return f"(t = {tick * TICK_SECONDS:.2f} s)"


def report(result: RunResult) -> str:
    head = f"shield {result.shield:<3} seed {result.seed}  {result.ticks} ticks: "
    if result.contact_tick is None:
        head += "no human contact"
    else:
        head += f"HUMAN CONTACT at tick {result.contact_tick} {at(result.contact_tick)}"
    if result.goal_tick is None:
        head += f"; goal not reached ({result.progress:.0%} of the way)"
    else:
        head += f"; goal reached at tick {result.goal_tick} {at(result.goal_tick)}"
    lines = [head]
    if result.shield == "on":
        no_reading = sum(":noProximityReadingBlocks" in s for s in result.sequence)
        counts = " · ".join(f"{k} {v}" for k, v in sorted(result.counts.items()))
        missing = result.counts.get(NO_VERDICT, 0)
        lines.append(
            f"  verdicts: {counts}  (no-reading :Blocked {no_reading}, {NO_VERDICT} {missing})"
        )
        lines.append(f"  sequence: {result.summary}")
        capped = result.counts.get(":Attenuated", 0)
        if capped:
            lines.append(
                f"  attenuated commands re-checked at {ATTENUATED_SPEED} m/s: {capped}, "
                f"of which {result.capped_refused} refused (released as zero)"
            )
        lat = result.latency()
        if lat is not None:
            lines.append(
                f"  decision time from Python (fresh World per tick, n={len(result.sequence)}): "
                f"p50 {lat['p50_us']:.0f} µs · p99 {lat['p99_us']:.0f} µs · "
                f"max {lat['max_us']:.0f} µs"
            )
    return "\n".join(lines)


def cost(off: RunResult, on: RunResult) -> dict[str, Any]:
    """What the shield cost, against the same policy with nothing in the way."""
    blocked = on.counts.get(":Blocked", 0) + on.counts.get(NO_VERDICT, 0)
    attenuated = on.counts.get(":Attenuated", 0)
    delay = None if on.goal_tick is None or off.goal_tick is None else on.goal_tick - off.goal_tick
    return {
        "blocked_ticks": blocked,
        "attenuated_ticks": attenuated,
        "goal_delay_ticks": delay,
        "goal_delay_seconds": None if delay is None else round(delay * TICK_SECONDS, 2),
        "progress_off": round(off.progress, 3),
        "progress_on": round(on.progress, 3),
        "contact_off": off.contact_tick is not None,
        "contact_on": on.contact_tick is not None,
    }


def cost_text(c: dict[str, Any]) -> str:
    late = (
        "never reaches the goal in this run"
        if c["goal_delay_ticks"] is None
        else f"reaches the goal {c['goal_delay_ticks']} ticks ({c['goal_delay_seconds']} s) later"
    )
    ticks = f"{c['blocked_ticks']} blocked + {c['attenuated_ticks']} attenuated ticks"
    progress = f"progress {c['progress_on']:.0%} vs {c['progress_off']:.0%} unshielded"
    return f"cost of the shield: {ticks}; {late}; {progress}"


def to_json(off: RunResult | None, on: RunResult | None) -> dict[str, Any]:
    def one(r: RunResult) -> dict[str, Any]:
        d = asdict(r)
        d["summary"] = r.summary
        d["latency_us"] = r.latency()
        d.pop("decision_seconds")
        return d

    out: dict[str, Any] = {
        "scene": SCENE.name,
        "ontology": ONTOLOGY,
        "mujoco": mujoco.__version__,
        "horismos": horismos.__version__,
        "tick_seconds": TICK_SECONDS,
        "attenuated_speed": ATTENUATED_SPEED,
        "dropout_ticks": [DROPOUT_TICKS.start, DROPOUT_TICKS.stop],
        "runs": {r.shield: one(r) for r in (off, on) if r is not None},
    }
    if off is not None and on is not None:
        out["cost"] = cost(off, on)
    return out


# --- the pinned sequence: `--check` and `--pin` -----------------------------------


def pin(seed: int, ticks: int) -> None:
    on = run(True, seed, ticks)
    EXPECTED.write_text(
        json.dumps(
            {
                "seed": seed,
                "ticks": ticks,
                "mujoco": mujoco.__version__,
                "summary": on.summary,
                "sequence": on.sequence,
            },
            indent=1,
            ensure_ascii=False,
        )
        + "\n"
    )
    print(f"pinned {len(on.sequence)} verdicts to {EXPECTED}\n{on.summary}")


def check(seed: int, ticks: int) -> int:
    """The CI assertion (``make demo-shield``): never pixels, never a skip."""
    expected = json.loads(EXPECTED.read_text())
    if (expected["seed"], expected["ticks"]) != (seed, ticks):
        seed, ticks = expected["seed"], expected["ticks"]
        print(f"checking the pinned seed {seed} over {ticks} ticks")
    if expected["mujoco"] != mujoco.__version__:
        print(
            f"warning: the sequence was pinned on mujoco {expected['mujoco']}, this is "
            f"{mujoco.__version__}; physics may differ",
        )
    failures: list[str] = []
    off = run(False, seed, ticks)
    print(report(off))
    if off.contact_tick is None:
        failures.append("shield off: expected a human contact and recorded none")
    on = run(True, seed, ticks)
    print(report(on))
    if on.contact_tick is not None:
        failures.append(f"shield on: recorded a human contact at tick {on.contact_tick}")
    pinned: list[str] = expected["sequence"]
    if on.sequence != pinned:
        pairs = zip(on.sequence, pinned, strict=False)
        first = next(
            (i for i, (a, b) in enumerate(pairs) if a != b), min(len(on.sequence), len(pinned))
        )
        got = on.sequence[first] if first < len(on.sequence) else "<end>"
        want = pinned[first] if first < len(pinned) else "<end>"
        failures.append(
            f"shield on: verdicts diverge at tick {first}: got {got!r}, pinned {want!r}"
        )
    if not any(s.startswith(":Attenuated") for s in pinned):
        failures.append("the pinned sequence holds no :Attenuated tick")
    if ":Blocked via :noProximityReadingBlocks" not in pinned:
        failures.append("the pinned sequence holds no no-reading :Blocked tick")
    again = run(True, seed, ticks)
    actuated = [t.actuated for t in on.trace], [t.actuated for t in again.trace]
    if again.sequence != on.sequence or actuated[0] != actuated[1]:
        failures.append("shield on: a second run with the same seed did not repeat the first")
    print(cost_text(cost(off, on)))
    if failures:
        print("\nFAILED:\n  " + "\n  ".join(failures))
        return 1
    print(
        f"\nOK: {len(on.sequence)} verdicts match the pinned sequence and repeat; "
        "shield off collided, shield on did not"
    )
    return 0


# --- CLI --------------------------------------------------------------------------


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    add = parser.add_argument
    add("--shield", choices=["both", "on", "off"], default="both")
    add("--seed", type=int, default=7)
    add("--ticks", type=int, default=300, help=f"control ticks of {TICK_SECONDS:g} s")
    add("--trace", action="store_true", help="print one line per tick")
    add("--json", type=Path, metavar="PATH", help="write the machine-readable result")
    add("--record", type=Path, metavar="DIR", help="render a clip per run (needs a GL context)")
    add("--check", action="store_true", help="assert the pinned outcome; exit 1 otherwise")
    add("--pin", action="store_true", help="rewrite expected-verdicts.json from this run")
    args = parser.parse_args(argv)

    if args.pin:
        pin(args.seed, args.ticks)
        return 0
    if args.check:
        return check(args.seed, args.ticks)

    off = on = None
    if args.shield in ("both", "off"):
        off = run(False, args.seed, args.ticks, trace=args.trace, record=args.record)
        print(report(off))
    if args.shield in ("both", "on"):
        on = run(True, args.seed, args.ticks, trace=args.trace, record=args.record)
        print(report(on))
    if off is not None and on is not None:
        print(cost_text(cost(off, on)))
    if args.json is not None:
        args.json.write_text(json.dumps(to_json(off, on), indent=1, ensure_ascii=False) + "\n")
        print(f"wrote {args.json}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
