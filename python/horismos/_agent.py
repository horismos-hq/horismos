"""The public ``Agent`` surface (doc 11 § Agent): a workflow bound to bridges,
run against a :class:`World` one whole run per call.

Everything that decides — the derived schema, the ``horis:confidence`` gate,
the fallback edges, the gated perception-provenance assertion, rule chaining,
the trace — happens in core inside one native crossing. This layer names
things and supplies the run's identity and ``horis:now``, which core
deliberately never mints itself."""

from __future__ import annotations

import json
import uuid
from collections.abc import Mapping
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import Any, final

from horismos import _horismos
from horismos._horismos import ProofNode, Ref
from horismos._world import World
from horismos.bridges import Bridge


@final
@dataclass(frozen=True)
class RunResult:
    """The outcome of one :meth:`Agent.run`."""

    terminal: str
    """The terminal step the run reached, as a CURIE (``":Done"``)."""
    decision: Any | None
    """The value of the workflow's ``horis:decides`` property on the case,
    when the run gave it one."""
    decides: str | None
    """That property, as a CURIE, when ``decision`` is set."""
    fallback_taken: bool
    """True when any step followed a fallback or error edge."""
    goal_reached: bool | None
    case: Ref | None
    """The instance perception asserted for the first extracted class."""
    proof: ProofNode | None
    """The derivation of ``decision``, when a rule derived it."""
    trace: dict[str, Any]
    """The ``horismos.trace/2`` trajectory — the audit record."""


def _workflow_error(text: str) -> _horismos.HorismosWorkflowError:
    err = _horismos.HorismosWorkflowError(text)
    err.domain_text = text
    err.suggestion = None
    err.data = {}
    return err


@final
class Agent:
    """Runs one declared ``horis:Workflow`` against ``world``.

    ``bridges`` maps each ``horis:model`` prefix the workflow names
    (``"jev"`` for ``"jev:jev-1.13"``) to the bridge serving it; with exactly
    one bridge bound, every step uses it. An undeclared workflow, a prefix no
    bridge serves, or a ``rust:`` tool binding fails here, at construction,
    never mid-run: tool implementations are Rust functions, which cannot be
    bound from Python yet. A binding scheme no build implements
    (``python:``, ``http:``, ``mcp:``) is not refused here; its step takes
    ``horis:onError`` when the run reaches it.
    """

    __slots__ = ("_bridges", "_workflow", "_world")

    def __init__(self, world: World, *, workflow: str, bridges: Mapping[str, Bridge]) -> None:
        self._world = world
        self._workflow = workflow
        self._bridges = list(bridges.items())
        _horismos.check_agent(world._onto._handle, workflow, self._natives())

    def _natives(self) -> list[tuple[str, Any]]:
        # Anything else is handed over as itself, for core's domain-term refusal.
        return [(prefix, getattr(bridge, "_native", bridge)) for prefix, bridge in self._bridges]

    def run(
        self, text: str, *, run_id: str | None = None, now: datetime | None = None
    ) -> RunResult:
        """Runs the workflow once on ``text``, from its ``horis:startsAt`` step
        to a terminal step. ``run_id`` (default: a fresh UUID) and ``now``
        (default: the current UTC time, read once, here) are stamped into the
        trace and every ``horis:now`` effect; pass both to make a run's trace
        reproducible."""
        stamp = datetime.now(UTC) if now is None else now
        if stamp.tzinfo is None:
            raise _workflow_error(
                "now must be timezone-aware: horis:now is an instant, not a wall-clock reading"
            )
        world = self._world
        try:
            outcome = _horismos.run_agent(
                world._native,
                self._workflow,
                self._natives(),
                text,
                uuid.uuid4().hex if run_id is None else run_id,
                stamp,
                bool(world._subs),
            )
        except _horismos.HorismosWorkflowError as err:
            # A step failed after earlier ones committed; those facts stand.
            committed: _horismos.Diff | None = getattr(err, "_diff", None)
            if committed is not None:
                world._dispatch(committed)
            raise
        # Subscribers see the run's committed facts once, after it finished —
        # the run's steps are several transactions, dispatched as one diff.
        if outcome.diff is not None:
            world._dispatch(outcome.diff)
        trace: dict[str, Any] = json.loads(outcome.trace_json)
        decides, decision = outcome.decision if outcome.decision is not None else (None, None)
        return RunResult(
            terminal=outcome.terminal,
            decision=decision,
            decides=decides,
            fallback_taken=outcome.fallback_taken,
            goal_reached=(trace.get("outcome") or {}).get("goal_reached"),
            case=outcome.case,
            proof=outcome.proof,
            trace=trace,
        )
