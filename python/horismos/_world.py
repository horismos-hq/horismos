"""The public ``World`` surface: keyword-argument naming sugar over the
native handle. Every mutation crosses the boundary whole (one call = one
transaction, gate + chaining in core); this layer only maps ``snake_case``
keyword names to declared property IRIs and unrolls list values — the
"pythonize the names, never the semantics" half of the boundary split."""

from __future__ import annotations

import difflib
import re
from collections.abc import Callable
from typing import Any, final

from horismos import _horismos
from horismos._horismos import ProofNode, Ref
from horismos._ontology import Ontology

_CAMEL = re.compile(r"(?<!^)(?=[A-Z])")

# "no value filter" sentinel — None is a meaningless filter here anyway,
# but a distinct sentinel keeps the intent unambiguous.
_ANY: Any = object()


def _snake(name: str) -> str:
    return _CAMEL.sub("_", name).lower()


@final
class Snapshot:
    """An immutable snapshot of a :class:`World` (persistent structure
    sharing — taking one is O(1))."""

    __slots__ = ("_native",)

    def __init__(self, native: _horismos.Snapshot) -> None:
        self._native = native

    def to_bytes(self) -> bytes:
        """The versioned binary wire form (doc 03 § Snapshots)."""
        return self._native.to_bytes()

    @staticmethod
    def from_bytes(onto: Ontology, data: bytes) -> Snapshot:
        """Decodes a wire form under ``onto`` — every fact re-enters through
        the validation gate; corrupt payloads and foreign ontologies are
        refused in domain terms."""
        return Snapshot(_horismos.Snapshot.from_bytes(onto._handle, data))


@final
class Subscription:
    """A handle on one registered callback; ``cancel()`` stops delivery."""

    __slots__ = ("_key", "_world")

    def __init__(self, world: World, key: int) -> None:
        self._world = world
        self._key = key

    def cancel(self) -> None:
        self._world._subs.pop(self._key, None)


