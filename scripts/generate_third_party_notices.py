"""Generate THIRD_PARTY_NOTICES.txt from the release closure of the native module.

The wheel ships compiled code from third-party crates, several of them MIT-only,
whose licences require their copyright and permission notices to travel with it.
Horismos's own LICENSE-MIT / LICENSE-APACHE do not carry those notices, and the
CycloneDX SBOM maturin writes carries SPDX identifiers, not notice text. This file
does.

The closure is the union, over every release target, of

    cargo tree --locked -p horismos-py --features <[tool.maturin] features> \
        --edges normal,build --target <triple>

which is what `[tool.maturin]` in pyproject.toml builds. The features are read from
that table rather than repeated here, so the closure cannot drift from the wheel. It is
a build closure: build scripts and procedural macros are included (and marked), so it
over-states what is linked rather than under-stating it.

Every licence, copyright and notice file a crate publishes is reproduced verbatim,
not only the licence one would elect under an `OR` expression. A crate that
publishes none must have an entry in SUPPLEMENTS saying where its notice was
established; otherwise the generator writes an explicit "not established" block
and exits non-zero rather than guess.

    uv run python scripts/generate_third_party_notices.py          # regenerate
    uv run python scripts/generate_third_party_notices.py --check  # fail if stale

`make notices` runs the first. The second runs in `make test`
(python/tests/test_third_party_notices.py).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
OUTPUT = REPO_ROOT / "THIRD_PARTY_NOTICES.txt"
SUPPLEMENT_DIR = REPO_ROOT / "scripts" / "third_party_notices"

# The release matrix in .github/workflows/release.yml.
TARGETS = (
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
)
ROOT_PACKAGE = "horismos-py"
# [tool.maturin] features in pyproject.toml: the features the published wheel is
# built with, bridges included.
FEATURES = ",".join(
    tomllib.loads((REPO_ROOT / "pyproject.toml").read_text(encoding="utf-8"))["tool"]["maturin"][
        "features"
    ]
)
CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"

NOTICE_FILE = re.compile(
    r"^(licen[cs]e|copying|copyright|notice|unlicen[cs]e)([-_.][^/]*)?$", re.IGNORECASE
)
SKIP_DIRS = {".git", "target"}


@dataclass(frozen=True)
class UpstreamFile:
    """A licence file absent from the published crate, fetched from the upstream
    repository at the exact commit the crate was published from."""

    commit: str
    url: str
    local: str  # path under SUPPLEMENT_DIR
    sha256: str


@dataclass(frozen=True)
class SourceHeader:
    """A notice that exists only as a comment header in the crate's own source."""

    path: str  # relative to the crate root
    first_line: int
    last_line: int
    sha256: str
    why: str


_OXIGRAPH = "4f8e1d890eee73062eaf809b779ed1b276aa11d4"
_OXIGRAPH_FILES = (
    UpstreamFile(
        commit=_OXIGRAPH,
        url=f"https://raw.githubusercontent.com/oxigraph/oxigraph/{_OXIGRAPH}/LICENSE-MIT",
        local=f"oxigraph-{_OXIGRAPH}/LICENSE-MIT",
        sha256="1f4f6736adc52ebfda18bb84947e0ef492bd86a408c0e83872efb75ed5e02838",
    ),
    UpstreamFile(
        commit=_OXIGRAPH,
        url=f"https://raw.githubusercontent.com/oxigraph/oxigraph/{_OXIGRAPH}/LICENSE-APACHE",
        local=f"oxigraph-{_OXIGRAPH}/LICENSE-APACHE",
        sha256="a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2",
    ),
)

# Crates whose published archive contains no licence file, keyed by exact version so
# that an upgrade has to re-establish the notice instead of inheriting a stale one.
SUPPLEMENTS: dict[tuple[str, str], tuple[UpstreamFile | SourceHeader, ...]] = {
    # The READMEs grant MIT OR Apache-2.0 and link licence files at the repository
    # root, which the per-crate archives leave out.
    ("oxrdf", "0.3.3"): _OXIGRAPH_FILES,
    ("oxttl", "0.2.3"): _OXIGRAPH_FILES,
    ("fxhash", "0.2.1"): (
        SourceHeader(
            path="lib.rs",
            first_line=1,
            last_line=9,
            sha256="5d392ff1ac69a18c9beb41720ecbfd56f7792a1ec4321428e73757d4283f5a61",
            why=(
                "neither the published crate nor the default branch of its upstream "
                "repository (https://github.com/cbreeden/fxhash) contains a standalone "
                "licence file; "
                "the copyright and permission notice is this source header"
            ),
        ),
    ),
}


@dataclass(frozen=True)
class Notice:
    label: str
    text: str


