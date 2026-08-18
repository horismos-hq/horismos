"""U6 — typed instance construction per D1 (the shipped generic form)."""

import horismos
import pytest
from horismos import Ontology, World


def test_snake_case_maps_to_the_declared_camel_case_property(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", heart_rate=128, has_symptom=[":ChestPain"])
    # The snake spelling and the authored CURIE read the same fact.
    assert world.value(p, "heart_rate") == 128
    assert world.value(p, ":heartRate") == 128
    assert world.values(p, ":hasSymptom") == [onto.ref(":ChestPain")]


def test_two_identically_built_worlds_are_byte_identical(onto: Ontology) -> None:
    def build() -> bytes:
        world = World(onto)
        world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
        return world.snapshot().to_bytes()

    assert build() == build()


def test_an_unknown_class_fails_at_the_gate_in_domain_terms(world: World) -> None:
    with pytest.raises(horismos.HorismosValidationError) as e:
        world.assert_(":Ghost")
    assert str(e.value) == ":Ghost is not a declared class in the ontology"
    assert e.value.constraint == "class_not_declared"


def test_an_unknown_prefix_in_the_class_term_is_a_naming_error(world: World) -> None:
    with pytest.raises(horismos.HorismosOntologyError):
        world.assert_("nope:Patient")


def test_list_values_fan_out_over_a_multi_valued_property(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", has_symptom=[":ChestPain", ":CrushingChestPain"])
    assert world.values(p, ":hasSymptom") == [
        onto.ref(":ChestPain"),
        onto.ref(":CrushingChestPain"),
    ]
