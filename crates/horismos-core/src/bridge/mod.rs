//! The Bridge contract: a model client behind a uniform interface.
//!
//! This module holds the **pure trait only** (perceive / narrate / rank,
//! confidence and timeout semantics). Implementations live in `horismos-bridges`
//! — `horismos-core` must never depend on a provider SDK. This inversion is the
//! enforced anti-wrapper boundary.
//!
//! Spec: `docs/framework/08-bridge.md` · Owner: PRP-004
