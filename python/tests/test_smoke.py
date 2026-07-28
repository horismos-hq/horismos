"""Skeleton smoke test — replaced by real suites when PRP-003 activates."""

import horismos


def test_native_module_roundtrip() -> None:
    assert horismos.__version__ == "0.1.0"
    assert horismos.PROFILE_VERSION == "0.1"
