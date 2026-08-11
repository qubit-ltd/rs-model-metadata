// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Derive macros for `qubit-model-metadata`.

mod attribute;
mod derive_model_impl;
mod expand;
mod input;
mod model_attribute;
mod model_options;
mod normalize;
mod runtime_path;
mod validate;

use proc_macro::TokenStream;

/// Declares a model and derives its standard capabilities and metadata.
///
/// The macro accepts named and unit structs, single-field tuple newtypes, and
/// fieldless enums. It emits compile errors for unsupported declaration shapes
/// or when required runtime dependencies cannot be resolved from the consuming
/// crate's dependencies.
///
/// # Parameters
///
/// - `args`: The type-level model arguments, including the required `id`.
/// - `input`: The token stream containing the model declaration and its
///   `#[field(...)]` attributes.
///
/// # Returns
///
/// Returns the rewritten declaration, generated implementations, and static
/// metadata, or compile-error tokens when the declaration is invalid.
///
/// # Errors
///
/// Diagnostics are returned as compile-error tokens for unsupported model
/// shapes, invalid attributes, and missing runtime dependencies.
#[proc_macro_attribute /* required by the style checker */]
#[allow(non_snake_case)]
pub fn Model(args: TokenStream, input: TokenStream) -> TokenStream {
    model_attribute::expand(args.into(), input.into()).into()
}
