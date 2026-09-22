"""Tests for the release workflow's real-index tag guard.

The guard is inline shell in `.github/workflows/release.yml`. These tests lift
that `run:` block out of the workflow verbatim and execute it under bash, so
they exercise the exact bytes a release runs, not a copy that could drift.
`REF_TYPE` / `REF_NAME` reach the shell as environment variables, as they do in
the workflow; the guard must never interpolate them into an expression.

Standard library only, so it runs anywhere a Python >= 3.11 does:

    python .github/scripts/test_release_guard.py
"""

from __future__ import annotations

import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import textwrap
import unittest

REPO = pathlib.Path(__file__).resolve().parents[2]
WORKFLOW = REPO / ".github" / "workflows" / "release.yml"
STEP_NAME = "Require a matching v* tag before any upload to the real index"
VERSION = "0.1.0-alpha.9"


def guard_script() -> str:
    """Return the guard step's `run: |` block, dedented, exactly as written."""
    lines = WORKFLOW.read_text(encoding="utf-8").splitlines()
    starts = [i for i, line in enumerate(lines) if line.strip() == f"- name: {STEP_NAME}"]
    if len(starts) != 1:
        raise AssertionError(f"expected one step named {STEP_NAME!r}, found {len(starts)}")
    step_indent = len(lines[starts[0]]) - len(lines[starts[0]].lstrip())
    for i in range(starts[0] + 1, len(lines)):
        line = lines[i]
        indent = len(line) - len(line.lstrip())
        if line.strip() and indent <= step_indent:
            break  # left the step without finding its script
        if line.strip() == "run: |":
            body: list[str] = []
            for block_line in lines[i + 1 :]:
                if block_line.strip() and len(block_line) - len(block_line.lstrip()) <= indent:
                    break
                body.append(block_line)
            return textwrap.dedent("\n".join(body)) + "\n"
    raise AssertionError(f"step {STEP_NAME!r} has no `run: |` block")


class ReleaseGuardTest(unittest.TestCase):
    def setUp(self) -> None:
        self.bash = shutil.which("bash")
        if self.bash is None:
            self.skipTest("bash is not available")
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.work = pathlib.Path(tmp.name)
        (self.work / "Cargo.toml").write_text(
            f'[workspace]\nmembers = []\n\n[workspace.package]\nversion = "{VERSION}"\n',
            encoding="utf-8",
        )
        # The guard calls `python`; point it at this interpreter (tomllib).
        shim = self.work / "bin"
        shim.mkdir()
        (shim / "python").symlink_to(sys.executable)
        self.path = f"{shim}{os.pathsep}{os.environ.get('PATH', '')}"
        self.script = guard_script()

    def run_guard(self, ref_type: str, ref_name: str) -> subprocess.CompletedProcess[str]:
        assert self.bash is not None
        return subprocess.run(
            [self.bash, "-c", self.script],
            cwd=self.work,
            env={"PATH": self.path, "REF_TYPE": ref_type, "REF_NAME": ref_name},
            capture_output=True,
            text=True,
            check=False,
        )

    def test_v_prefixed_tag_matching_the_workspace_version_passes(self) -> None:
        result = self.run_guard("tag", f"v{VERSION}")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(f"matches the workspace version '{VERSION}'", result.stdout)

    def test_v_prefixed_tag_with_a_different_version_is_refused(self) -> None:
        result = self.run_guard("tag", "v0.0.0")
        self.assertEqual(result.returncode, 1)
        self.assertIn("carries version '0.0.0'", result.stdout)

    def test_unprefixed_tag_equal_to_the_workspace_version_is_refused(self) -> None:
        # The case `${REF_NAME#v}` alone let through: stripping an absent
        # prefix leaves the name unchanged, and it equals the version.
        result = self.run_guard("tag", VERSION)
        self.assertEqual(result.returncode, 1)
        self.assertIn("are v-prefixed", result.stdout)
        self.assertNotIn("matches the workspace version", result.stdout)

    def test_branch_ref_is_refused_even_when_named_like_a_release(self) -> None:
        result = self.run_guard("branch", f"v{VERSION}")
        self.assertEqual(result.returncode, 1)
        self.assertIn("that ref is a branch, not a tag", result.stdout)

    def test_shell_metacharacters_in_the_ref_stay_literal(self) -> None:
        marker = self.work / "INJECTED"
        result = self.run_guard("tag", f"v{VERSION};touch {marker}")
        self.assertEqual(result.returncode, 1)
        self.assertFalse(marker.exists(), "REF_NAME was executed as shell")


if __name__ == "__main__":
    unittest.main(verbosity=2)
