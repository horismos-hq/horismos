"""Release gate: run the doc-11 symbolic quickstart verbatim against an installed
``horismos``.

Extracts every ``python`` block from the "Quickstart (symbolic ...)" section of
``docs/framework/11-python-api.md`` and executes them in order, from the repository
root. Both the wheel job and the source-distribution job in
``.github/workflows/release.yml`` run this one file, so the two gates cannot drift.
"""

import pathlib
import re
import sys

# The quickstart prints a proof tree, which renders with box-drawing characters
# and the subsumption sign. On Windows, Python's stdout defaults to the console
# code page (cp1252 on the runners), which has no mapping for those, so that
# print failed the gate with `UnicodeEncodeError`. Reading the doc has the same
# exposure the other way round: `read_text()` with no encoding uses the locale's,
# not the UTF-8 the file is written in. Both are pinned to UTF-8 here so the gate
# behaves identically on every runner.
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

text = pathlib.Path("docs/framework/11-python-api.md").read_text(encoding="utf-8")
section = re.split(r"^#+ .*Quickstart \(symbolic.*$", text, flags=re.M)[1]
section = re.split(r"^#+ ", section, maxsplit=1, flags=re.M)[0]
blocks = re.findall(r"```python\n(.*?)```", section, flags=re.S)
assert blocks, "no python blocks found in the doc-11 symbolic quickstart"
namespace: dict[str, object] = {}
for block in blocks:
    exec(compile(block, "<quickstart>", "exec"), namespace)
