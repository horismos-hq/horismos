"""U1 — the Ontology handle across the boundary."""

import json
import subprocess
import sys
from pathlib import Path

import horismos
import pytest
from horismos import HorismosOntologyError, HorismosVerificationError, Ontology

from .conftest import SNOMED, TRIAGE


def test_load_with_imports_sees_the_triage_prefix(onto: Ontology) -> None:
    assert onto.namespaces[""] == "https://example.org/triage#"
    assert onto.namespaces["sct"] == "http://snomed.info/id/"


def test_load_accepts_string_paths() -> None:
    # Path objects are covered by the session fixture (conftest passes Paths).
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
    assert ":Ghost" in str(e.value)
    assert e.value.data["term"] == ":Ghost"


def test_rules_are_introspectable(onto: Ontology) -> None:
    rules = onto.rules
    assert [r.curie for r in rules] == [":r1", ":r2", ":r3", ":r4", ":r6", ":r5"]
    r1 = rules[0]
    assert r1.priority == 10
    assert r1.explain == "Tachycardia with chest pain — immediate track"


def test_compact_inverts_expand(onto: Ontology) -> None:
    assert onto.expand(":Patient") == "https://example.org/triage#Patient"
    assert onto.compact("https://example.org/triage#Patient") == ":Patient"


def test_the_introspection_tables_cover_the_declared_schema(onto: Ontology) -> None:
    props = {local: (iri, is_object) for iri, local, is_object, _ in onto._handle.property_table()}
    assert props["heartRate"] == ("https://example.org/triage#heartRate", False)
    assert props["hasSymptom"] == ("https://example.org/triage#hasSymptom", True)

    classes = {local: one_of for _, local, one_of in onto._handle.class_table()}
    assert "Patient" in classes
    assert classes["AcuityLevel"] == [
        "https://example.org/triage#Immediate",
        "https://example.org/triage#Urgent",
        "https://example.org/triage#Standard",
    ]


def test_verify_returns_the_complete_typed_report(vonto: Ontology) -> None:
    report = vonto.verify()
    assert report.ok
    assert report.exit_code == 0
    assert report.assumptions.input_closure == ()
    assert report.assumptions.external_profile_versions == {}
    assert report.assumptions.mapping_version is None
    assert report.assumptions.supported_subset == ()
    assert report.assumptions.unverified_external_effects == ()
    assert json.loads(report.to_json())["assumptions"] == {
        "input_closure": [],
        "external_profile_versions": {},
        "mapping_version": None,
        "supported_subset": [],
        "unverified_external_effects": [],
    }


def test_default_hard_load_attaches_the_complete_report(tmp_path: Path) -> None:
    incomplete = tmp_path / "incomplete.ttl"
    incomplete.write_text(
        """\
@prefix : <https://example.org/incomplete#> .
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
:Patient a rdfs:Class .
:decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
    horis:assignedBy horis:RulesOnly .
"""
    )
    with pytest.raises(HorismosVerificationError) as raised:
        Ontology.load(incomplete)
    assert raised.value.exit_code == 1
    assert raised.value.report.ok is False
    assert raised.value.report.complete == {":decision": False}
    assert raised.value.report.assumptions.input_closure == ()
    assert str(raised.value) == raised.value.report.witnesses[0].domain_text

    allowed = Ontology.load(incomplete, allow_incomplete=True)
    assert allowed.verify().to_json() == raised.value.report.to_json()


def test_explicit_imports_are_verified_as_one_assembled_ontology(tmp_path: Path) -> None:
    root = tmp_path / "root.ttl"
    rules = tmp_path / "rules.ttl"
    prefixes = """\
@prefix : <https://example.org/assembled#> .
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
"""
    root.write_text(
        prefixes
        + """\
:Patient a rdfs:Class .
:flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ; horis:cardinality "1" .
:decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
    horis:assignedBy horis:RulesOnly .
"""
    )
    rules.write_text(
        prefixes
        + """\
:cover a horis:Rule ;
    horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
    horis:then [ horis:property :decision ; horis:value true ] .
"""
    )
    assert Ontology.load(root, imports=[rules]).verify().ok


