"""U2 — the HorismosError hierarchy (doc 11 § Errors).

The gate-rejection message pins that need a World mutation live in
test_world.py (U3); this file pins the hierarchy itself and the
ontology-error side.
"""

import horismos
import pytest
from horismos import (
    HorismosBridgeError,
    HorismosError,
    HorismosOntologyError,
    HorismosToolError,
    HorismosValidationError,
    HorismosVerificationError,
    HorismosWorkflowError,
    Ontology,
)


def test_every_subclass_is_a_horismos_error() -> None:
    for sub in (
        HorismosOntologyError,
        HorismosVerificationError,
        HorismosValidationError,
        HorismosWorkflowError,
        HorismosBridgeError,
        HorismosToolError,
    ):
        assert issubclass(sub, HorismosError)
    assert issubclass(HorismosError, Exception)


def test_the_hierarchy_is_exported_at_top_level() -> None:
    for name in (
        "HorismosError",
        "HorismosOntologyError",
        "HorismosVerificationError",
        "HorismosValidationError",
        "HorismosWorkflowError",
        "HorismosBridgeError",
        "HorismosToolError",
    ):
        assert name in horismos.__all__


def test_an_ontology_error_is_catchable_as_the_base(onto: Ontology) -> None:
    with pytest.raises(HorismosError):
        onto.ref("nope:Immediate")


def test_an_ontology_error_carries_domain_text(onto: Ontology) -> None:
    try:
        onto.ref(":Ghost")
    except HorismosOntologyError as e:
        assert str(e) == e.domain_text
        assert e.suggestion is None
    else:  # pragma: no cover
        raise AssertionError("expected HorismosOntologyError")
