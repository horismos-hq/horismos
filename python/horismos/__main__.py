"""Command-line verification surface for CI and ontology authoring."""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence

from horismos import HorismosError, Ontology


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="python -m horismos")
    commands = parser.add_subparsers(dest="command", required=True)
    verify = commands.add_parser("verify", help="verify an ontology and emit JSON")
    verify.add_argument("path")
    verify.add_argument("--import", dest="imports", action="append", default=[])
    args = parser.parse_args(argv)

    if args.command == "verify":
        try:
            ontology = Ontology.load(args.path, imports=args.imports, allow_incomplete=True)
            report = ontology.verify()
        except HorismosError as error:
            print(str(error), file=sys.stderr)
            return 1
        print(report.to_json())
        return report.exit_code
    return 1


if __name__ == "__main__":  # pragma: no cover - exercised via subprocess
    raise SystemExit(main())
