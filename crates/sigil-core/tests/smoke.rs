//! Skeleton smoke test — replaced by real suites as PRPs activate.

#[test]
fn versions_are_declared() {
    assert_eq!(sigil_core::PROFILE_VERSION, "0.1");
    assert!(!sigil_core::VERSION.is_empty());
}
