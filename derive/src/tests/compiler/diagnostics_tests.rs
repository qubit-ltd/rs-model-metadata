// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for combined compiler diagnostics.

use proc_macro2::Span;
use syn::Error;

use crate::compiler::diagnostics::Diagnostics;

/// Confirms independent diagnostics are retained in the emitted error.
#[test]
fn test_finish_emits_all_combined_errors() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.push(Error::new(Span::call_site(), "first error"));
    diagnostics.push(Error::new(Span::call_site(), "second error"));

    let error = diagnostics.finish().expect_err("combined diagnostics must fail");
    let tokens = error.into_compile_error().to_string();

    assert!(tokens.contains("first error"));
    assert!(tokens.contains("second error"));
}
