"""The text -> decisions demo: ``python -m horismos demo``.

A clinical note goes in; TypeSafe's Jev proposes ``vte.ttl``'s four booleans;
the workflow's ``horis:confidence 0.85`` gate accepts or refuses the proposal;
and ``vte.ttl``'s rules - never the model - decide ``:recommendation``, with a
proof saying which rule fired on which perceived facts.

    python -m horismos demo                        # MockBridge: no network, no key
    HORISMOS_DEMO_LIVE=1 OPENROUTER_API_KEY=... python -m horismos demo   # Jev, live
    HORISMOS_DEMO_LIVE=1 TYPESAFE_API_KEY=...   python -m horismos demo   # Jev, live

The mock is the default whatever else is set: a key in the environment is not
consent to spend it, so the live model runs only when ``HORISMOS_DEMO_LIVE=1``
asks for it. The mock's probabilities are authored stand-ins, not measurements
of any model, and the output says so. The notes, the question wording and those
stand-ins are the ones ``crates/horismos-bridges/examples/jev_vte_demo.rs``
uses, so the two demos agree. The difference is the loop: this one runs the
declared workflow (the packaged ``vte/perception.ttl``) through
``horismos.Agent``, so the step's confidence gate and its fallback edge are in
play: the second note's weakest scripted answer, 0.61, sends it to a clinician
instead of into the record.
"""

from __future__ import annotations

import io
import os
import sys
import textwrap
from datetime import UTC, datetime
from typing import Any

from horismos import Agent, Ontology, World
from horismos.bridges import Bridge, JevBridge, MockBridge, PropertyHint, compiled
from horismos.examples import path

LIVE = "HORISMOS_DEMO_LIVE"
KEYS = ("OPENROUTER_API_KEY", "TYPESAFE_API_KEY")

# The probe's default note: reduced mobility is plain, the malignancy is
# historical, and nothing bleeds.
NOTE_CLEAR = (
    "72-year-old admitted yesterday following a fall at home with a fractured right neck "
    "of femur, awaiting theatre. Mobilising only with assistance and largely confined to "
    "bed since admission. Past history of breast carcinoma treated eight years ago, in "
    "remission, no current therapy. No previous clots documented. Haemoglobin stable, "
    "platelets 210, not anticoagulated, no bleeding noted on the ward round."
)
# The probe's borderline note. "On warfarin" is the ambiguity.
NOTE_UNCERTAIN = "78yo, post-op day 2, on warfarin, bed-bound"

# The ontology has no vocabulary for question wording (doc 02), so a
# deployment supplies it alongside the bridge.
HINTS = {
    "reducedMobility": PropertyHint(
        "Is the patient's mobility reduced relative to their usual baseline?",
        "Bed-bound, immobile, confined to a chair, or mobilising only with assistance.",
        "Independently mobile at their usual baseline.",
    ),
    "activeCancer": PropertyHint(
        "Does the patient have active malignancy?",
        "Malignancy currently present or under active treatment.",
        "No malignancy, or a historical malignancy in remission with no current treatment.",
    ),
    "priorVte": PropertyHint(
        "Has the patient previously had a venous thromboembolism?",
        "A documented previous DVT or pulmonary embolism.",
        "No previous venous thromboembolism documented.",
    ),
    "bleedingRisk": PropertyHint(
        "Is there a contraindication to pharmacologic prophylaxis on bleeding grounds?",
        "Active bleeding, a recent major bleed, severe thrombocytopenia, or full "
        "anticoagulation already.",
        "No bleeding contraindication identified.",
    ),
}


def scripted(facts: dict[str, tuple[bool, float]]) -> dict[str, Any]:
    """One Jev-shaped answer: the call's confidence is its weakest fact. The
    probabilities passed in are authored stand-ins, not model output."""
    weakest = min(p for _, p in facts.values())
    instance: dict[str, Any] = {"class": ":VteCase"}
    instance.update({name: value for name, (value, _) in facts.items()})
    return {"instances": [instance], "confidence": weakest, "fact_confidence": {"0": weakest}}


