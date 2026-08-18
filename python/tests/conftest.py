"""Shared fixtures: the flagship triage ontology, loaded once per session."""

from pathlib import Path

import horismos
import pytest

REPO = Path(__file__).resolve().parents[2]
TRIAGE = REPO / "examples" / "triage" / "triage.ttl"
SNOMED = REPO / "examples" / "triage" / "snomed-ct-subset.ttl"


@pytest.fixture(scope="session")
def onto() -> horismos.Ontology:
    return horismos.Ontology.load(TRIAGE, imports=[SNOMED])


@pytest.fixture()
def world(onto: horismos.Ontology) -> horismos.World:
    return horismos.World(onto)


VALUES_FIXTURE = """\
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


@pytest.fixture(scope="session")
def vonto(tmp_path_factory: pytest.TempPathFactory) -> horismos.Ontology:
    """A small ontology declaring the datatypes triage.ttl does not."""
    path = tmp_path_factory.mktemp("values") / "values.ttl"
    path.write_text(VALUES_FIXTURE)
    return horismos.Ontology.load(path)


@pytest.fixture()
def vworld(vonto: horismos.Ontology) -> horismos.World:
    return horismos.World(vonto)
