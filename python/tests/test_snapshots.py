"""U5 — snapshots, diffs, and the wire form across the boundary."""

import horismos
import pytest
from horismos import Ontology, World

from .conftest import SNOMED


def test_diff_shows_exactly_the_added_facts(onto: Ontology, world: World) -> None:
    before = world.snapshot()
    p = world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])

    diff = world.diff(before)
    assert diff.removed == []
    assert len(diff.added) == 4, "class + heartRate + hasSymptom + derived acuity"
    added = {(f.kind, f.subject.iri, f.property, f.value) for f in diff.added}
    assert (
        "class",
        p.iri,
        None,
        "https://example.org/triage#Patient",
    ) in added
    assert (
        "property",
        p.iri,
        "https://example.org/triage#heartRate",
        128,
    ) in added
    # r1's derived conclusion is part of the same transaction's diff.
    assert (
        "property",
        p.iri,
        "https://example.org/triage#acuity",
        onto.ref(":Immediate"),
    ) in added


def test_from_snapshot_restores_an_equal_world(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    snap = world.snapshot()
    restored = World.from_snapshot(onto, snap)
    assert restored.value(p, ":acuity") == onto.ref(":Immediate")
    assert restored.diff(snap).added == []


def test_the_wire_form_round_trips(onto: Ontology, world: World) -> None:
    world.assert_(":Patient", heart_rate=128)
    snap = world.snapshot()
    wire = snap.to_bytes()
    assert isinstance(wire, bytes)
    again = horismos.Snapshot.from_bytes(onto, wire)
    restored = World.from_snapshot(onto, again)
    assert restored.diff(snap).added == []


def test_a_corrupted_wire_form_raises_the_format_error(onto: Ontology) -> None:
    with pytest.raises(horismos.HorismosValidationError) as e:
        horismos.Snapshot.from_bytes(onto, b"not a snapshot")
    assert e.value.constraint == "snapshot_format"


def test_restoring_under_a_different_ontology_is_refused(onto: Ontology, world: World) -> None:
    world.assert_(":Patient", heart_rate=128)
    other = Ontology.load(SNOMED)  # a different root graph entirely
    with pytest.raises(horismos.HorismosValidationError) as e:
        World.from_snapshot(other, world.snapshot())
    assert "different ontology" in str(e.value)
