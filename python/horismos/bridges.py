"""Bridges: the model clients a workflow's perception and narration steps call
(doc 08).

A bridge is a deliberately dumb client. It never asserts anything and it
decides nothing: hand one to :class:`horismos.Agent`, keyed by the
``horis:model`` prefix it serves, and the agent's run is what calls it. That
run derives the schema the model is shown (a ``horis:RulesOnly`` property is
absent from it, so a model has no field in which to return a decision),
compares the answer's confidence with the step's ``horis:confidence``, takes
the step's ``horis:fallback`` edge on low confidence or on any bridge error,
and asserts accepted instances through the validation gate with perception
provenance. There is no public way to call a bridge directly.

``MockBridge`` is always available. ``JevBridge`` and ``OpenRouterBridge``
are compiled into the published wheel; a build from source made without their
``bridges-*`` features refuses to construct them, in domain terms.
:func:`compiled` says which are present.
"""

from __future__ import annotations

import json
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Any, Literal, final

from horismos import _horismos


def compiled() -> frozenset[str]:
    """The bridges compiled into this build, by name: always ``"mock"``, plus
    ``"jev"`` and ``"openrouter"`` in the published wheel."""
    return frozenset(_horismos.BRIDGES)


def _not_compiled(bridge: str, feature: str) -> _horismos.HorismosBridgeError:
    text = (
        f"{bridge} is not compiled into this build of horismos: it was built "
        f"without the {feature} feature. The published wheel includes it; a source "
        f"build needs `--features {feature}`."
    )
    err = _horismos.HorismosBridgeError(text)
    err.domain_text = text
    err.suggestion = "pip install horismos"
    err.data = {"bridge": bridge, "feature": feature}
    return err


@final
@dataclass(frozen=True)
class PropertyHint:
    """Deployment-side wording and ordinality for one property, which the
    ontology has no vocabulary for (doc 08 § ``jev:``). ``instructions`` is
    the question put to the model; ``when_true``/``when_false`` are a boolean
    property's criteria; ``ordinal`` sends an enumeration as an ordered
    ``score`` instead of an unordered ``choice``."""

    instructions: str | None = None
    when_true: str | None = None
    when_false: str | None = None
    ordinal: bool = False


def _hint_rows(
    hints: Mapping[str, PropertyHint] | None,
) -> list[tuple[str, str | None, str | None, str | None, bool]]:
    return [
        (name, h.instructions, h.when_true, h.when_false, h.ordinal)
        for name, h in (hints or {}).items()
    ]


@final
class MockBridge:
    """The scripted, deterministic bridge every test runs on (doc 08 § The
    mock backend): no network, no key.

    ``script`` maps an input text (exact match, or ``"*"`` for any other) to
    the perception result to return — ``{"instances": [...], "confidence":
    0.94}``, optionally with ``"fact_confidence"``. Unscripted text gets the
    documented low-confidence default, which takes the step's fallback edge.
    ``narrate`` maps a rendered proof (or ``"*"``) to narration text. Every
    call is recorded; :meth:`invocations` returns them.
    """

    __slots__ = ("_native",)

    def __init__(
        self,
        script: Mapping[str, Mapping[str, Any]] | None = None,
        *,
        narrate: Mapping[str, str] | None = None,
        id: str = "mock:default",
    ) -> None:
        self._native = _horismos.MockBridge(
            id,
            [(pattern, json.dumps(result)) for pattern, result in (script or {}).items()],
            [(pattern, json.dumps({"text": text})) for pattern, text in (narrate or {}).items()],
        )

    @property
    def id(self) -> str:
        """The model identifier the trace records for calls to this bridge."""
        return self._native.id

    def invocations(self) -> list[dict[str, Any]]:
        """Every call served so far, in order — each the whole request the
        runtime sent (for perception: the input ``text`` and the derived
        ``schema``), plus its ``kind``."""
        return [{"kind": kind, **json.loads(req)} for kind, req in self._native.invocations()]

    def __repr__(self) -> str:
        return f"MockBridge(id={self.id!r})"


@final
class JevBridge:
    """TypeSafe's Jev, a text → decisions model (doc 08 § ``jev:``), over
    OpenRouter's Decisions endpoint or TypeSafe's own.

    Jev answers typed questions only, so it refuses — before any request is
    sent — a schema asking for a number or a free string, or for more than one
    class; the refusal is a bridge error and the step takes its fallback edge.
    The key is ``api_key`` when given, else the transport's environment
    variable (``OPENROUTER_API_KEY`` / ``TYPESAFE_API_KEY``, or its ``_FILE``
    variant), read at call time. It is never echoed into a repr or a trace.
    """

    __slots__ = ("_native",)

    def __init__(
        self,
        transport: Literal["openrouter", "typesafe"] = "openrouter",
        *,
        model: str | None = None,
        api_key: str | None = None,
        hints: Mapping[str, PropertyHint] | None = None,
    ) -> None:
        native = getattr(_horismos, "JevBridge", None)
        if native is None:
            raise _not_compiled("JevBridge", "bridges-jev")
        self._native = native(transport, model, api_key, _hint_rows(hints))

    @classmethod
    def from_env(cls, *, hints: Mapping[str, PropertyHint] | None = None) -> JevBridge:
        """OpenRouter when its key is configured, otherwise TypeSafe;
        ``JEV_MODEL`` overrides the model alias."""
        native = getattr(_horismos, "JevBridge", None)
        if native is None:
            raise _not_compiled("JevBridge", "bridges-jev")
        bridge = cls.__new__(cls)
        bridge._native = native.from_env(_hint_rows(hints))
        return bridge

    @property
    def id(self) -> str:
        return str(self._native.id)

    def __repr__(self) -> str:
        return f"JevBridge(id={self.id!r})"


@final
class OpenRouterBridge:
    """Any OpenRouter-routed chat model, forced into the derived schema
    through a tool call (doc 08). The key is read from
    ``OPENROUTER_API_KEY`` / ``OPENROUTER_API_KEY_FILE`` at call time."""

    __slots__ = ("_native",)

    def __init__(self, model: str) -> None:
        native = getattr(_horismos, "OpenRouterBridge", None)
        if native is None:
            raise _not_compiled("OpenRouterBridge", "bridges-openrouter")
        self._native = native(model)

    @classmethod
    def from_env(cls) -> OpenRouterBridge:
        """``OPENROUTER_MODEL``, else the bridge's default model."""
        native = getattr(_horismos, "OpenRouterBridge", None)
        if native is None:
            raise _not_compiled("OpenRouterBridge", "bridges-openrouter")
        bridge = cls.__new__(cls)
        bridge._native = native.from_env()
        return bridge

    @property
    def id(self) -> str:
        return str(self._native.id)

    def __repr__(self) -> str:
        return f"OpenRouterBridge(id={self.id!r})"


Bridge = MockBridge | JevBridge | OpenRouterBridge
"""Any bridge :class:`horismos.Agent` accepts."""

__all__ = [
    "Bridge",
    "JevBridge",
    "MockBridge",
    "OpenRouterBridge",
    "PropertyHint",
    "compiled",
]