@dataclass(frozen=True)
class Component:
    name: str
    version: str
    license: str
    build_only: bool
    source: str
    repository: str | None
    vcs_commit: str | None
    vcs_path: str | None
    notices: tuple[Notice, ...]
    established: bool


def _cargo(*args: str) -> str:
    return subprocess.run(
        ["cargo", *args], cwd=REPO_ROOT, check=True, capture_output=True, text=True
    ).stdout


def _tree(edges: str, target: str) -> set[tuple[str, str]]:
    out = _cargo(
        "tree",
        "--locked",
        "-p",
        ROOT_PACKAGE,
        "--features",
        FEATURES,
        "--edges",
        edges,
        "--target",
        target,
        "--prefix",
        "none",
        "--format",
        "{p}",
    )
    packages = set()
    for line in out.splitlines():
        if not line.strip():
            continue
        name, version = line.split()[:2]
        packages.add((name, version.removeprefix("v")))
    return packages


def _closure() -> tuple[set[tuple[str, str]], set[tuple[str, str]]]:
    """(everything built, the part reachable without a build script or proc macro)."""
    built: set[tuple[str, str]] = set()
    normal: set[tuple[str, str]] = set()
    for target in TARGETS:
        built |= _tree("normal,build", target)
        normal |= _tree("normal,no-proc-macro", target)
    return built, normal


def _read_text(path: Path) -> str:
    text = path.read_bytes().decode("utf-8").removeprefix("\ufeff")
    return text.replace("\r\n", "\n").replace("\r", "\n").rstrip() + "\n"


def _sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _crate_notices(root: Path) -> list[Notice]:
    found = []
    for path in sorted(root.rglob("*")):
        rel = path.relative_to(root)
        if SKIP_DIRS.intersection(rel.parts[:-1]) or not path.is_file():
            continue
        if NOTICE_FILE.match(path.name):
            found.append(Notice(label=rel.as_posix(), text=_read_text(path)))
    return found


def _supplement_notices(
    name: str, version: str, root: Path, vcs_commit: str | None
) -> list[Notice]:
    notices = []
    for item in SUPPLEMENTS.get((name, version), ()):
        if isinstance(item, UpstreamFile):
            if item.commit != vcs_commit:
                raise SystemExit(
                    f"{name} {version} was published from commit {vcs_commit}, but its "
                    f"supplement is pinned to {item.commit}; re-establish the notice"
                )
            text = _read_text(SUPPLEMENT_DIR / item.local)
            label = (
                f"{Path(item.local).name} (absent from the published crate; from "
                f"{item.url}, the commit the crate was published from)"
            )
        else:
            lines = _read_text(root / item.path).splitlines()
            text = "\n".join(lines[item.first_line - 1 : item.last_line]) + "\n"
            label = (
                f"{item.path} lines {item.first_line}-{item.last_line} (source header: {item.why})"
            )
        if _sha256(text) != item.sha256:
            raise SystemExit(
                f"{name} {version}: supplement text has sha256 {_sha256(text)}, "
                f"expected {item.sha256}"
            )
        notices.append(Notice(label=label, text=text))
    return notices


def _version_key(version: str) -> tuple[tuple[int, ...], str]:
    release, _, pre = version.partition("-")
    return tuple(int(part) for part in release.split(".")), pre


def collect() -> list[Component]:
    built, normal = _closure()
    metadata = json.loads(
        _cargo(
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
            f"crates/{ROOT_PACKAGE}/Cargo.toml",
            "--features",
            FEATURES,
        )
    )
    members = set(metadata["workspace_members"])
    packages = {(p["name"], p["version"]): p for p in metadata["packages"]}

    components = []
    for key in sorted(built, key=lambda k: (k[0], _version_key(k[1]))):
        package = packages.get(key)
        if package is None:
            raise SystemExit(f"{key[0]} {key[1]} is in the closure but not in cargo metadata")
        if package["id"] in members:
            continue  # first party: covered by LICENSE-MIT / LICENSE-APACHE
        root = Path(package["manifest_path"]).parent
        vcs: dict[str, str] = {}
        vcs_file = root / ".cargo_vcs_info.json"
        if vcs_file.is_file():
            info = json.loads(vcs_file.read_text(encoding="utf-8"))
            vcs = {
                "commit": info.get("git", {}).get("sha1", ""),
                "path": info.get("path_in_vcs", ""),
            }
        commit = vcs.get("commit") or None
        notices = _crate_notices(root) + _supplement_notices(*key, root, commit)
        components.append(
            Component(
                name=key[0],
                version=key[1],
                license=package.get("license") or "(none declared)",
                build_only=key not in normal,
                source=package.get("source") or "(local path)",
                repository=package.get("repository"),
                vcs_commit=commit,
                vcs_path=vcs.get("path") or None,
                notices=tuple(notices),
                established=bool(notices),
            )
        )
    return components


