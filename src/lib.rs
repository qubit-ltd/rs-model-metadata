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
mod normalize;
mod runtime_path;
mod validate;

use derive_model_impl::derive_model_impl;
use proc_macro::TokenStream;

/// Derives static model metadata for supported Rust model declarations.
///
/// The macro accepts named and unit structs, single-field tuple newtypes, and
/// fieldless enums. It emits compile errors for unsupported declaration shapes
/// or when the runtime metadata crate cannot be resolved from the consuming
/// crate's dependencies.
///
/// # Parameters
///
/// - `input`: The token stream containing the model declaration and its
///   `#[model(...)]` attributes.
///
/// # Returns
///
/// Returns generated implementations and static metadata, or compile-error
/// tokens when the declaration is invalid.
///
/// # Errors
///
/// Diagnostics are returned as compile-error tokens for unsupported model
/// shapes, invalid attributes, and missing runtime dependencies.
#[proc_macro_derive(Model, attributes(model))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    derive_model_impl(input)
}

/// Derives static model metadata using the legacy macro name.
///
/// Use [`Model`] for new code. This compatibility entry point remains
/// available for existing users of the pre-`Model` API.
#[proc_macro_derive(ModelMetadata, attributes(model))]
pub fn derive_model_metadata(input: TokenStream) -> TokenStream {
    derive_model_impl(input)
}
