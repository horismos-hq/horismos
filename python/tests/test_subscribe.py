"""U8 — subscriptions (post-commit dispatch, B2) and the GIL policy (D3)."""

import threading
import time
from pathlib import Path
from typing import Any

import horismos
import pytest
from horismos import Ontology, Ref, World


def test_a_callback_fires_once_per_matching_commit(onto: Ontology, world: World) -> None:
    seen: list[tuple[Ref, Any]] = []
    world.subscribe("acuity", lambda s, v: seen.append((s, v)), value=onto.ref(":Immediate"))

    p = world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    assert seen == [(p, onto.ref(":Immediate"))]

    # A non-matching commit adds nothing for the pattern.
    world.assert_(":Patient", heart_rate=70)
    assert len(seen) == 1


def test_a_rejected_transaction_notifies_nobody(onto: Ontology, world: World) -> None:
    fired: list[Ref] = []
    world.subscribe("heart_rate", lambda s, v: fired.append(s))
    with pytest.raises(horismos.HorismosValidationError):
        world.assert_(":Patient", heart_rate=1280)
    assert fired == []


def test_cancel_stops_delivery(onto: Ontology, world: World) -> None:
    fired: list[Ref] = []
    sub = world.subscribe("heart_rate", lambda s, v: fired.append(s))
    world.assert_(":Patient", heart_rate=100)
    sub.cancel()
    world.assert_(":Patient", heart_rate=101)
    assert len(fired) == 1


def test_a_mutation_from_a_callback_is_its_own_later_transaction(
    onto: Ontology, world: World
) -> None:
    order: list[str] = []
    companion: list[Ref] = []

    def on_acuity(subject: Ref, value: Any) -> None:
        order.append("callback")
        # Re-entrant mutation: a new transaction, dispatched after this one.
        companion.append(world.assert_(":Patient", heart_rate=60))

    world.subscribe("acuity", on_acuity)
    world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    order.append("after")

    assert order == ["callback", "after"]
    assert world.value(companion[0], ":heartRate") == 60


def test_two_threads_share_a_world_without_deadlocking(onto: Ontology, world: World) -> None:
    """P3 (revised): the lock is never blocked on while holding the GIL, so
    a writer mid-transaction and a reader on another thread make progress
    instead of deadlocking (the batch review's reproduced hang)."""
    deadline = time.perf_counter() + 0.5
    failures: list[BaseException] = []

    p = world.assert_(":Patient", subject=":shared", heart_rate=99)

    def writer() -> None:
        try:
            while time.perf_counter() < deadline:
                world.assert_(":Patient", heart_rate=100)
        except BaseException as e:
            failures.append(e)

    def reader() -> None:
        try:
            while time.perf_counter() < deadline:
                world.value(p, ":heartRate")
        except BaseException as e:
            failures.append(e)

    threads = [
        threading.Thread(target=writer, daemon=True),
        threading.Thread(target=reader, daemon=True),
    ]
    for t in threads:
        t.start()
    for t in threads:
        t.join(timeout=15)
    assert not any(t.is_alive() for t in threads), "threads wedged: deadlock"
    assert failures == []


def test_dispatch_fires_callbacks_on_the_asserting_thread(onto: Ontology, world: World) -> None:
    """B2: with two threads sharing a World and a subscription registered,
    each thread's committed facts must dispatch on that same thread — an
    invariant, not a timing race, so thread-local dispatch state can never
    let one thread drain another's queued callbacks."""
    callback_ident_by_ref: dict[Ref, int] = {}
    lock = threading.Lock()

    def on_heart_rate(subject: Ref, value: Any) -> None:
        with lock:
            callback_ident_by_ref[subject] = threading.get_ident()

    world.subscribe("heart_rate", on_heart_rate)

    own_refs: dict[int, list[Ref]] = {}
    failures: list[BaseException] = []

    def worker() -> None:
        try:
            ident = threading.get_ident()
            refs = [world.assert_(":Patient", heart_rate=100 + i) for i in range(50)]
            own_refs[ident] = refs
        except BaseException as e:
            failures.append(e)

    threads = [threading.Thread(target=worker, daemon=True) for _ in range(2)]
    for t in threads:
        t.start()
    for t in threads:
        t.join(timeout=15)

    assert not any(t.is_alive() for t in threads), "threads wedged: deadlock"
    assert failures == []
    assert len(own_refs) == 2
    for ident, refs in own_refs.items():
        for ref in refs:
            assert callback_ident_by_ref[ref] == ident, (
                "a callback for a fact asserted on one thread fired on another"
            )


def test_a_long_native_call_releases_the_gil(tmp_path: Path) -> None:
    """D3: while Rust loads/validates a large ontology, other Python
    threads keep running. If the GIL were held across the native call,
    the main thread could not tick at all during the load window."""
    big = tmp_path / "big.ttl"
    lines = [
        "@prefix : <https://example.org/big#> .",
        "@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .",
    ]
    lines.extend(f":C{i} a rdfs:Class ." for i in range(60_000))
    big.write_text("\n".join(lines))

    # Baseline: how fast does this machine tick, solo?
    def tick_for(seconds: float) -> int:
        n = 0
        end = time.perf_counter() + seconds
        while time.perf_counter() < end:
            n += 1
        return n

    rate = tick_for(0.1) / 0.1

    window: dict[str, float] = {}

    def load() -> None:
        window["start"] = time.perf_counter()
        Ontology.load(big)
        window["end"] = time.perf_counter()

    loader = threading.Thread(target=load)
    loader.start()
    ticks = 0
    while loader.is_alive():
        ticks += 1
    loader.join()

    duration = window["end"] - window["start"]
    assert duration > 0.02, "the fixture must be big enough to measure"
    expected = rate * duration
    assert ticks > expected * 0.2, (
        f"main thread starved during the native load: {ticks} ticks vs "
        f"~{expected:.0f} expected over {duration * 1000:.0f}ms — the GIL "
        "was not released"
    )
