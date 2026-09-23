"""The agent loop from Python: natural language in, typed instances asserted
into the World through a bridge — with the schema, the confidence gate, the
fallback edge and perception provenance all core's, never this package's.

Every test runs on ``MockBridge`` (no network, no key) except the Jev ones,
which stay offline by construction: Jev refuses a numeric schema before it
resolves a key or sends a request."""

import json
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

import horismos
import pytest
from horismos import Agent, Ontology, World, _horismos
from horismos.bridges import JevBridge, MockBridge, OpenRouterBridge, PropertyHint, compiled

from .conftest import REPO, SNOMED, TRIAGE

PERCEPTION = REPO / "examples" / "vte" / "perception.ttl"
WORKFLOW = ":NoteToRecommendation"
NOW = datetime(2026, 9, 23, 9, 0, tzinfo=UTC)
NOTE = "78yo, post-op day 2, bed-bound, no history of clots or cancer, no bleeding"
AT_RISK = {
    "class": ":VteCase",
    "reducedMobility": True,
    "activeCancer": False,
    "priorVte": False,
    "bleedingRisk": False,
}


@pytest.fixture(scope="module")
def vte() -> Ontology:
    return Ontology.load(PERCEPTION)


def perceiving(instance: dict[str, Any], confidence: float = 0.93) -> MockBridge:
    return MockBridge(
        {NOTE: {"instances": [instance], "confidence": confidence}}, id="mock:jev-1.13"
    )


def run(vte: Ontology, bridge: MockBridge, text: str = NOTE) -> tuple[World, horismos.RunResult]:
    world = World(vte)
    result = Agent(world, workflow=WORKFLOW, bridges={"jev": bridge}).run(
        text, run_id="test-run", now=NOW
    )
    return world, result


# -- the two load-bearing properties -------------------------------------------


def test_a_rules_only_property_is_absent_from_the_schema_the_model_is_shown(
    vte: Ontology,
) -> None:
    # Non-vacuous: vte.ttl declares :recommendation, and the rules decide it.
    assert vte.ref(":recommendation")
    bridge = perceiving(AT_RISK)
    _, result = run(vte, bridge)
    assert result.decision == vte.ref(":PharmacologicProphylaxis")

    (call,) = bridge.invocations()
    assert call["kind"] == "perceive"
    assert call["text"] == NOTE
    (case_schema,) = call["schema"]["properties"]["instances"]["items"]["oneOf"]
    assert set(case_schema["properties"]) == {
        "class",
        "reducedMobility",
        "activeCancer",
        "priorVte",
        "bleedingRisk",
    }
    # Not marked read-only, not discouraged — absent, anywhere in the schema.
    assert "recommendation" not in json.dumps(call["schema"])


def test_a_rules_only_value_proposed_anyway_never_enters_the_world(vte: Ontology) -> None:
    # A model that answers with the decision field regardless: the gate refuses
    # perception provenance on a horis:RulesOnly property, the whole instance
    # is rejected, and the step takes its fallback edge.
    bridge = perceiving({**AT_RISK, "recommendation": ":HoldProphylaxis"})
    world, result = run(vte, bridge)

    assert result.terminal == ":ClinicianReview"
    assert result.fallback_taken
    assert result.decision is None
    assert result.case is None
    assert world.instances(":VteCase") == []
    assert "HoldProphylaxis" not in json.dumps(result.trace)


def test_perceived_facts_carry_perception_provenance_in_their_proof(vte: Ontology) -> None:
    world, result = run(vte, perceiving(AT_RISK))
    assert result.case is not None
    proof = result.proof
    assert proof is not None
    assert proof.rule == ":risk-without-contraindication"

    leaves = [n for n in _walk(proof) if n.provenance is not None]
    assert leaves, proof.render()
    assert {leaf.provenance for leaf in leaves} == {"perception"}
    assert "(asserted by perception)" in proof.render()
    assert "the api" not in proof.render()

    # The same proof through the public read surface, after the run.
    explained = world.explain(result.case, "recommendation")
    assert explained is not None
    assert explained.render() == proof.render()

    # And in the trace: every asserted fact says who asserted it.
    (perception, *_) = result.trace["steps"]
    assert perception["kind"] == "perception"
    assert {fact["provenance"] for fact in perception["asserted"]} == {"perception"}


def test_the_same_facts_asserted_through_the_api_say_so(vte: Ontology) -> None:
    # The contrast the product rests on: identical facts, different provenance.
    world = World(vte)
    case = world.assert_(
        ":VteCase",
        reduced_mobility=True,
        active_cancer=False,
        prior_vte=False,
        bleeding_risk=False,
    )
    proof = world.explain(case, "recommendation")
    assert proof is not None
    assert {n.provenance for n in _walk(proof) if n.provenance is not None} == {"api"}
    assert "(asserted by the api)" in proof.render()
    assert "perception" not in proof.render()


