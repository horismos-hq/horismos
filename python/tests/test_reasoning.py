"""U4 — the read surface and proof trees across the boundary."""

from horismos import Ontology, World

R1_PROOF = (
    ':acuity = :Immediate   via r1 — "Tachycardia with chest pain — immediate track"\n'
    "  ├─ heartRate 128 exceeds 120   (asserted by the api)\n"
    "  └─ hasSymptom :CrushingChestPain ⊑ :ChestPain   (subsumption)\n"
)


def test_explain_renders_the_r1_proof_byte_identically(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    proof = world.explain(p, ":acuity")
    assert proof is not None
    assert proof.render() == R1_PROOF
    assert proof.rule == ":r1"
    # The tree is real (P6): children marshaled recursively, each rendering
    # its own subtree.
    assert len(proof.children) == 2
    assert "heartRate 128 exceeds 120" in proof.children[0].render()
    assert proof.children[0].children == []


def test_explain_is_none_for_a_base_fact(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", heart_rate=128)
    assert world.explain(p, ":heartRate") is None


def test_prove_finds_a_derivation_and_refuses_an_unprovable_goal(
    onto: Ontology, world: World
) -> None:
    p = world.assert_(":Patient", heart_rate=128, has_symptom=[":CrushingChestPain"])
    proof = world.prove(p, ":acuity", onto.ref(":Immediate"))
    assert proof is not None
    assert ":Immediate" in proof.render()

    assert world.prove(p, ":acuity", onto.ref(":Standard")) is None


def test_instances_are_subsumption_aware(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", heart_rate=128)
    assert world.instances(":Patient") == [p]
    assert world.instances(":AcuityLevel") == []


def test_values_of_an_unassigned_property_are_empty(onto: Ontology, world: World) -> None:
    p = world.assert_(":Patient", heart_rate=128)
    assert world.values(p, ":hasSymptom") == []
    assert world.value(p, ":hasSymptom") is None
