"""Horismos: ontology-first neuro-symbolic agent framework.

Rust core, Python bindings. The ontology is simultaneously the typed
world-state, the reasoning substrate, and the agent workflow. Neural
proposes; symbolic decides.

The public API (``Ontology``, ``World``, ...) lands with PRP-003; until then
this package only proves the Rust->Python roundtrip.
"""

from horismos._horismos import PROFILE_VERSION, __version__

__all__ = ["PROFILE_VERSION", "__version__"]
