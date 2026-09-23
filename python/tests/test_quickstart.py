"""U10 — the DoD run: the documented symbolic quickstarts execute
**verbatim**. The code under test is extracted from the markdown sources,
so a docs edit that breaks the quickstart breaks this test."""

import contextlib
import io
import os
import re
from pathlib import Path
from typing import Any

import pytest

REPO = Path(__file__).resolve().parents[2]


def _python_blocks(markdown: Path, heading: str) -> list[str]:
    """Every ```python block between `heading` and the next same-level
    heading."""
    text = markdown.read_text()
    section = re.split(rf"^#+ .*{re.escape(heading)}.*$", text, flags=re.M)[1]
    section = re.split(r"^#+ ", section, maxsplit=1, flags=re.M)[0]
    blocks = re.findall(r"```python\n(.*?)```", section, flags=re.S)
    assert blocks, f"no python blocks under {heading!r} in {markdown}"
    return blocks


def _run_verbatim(blocks: list[str], where: Path = REPO) -> None:
    cwd = os.getcwd()
    os.chdir(where)
    try:
        namespace: dict[str, Any] = {}
        out = io.StringIO()
        for block in blocks:
            with contextlib.redirect_stdout(out):
                exec(compile(block, "<quickstart>", "exec"), namespace)
        rendered = out.getvalue()
        assert "via r1" in rendered, "the quickstart prints the r1 proof"
    finally:
        os.chdir(cwd)


def test_the_doc_11_symbolic_quickstart_runs_verbatim(tmp_path: Path) -> None:
    blocks = _python_blocks(
        REPO / "docs" / "framework" / "11-python-api.md",
        "Quickstart (symbolic",
    )
    # From an empty directory: the published quickstart loads the package's own
    # examples, never a repository path (the release gate runs it the same way
    # against every built wheel).
    _run_verbatim(blocks, where=tmp_path)


def test_the_prp_003_example_1_runs_verbatim() -> None:
    blocks = _python_blocks(
        REPO / "PRPs" / "PRP-003-python-bindings" / "01-objective.md",
        "Example 1",
    )
    _run_verbatim(blocks)  # an internal design doc: repo-relative fixture paths


def test_example_2_runs_verbatim_where_it_raises() -> None:
    """Example 2's first snippet is a try/except that must catch; the
    second snippet is documented to raise — both behaviors are pinned."""
    import horismos
    from horismos import Ontology, World

    onto = Ontology.load(
        REPO / "examples/triage/triage.ttl",
        imports=[REPO / "examples/triage/snomed-ct-subset.ttl"],
    )
    world = World(onto)
    try:
        world.assert_(":Patient", heart_rate=1280)
    except horismos.HorismosValidationError as e:
        assert str(e) == "heartRate 1280 exceeds horis:max 300 (bpm) on :Patient"
        assert e.constraint == "max"
        assert e.suggestion is None
    else:  # pragma: no cover
        raise AssertionError("Example 2 must raise")

    with pytest.raises(horismos.HorismosValidationError) as exc:
        world.assert_(":Patient", acuity=":Immediate")
    assert "horis:RulesOnly" in str(exc.value)
