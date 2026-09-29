// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Fixed-predicate diagnostics for conditional property candidates.

/// Verifies conditional duplicates and compatibility errors with source spans.
#[test]
fn test_conditional_accessor_diagnostics() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui_cfg/pass/*.rs");
    cases.compile_fail("tests/ui_cfg/fail/*.rs");
}
