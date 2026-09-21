//! # horismos-bridges
//!
//! Implementations of the `horismos_core::bridge` contract: provider clients
//! behind `bridges-*` feature flags, plus the mock backend for tests and the
//! authoring loop. Provider SDKs are allowed **only** in this crate — the
//! symbolic chain in `horismos-core` never depends on it.
//!
//! Spec: `docs/framework/08-bridge.md` · Owner: PRP-004

pub mod mock;

#[cfg(feature = "bridges-anthropic")]
pub mod anthropic;
#[cfg(feature = "bridges-jev")]
pub mod jev;
#[cfg(feature = "bridges-onnx")]
pub mod onnx;
#[cfg(feature = "bridges-openai")]
pub mod openai;
#[cfg(feature = "bridges-openrouter")]
pub mod openrouter;
