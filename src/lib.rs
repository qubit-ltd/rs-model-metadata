// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Derive macros for `qubit-model-metadata`.

mod attribute;
mod expand;
mod input;
mod normalize;
mod runtime_path;
mod validate;

use proc_macro::TokenStream;
use quote::quote;

/// Derives static model metadata for supported Rust model declarations.
///
/// The macro accepts named and unit structs, single-field tuple newtypes, and
/// fieldless enums. It emits compile errors for unsupported declaration shapes
/// or when the runtime metadata crate cannot be resolved from the consuming
/// crate's dependencies.
#[proc_macro_derive(ModelMetadata, attributes(model))]
pub fn derive_model_metadata(input: TokenStream) -> TokenStream {
    let derive_input = match syn::parse(input) {
        Ok(derive_input) => derive_input,
        Err(error) => return error.into_compile_error().into(),
    };
    let result = input::ModelInput::parse(derive_input).and_then(|model| {
        let model = normalize::normalize(model);
        let validation_error = validate::validate(&model).err();
        let runtime_path = match runtime_path::runtime_path() {
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
                let independent_diagnostics =
                    expand::expand_independent_diagnostics(
                        &model,
                        &runtime_path,
                    );
                quote!(#diagnostics #independent_diagnostics)
            }
            None => expand::expand(&model, &runtime_path)?,
        })
    });

    match result {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}