# -- the confidence gate and its mandatory fallback -----------------------------


def test_confidence_below_the_step_threshold_takes_the_fallback_and_asserts_nothing(
    vte: Ontology,
) -> None:
    world, result = run(vte, perceiving(AT_RISK, confidence=0.84))

    assert result.terminal == ":ClinicianReview"
    assert result.fallback_taken
    assert result.case is None
    assert world.instances(":VteCase") == []
    (step,) = result.trace["steps"]
    assert (step["confidence"], step["threshold"], step["accepted"]) == (0.84, 0.85, False)
    assert step["asserted"] == []


def test_confidence_at_the_threshold_is_accepted(vte: Ontology) -> None:
    _, result = run(vte, perceiving(AT_RISK, confidence=0.85))
    assert result.terminal == ":Done"
    assert not result.fallback_taken


def test_unscripted_input_fails_safe_into_the_fallback(vte: Ontology) -> None:
    # The mock's documented default is low confidence: never a false accept.
    _, result = run(vte, perceiving(AT_RISK), text="a note nobody scripted")
    assert result.terminal == ":ClinicianReview"
    assert result.fallback_taken


def test_a_gate_rejected_proposal_takes_the_fallback(vte: Ontology) -> None:
    # Schema-shaped but not type-correct: the gate is the backstop.
    world, result = run(vte, perceiving({**AT_RISK, "bleedingRisk": "maybe"}))
    assert result.terminal == ":ClinicianReview"
    assert world.instances(":VteCase") == []


# -- the run result and the trace ------------------------------------------------


def test_an_accepted_run_reaches_the_decision_through_the_rules(vte: Ontology) -> None:
    world, result = run(vte, perceiving({**AT_RISK, "bleedingRisk": True}))

    assert result.terminal == ":Done"
    assert not result.fallback_taken
    assert result.goal_reached is True
    assert result.decides == ":recommendation"
    assert result.decision == vte.ref(":HoldProphylaxis")
    assert result.case is not None
    assert world.value(result.case, "recommendation") == vte.ref(":HoldProphylaxis")
    assert result.trace["outcome"]["terminal"] == ":Done"
    assert result.trace["steps"][0]["model"] == "mock:jev-1.13"
    assert [s["kind"] for s in result.trace["steps"]] == ["perception", "reasoning"]


def test_a_fixed_run_id_and_instant_reproduce_the_trace(vte: Ontology) -> None:
    _, first = run(vte, perceiving(AT_RISK))
    _, second = run(vte, perceiving(AT_RISK))
    assert first.trace == second.trace
    assert first.trace["run_id"] == "test-run"
    assert first.trace["started_at"] == "2026-09-23T09:00:00+00:00"


def test_a_default_run_mints_its_own_id_and_instant(vte: Ontology) -> None:
    world = World(vte)
    agent = Agent(world, workflow=WORKFLOW, bridges={"jev": perceiving(AT_RISK)})
    result = agent.run(NOTE)
    assert len(result.trace["run_id"]) == 32
    assert result.trace["started_at"].endswith("+00:00")


def test_a_naive_instant_is_refused(vte: Ontology) -> None:
    agent = Agent(World(vte), workflow=WORKFLOW, bridges={"jev": perceiving(AT_RISK)})
    with pytest.raises(horismos.HorismosWorkflowError, match="timezone-aware"):
        agent.run(NOTE, now=datetime(2026, 9, 23))


def test_subscribers_see_the_facts_a_run_committed(vte: Ontology) -> None:
    world = World(vte)
    seen: list[Any] = []
    world.subscribe("recommendation", lambda _subject, value: seen.append(value))
    Agent(world, workflow=WORKFLOW, bridges={"jev": perceiving(AT_RISK)}).run(
        NOTE, run_id="r", now=NOW
    )
    assert seen == [vte.ref(":PharmacologicProphylaxis")]


# Perception commits, then the run reaches a step kind the loop defers (doc 05).
CHOICE_AFTER_PERCEPTION = """
@prefix :      <https://example.org/vte#> .
@prefix horis: <https://w3id.org/horismos#> .

:ReadThenChoose a horis:Workflow ; horis:startsAt :Read .
:Read a horis:PerceptionStep ;
    horis:extracts :VteCase ; horis:model "jev:jev-1.13" ;
    horis:confidence 0.85 ; horis:fallback :Review ; horis:next :Choose .
:Choose a horis:ChoiceStep ; horis:branches (
    [ horis:when [ horis:property :bleedingRisk ; horis:is true ] ; horis:next :Review ]
    [ horis:when [ horis:property :bleedingRisk ; horis:is false ] ; horis:next :Review ]
) .
:Review a horis:Step ; horis:terminal true .
"""