@final
class World:
    """The typed, gate-guarded instance store over a loaded ontology
    (doc 11 § World)."""

    __slots__ = (
        "_dispatching",
        "_native",
        "_next_sub",
        "_onto",
        "_pending",
        "_prop_by_snake",
        "_subs",
    )

    def __init__(self, onto: Ontology) -> None:
        self._onto = onto
        self._native = _horismos.World(onto._handle)
        self._prop_by_snake = _property_names(onto)
        self._init_subscriptions()

    def _init_subscriptions(self) -> None:
        self._subs: dict[int, tuple[str, object, Callable[[Ref, Any], None]]] = {}
        self._next_sub = 0
        self._dispatching = False
        self._pending: list[tuple[Callable[[Ref, Any], None], Ref, Any]] = []

    # -- naming sugar (this layer's entire job) -----------------------------

    def _property_iri(self, term: str) -> str:
        """A property named as a snake_case keyword, a CURIE, or a full IRI."""
        mapped = self._prop_by_snake.get(term)
        if mapped is not None:
            return mapped
        if term in self._prop_by_snake:  # present but ambiguous
            raise _naming_error(
                f'property name "{term}" is ambiguous across namespaces; '
                "write it as a CURIE (prefix:localName)"
            )
        if ":" in term:
            return self._onto.expand(term)
        raise _naming_error(
            f'no property named "{term}" is declared by the ontology',
            suggestion=_nearest(term, self._prop_by_snake),
        )

    def _pairs(self, properties: dict[str, Any]) -> list[tuple[str, Any]]:
        """Kwargs → flat ``(property IRI, value)`` pairs, lists unrolled."""
        pairs: list[tuple[str, Any]] = []
        for name, value in properties.items():
            iri = self._property_iri(name)
            values = value if isinstance(value, (list, tuple)) else [value]
            pairs.extend((iri, v) for v in values)
        return pairs

    # -- mutations (whole transactions) --------------------------------------

    def assert_(self, class_: str, *, subject: str | None = None, **properties: Any) -> Ref:
        """Asserts one instance of ``class_`` — the whole payload crosses in
        one call; the five-check gate and forward chaining to the fixed
        point run inside that same transaction (doc 03).

        The parameter names ``class_`` and ``subject`` are reserved by this
        signature: a property whose local name collides with them (or with
        another namespace's property) cannot be spelled as a keyword here.
        """
        if not self._subs:
            return self._native.assert_(self._onto.expand(class_), self._pairs(properties), subject)
        # Post-commit subscription dispatch (B2): callbacks fire after the
        # transaction committed, matched against the core-computed diff —
        # a rejected transaction raised above and notifies nobody.
        before = self._native.snapshot()
        ref = self._native.assert_(self._onto.expand(class_), self._pairs(properties), subject)
        self._dispatch(self._native.diff(before))
        return ref

    def retract(self, subject: Ref, property: str, value: object) -> None:
        """Retracts one fact through the retraction gate (doc 03)."""
        self._native.retract(subject, self._property_iri(property), value)

    # -- reads ---------------------------------------------------------------

    def instances(self, class_: str) -> list[Ref]:
        """Every instance of ``class_``, subsumption-aware, in the store's
        deterministic order."""
        return self._native.instances(self._onto.expand(class_))

    def value(self, subject: Ref, property: str) -> Any | None:
        """The value of a single-valued property (first value otherwise),
        or None when unassigned."""
        return self._native.value(subject, self._property_iri(property))

    def values(self, subject: Ref, property: str) -> list[Any]:
        """Every value of ``(subject, property)``, in assertion order."""
        return self._native.values(subject, self._property_iri(property))

    def query(self, class_: str, **patterns: Any) -> list[Ref]:
        """The tier-2 pattern query (doc 03 § Queries): every instance of
        ``class_`` (subsumption-aware) holding, for each keyword, a value
        equal to the given one — conjunction only, deterministic order.
        Undeclared classes or properties are domain-term errors, never
        silently empty results."""
        return self._native.query(self._onto.expand(class_), self._pairs(patterns))

    # -- reasoning queries (doc 04) ------------------------------------------

    def explain(self, subject: Ref, property: str) -> ProofNode | None:
        """The recorded derivation of an existing rule-derived fact —
        "why *is* this established?" — or None for base facts."""
        return self._native.explain(subject, self._property_iri(property))

    def prove(self, subject: Ref, property: str, value: object) -> ProofNode | None:
        """Goal-directed search — "could ``(subject, property, value)`` be
        established?" — returning a proof tree, or None."""
        return self._native.prove(subject, self._property_iri(property), value)

    # -- subscriptions (doc 03 § Subscriptions; B2's PRP-003 half) ------------

    def subscribe(
        self,
        property: str,
        callback: Callable[[Ref, Any], None],
        *,
        value: object = _ANY,
    ) -> Subscription:
        """Registers ``callback`` for facts added by future transactions:
        it fires once per committed ``(subject, property, value)`` fact
        whose property matches (and whose value equals ``value``, when
        given — pass a :class:`Ref` for node values), strictly after the
        transaction committed, on the mutating thread. A mutation made
        from inside a callback is its own new transaction, dispatched
        after the current dispatch drains. The runtime's transactional
        notification stage is PRP-004's; this is the registration surface.
        """
        key = self._next_sub
        self._next_sub += 1
        self._subs[key] = (self._property_iri(property), value, callback)
        return Subscription(self, key)

    def _dispatch(self, diff: _horismos.Diff) -> None:
        for fact in diff.added:
            if fact.kind != "property":
                continue
            for prop_iri, wanted, callback in list(self._subs.values()):
                if fact.property != prop_iri:
                    continue
                if wanted is not _ANY and fact.value != wanted:
                    continue
                self._pending.append((callback, fact.subject, fact.value))
        if self._dispatching:
            return  # the outer dispatch loop drains what we queued
        self._dispatching = True
        try:
            while self._pending:
                callback, subject, fact_value = self._pending.pop(0)
                callback(subject, fact_value)
        finally:
            self._dispatching = False

    # -- snapshots (doc 03) ---------------------------------------------------

    def snapshot(self) -> Snapshot:
        return Snapshot(self._native.snapshot())

    @classmethod
    def from_snapshot(cls, onto: Ontology, snapshot: Snapshot) -> World:
        """A live world restored from ``snapshot`` — refused when it was
        taken over a different ontology."""
        world = cls.__new__(cls)
        world._onto = onto
        world._native = _horismos.World.from_snapshot(onto._handle, snapshot._native)
        world._prop_by_snake = _property_names(onto)
        world._init_subscriptions()
        return world

    def diff(self, snapshot: Snapshot) -> _horismos.Diff:
        """Added/removed facts since ``snapshot``, in deterministic order."""
        return self._native.diff(snapshot._native)


def _property_names(onto: Ontology) -> dict[str, str | None]:
    """snake_case keyword → property IRI; a name two properties share maps
    to None and must be written as a CURIE instead."""
    names: dict[str, str | None] = {}
    for iri, local, _is_object, _domain in onto._handle.property_table():
        key = _snake(local)
        names[key] = None if key in names else iri
    return names


def _nearest(term: str, names: dict[str, str | None]) -> str | None:
    close = difflib.get_close_matches(term, names, n=1)
    return f"did you mean {close[0]!r}?" if close else None


def _naming_error(text: str, suggestion: str | None = None) -> Exception:
    err = _horismos.HorismosOntologyError(text)
    err.domain_text = text
    err.suggestion = suggestion
    return err
