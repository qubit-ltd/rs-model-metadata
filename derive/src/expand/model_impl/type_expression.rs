// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Lowers property accessor Rust types to context-independent expression facts.

use proc_macro2::TokenStream;
use syn::Ident;
use syn::Type;

/// Converts Rust syntax using the shared model-field expression converter.
pub(super) fn type_expression_data(ty: &Type, runtime: &TokenStream, generic_environment: &[Ident]) -> TokenStream {
    super::super::fields::type_expression_data_with_parameters(ty, runtime, generic_environment)
}
