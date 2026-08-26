// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-internal tests for model-input parsing diagnostics.

use syn::DeriveInput;
use syn::parse_quote;

use crate::input::ModelInput;

/// Verifies enum payload validation retains every unsupported helper
/// diagnostic.
#[test]
fn test_enum_payload_scope_validation_reports_every_invalid_helper() {
    let input: DeriveInput = parse_quote! {
        #[model(id = "test.derive.Invalid")]
        enum Invalid {
            First(#[unique] String, #[indexed] i64),
            Second(#[reference(entity = "test.derive.Target")] i64),
        }
    };

    let Err(error) = ModelInput::parse(input) else {
        panic!("payload helpers should be rejected");
    };
    let message = error.into_compile_error().to_string();

    assert!(message.contains("`unique` is not supported on enum variant fields"));
    assert!(message.contains("`indexed` is not supported on enum variant fields"));
    assert!(message.contains("`reference` is not supported on enum variant fields"));
}

/// Verifies payload scope diagnostics survive unrelated helper parse failures.
#[test]
fn test_enum_payload_scope_validation_combines_parse_and_scope_errors() {
    let input: DeriveInput = parse_quote! {
        #[model(id = "test.derive.Invalid")]
        enum Invalid {
            Value(#[text(unknown = 1)] #[unique] String),
        }
    };

    let Err(error) = ModelInput::parse(input) else {
        panic!("payload helpers should be rejected");
    };
    let message = error.into_compile_error().to_string();

    assert!(message.contains("unknown `text` argument"));
    assert!(message.contains("`unique` is not supported on enum variant fields"));
}