def test_a_run_that_fails_after_committing_still_notifies_subscribers(tmp_path: Path) -> None:
    path = tmp_path / "choice.ttl"
    path.write_text(CHOICE_AFTER_PERCEPTION)
    onto = Ontology.load(path, imports=[REPO / "examples" / "vte" / "vte.ttl"])
    world = World(onto)
    seen: list[Any] = []
    world.subscribe("reduced_mobility", lambda _subject, value: seen.append(value))
    agent = Agent(world, workflow=":ReadThenChoose", bridges={"jev": perceiving(AT_RISK)})

    with pytest.raises(horismos.HorismosWorkflowError, match="ChoiceStep"):
        agent.run(NOTE, run_id="r", now=NOW)
    # The perceived case stands in the World, so its subscribers heard of it.
    (case,) = world.instances(":VteCase")
    assert world.value(case, "reduced_mobility") is True
    assert seen == [True]


# -- construction fails early, in domain terms -----------------------------------


def test_a_prefix_no_bridge_serves_fails_at_construction(vte: Ontology) -> None:
    bridges = {"mock": perceiving(AT_RISK), "claude": perceiving(AT_RISK)}
    with pytest.raises(horismos.HorismosWorkflowError, match='"jev" model prefix'):
        Agent(World(vte), workflow=WORKFLOW, bridges=bridges)


def test_an_undeclared_workflow_fails_at_construction(vte: Ontology) -> None:
    with pytest.raises(horismos.HorismosWorkflowError, match=":NoSuchWorkflow"):
        Agent(World(vte), workflow=":NoSuchWorkflow", bridges={"jev": perceiving(AT_RISK)})


def test_a_workflow_needing_a_rust_tool_is_refused_by_name() -> None:
    triage = Ontology.load(TRIAGE, imports=[SNOMED])
    with pytest.raises(
        horismos.HorismosWorkflowError,
        match=r'no Tool implementation registered for "(assign_queue|notify_nurse)"',
    ):
        Agent(World(triage), workflow=":Triage", bridges={"claude": MockBridge()})


@pytest.mark.parametrize("impostor", [object(), print, "jev"])
def test_only_a_bridge_is_accepted_as_one(vte: Ontology, impostor: object) -> None:
    with pytest.raises(horismos.HorismosBridgeError, match="is not a Horismos bridge"):
        Agent(World(vte), workflow=WORKFLOW, bridges={"jev": impostor})  # type: ignore[dict-item]


def test_a_mock_script_must_be_a_whole_perceive_result() -> None:
    with pytest.raises(horismos.HorismosBridgeError, match="not a PerceiveResult"):
        MockBridge({NOTE: {"instances": "not a list"}})


# -- the compiled provider bridges ------------------------------------------------

needs_jev = pytest.mark.skipif("jev" not in compiled(), reason="built without bridges-jev")

VITALS = """\
@prefix :      <https://example.org/vitals#> .
@prefix horis: <https://w3id.org/horismos#> .
@prefix rdf:   <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs:  <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd:   <http://www.w3.org/2001/XMLSchema#> .

:Vitals a rdfs:Class .
:heartRate a rdf:Property ; rdfs:domain :Vitals ; rdfs:range xsd:decimal ;
    horis:cardinality "1" .

:ReadVitals a horis:Workflow ; horis:startsAt :Read .
:Read a horis:PerceptionStep ;
    horis:extracts :Vitals ; horis:model "jev:jev-1.13" ; horis:confidence 0.85 ;
    horis:fallback :Review ; horis:next :Done .
:Done a horis:Step ; horis:terminal true .
:Review a horis:Step ; horis:terminal true .
"""


@needs_jev
def test_a_jev_refusal_takes_the_fallback_edge_and_never_leaks_the_key(tmp_path: Path) -> None:
    # Jev cannot produce a number, so it refuses :heartRate by name before it
    # resolves a key or sends anything; the runtime treats the refusal exactly
    # like low confidence. The key it was given appears nowhere.
    path = tmp_path / "vitals.ttl"
    path.write_text(VITALS)
    world = World(Ontology.load(path))
    secret = "sk-or-v1-not-a-real-key-0123456789"
    bridge = JevBridge(api_key=secret, hints={"heartRate": PropertyHint(ordinal=True)})
    result = Agent(world, workflow=":ReadVitals", bridges={"jev": bridge}).run(
        "HR 128", run_id="r", now=NOW
    )

    assert result.terminal == ":Review"
    assert result.fallback_taken
    assert world.instances(":Vitals") == []
    assert secret not in json.dumps(result.trace)
    assert secret not in repr(bridge)


