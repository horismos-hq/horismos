"""Release gate: run the doc-11 symbolic quickstart verbatim against an installed
``horismos``.

Extracts every ``python`` block from the "Quickstart (symbolic ...)" section of
``docs/framework/11-python-api.md`` and executes them in order, from the repository
root. Both the wheel job and the source-distribution job in
``.github/workflows/release.yml`` run this one file, so the two gates cannot drift.
"""

import pathlib
import re

text = pathlib.Path("docs/framework/11-python-api.md").read_text()
section = re.split(r"^#+ .*Quickstart \(symbolic.*$", text, flags=re.M)[1]
section = re.split(r"^#+ ", section, maxsplit=1, flags=re.M)[0]
blocks = re.findall(r"```python\n(.*?)```", section, flags=re.S)
assert blocks, "no python blocks found in the doc-11 symbolic quickstart"
namespace: dict[str, object] = {}
for block in blocks:
    exec(compile(block, "<quickstart>", "exec"), namespace)
