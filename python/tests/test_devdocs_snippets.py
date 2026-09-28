"""The developer docs site's runnable snippets execute verbatim.

Every fenced block written exactly as ```python on the pages below is
extracted and executed in order, in one namespace per page, from an empty
temporary directory — the site is public and the repository is not, so a
sample may load only what the installed package ships (``horismos.examples``).
The same discipline ``test_quickstart.py`` applies to doc 11.
Illustrative blocks carry a title (```python title="…") and are deliberately
not matched. A docs edit that breaks a sample breaks this test.

Every ```text fence that immediately follows a runnable fence is the output the
page shows for it, and is pinned to the page's real stdout — the site's promise
is that a shown proof is verbatim. Both guards run only where ``make test`` runs;
the repository has no pull-request-triggered check yet (PRP-006).
"""

import contextlib
import difflib
import functools
import io
import os
import re
import tempfile
from pathlib import Path
from typing import Any

import pytest

REPO = Path(__file__).resolve().parents[2]
DOCS = REPO / "docs"

# page → a substring its combined stdout must contain: proof that the sample
# did what the page says, not merely that it ran without raising.
PAGES: dict[str, str] = {
    "index.md": "via noProximityReadingBlocks",
    "quickstart.md": "hasSymptom sct:4557003 ⊑ :ChestPain",
    "coming-from-langgraph.md": 'via r3 — "Heart rate at most 100',
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

# A ```text fence is that sample's shown output only when it immediately follows a
# runnable fence at the same indent, with nothing but blank lines between them. A
# diagram or a shell transcript elsewhere on the page never matches.
_SHOWN_OUTPUT = re.compile(
    r"^(?P<indent>[ \t]*)```python\n.*?^(?P=indent)```\n(?:[ \t]*\n)*"
    r"(?P=indent)```text\n(?P<body>.*?)^(?P=indent)```",
    re.M | re.S,
)


def _dedent(indent: str, body: str) -> str:
    lines = [line.removeprefix(indent) if line.strip() else "" for line in body.splitlines()]
    return "\n".join(lines) + "\n"


def _runnable_blocks(page: Path) -> list[str]:
    return [
        _dedent(m.group("indent"), m.group("body")) for m in _RUNNABLE.finditer(page.read_text())
    ]


def _shown_output_blocks(page: Path) -> list[tuple[int, str]]:
    """(line number, dedented body) of every fence the page presents as a sample's output."""
    text = page.read_text()
    return [
        (
            text.count("\n", 0, m.start("body")) + 1,
            _dedent(m.group("indent"), m.group("body")).strip("\n"),
        )
        for m in _SHOWN_OUTPUT.finditer(text)
    ]


@functools.cache
def _run_page(page: str) -> str:
    blocks = _runnable_blocks(DOCS / page)
    assert blocks, f"{page} has no runnable ```python blocks"
    cwd = os.getcwd()
    with tempfile.TemporaryDirectory() as empty:
        os.chdir(empty)  # no repository path resolves from here
        try:
            namespace: dict[str, Any] = {}
            out = io.StringIO()
            for index, block in enumerate(blocks):
                with contextlib.redirect_stdout(out):
                    exec(compile(block, f"<{page} block {index}>", "exec"), namespace)
            return out.getvalue()
        finally:
            os.chdir(cwd)


def _pages_with_runnable_blocks() -> set[str]:
    """Every site page outside docs/framework/ (which test_quickstart.py owns)
    that carries at least one plain ```python fence."""
    framework = DOCS / "framework"
    return {
        page.relative_to(DOCS).as_posix()
        for page in DOCS.rglob("*.md")
        if not page.is_relative_to(framework) and _runnable_blocks(page)
    }


def test_every_page_with_runnable_blocks_is_listed() -> None:
    assert _pages_with_runnable_blocks() == set(PAGES), (
        "a site page gained or lost plain ```python fences; update PAGES so every "
        "runnable sample on the site is executed"
    )


@pytest.mark.parametrize(("page", "expected"), sorted(PAGES.items()))
def test_the_page_runs_verbatim(page: str, expected: str) -> None:
    assert expected in _run_page(page)


def _closest_window(documented: list[str], actual: list[str]) -> list[str]:
    """The slice of the real stdout that best lines up with the documented block."""
    anchors = difflib.SequenceMatcher(
        None, documented, actual, autojunk=False
    ).get_matching_blocks()
    start = next((m.b - m.a for m in anchors if m.size), 0)
    start = max(0, min(start, len(actual)))
    return actual[start : start + len(documented)]


@pytest.mark.parametrize("page", sorted(PAGES))
def test_every_shown_output_block_is_the_real_stdout(page: str) -> None:
    shown = _shown_output_blocks(DOCS / page)
    assert shown, f"{page} presents no output for its runnable samples"
    stdout = _run_page(page)
    stale = []
    for line, documented in shown:
        if documented in stdout:
            continue
        actual = _closest_window(documented.splitlines(), stdout.splitlines())
        diff = "\n".join(
            difflib.unified_diff(
                documented.splitlines(),
                actual,
                fromfile=f"{page}:{line} (documented)",
                tofile=f"{page}:{line} (actual stdout)",
                lineterm="",
            )
        )
        stale.append(diff)
    assert not stale, (
        f"{len(stale)} shown output block(s) on {page} no longer match what the samples print — "
        "regenerate the block from the sample's real stdout:\n\n" + "\n\n".join(stale)
    )


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
