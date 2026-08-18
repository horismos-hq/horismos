"""U3 (review follow-up) — value marshaling for the datatypes the triage
fixture does not declare: boolean, dateTime, duration, string."""

from datetime import UTC, datetime
from pathlib import Path

import horismos
import pytest
from horismos import Ontology, World

FIXTURE = """\
@prefix :      <https://example.org/v#> .
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .

:Reading a rdfs:Class .
:flagged a rdf:Property ; rdfs:domain :Reading ; rdfs:range xsd:boolean ;
    horis:cardinality "0..1" .
:takenAt a rdf:Property ; rdfs:domain :Reading ; rdfs:range xsd:dateTime ;
    horis:cardinality "0..1" .
:window a rdf:Property ; rdfs:domain :Reading ; rdfs:range xsd:duration ;
    horis:cardinality "0..1" .
:note a rdf:Property ; rdfs:domain :Reading ; rdfs:range xsd:string ;
    horis:cardinality "0..1" .
"""


@pytest.fixture(scope="module")
def vonto(tmp_path_factory: pytest.TempPathFactory) -> Ontology:
    path: Path = tmp_path_factory.mktemp("values") / "values.ttl"
    path.write_text(FIXTURE)
    return Ontology.load(path)


@pytest.fixture()
def vworld(vonto: Ontology) -> World:
    return World(vonto)


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
