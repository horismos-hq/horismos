"""U3 — whole-transaction World mutations across the boundary."""

import horismos
import pytest
from horismos import HorismosValidationError, Ontology, World


def test_example_1_the_assert_derives_acuity_inside_the_transaction(
    onto: Ontology, world: World
) -> None:
    p = world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    assert world.value(p, ":acuity") == onto.ref(":Immediate")


def test_example_2_the_rejection_carries_the_domain_sentence(onto: Ontology, world: World) -> None:
    with pytest.raises(HorismosValidationError) as e:
        world.assert_(":Patient", heart_rate=1280)
    exc = e.value
    assert str(exc) == "heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"
    assert exc.domain_text == str(exc)
    assert exc.constraint == "max"
    assert exc.data["bound"] == "300"
    assert exc.data["unit"] == "bpm"
    assert exc.suggestion is None
    assert isinstance(exc, horismos.HorismosError)


def test_example_2_rules_only_provenance_is_refused(world: World) -> None:
    with pytest.raises(HorismosValidationError) as e:
        world.assert_(":Patient", acuity=":Immediate")
    exc = e.value
    assert exc.constraint == "provenance_refused"
    assert "horis:RulesOnly" in str(exc)
    assert exc.provenance == "api"
    assert exc.operation == "assert"


def test_a_rejected_assert_leaves_the_world_unchanged(world: World) -> None:
    p = world.assert_(":Patient", heart_rate=100)
    before = world.snapshot()
    with pytest.raises(HorismosValidationError):
        world.assert_(":Patient", heart_rate=1280)
    diff = world.diff(before)
    assert diff.added == [] and diff.removed == []
    assert world.values(p, ":heartRate") == [100]


def test_named_subjects_round_trip(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", subject=":p7", heart_rate=128)
    assert p.iri == "https://example.org/triage#p7"
    assert world.value(p, ":heartRate") == 128


def test_retracting_a_derived_fact_raises_the_prp_002_ruling(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    with pytest.raises(HorismosValidationError) as e:
        world.retract(p, ":acuity", onto.ref(":Immediate"))
    assert e.value.constraint == "derived_fact_retraction"
    assert ":r1" in str(e.value)


def test_retract_removes_a_base_fact(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", heart_rate=100, has_symptom=[":CrushingChestPain"])
    world.retract(p, ":hasSymptom", onto.ref(":CrushingChestPain"))
    assert world.values(p, ":hasSymptom") == []


def test_value_marshaling_round_trips(onto: Ontology, world: World) -> None:
    from decimal import Decimal

    p = world.assert_(":Patient", heart_rate=Decimal("128.5"))
    assert world.value(p, ":heartRate") == Decimal("128.5")

    q = world.assert_(":Patient", has_symptom=[onto.ref(":ChestPain")])
    assert world.values(q, ":hasSymptom") == [onto.ref(":ChestPain")]


def test_an_unknown_kwarg_property_is_a_domain_error_with_a_suggestion(
    world: World,
) -> None:
    with pytest.raises(horismos.HorismosOntologyError) as e:
        world.assert_(":Patient", heart_rat=128)
    assert "heart_rat" in str(e.value)
    assert e.value.suggestion is not None
    assert "heart_rate" in e.value.suggestion
