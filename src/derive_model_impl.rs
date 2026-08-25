// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared expansion entry point for the public derive macros.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Result;
use syn::parse2;

use crate::expand;
use crate::input;
use crate::normalize;
use crate::validate;

/// Expands a parsed derive input with a previously resolved runtime path.
///
/// # Parameters
///
/// - `input`: The token stream for one derive declaration.
/// - `runtime_path`: The runtime crate path resolved by the proc-macro entry
///   point.
///
/// # Returns
///
/// Returns generated metadata tokens or compile-error tokens that preserve
/// syntax, validation, and runtime-resolution diagnostics.
pub(crate) fn derive_model_tokens(input: TokenStream, runtime_path: Result<TokenStream>) -> TokenStream {
    let derive_input = match parse2(input) {
        Ok(derive_input) => derive_input,
        Err(error) => return error.into_compile_error(),
    };
    let result = input::ModelInput::parse(derive_input).and_then(|model| {
        let model = normalize::normalize(model);
        let validation_error = validate::validate(&model).err();
        let runtime_path = match runtime_path {
            Ok(path) => path,
            Err(mut runtime_error) => {
                if let Some(validation_error) = validation_error {
                    runtime_error.combine(validation_error);
                }
                return Err(runtime_error);
            }
        };
        Ok(match validation_error {
            Some(error) => {
                let diagnostics = error.into_compile_error();
                let independent_diagnostics = expand::expand_independent_diagnostics(&model, &runtime_path);
                quote!(#diagnostics #independent_diagnostics)
            }
            None => expand::expand(&model, &runtime_path),
        })
    });

    match result {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error(),
    }
}
