// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Compile-time contracts for the curated reflection facade.

#[test]
fn test_reflection_facade_is_curated() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/pass/*.rs");
    cases.compile_fail("tests/ui/fail/*.rs");
}

#[test]
#[cfg(not(feature = "generic"))]
fn test_generic_models_require_feature() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/feature/generic_requires_feature_tests.rs");
}
