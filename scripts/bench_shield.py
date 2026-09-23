"""PRP-009 P1: per-decision latency of the motion-command shield through the
public Python API — p50 / p99 / max over a stated sample count.

The measured path is a **release** build of the extension module. `make develop`
builds it *without* `--release` (a debug build, several times slower), so a
number from that build is not the measured path and must not be reported as one.
Build the release module into the same venv first:

    uv run maturin develop --uv --release
    uv run python scripts/bench_shield.py [--samples N] [--sweep-samples N] [--warmup N]

or run the script against an installed wheel (`pip install horismos`), which is
always a release build. Nothing on the Python side can tell the two apart, so the
build command is part of the record: the README beside the Rust benches
(`crates/horismos-core/benches/README.md`) states which one each recorded run used.

What one *decision* is (doc 03): `World.assert_` with the command's whole state
in one call — the five-check gate and forward chaining to the fixed point run
inside that transaction — then `World.value` reads `:verdict`. `explain` is timed
apart, never inside the decision. Rows mirror the Rust `shield_percentiles`
bench: the three verdict paths on a standing command, the first transaction of a
fresh `World`, `explain`, and a world-size sweep. Every transaction resets and
re-derives all standing derivations, and the forward chainer's iteration cap
counts firings, so the sweep reports the sizes the kernel refuses as refused
rows, with the error's own text.

This is a measurement script, not a test and not a product claim: the published
latency claim is PRP-009 decision 2.
"""

from __future__ import annotations

import argparse
import platform
import sys
import time
from collections.abc import Callable, Iterator
from dataclasses import dataclass

import horismos
from horismos import HorismosError, Ontology, Ref, World
from horismos.examples import path

# The three verdict paths as the Physical AI page exercises them; the same
# lexical values as the Rust fixture (`benches/shield_common/mod.rs`).
CASES: dict[str, tuple[dict[str, object], str]] = {
    "permitted_open_floor": (
        {
            "requested_speed": 1.8,
            "nearest_human_distance": 2.4,
            "e_stop_engaged": False,
            "zone": ":OpenFloor",
        },
        ":Permitted",
    ),
    "attenuated_shared_cell": (
        {
            "requested_speed": 1.2,
            "nearest_human_distance": 0.8,
            "e_stop_engaged": False,
            "zone": ":SharedCell",
        },
        ":Attenuated",
    ),
    "blocked_no_reading": (
        {"requested_speed": 1.0, "e_stop_engaged": False, "zone": ":OpenFloor"},
        ":Blocked",
    ),
}

# Beside `:cmd`: 99 further standing verdicts fit under the 100-firing cap,
# 100 do not (see `shield_common::WORLD_SIZES` on the Rust side).
WORLD_SIZES = (1, 10, 50, 99, 100, 1000)


@dataclass(frozen=True)
class Row:
    name: str
    samples: int
    p50_ns: int
    p99_ns: int
    max_ns: int


def measure(
    name: str,
    warmup: int,
    samples: int,
    prepare: Callable[[], object],
    run: Callable[[object], None],
) -> Row:
    """Times ``run`` once per sample after ``warmup`` untimed calls; ``prepare``
    runs before each call, outside the timed section."""
    for _ in range(warmup):
        run(prepare())
    times: list[int] = []
    for _ in range(samples):
        state = prepare()
        start = time.perf_counter_ns()
        run(state)
        times.append(time.perf_counter_ns() - start)
    times.sort()
    last = len(times) - 1
    return Row(
        name=name,
        samples=samples,
        p50_ns=times[round(last * 0.50)],
        p99_ns=times[round(last * 0.99)],
        max_ns=times[last],
    )


def fmt(ns: int) -> str:
    us = ns / 1000.0
    return f"{us / 1000.0:.3f} ms" if us >= 1000.0 else f"{us:.2f} µs"


def decide(world: World, subject: str, state: dict[str, object]) -> Ref | None:
    """One decision: the whole state in one transaction, then read `:verdict`."""
    ref = world.assert_(":MotionCommand", subject=subject, **state)
    verdict = world.value(ref, ":verdict")
    return verdict if isinstance(verdict, Ref) else None


