"""U7 — the tier-2 pattern query per D2 (03-decision-query.md)."""

import horismos
import pytest
from horismos import Ontology, World


def test_patients_whose_acuity_is_immediate(onto: Ontology, world: World) -> None:
    p1 = world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    world.assert_(":Patient", heart_rate=70)

    assert world.query(":Patient", acuity=":Immediate") == [p1]
    assert world.query(":Patient", acuity=onto.ref(":Immediate")) == [p1]


def test_an_empty_conjunction_is_the_class_scan(onto: Ontology, world: World) -> None:
    p1 = world.assert_(":Patient", heart_rate=128)
    p2 = world.assert_(":Patient", heart_rate=70)
    assert world.query(":Patient") == [p1, p2]


def test_conjunction_requires_every_pattern(onto: Ontology, world: World) -> None:
    world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    assert world.query(":Patient", acuity=":Immediate", heart_rate=70) == []


def test_an_undeclared_class_is_an_error_not_an_empty_result(world: World) -> None:
    with pytest.raises(horismos.HorismosOntologyError) as e:
        world.query(":Ghost")
    assert str(e.value) == ":Ghost names no term declared by the ontology"


def test_an_unknown_property_keyword_is_an_error_with_a_suggestion(
    world: World,
) -> None:
    with pytest.raises(horismos.HorismosOntologyError) as e:
        world.query(":Patient", acutiy=":Immediate")
    assert e.value.suggestion is not None
    assert "acuity" in e.value.suggestion
