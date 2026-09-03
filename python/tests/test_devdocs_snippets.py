"""The developer docs site's runnable snippets execute verbatim.

Every fenced block written exactly as ```python on the pages below is
extracted and executed in order, in one namespace per page, from the repo
root — the same discipline ``test_quickstart.py`` applies to doc 11.
Illustrative blocks carry a title (```python title="…") and are deliberately
not matched. A docs edit that breaks a sample breaks this test.
"""

import contextlib
import io
import os
import re
from pathlib import Path
from typing import Any

import pytest

REPO = Path(__file__).resolve().parents[2]
DOCS = REPO / "docs"

# page → a substring its combined stdout must contain: proof that the sample
# did what the page says, not merely that it ran without raising.
PAGES: dict[str, str] = {
    "index.md": "via r1",
    "quickstart.md": "hasSymptom sct:4557003 ⊑ :ChestPain",
    "coming-from-langgraph.md": "(None, None)",
    "governing-a-coding-agent.md": "via approveRoutineChange",
    "guarantees.md": "two independent worlds, identical bytes",
    "use-cases/health.md": "via highRiskWithContraindication",
    "use-cases/coding.md": "via flakyStillFailingEscalates",
    "use-cases/insurance.md": "via pendMissingPriorAuth",
    "use-cases/physical-ai.md": "via blockOnEStop",
    "use-cases/customer-ops.md": "via alreadyFlown",
}

# A fence may sit inside a tabbed/admonition block, indented by a fixed prefix;
# the closing fence carries the same prefix, and every line in between is
# dedented by exactly that prefix before it runs.
_RUNNABLE = re.compile(r"^(?P<indent>[ \t]*)```python\n(?P<body>.*?)^(?P=indent)```", re.M | re.S)


def _runnable_blocks(page: Path) -> list[str]:
    blocks = []
    for match in _RUNNABLE.finditer(page.read_text()):
        indent, body = match.group("indent"), match.group("body")
        lines = [line.removeprefix(indent) if line.strip() else "" for line in body.splitlines()]
        blocks.append("\n".join(lines) + "\n")
    return blocks


def _run_page(page: str) -> str:
    blocks = _runnable_blocks(DOCS / page)
    assert blocks, f"{page} has no runnable ```python blocks"
    cwd = os.getcwd()
    os.chdir(REPO)  # the samples use repo-relative example paths
    try:
        namespace: dict[str, Any] = {}
        out = io.StringIO()
        for index, block in enumerate(blocks):
            with contextlib.redirect_stdout(out):
                exec(compile(block, f"<{page} block {index}>", "exec"), namespace)
        return out.getvalue()
    finally:
        os.chdir(cwd)


@pytest.mark.parametrize(("page", "expected"), sorted(PAGES.items()))
def test_the_page_runs_verbatim(page: str, expected: str) -> None:
    assert expected in _run_page(page)


def test_the_change_gate_withdraws_approval_when_the_base_drifts() -> None:
    """The coding-agent page's last listing is explained in prose; pin the
    behaviour it describes."""
    from horismos import Ontology, World

    onto = Ontology.load(REPO / "examples/change-gate/change-gate.ttl")
    world = World(onto)
    pr = world.assert_(
        ":ChangeProposal",
        subject=":pr-45",
        proposed_by=":Bot",
        base_matches_head=True,
        tests_passed=True,
        lint_passed=True,
        touches_protected_path=False,
        lines_changed=900,
        human_approvals=0,
    )
    assert world.value(pr, ":gateDecision") == onto.ref(":NeedsHuman")
    world.assert_(":ChangeProposal", subject=":pr-45", human_approvals=1)
    assert world.query(":ChangeProposal", gate_decision=":Approved") == [pr]
    world.assert_(":ChangeProposal", subject=":pr-45", base_matches_head=False)
    assert world.query(":ChangeProposal", gate_decision=":Approved") == []
    assert world.value(pr, ":gateDecision") == onto.ref(":Rejected")
