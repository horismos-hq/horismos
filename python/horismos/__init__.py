"""Horismos: ontology-first neuro-symbolic agent framework.

Rust core, Python bindings. The ontology is simultaneously the typed
world-state, the reasoning substrate, and the agent workflow. Neural
proposes; symbolic decides.

This package is the public surface (doc 11); ``horismos._horismos`` is the
private native module behind it — never import it directly.
"""

from horismos._horismos import (
    PROFILE_VERSION,
    HorismosBridgeError,
    HorismosError,
    HorismosOntologyError,
    HorismosToolError,
    HorismosValidationError,
    HorismosVerificationError,
    HorismosWorkflowError,
    Ref,
    Rule,
    __version__,
)
from horismos._ontology import Ontology, VerificationReport

__all__ = [
    "PROFILE_VERSION",
    "HorismosBridgeError",
    "HorismosError",
    "HorismosOntologyError",
    "HorismosToolError",
    "HorismosValidationError",
    "HorismosVerificationError",
    "HorismosWorkflowError",
    "Ontology",
    "Ref",
    "Rule",
    "VerificationReport",
    "__version__",
]
