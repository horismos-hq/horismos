"""THIRD_PARTY_NOTICES.txt is current for Cargo.lock and reaches the built artefacts.

The native module links MIT-only crates whose copyright notices must travel with
it. The notices once existed nowhere in the wheel or sdist, so these tests check
the artefacts themselves, not only the repository file.
"""

import shutil
import subprocess
import sys
import tarfile
from email.parser import Parser
from importlib.metadata import distribution
from importlib.util import find_spec
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[2]
NOTICES = "THIRD_PARTY_NOTICES.txt"
LICENSE_FILES = ("LICENSE-APACHE", "LICENSE-MIT", NOTICES)

# One copyright line from each MIT-only crate the dependency audit found missing.
SENTINELS = (
    "Copyright (c) 2019-2021 Sybila Systems Biology Laboratory",  # biodivine-lib-bdd
    "Copyright (c) 2016 Paul Mason",  # rust_decimal
)


def _assert_substantive(text: str) -> None:
    assert len(text) > 100_000, "the notices file is implausibly small"
    for sentinel in SENTINELS:
        assert sentinel in text
    assert "NOT ESTABLISHED" not in text
    assert "Mozilla Public License Version 2.0" in text  # imbl and its chunks
    assert "UNICODE, INC. LICENSE AGREEMENT" in text  # regex-syntax's nested Unicode data


def test_the_notices_file_matches_the_release_closure() -> None:
    if shutil.which("cargo") is None:
        pytest.skip("cargo is needed to resolve the release closure")
    result = subprocess.run(
        [sys.executable, str(REPO / "scripts" / "generate_third_party_notices.py"), "--check"],
        check=False,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr


def test_the_notices_file_carries_real_notice_text() -> None:
    _assert_substantive((REPO / NOTICES).read_text(encoding="utf-8"))


def test_the_sdist_ships_the_notices(tmp_path: Path) -> None:
    if find_spec("maturin") is None:
        pytest.skip("maturin (a dev dependency) is needed to build the sdist")
    subprocess.run(
        [sys.executable, "-m", "maturin", "sdist", "--out", str(tmp_path)],
        cwd=REPO,
        check=True,
        capture_output=True,
    )
    [sdist] = tmp_path.glob("horismos-*.tar.gz")
    with tarfile.open(sdist) as archive:
        root = archive.getnames()[0].split("/")[0]

        def member(name: str) -> bytes:
            extracted = archive.extractfile(f"{root}/{name}")
            assert extracted is not None, f"{name} missing from {sdist.name}"
            return extracted.read()

        for name in LICENSE_FILES:
            assert member(name) == (REPO / name).read_bytes(), name
        metadata = Parser().parsestr(member("PKG-INFO").decode("utf-8"))
    assert set(metadata.get_all("License-File") or []) >= set(LICENSE_FILES)


def test_the_installed_wheel_ships_the_notices() -> None:
    # The dev install is a wheel built by maturin; it may predate the latest
    # regeneration, so this checks presence and substance, not byte equality.
    dist = distribution("horismos")
    assert set(dist.metadata.get_all("License-File") or []) >= set(LICENSE_FILES)
    text = dist.read_text(f"licenses/{NOTICES}")
    assert text is not None, f"{NOTICES} missing from the installed dist-info"
    _assert_substantive(text)
