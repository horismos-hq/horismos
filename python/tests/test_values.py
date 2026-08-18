"""U3 (review follow-up) — value marshaling for the datatypes the triage
fixture does not declare: boolean, dateTime, duration, string."""

from datetime import UTC, datetime

import horismos
import pytest
from horismos import World


def test_booleans_round_trip_as_real_bools(vworld: World) -> None:
    r = vworld.assert_(":Reading", flagged=True)
    assert vworld.value(r, ":flagged") is True


def test_tz_aware_datetimes_round_trip(vworld: World) -> None:
    taken = datetime(2026, 8, 18, 12, 30, tzinfo=UTC)
    r = vworld.assert_(":Reading", taken_at=taken)
    assert vworld.value(r, ":takenAt") == taken


def test_a_naive_datetime_is_refused_in_domain_terms(vworld: World) -> None:
    with pytest.raises(horismos.HorismosError):
        vworld.assert_(":Reading", taken_at=datetime(2026, 8, 18, 12, 30))


def test_durations_cross_as_their_lexical_form(vworld: World) -> None:
    r = vworld.assert_(":Reading", window="PT2H")
    assert vworld.value(r, ":window") == "PT2H"  # dated gap note: str, not a handle


def test_a_malformed_duration_is_a_core_worded_error(vworld: World) -> None:
    with pytest.raises(horismos.HorismosOntologyError) as e:
        vworld.assert_(":Reading", window="2 hours")
    assert "duration" in str(e.value)


def test_strings_round_trip(vworld: World) -> None:
    r = vworld.assert_(":Reading", note="pre-op baseline")
    assert vworld.value(r, ":note") == "pre-op baseline"


def test_an_oversized_int_is_not_a_bare_overflow_error(world: World) -> None:
    with pytest.raises(horismos.HorismosOntologyError) as e:
        world.assert_(":Patient", heart_rate=2**63)
    assert "64-bit" in str(e.value)


def test_a_nan_float_is_refused_in_core_words(world: World) -> None:
    with pytest.raises(horismos.HorismosOntologyError):
        world.assert_(":Patient", heart_rate=float("nan"))
