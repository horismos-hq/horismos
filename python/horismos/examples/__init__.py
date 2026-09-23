"""The example ontologies, shipped inside the package so that every sample in the
docs runs after ``pip install horismos``, with no clone and no path to know.

``path("triage/triage.ttl")`` is the installed file, ready for
:meth:`horismos.Ontology.load`. The names mirror the repository's ``examples/``
directory, which stays the source of truth: each file here is a byte-identical
copy of the one there, and the test suite holds them to it.

``python -m horismos demo`` runs :mod:`horismos.examples.jev_vte`, the VTE
text → decisions demo, on the scripted mock bridge.
"""

from __future__ import annotations

from importlib.resources import files
from pathlib import Path

__all__ = ["names", "path"]


def _root() -> Path:
    root = files(__name__)
    if not isinstance(root, Path):  # pragma: no cover - only a zipped install
        raise RuntimeError(
            "horismos is installed as a zip archive; the example ontologies need an "
            "unpacked install (pip and uv always unpack wheels)"
        )
    return root


def names() -> list[str]:
    """Every packaged example ontology, as the names :func:`path` accepts."""
    root = _root()
    return sorted(p.relative_to(root).as_posix() for p in root.rglob("*.ttl"))


def path(name: str) -> Path:
    """The installed file for one packaged example, by its name under
    ``examples/`` (``"vte/perception.ttl"``)."""
    if name in names():  # a closed list: no path outside the package resolves
        return _root().joinpath(*name.split("/"))
    raise FileNotFoundError(
        f"no packaged example named {name!r}; the package ships: {', '.join(names())}"
    )
