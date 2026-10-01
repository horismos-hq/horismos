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
is that a shown proof is verbatim.

Every ```turtle fence whose title names a file under ``examples/`` is an owned
excerpt of that file: each blank-line-separated chunk of its body must be a
verbatim contiguous slice of the file, so an edit to the overlay fails here
loudly instead of the public page silently showing the wrong lines. An untitled
Turtle fence is illustrative and is not matched. All three guards run only where
``make test`` runs; the repository has no pull-request-triggered check yet (PRP-006).
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
    "quickstart.md": "via blockOnEStop",
    "quickstart-clinical.md": "hasSymptom sct:4557003 ⊑ :ChestPain",
    "coming-from-langgraph.md": 'via r3 — "Heart rate at most 100',
    "governing-a-coding-agent.md": "via approveRoutineChange",
    "guarantees.md": "two independent worlds, identical bytes",
    "use-cases/health.md": "via highRiskWithContraindication",
    "use-cases/coding.md": "via flakyStillFailingEscalates",
    "use-cases/insurance.md": "via pendMissingPriorAuth",
    "use-cases/physical-ai.md": "via blockOnEStop",
    "use-cases/customer-ops.md": "via alreadyFlown",
}

# page → the examples/ file it must present a titled Turtle excerpt of: without this
# sentinel, a retitled fence would drop out of the excerpt guard and it would pass empty.
EXCERPTS: dict[str, str] = {
    "quickstart.md": "examples/gallery/safety-envelope.ttl",
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

# A Turtle fence titled with an examples/ path is an excerpt contract with that file.
_EXCERPT = re.compile(
    r"^(?P<indent>[ \t]*)```turtle title=\"(?P<file>examples/[^\"\s]+)[^\"]*\"\n"
    r"(?P<body>.*?)^(?P=indent)```",
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


def _excerpt_blocks(page: Path) -> list[tuple[int, str, list[str]]]:
    """(line number, source file, blank-line-separated chunks) of every Turtle
    fence the page presents as an excerpt of a file under examples/."""
    text = page.read_text()
    return [
        (
            text.count("\n", 0, m.start()) + 1,
            m.group("file"),
            [
                chunk
                for chunk in re.split(
                    r"\n(?:[ \t]*\n)+", _dedent(m.group("indent"), m.group("body")).strip("\n")
                )
                if chunk.strip()
            ],
        )
        for m in _EXCERPT.finditer(text)
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


def test_every_titled_turtle_excerpt_is_listed() -> None:
    found = {(page, file) for page in PAGES for _, file, _ in _excerpt_blocks(DOCS / page)}
    assert found == set(EXCERPTS.items()), (
        "a listed page gained or lost a ```turtle fence titled with an examples/ file; "
        "update EXCERPTS so every excerpt on the site is guarded against drift"
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


@pytest.mark.parametrize(("page", "expected_file"), sorted(EXCERPTS.items()))
def test_every_titled_turtle_excerpt_is_a_verbatim_slice_of_its_file(
    page: str, expected_file: str
) -> None:
    excerpts = _excerpt_blocks(DOCS / page)
    assert any(file == expected_file for _, file, _ in excerpts), (
        f"{page} lost its excerpt of {expected_file}, or its fence title no longer names "
        "that file — the excerpt guard would otherwise pass without checking anything"
    )
    drifted = []
    for line, file, chunks in excerpts:
        source = "\n" + (REPO / file).read_text().rstrip("\n") + "\n"
        assert chunks, f"{page}:{line} is an empty excerpt of {file}"
        for chunk in chunks:
            if f"\n{chunk}\n" not in source:
                drifted.append(f"{page}:{line} ({file}):\n{chunk}")
    assert not drifted, (
        f"{len(drifted)} excerpt chunk(s) on {page} are no longer a verbatim contiguous slice "
        "of the file they are titled with — re-copy them from the file:\n\n" + "\n\n".join(drifted)
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