HEADER = """\
THIRD-PARTY SOFTWARE NOTICES
============================

Horismos is licensed under MIT OR Apache-2.0; see LICENSE-MIT and LICENSE-APACHE.
Those files cover Horismos's own crates only, which are therefore not listed
here. The native extension module in this package (horismos._horismos) is built
from the third-party Rust crates listed below, each under its own licence. This
file reproduces their copyright, permission and licence notices.

Generated by scripts/generate_third_party_notices.py; do not edit by hand.

Which crates are listed
-----------------------
The union, over the release targets

{targets}

of `cargo tree --locked -p horismos-py --features {features}
--edges normal,build --target <target>`.

This is the set of crates *built* to produce the module, not an exact
inventory of what is linked into it. Crates marked "build-time only" are
reachable only through a build script or a procedural macro; they are listed
conservatively and are not expected to be present in the compiled module.
Crates on a normal path may still be only partly retained by the linker.

What is reproduced
------------------
For every crate, each licence, copyright and notice file found anywhere in the
published crate is reproduced verbatim (line endings normalised to LF). Where a
crate is offered under a choice of licences, the texts of every offered licence
it ships are reproduced; no choice between them is made here. Where a published
crate contains no such file, the entry says where the notice was established
instead.

Identical texts are printed once, in the "Notice texts" section, preceded by
the list of crates and files they apply to; each component entry names the
texts that apply to it as [T<n>].

For components under the Mozilla Public License 2.0, the Source Code Form is
available from the source archive URL given with the component.
"""


def render(components: list[Component]) -> str:
    texts: dict[str, int] = {}
    applies: dict[int, list[str]] = {}
    for c in components:
        for n in c.notices:
            index = texts.setdefault(n.text, len(texts) + 1)
            applies.setdefault(index, []).append(f"{c.name} {c.version}: {n.label}")

    out = [HEADER.format(targets="\n".join(f"  {t}" for t in TARGETS), features=FEATURES)]
    build_only = sum(c.build_only for c in components)
    out.append(
        f"\nComponents ({len(components)}: {len(components) - build_only} on a normal "
        f"dependency path, {build_only} build-time only)\n" + "-" * 72 + "\n"
    )
    for c in components:
        out.append(f"\n{c.name} {c.version}\n")
        out.append(f"  Licence: {c.license}\n")
        role = (
            "build-time only (build script / procedural macro)"
            if c.build_only
            else "normal dependency path"
        )
        out.append(f"  Role: {role}\n")
        if c.source == CRATES_IO:
            out.append(f"  Published source: https://crates.io/crates/{c.name}/{c.version}\n")
            out.append(
                f"  Source archive: https://static.crates.io/crates/{c.name}/"
                f"{c.name}-{c.version}.crate\n"
            )
        else:
            out.append(f"  Source: {c.source}\n")
        if c.repository:
            out.append(f"  Repository (as declared): {c.repository}\n")
        if c.vcs_commit:
            where = f", path {c.vcs_path!r}" if c.vcs_path else ""
            out.append(f"  Published from commit: {c.vcs_commit}{where}\n")
        if c.established:
            out.append("  Notices:\n")
            for n in c.notices:
                out.append(f"    [T{texts[n.text]}] {n.label}\n")
        else:
            out.append(
                "  Notices: NOT ESTABLISHED. The published crate contains no licence,\n"
                "    copyright or notice file and no supplement records where its\n"
                "    notice can be found. Its declared licence is given above.\n"
            )

    out.append("\n\nNotice texts\n" + "-" * 72 + "\n")
    for text, index in texts.items():
        out.append(f"\n{'=' * 72}\n[T{index}] applies to:\n")
        out.extend(f"  {line}\n" for line in applies[index])
        out.append(f"{'=' * 72}\n\n{text}")
    return "".join(out)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--check", action="store_true", help=f"fail if {OUTPUT.name} is not up to date"
    )
    args = parser.parse_args(argv)

    components = collect()
    rendered = render(components)
    missing = [f"{c.name} {c.version}" for c in components if not c.established]

    if args.check:
        current = OUTPUT.read_bytes().decode("utf-8") if OUTPUT.is_file() else ""
        if current != rendered:
            print(
                f"{OUTPUT.name} is stale for the current Cargo.lock; run `make notices`",
                file=sys.stderr,
            )
            return 1
    else:
        OUTPUT.write_bytes(rendered.encode("utf-8"))
        print(f"wrote {OUTPUT.name}: {len(components)} third-party components")

    if missing:
        print(
            "notice not established for: " + ", ".join(missing) + " — add a SUPPLEMENTS "
            "entry recording where each crate's notice is published",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
