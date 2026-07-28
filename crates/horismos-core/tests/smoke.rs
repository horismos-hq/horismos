//! Skeleton smoke test — replaced by real suites as PRPs activate.

#[test]
fn versions_are_declared() {
    assert_eq!(horismos_core::PROFILE_VERSION, "0.1");
    assert!(!horismos_core::VERSION.is_empty());
}
