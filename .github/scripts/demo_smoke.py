"""Release gate: the installed package runs its own demo, as an outside developer
would, with nothing from this repository.

Run after the built wheel (or the source archive) is installed into the job's clean
interpreter. It refuses to count an import from the checkout, then runs
``python -m horismos demo`` from an empty temporary directory with the live-model
opt-in and every provider key removed, so the demo can only succeed on the example
ontologies the package itself ships, offline. Every job that installs a release
artefact runs this one file, so the gates cannot drift.
"""

import os
import pathlib
import subprocess
import sys
import tempfile

import horismos

checkout = pathlib.Path.cwd().resolve()
installed = pathlib.Path(horismos.__file__).resolve()
assert not installed.is_relative_to(checkout), (
    f"horismos was imported from the checkout ({installed}), not from the installed artefact"
)

scrubbed = {"HORISMOS_DEMO_LIVE"} | {
    f"{key}{suffix}"
    for key in ("OPENROUTER_API_KEY", "TYPESAFE_API_KEY")
    for suffix in ("", "_FILE")
}
env = {name: value for name, value in os.environ.items() if name not in scrubbed}

with tempfile.TemporaryDirectory() as empty:
    run = subprocess.run(
        [sys.executable, "-m", "horismos", "demo"],
        cwd=empty,
        env=env,
        capture_output=True,
        encoding="utf-8",
        timeout=300,
        check=False,
    )

sys.stdout.reconfigure(encoding="utf-8")  # type: ignore[union-attr]
print(run.stdout)
print(run.stderr, file=sys.stderr)
assert run.returncode == 0, f"python -m horismos demo exited {run.returncode}"
for expected in (
    "mock bridge (offline)",
    "served by mock:jev-1.13",
    "authored",  # the mock's numbers are labelled as stand-ins
    "→ accepted",
    ":recommendation = :PharmacologicProphylaxis",
    "(asserted by perception)",
    "→ REFUSED",
    "→ :ClinicianReview",
    "schema properties: activeCancer, bleedingRisk, priorVte, reducedMobility",
):
    assert expected in run.stdout, f"the demo's output lacks {expected!r}"
print(f"demo ran from the installed package at {installed.parent}")
