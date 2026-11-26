"""Horismos: ontology-first neuro-symbolic agent framework.

Rust core, Python bindings. The ontology is simultaneously the typed
world-state, the reasoning substrate, and the agent workflow. Neural
proposes; symbolic decides.

This package is the public surface (doc 11); ``horismos._horismos`` is the
private native module behind it — never import it directly.
"""

from importlib.metadata import PackageNotFoundError as _PackageNotFoundError
from importlib.metadata import version as _distribution_version

from horismos._agent import Agent, RunResult
from horismos._horismos import (
    PROFILE_VERSION,
    Diff,
    Fact,
    HorismosBridgeError,
    HorismosError,
    HorismosOntologyError,
    HorismosToolError,
    HorismosValidationError,
    HorismosVerificationError,
    HorismosWorkflowError,
    ProofNode,
    Ref,
    Rule,
)
from horismos._ontology import (
    AssumptionManifest,
    Bound,
    Conflict,
    Ontology,
    Region,
    RegionPredicate,
    VerificationReport,
    Witness,
    WorkflowVerification,
)
from horismos._world import Snapshot, Subscription, World

# The distribution version, in PEP 440 form (``X.Y.Z[aN]``). The native module
# carries the same version in Cargo's SemVer spelling (``X.Y.Z[-alpha.N]``);
# ``horismos.__version__`` is the Python-facing one, so it is read from the
# installed distribution metadata that maturin normalised. The native value is
# the fallback for a source tree that was never installed.
try:
    __version__ = _distribution_version("horismos")
except _PackageNotFoundError:  # pragma: no cover - uninstalled source tree
    from horismos._horismos import __version__

__all__ = [
    "PROFILE_VERSION",
    "Agent",
    "AssumptionManifest",
    "Bound",
    "Conflict",
    "Diff",
    "Fact",
    "HorismosBridgeError",
    "HorismosError",
    "HorismosOntologyError",
    "HorismosToolError",
    "HorismosValidationError",
    "HorismosVerificationError",
    "HorismosWorkflowError",
    "Ontology",
    "ProofNode",
    "Ref",
    "Region",
    "RegionPredicate",
    "Rule",
    "RunResult",
    "Snapshot",
    "Subscription",
    "VerificationReport",
    "Witness",
    "WorkflowVerification",
    "World",
    "__version__",
]