def test_verify_cli_emits_json_and_exit_one_for_findings(tmp_path: Path) -> None:
    incomplete = tmp_path / "cli-incomplete.ttl"
    incomplete.write_text(
        """\
@prefix : <https://example.org/cli#> .
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
:Patient a rdfs:Class .
:decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
    horis:assignedBy horis:RulesOnly .
"""
    )
    command = [sys.executable, "-m", "horismos", "verify", str(incomplete)]
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    repeated = subprocess.run(command, check=False, capture_output=True, text=True)
    assert result.returncode == 1
    assert result.stderr == ""
    assert repeated.returncode == result.returncode
    assert repeated.stderr == result.stderr
    assert repeated.stdout == result.stdout
    payload = json.loads(result.stdout)
    assert payload["ok"] is False
    assert payload["assumptions"] == {
        "input_closure": [],
        "external_profile_versions": {},
        "mapping_version": None,
        "supported_subset": [],
        "unverified_external_effects": [],
    }


def test_verify_cli_exit_zero_is_byte_deterministic() -> None:
    command = [
        sys.executable,
        "-m",
        "horismos",
        "verify",
        str(TRIAGE),
        "--import",
        str(SNOMED),
    ]
    first = subprocess.run(command, check=False, capture_output=True, text=True)
    second = subprocess.run(command, check=False, capture_output=True, text=True)

    assert first.returncode == 0
    assert first.stderr == ""
    assert second.returncode == first.returncode
    assert second.stderr == first.stderr
    assert second.stdout == first.stdout
    payload = json.loads(first.stdout)
    assert payload["ok"] is True
    assert payload["complete"] == {":acuity": True}
    assert [(w["kind"], w["severity"]) for w in payload["witnesses"]] == [
        ("constraint_vacuity", "info")
    ]


def test_informational_notes_do_not_fail_the_report(onto: Ontology) -> None:
    report = onto.verify()
    assert report.ok
    assert report.exit_code == 0
    assert [witness.severity for witness in report.witnesses] == ["info"]
    assert report.witnesses[0].kind == "constraint_vacuity"
    assert report.witnesses[0].domain_text.startswith("hasSymptom horis:cardinality")
    assert set(report.workflows[":Triage"].__dict__) == {
        "terminating",
        "requires_satisfiable",
        "fallbacks_total",
        "choices_total",
    }


def test_conflicts_carry_the_priority_winner(tmp_path: Path) -> None:
    conflicting = tmp_path / "conflict.ttl"
    conflicting.write_text(
        """\
@prefix : <https://example.org/conflict#> .
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
:Patient a rdfs:Class .
:flag a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
    horis:cardinality "1" .
:decision a rdf:Property ; rdfs:domain :Patient ; rdfs:range xsd:boolean ;
    horis:assignedBy horis:RulesOnly ; horis:cardinality "0..1" .
:high a horis:Rule ; horis:priority 10 ;
    horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
    horis:then [ horis:property :decision ; horis:value true ] .
:low a horis:Rule ; horis:priority 1 ;
    horis:when [ horis:subject :Patient ; horis:property :flag ; horis:isAssigned true ] ;
    horis:then [ horis:property :decision ; horis:value false ] .
"""
    )
    with pytest.raises(HorismosVerificationError) as raised:
        Ontology.load(conflicting)
    report = raised.value.report
    assert report.ok is False
    (conflict,) = report.conflicts
    assert (conflict.first_rule, conflict.second_rule) == (":high", ":low")
    assert (conflict.first_priority, conflict.second_priority) == (10, 1)
    assert conflict.winner == ":high"
    (witness,) = [w for w in report.witnesses if w.kind == "conflict"]
    assert witness.severity == "error"
    assert witness.domain_text.endswith("— :high wins by priority; confirm intended")


def test_a_missing_file_raises_an_ontology_error() -> None:
    with pytest.raises(HorismosOntologyError):
        Ontology.load("does-not-exist.ttl")


def test_a_malformed_ontology_raises_an_ontology_error(tmp_path: Path) -> None:
    bad = tmp_path / "bad.ttl"
    bad.write_text("@prefix : <https://example.org/x#> .\n:this is not turtle")
    with pytest.raises(HorismosOntologyError) as e:
        Ontology.load(bad)
    assert isinstance(e.value, horismos.HorismosError)
