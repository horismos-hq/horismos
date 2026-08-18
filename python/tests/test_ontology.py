"""U1 — the Ontology handle across the boundary."""

from pathlib import Path

import horismos
import pytest
from horismos import HorismosOntologyError, Ontology

from .conftest import SNOMED, TRIAGE


def test_load_with_imports_sees_the_triage_prefix(onto: Ontology) -> None:
    assert onto.namespaces[""] == "https://example.org/triage#"
    assert onto.namespaces["sct"] == "http://snomed.info/id/"


def test_load_accepts_path_objects_and_strings() -> None:
    a = Ontology.load(str(TRIAGE), imports=[str(SNOMED)])
    assert a.namespaces[""] == "https://example.org/triage#"


def test_ref_round_trips_a_curie(onto: Ontology) -> None:
    r = onto.ref(":Immediate")
    assert r.iri == "https://example.org/triage#Immediate"
    assert r.curie == ":Immediate"
    assert r == onto.ref("https://example.org/triage#Immediate")
    assert hash(r) == hash(onto.ref(":Immediate"))


def test_ref_rejects_an_unknown_prefix(onto: Ontology) -> None:
    with pytest.raises(HorismosOntologyError) as e:
        onto.ref("nope:Immediate")
    assert "nope" in str(e.value)


def test_ref_rejects_an_undeclared_term(onto: Ontology) -> None:
    with pytest.raises(HorismosOntologyError) as e:
        onto.ref(":Ghost")
    assert ":Ghost" in str(e.value) or ":Ghost" in str(e.value)


def test_rules_are_introspectable(onto: Ontology) -> None:
    rules = onto.rules
    assert [r.curie for r in rules] == [":r1"]
    (r1,) = rules
    assert r1.priority == 10
    assert r1.explain == "Tachycardia with chest pain — immediate track"


def test_verify_is_a_stub_until_prp_005(onto: Ontology) -> None:
    report = onto.verify()
    assert report.ok
    assert "PRP-005" in report.notes


def test_a_missing_file_raises_an_ontology_error() -> None:
    with pytest.raises(HorismosOntologyError):
        Ontology.load("does-not-exist.ttl")


def test_a_malformed_ontology_raises_an_ontology_error(tmp_path: Path) -> None:
    bad = tmp_path / "bad.ttl"
    bad.write_text("@prefix : <https://example.org/x#> .\n:this is not turtle")
    with pytest.raises(HorismosOntologyError) as e:
        Ontology.load(bad)
    assert isinstance(e.value, horismos.HorismosError)
