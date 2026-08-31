// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Compile-fail coverage for role shape and shared-pipeline diagnostics.

#[test]
fn test_role_shape_diagnostics() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui_roles/fail/*.rs");
    tests.pass("tests/ui_roles/pass/*.rs");
}
