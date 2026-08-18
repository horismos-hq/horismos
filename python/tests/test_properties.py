"""U9 — Hypothesis property tests: the boundary invariants (B4).

These mirror the Rust proptest suites at the Python surface: atomic
rejection, byte-identical determinism, value round-trip.
"""

from datetime import UTC, datetime
from decimal import Decimal

import horismos
from horismos import Ontology, World
from hypothesis import given
from hypothesis import strategies as st

SYMPTOMS = [":ChestPain", ":CrushingChestPain", ":Symptom"]


@given(
    heart_rate=st.integers(min_value=-(2**70), max_value=2**70),
    symptoms=st.lists(st.sampled_from(SYMPTOMS), max_size=3),
)
def test_any_rejected_payload_leaves_the_world_byte_identical(
    onto: Ontology, heart_rate: int, symptoms: list[str]
) -> None:
    world = World(onto)
    world.assert_(":Patient", heart_rate=100)
    before = world.snapshot()
    try:
        world.assert_(":Patient", heart_rate=heart_rate, has_symptom=symptoms)
    except horismos.HorismosError:
        diff = world.diff(before)
        assert diff.added == [] and diff.removed == []


@given(
    rates=st.lists(st.integers(min_value=0, max_value=300), min_size=1, max_size=5),
    symptoms=st.lists(st.sampled_from(SYMPTOMS), max_size=3),
)
def test_two_identically_built_worlds_agree_byte_for_byte(
    onto: Ontology, rates: list[int], symptoms: list[str]
) -> None:
    def build() -> World:
        world = World(onto)
        for rate in rates:
            world.assert_(":Patient", heart_rate=rate, has_symptom=symptoms)
        return world

    a, b = build(), build()
    assert a.snapshot().to_bytes() == b.snapshot().to_bytes()
    # Proof rendering is part of the determinism claim.
    for pa, pb in zip(a.instances(":Patient"), b.instances(":Patient"), strict=True):
        ea, eb = a.explain(pa, ":acuity"), b.explain(pb, ":acuity")
        assert (ea is None) == (eb is None)
        if ea is not None and eb is not None:
            assert ea.render() == eb.render()


@given(rate=st.integers(min_value=0, max_value=300))
def test_in_range_integers_round_trip_exactly(onto: Ontology, rate: int) -> None:
    world = World(onto)
    p = world.assert_(":Patient", heart_rate=rate)
    assert world.value(p, ":heartRate") == rate


@given(
    rate=st.one_of(
        st.just(0.0),
        # xsd:decimal is rust_decimal in core: at most 28 decimal places,
        # so a float whose shortest repr needs more fractional digits is
        # truncated (dated gap note, 02-tasks-and-gaps.md). ≥1e-10 keeps
        # any f64's shortest repr within 28 places, so the round-trip is
        # exact on this range; the floor itself is core's representation,
        # not the boundary's.
        st.floats(min_value=1e-10, max_value=300.0, allow_nan=False, allow_infinity=False),
    )
)
def test_floats_round_trip_through_decimal_losslessly(onto: Ontology, rate: float) -> None:
    world = World(onto)
    p = world.assert_(":Patient", heart_rate=rate)
    out = world.value(p, ":heartRate")
    assert isinstance(out, Decimal)
    # Rust renders the shortest round-trip decimal form of the f64.
    assert float(out) == rate


@given(
    note=st.text(max_size=200),
    flagged=st.booleans(),
    taken=st.datetimes(
        min_value=datetime(1900, 1, 1),
        max_value=datetime(2200, 1, 1),
        timezones=st.just(UTC),
    ),
)
def test_strings_bools_and_datetimes_round_trip(
    vonto: Ontology, note: str, flagged: bool, taken: datetime
) -> None:
    world = World(vonto)
    r = world.assert_(":Reading", note=note, flagged=flagged, taken_at=taken)
    assert world.value(r, ":note") == note
    assert world.value(r, ":flagged") is flagged
    assert world.value(r, ":takenAt") == taken