def standing_world(onto: Ontology, case: str, padding: int) -> tuple[World, Ref]:
    """A world at its fixed point: ``padding`` standing `:Permitted` commands,
    then `:cmd` decided for ``case``. Raises the kernel's own error when the
    iteration cap refuses the world."""
    world = World(onto)
    permitted, _ = CASES["permitted_open_floor"]
    for i in range(padding):
        world.assert_(":MotionCommand", subject=f":standing-{i}", **permitted)
    state, expected = CASES[case]
    ref = decide(world, ":cmd", state)
    if ref != onto.ref(expected):
        raise AssertionError(f"{case}: expected {expected}, got {ref}")
    return world, onto.ref(expected)


def replace_standing_row(onto: Ontology, case: str, warmup: int, samples: int) -> Row:
    state, _ = CASES[case]
    world, expected = standing_world(onto, case, 0)

    def run(_: object) -> None:
        assert decide(world, ":cmd", state) == expected

    return measure(f"replace_standing/{case}", warmup, samples, lambda: None, run)


def cold_world_row(onto: Ontology, case: str, warmup: int, samples: int) -> Row:
    state, expected_term = CASES[case]
    expected = onto.ref(expected_term)

    def run(world: object) -> None:
        assert isinstance(world, World)
        assert decide(world, ":cmd", state) == expected

    return measure(f"cold_world/{case}", warmup, samples, lambda: World(onto), run)


def explain_row(onto: Ontology, warmup: int, samples: int) -> Row:
    world, _ = standing_world(onto, "attenuated_shared_cell", 0)
    cmd = world.instances(":MotionCommand")[0]

    def run(_: object) -> None:
        proof = world.explain(cmd, ":verdict")
        assert proof is not None and proof.children

    return measure("explain/attenuated_shared_cell", warmup, samples, lambda: None, run)


def world_size_row(onto: Ontology, padding: int, warmup: int, samples: int) -> Row | str:
    case = "attenuated_shared_cell"
    state, _ = CASES[case]
    try:
        world, expected = standing_world(onto, case, padding)
    except HorismosError as error:
        return f"`world_size/{padding}`: {error}"

    def run(_: object) -> None:
        assert decide(world, ":cmd", state) == expected

    return measure(f"world_size/{padding}", min(warmup, samples), samples, lambda: None, run)


def rows(onto: Ontology, warmup: int, samples: int, sweep_samples: int) -> Iterator[Row | str]:
    for case in CASES:
        yield replace_standing_row(onto, case, warmup, samples)
    for case in CASES:
        yield cold_world_row(onto, case, warmup, samples)
    yield explain_row(onto, warmup, samples)
    for padding in WORLD_SIZES:
        yield world_size_row(onto, padding, warmup, sweep_samples)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--samples", type=int, default=20_000)
    parser.add_argument("--sweep-samples", type=int, default=2_000)
    parser.add_argument("--warmup", type=int, default=1_000)
    args = parser.parse_args(argv)

    onto = Ontology.load(path("gallery/safety-envelope.ttl"))
    print(
        f"horismos {horismos.__version__} · Python {platform.python_version()} · "
        f"{platform.system()} {platform.machine()} · "
        "build profile: see the command that built the module (not detectable here)"
    )
    print()
    print("| case | samples | p50 | p99 | max |")
    print("|---|---:|---:|---:|---:|")
    refused: list[str] = []
    for row in rows(onto, args.warmup, args.samples, args.sweep_samples):
        if isinstance(row, str):
            refused.append(row)
            name = row.split(":", 1)[0]
            print(f"| {name} | — | refused | refused | refused |")
            continue
        print(
            f"| `{row.name}` | {row.samples} | {fmt(row.p50_ns)} | {fmt(row.p99_ns)} | "
            f"{fmt(row.max_ns)} |"
        )
    for line in refused:
        print()
        print(line)
    return 0


if __name__ == "__main__":
    sys.exit(main())