@needs_jev
def test_jev_bridges_construct_and_report_their_model() -> None:
    assert JevBridge().id == "jev:typesafe/jev-1.13"
    assert JevBridge("typesafe", model="jev-1.13").id == "jev:jev-1.13"
    assert JevBridge.from_env().id.startswith("jev:")


@needs_jev
def test_an_unknown_jev_transport_is_refused() -> None:
    with pytest.raises(horismos.HorismosBridgeError, match="unknown Jev transport"):
        JevBridge("carrier-pigeon")  # type: ignore[arg-type]


@pytest.mark.skipif("openrouter" not in compiled(), reason="built without bridges-openrouter")
def test_openrouter_bridges_construct_and_report_their_model() -> None:
    assert OpenRouterBridge("openai/gpt-4o-mini").id == "openrouter:openai/gpt-4o-mini"
    assert OpenRouterBridge.from_env().id.startswith("openrouter:")


def test_a_bridge_missing_from_the_build_is_refused_by_feature(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delattr(_horismos, "JevBridge", raising=False)
    with pytest.raises(horismos.HorismosBridgeError, match="bridges-jev") as caught:
        JevBridge()
    assert caught.value.data == {"bridge": "JevBridge", "feature": "bridges-jev"}


def test_the_published_wheel_compiles_every_implemented_bridge() -> None:
    # pyproject.toml's [tool.maturin] features is what `make develop` and the
    # release both build with, so this build is the wheel's configuration.
    assert compiled() == {"mock", "jev", "openrouter"}


def _walk(node: horismos.ProofNode) -> list[horismos.ProofNode]:
    return [node, *(n for child in node.children for n in _walk(child))]


def test_the_packaged_demo_runs_offline_even_with_a_key_set(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    # `python -m horismos demo` without the explicit opt-in: the mock path end to
    # end, whatever keys the environment happens to hold. A key is not consent.
    monkeypatch.delenv("HORISMOS_DEMO_LIVE", raising=False)
    monkeypatch.setenv("OPENROUTER_API_KEY", "sk-or-v1-not-a-real-key-0123456789")
    from horismos.examples import jev_vte

    assert jev_vte.main() == 0
    out = capsys.readouterr().out
    assert "mock bridge (offline)" in out
    assert "An API key is set, but only HORISMOS_DEMO_LIVE=1 spends it" in out
    assert "served by mock:jev-1.13" in out
    assert "authored" in out  # the mock's numbers are labelled as stand-ins
    assert "confidence 0.880 (authored) against the step's horis:confidence 0.85 → accepted" in out
    assert ":recommendation = :PharmacologicProphylaxis" in out
    assert "(asserted by perception)" in out
    assert "confidence 0.610 (authored) against the step's horis:confidence 0.85 → REFUSED" in out
    assert "→ :ClinicianReview: nothing was asserted" in out
    assert "schema properties: activeCancer, bleedingRisk, priorVte, reducedMobility" in out
    assert "sk-or-v1" not in out


def test_the_live_demo_without_a_key_refuses_before_any_request(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv("HORISMOS_DEMO_LIVE", "1")
    for var in ("OPENROUTER_API_KEY", "TYPESAFE_API_KEY"):
        monkeypatch.delenv(var, raising=False)
        monkeypatch.delenv(f"{var}_FILE", raising=False)
    from horismos.examples import jev_vte

    assert jev_vte.main() == 2
    captured = capsys.readouterr()
    assert captured.out == ""
    assert "HORISMOS_DEMO_LIVE=1 needs OPENROUTER_API_KEY or TYPESAFE_API_KEY" in captured.err


def test_the_doc_11_agent_quickstart_runs_verbatim(tmp_path: Path) -> None:
    import contextlib
    import io
    import os
    import re

    doc = (REPO / "docs" / "framework" / "11-python-api.md").read_text()
    section = re.split(r"^## Quickstart \(the agent loop\)$", doc, flags=re.M)[1]
    section = re.split(r"^## ", section, maxsplit=1, flags=re.M)[0]
    (block,) = re.findall(r"```python\n(.*?)```", section, flags=re.S)
    out = io.StringIO()
    cwd = os.getcwd()
    os.chdir(tmp_path)  # nothing from the repository: the package's own examples
    try:
        with contextlib.redirect_stdout(out):
            exec(compile(block, "<doc 11 agent quickstart>", "exec"), {})
    finally:
        os.chdir(cwd)
    printed = out.getvalue()
    assert printed.startswith("Ref(:HoldProphylaxis)\n")
    assert "via bleeding-contraindication" in printed
    assert "(asserted by perception)" in printed
