"""Public ontology and verification values over the native core handle.

All verification runs in ``horismos-core``. This module only converts one
complete native JSON payload into typed Python values.
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from dataclasses import dataclass, field
from os import PathLike
from types import MappingProxyType
from typing import Any, final

from horismos import _horismos
from horismos._horismos import Ref, Rule


@dataclass(frozen=True)
class Bound:
    value: str
    inclusive: bool


@dataclass(frozen=True)
class RegionPredicate:
    kind: str
    property: str
    lower: Bound | None = None
    upper: Bound | None = None
    unit: str | None = None
    value: str | None = None
    assigned: bool | None = None


@dataclass(frozen=True)
class Region:
    subject_class: str | None
    predicates: tuple[RegionPredicate, ...]


@dataclass(frozen=True)
class Witness:
    kind: str
    severity: str
    domain_text: str
    region: Region
    suggestion: str | None


@dataclass(frozen=True)
class Conflict:
    first_rule: str
    second_rule: str
    property: str
    first_value: str
    second_value: str
    first_priority: int
    second_priority: int
    winner: str | None
    region: Region


@dataclass(frozen=True)
class WorkflowVerification:
    terminating: bool
    requires_satisfiable: bool
    fallbacks_total: bool
    choices_total: bool


@dataclass(frozen=True)
class AssumptionManifest:
    """The five explicit bounds under which the report's guarantees hold.

    Value types are provisional until PRP-008's external snapshot types;
    the five fields themselves are permanent.
    """

    input_closure: tuple[str, ...]
    external_profile_versions: Mapping[str, str]
    mapping_version: str | None
    supported_subset: tuple[str, ...]
    unverified_external_effects: tuple[str, ...]


@dataclass(frozen=True)
class VerificationReport:
    ok: bool
    complete: Mapping[str, bool]
    conflicts: tuple[Conflict, ...]
    dead_rules: tuple[str, ...]
    terminating: bool
    workflows: Mapping[str, WorkflowVerification]
    witnesses: tuple[Witness, ...]
    assumptions: AssumptionManifest
    exit_code: int
    _json: str = field(repr=False, compare=False)

    def to_json(self) -> str:
        """Returns the core's canonical deterministic JSON payload."""
        return self._json


def _bound(value: Mapping[str, Any] | None) -> Bound | None:
    if value is None:
        return None
    return Bound(value=str(value["value"]), inclusive=bool(value["inclusive"]))


def _region(value: Mapping[str, Any]) -> Region:
    predicates = tuple(
        RegionPredicate(
            kind=str(predicate["kind"]),
            property=str(predicate["property"]),
            lower=_bound(predicate.get("lower")),
            upper=_bound(predicate.get("upper")),
            unit=predicate.get("unit"),
            value=predicate.get("value"),
            assigned=predicate.get("assigned"),
        )
        for predicate in value["predicates"]
    )
    return Region(subject_class=value.get("subject_class"), predicates=predicates)


def _report(payload: str, exit_code: int) -> VerificationReport:
    value: dict[str, Any] = json.loads(payload)
    assumptions = value["assumptions"]
    return VerificationReport(
        ok=bool(value["ok"]),
        complete=MappingProxyType(dict(value["complete"])),
        conflicts=tuple(
            Conflict(
                first_rule=str(conflict["first_rule"]),
                second_rule=str(conflict["second_rule"]),
                property=str(conflict["property"]),
                first_value=str(conflict["first_value"]),
                second_value=str(conflict["second_value"]),
                first_priority=int(conflict["first_priority"]),
                second_priority=int(conflict["second_priority"]),
                winner=conflict["winner"],
                region=_region(conflict["region"]),
            )
            for conflict in value["conflicts"]
        ),
        dead_rules=tuple(value["dead_rules"]),
        terminating=bool(value["terminating"]),
        workflows=MappingProxyType(
            {
                name: WorkflowVerification(**workflow)
                for name, workflow in value["workflows"].items()
            }
        ),
        witnesses=tuple(
            Witness(
                kind=str(witness["kind"]),
                severity=str(witness["severity"]),
                domain_text=str(witness["domain_text"]),
                region=_region(witness["region"]),
                suggestion=witness.get("suggestion"),
            )
            for witness in value["witnesses"]
        ),
        assumptions=AssumptionManifest(
            input_closure=tuple(assumptions["input_closure"]),
            external_profile_versions=MappingProxyType(
                dict(assumptions["external_profile_versions"])
            ),
            mapping_version=assumptions["mapping_version"],
            supported_subset=tuple(assumptions["supported_subset"]),
            unverified_external_effects=tuple(assumptions["unverified_external_effects"]),
        ),
        exit_code=exit_code,
        _json=payload,
    )


@final
class Ontology:
    """A loaded, profile-valid and, by default, verified ontology."""

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
        allow_incomplete: bool = False,
        permitted_import_roots: list[str | PathLike[str]] | None = None,
    ) -> Ontology:
        """Loads and verifies one ontology plus its explicit imports.

        A declared ``horis:imports`` target may only name a file under the
        root document's own directory. ``permitted_import_roots`` names
        further directories a deployment deliberately shares.
        """
        try:
            handle = _horismos.Ontology.load(
                path,
                list(imports or []),
                allow_incomplete,
                list(permitted_import_roots or []),
            )
        except _horismos.HorismosVerificationError as error:
            error.report = _report(error.report_json, error.exit_code)
            raise
        return cls(handle)

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
        """A node handle for a declared term."""
        return self._handle.ref(term)

    @property
    def rules(self) -> tuple[Rule, ...]:
        """Every parsed rule, in the engine's firing order."""
        return tuple(self._handle.rules())

    def verify(self) -> VerificationReport:
        """Re-runs the complete deterministic verification pass."""
        payload, exit_code = self._handle.verification_payload()
        return _report(payload, exit_code)
