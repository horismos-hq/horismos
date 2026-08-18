"""The public ``Ontology`` surface: a thin pythonization over the native
handle. Loading, naming, and lookups all happen in ``horismos-core``; this
layer only shapes signatures (keyword-only ``imports``, properties instead
of getter calls) and hosts the PRP-005 ``verify()`` stub."""

from __future__ import annotations

from dataclasses import dataclass
from os import PathLike
from typing import final

from horismos import _horismos
from horismos._horismos import Ref, Rule


@dataclass(frozen=True)
class VerificationReport:
    """What ``Ontology.verify()`` returns.

    A stub until PRP-005: ``ok`` is True for any loaded ontology, because
    everything verified today (Turtle well-formedness, the RDFS-plus
    profile, schema construction, rule validation) already ran at load —
    an ``Ontology`` value cannot exist otherwise. The BDD checks with
    domain-term witnesses land with PRP-005.
    """

    ok: bool
    notes: str


@final
class Ontology:
    """A loaded, profile-valid ontology (doc 11 § Ontology)."""

    __slots__ = ("_handle", "_namespaces")

    def __init__(self, handle: _horismos.Ontology) -> None:
        self._handle = handle
        self._namespaces: dict[str, str] = handle.namespaces()

    @classmethod
    def load(
        cls,
        path: str | PathLike[str],
        *,
        imports: list[str | PathLike[str]] | None = None,
    ) -> Ontology:
        """Parses ``path`` (Turtle), resolves declared imports plus every
        entry of ``imports``, validates the RDFS-plus profile, and builds
        the schema and rules — one native call, GIL released while it runs.
        """
        return cls(_horismos.Ontology.load(path, list(imports or [])))

    @property
    def namespaces(self) -> dict[str, str]:
        """The unified prefix map, ``{prefix: namespace IRI}``."""
        return dict(self._namespaces)

    def expand(self, term: str) -> str:
        """A CURIE's full IRI (full IRIs pass through unchanged)."""
        return self._handle.expand(term)

    def compact(self, iri: str) -> str:
        """A full IRI's Turtle prefixed form (``:Patient``)."""
        return self._handle.compact(iri)

    def ref(self, term: str) -> Ref:
        """A node handle for a declared term — what query results and
        derived values compare equal to."""
        return self._handle.ref(term)

    @property
    def rules(self) -> tuple[Rule, ...]:
        """Every parsed rule, in the engine's firing order."""
        return tuple(self._handle.rules())

    def verify(self) -> VerificationReport:
        """Re-runs verification on demand. Stub until PRP-005 (see
        :class:`VerificationReport`)."""
        return VerificationReport(
            ok=True,
            notes=(
                "stub until PRP-005: load-time checks (Turtle, RDFS-plus "
                "profile, schema, rules) already passed; BDD verification "
                "with witnesses is not yet implemented"
            ),
        )
