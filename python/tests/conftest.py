"""Shared fixtures: the flagship triage ontology, loaded once per session."""

from pathlib import Path

import horismos
import pytest

REPO = Path(__file__).resolve().parents[2]
TRIAGE = REPO / "examples" / "triage" / "triage.ttl"
SNOMED = REPO / "examples" / "triage" / "snomed-ct-subset.ttl"


@pytest.fixture(scope="session")
def onto() -> horismos.Ontology:
    return horismos.Ontology.load(TRIAGE, imports=[SNOMED])
