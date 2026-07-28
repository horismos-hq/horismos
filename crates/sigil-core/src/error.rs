//! The `SigilError` hierarchy, mirrored by the Python `SigilError` classes.
//!
//! Every error carries `domain_text()` — the failing constraint stated in
//! domain terms ("heartRate 1280 exceeds sigil:max 300 (bpm) on :Patient"),
//! plus a machine-readable structure used by the perception retry loop.
//!
//! Spec: `docs/framework/12-rust-api.md` (§ Errors) · Owner: PRP-001
