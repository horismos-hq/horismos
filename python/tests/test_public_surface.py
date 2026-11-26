"""U10 (with U9) — every public symbol exists, is exported deliberately,
and the exact public surface is pinned."""

import horismos

EXPECTED = {
    "PROFILE_VERSION",
    "Agent",
    "AssumptionManifest",
    "Bound",
    "Conflict",
    "Diff",
    "Fact",
    "HorismosBridgeError",
    "HorismosError",
    "HorismosOntologyError",
    "HorismosToolError",
    "HorismosValidationError",
    "HorismosVerificationError",
    "HorismosWorkflowError",
    "Ontology",
    "ProofNode",
    "Ref",
    "Region",
    "RegionPredicate",
    "Rule",
    "RunResult",
    "Snapshot",
    "Subscription",
    "VerificationReport",
    "Witness",
    "WorkflowVerification",
    "World",
    "__version__",
}


def test_the_public_surface_is_exactly_the_documented_one() -> None:
    assert set(horismos.__all__) == EXPECTED


def test_every_public_symbol_resolves() -> None:
    for name in horismos.__all__:
        assert getattr(horismos, name) is not None


def test_the_version_strings_are_sane() -> None:
    # PEP 440 form of the workspace Cargo version `0.1.0`, as maturin
    # normalises it into the distribution metadata.
    assert horismos.__version__ == "0.1.0"
    assert horismos.PROFILE_VERSION == "0.1"
