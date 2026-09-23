"""The example ontologies ship inside the package (``horismos.examples``), so the
docs' samples and ``python -m horismos demo`` run after ``pip install`` alone.

The repository's ``examples/`` stays the source of truth (the Rust demos and
tests read it); the packaged files are byte-identical copies, held to it here.
That the *built wheel* carries them is the release gate's job
(``.github/scripts/demo_smoke.py``), which runs the demo from an installed
artefact; these tests run against the source tree."""

import os
import subprocess
import sys
from pathlib import Path

import pytest
from horismos import Ontology, examples

from .conftest import REPO

PACKAGED = Path(examples.__file__).parent


def test_every_packaged_ontology_is_a_byte_identical_copy_of_examples() -> None:
    shipped = examples.names()
    assert shipped, "the package ships no example ontologies"
    stale = [
        name
        for name in shipped
        if examples.path(name).read_bytes() != (REPO / "examples" / name).read_bytes()
    ]
    assert not stale, (
        f"packaged copies differ from examples/: {stale}; re-copy each with "
        "`cp examples/<name> python/horismos/examples/<name>`"
    )


def test_every_example_the_docs_load_ships() -> None:
    for name in (
        "vte/vte.ttl",
        "vte/perception.ttl",
        "triage/triage.ttl",
        "triage/snomed-ct-subset.ttl",
        "change-gate/change-gate.ttl",
        "gallery/ci-triage.ttl",
        "gallery/claims-adjudication.ttl",
        "gallery/rebooking.ttl",
        "gallery/safety-envelope.ttl",
        "gallery/vte-prophylaxis.ttl",
    ):
        assert examples.path(name).is_file(), name


def test_a_packaged_path_loads_and_its_relative_import_resolves_beside_it() -> None:
    # perception.ttl imports <./vte.ttl>: the copy next to it in the package.
    onto = Ontology.load(examples.path("vte/perception.ttl"))
    assert onto.ref(":NoteToRecommendation")  # declared in perception.ttl
    assert onto.ref(":recommendation")  # declared in the vte.ttl it imports
    assert examples.path("vte/perception.ttl").parent == PACKAGED / "vte"


@pytest.mark.parametrize("name", ["vte/nope.ttl", "../__init__.py", "jev_vte.py", ""])
def test_an_unknown_name_is_refused_with_the_list_that_ships(name: str) -> None:
    with pytest.raises(FileNotFoundError, match=r"the package ships: change-gate/change-gate\.ttl"):
        examples.path(name)


def test_the_demo_command_runs_from_an_empty_directory(tmp_path: Path) -> None:
    env = {
        k: v
        for k, v in os.environ.items()
        if k != "HORISMOS_DEMO_LIVE" and not k.endswith(("_API_KEY", "_API_KEY_FILE"))
    }
    run = subprocess.run(
        [sys.executable, "-m", "horismos", "demo"],
        cwd=tmp_path,
        env=env,
        capture_output=True,
        encoding="utf-8",
        timeout=120,
        check=False,
    )
    assert run.returncode == 0, run.stderr
    assert "mock bridge (offline)" in run.stdout
    assert "→ accepted" in run.stdout
    assert "→ REFUSED" in run.stdout
    assert list(tmp_path.iterdir()) == []  # it wrote nothing where it ran
