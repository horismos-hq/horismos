# D1 — Typed instance construction: decision record

**Decided 2026-08-18**: ship the **dict-like generic form** (CURIE class
term + snake_case keyword properties) as the one blessed spelling; do
**not** ship generated proxy classes in v1. Owns and strikes doc 11's
`[DECIDE: codegen for IDE/mypy support (horismos stubgen)]`.

## The prototypes (both ran against the triage quickstart)

Candidate B — generic (shipped since U3, exercised by the whole suite):

```python
p = world.assert_(":Patient", heart_rate=128,
                  has_symptom=[":CrushingChestPain"])
assert world.value(p, ":acuity") == onto.ref(":Immediate")
```

Candidate A — generated proxy namespace (prototype:
`.superpowers/prp003/spike_proxies.py`, preserved below since that
directory is untracked scratch): `T = proxies(onto)`;
`T.Patient(heart_rate=128, has_symptom=[T.CrushingChestPain])`;
`world.value(p, ":acuity") == T.Immediate`. The prototype ran the full
quickstart correctly, including the byte-identical r1 proof, and caught
`T.Pateint` at attribute access, before the boundary.

## Evidence, against the criterion (quickstart ergonomics)

1. **Namespace pollution kills A's discoverability win.** The triage
   quickstart loads the SNOMED subset, so `T.<TAB>` lists ~50 *numeric*
   SNOMED locals (`102556003`, …) ahead of the domain names — and numeric
   locals are not legal Python attribute names at all (`T.102556003` is a
   syntax error). A's headline benefit (browse the domain by tab) fails on
   the flagship example precisely because real ontologies import external
   vocabularies.
2. **One vocabulary, everywhere.** B's strings are the CURIEs the author
   wrote in `triage.ttl` and the exact rendering in every domain-term
   error and proof (`:Patient`, `:Immediate`). A introduces a second,
   Python-local naming system that diverges from what the errors and
   proofs print.
3. **B is one concept shorter.** No `T`, no builder objects, no
   class-as-value vs enum-member distinction (in A, `T.CrushingChestPain`
   is a class acting as a value while `T.Immediate` is a `oneOf` member —
   two different underlying kinds behind one surface).
4. **Typo experience is a wash.** A catches typos at attribute access; B
   catches them at the call with a domain-term error *plus a suggestion*
   (`no property named "heart_rat" … did you mean 'heart_rate'?`) — both
   fail fast and clearly, on one crossing.
5. **Static typing favors neither today.** A is `Any`-typed without a
   stubgen; B is string-keyed. The stubgen question (doc 11's marker) is
   therefore not load-bearing for v1 and is deferred, not decided-by-default:
   see the dated gap note in `02-tasks-and-gaps.md` — revisit when PRP-007's
   authoring agent generates ontology artifacts anyway.

Not a coin-flip: (1) alone is disqualifying on the stated criterion, so no
escalation. The prototype is preserved verbatim below as evidence.

## Appendix — candidate A prototype (spike, not shipped)

```python
"""D1 spike — candidate A: generated proxy namespace over the loaded
ontology."""

from __future__ import annotations

import re
from typing import Any

from horismos import Ontology, Ref, World

_CAMEL = re.compile(r"(?<!^)(?=[A-Z])")


def _snake(name: str) -> str:
    return _CAMEL.sub("_", name).lower()


class ProxyInstance:
    """A builder: nothing exists until world.assert_(p) passes the gate."""

    def __init__(self, class_iri: str, props: dict[str, Any]) -> None:
        self.class_iri = class_iri
        self.props = props


class ProxyClass:
    def __init__(self, iri: str, local: str) -> None:
        self.iri = iri
        self.local = local

    def __call__(self, **props: Any) -> ProxyInstance:
        return ProxyInstance(self.iri, props)


class Proxies:
    """T = proxies(onto); T.Patient(...), T.CrushingChestPain, T.Immediate."""

    def __init__(self, onto: Ontology) -> None:
        self._by_name: dict[str, ProxyClass | Ref] = {}
        for iri, local, one_of in onto._handle.class_table():
            self._by_name[local] = ProxyClass(iri, local)
            for member in one_of or []:
                self._by_name[member.rsplit("#", 1)[-1]] = onto.ref(member)

    def __getattr__(self, name: str) -> ProxyClass | Ref:
        try:
            return self._by_name[name]
        except KeyError:
            raise AttributeError(
                f"the ontology declares no class or member named {name!r}"
            ) from None


def assert_proxy(world: World, p: ProxyInstance) -> Ref:
    def unwrap(v: Any) -> Any:
        if isinstance(v, ProxyClass):
            return v.iri
        if isinstance(v, list):
            return [unwrap(x) for x in v]
        return v

    props = {k: unwrap(v) for k, v in p.props.items()}
    return world.assert_(p.class_iri, **props)
```
