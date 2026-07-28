"""Skeleton smoke test — replaced by real suites when PRP-003 activates."""

import sigil


def test_native_module_roundtrip() -> None:
    assert sigil.__version__ == "0.1.0"
    assert sigil.PROFILE_VERSION == "0.1"