MOCK_SCRIPT = {
    NOTE_CLEAR: scripted(
        {
            "reducedMobility": (True, 0.96),
            "activeCancer": (False, 0.88),
            "priorVte": (False, 0.96),
            "bleedingRisk": (False, 0.93),
        }
    ),
    NOTE_UNCERTAIN: scripted(
        {
            "reducedMobility": (True, 0.94),
            "activeCancer": (False, 0.91),
            "priorVte": (False, 0.72),
            "bleedingRisk": (True, 0.61),
        }
    ),
}


def key_configured() -> bool:
    return any(os.environ.get(var) or os.environ.get(f"{var}_FILE") for var in KEYS)


def build_bridge() -> Bridge | str:
    """The bridge to run, or why the live one asked for cannot run."""
    if os.environ.get(LIVE) != "1":
        return MockBridge(MOCK_SCRIPT, id="mock:jev-1.13")
    if "jev" not in compiled():
        return "this build has no JevBridge (the bridges-jev feature)"
    if not key_configured():
        return f"{LIVE}=1 needs {' or '.join(KEYS)} (or its _FILE variant) set"
    return JevBridge.from_env(hints=HINTS)


def rule(title: str) -> None:
    print(f"\n{'─' * 78}\n{title}\n{'─' * 78}")


def main() -> int:
    # The proof trees print box-drawing characters; a Windows pipe defaults
    # to the locale's code page, which has none.
    if isinstance(sys.stdout, io.TextIOWrapper):
        sys.stdout.reconfigure(encoding="utf-8")
    bridge = build_bridge()
    if isinstance(bridge, str):
        print(f"horismos demo: cannot run the live model: {bridge}", file=sys.stderr)
        return 2
    mock = isinstance(bridge, MockBridge)
    onto = Ontology.load(path("vte/perception.ttl"))
    mode = "mock bridge (offline)" if mock else "live Jev"
    print(f"Horismos VTE text→decisions demo — {mode}")
    print(f"workflow: :NoteToRecommendation (vte/perception.ttl)  ·  model: {bridge.id}")
    if mock:
        print("The mock replays scripted answers. Every probability below is an authored")
        print(f"stand-in, not a measurement of any model. {LIVE}=1 runs the live model.")
        if key_configured():
            print(f"An API key is set, but only {LIVE}=1 spends it: this run sent no request.")

    for label, note in (("1", NOTE_CLEAR), ("2", NOTE_UNCERTAIN)):
        rule(f"Note {label}")
        print(textwrap.fill(note, width=78, initial_indent="  ", subsequent_indent="  "))

        world = World(onto)
        agent = Agent(world, workflow=":NoteToRecommendation", bridges={"jev": bridge})
        # A fixed run id and instant make the mock run's trace reproducible.
        result = agent.run(
            note, run_id=f"jev-demo-note-{label}", now=datetime(2026, 9, 23, tzinfo=UTC)
        )
        steps = result.trace["steps"]
        perception = steps[0] if steps else None

        if perception is None:
            # A bridge error or refusal: no answer, so nothing to record.
            print("\n  The bridge returned no usable answer; the step took its fallback edge.")
        else:
            verdict = "accepted" if perception["accepted"] else "REFUSED"
            authored = " (authored)" if mock else ""
            print(f"\n  served by {perception['model']} (the trace's model field)")
            print(
                f"  confidence {perception['confidence']:.3f}{authored} against the step's "
                f"horis:confidence {perception['threshold']:.2f} → {verdict}"
            )
            for fact in perception["asserted"]:
                print(
                    f"    {fact['p'].lstrip(':'):<18} {fact['o']!s:<5}  "
                    f"asserted by {fact['provenance']}"
                )

        if result.fallback_taken:
            print(f"\n  → {result.terminal}: nothing was asserted, and no rule had anything to")
            print("    decide. A person reads this note; the record is untouched.")
        elif result.proof is not None:
            print("\n  The rules decided. The model had no vote:")
            print(textwrap.indent(result.proof.render().rstrip(), "    "))

    if isinstance(bridge, MockBridge):  # the mock records what the model was shown
        (first, *_) = bridge.invocations()
        (case,) = first["schema"]["properties"]["instances"]["items"]["oneOf"]
        asked = sorted(name for name in case["properties"] if name != "class")
        rule("What the model was shown")
        print(f"  schema properties: {', '.join(asked)}")
        print("  :recommendation is horis:RulesOnly, so the schema has no field for it —")
        print("  not discouraged, absent. The model proposes facts; the rules decide.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
