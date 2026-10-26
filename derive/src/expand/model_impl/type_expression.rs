// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Lowers property accessor Rust types to context-independent expression facts.

use proc_macro2::TokenStream;
use syn::Ident;
use syn::Type;

/// Converts a property accessor's Rust type into tokens for its expression
/// metadata.
///
/// `runtime` selects the metadata facade used by the generated tokens, while
/// `generic_environment` identifies type and const parameters that must remain
/// parameter references instead of being represented as concrete types.
#[must_use]
#[inline]
pub(super) fn type_expression_data(ty: &Type, runtime: &TokenStream, generic_environment: &[Ident]) -> TokenStream {
    super::super::fields::type_expression_data_with_parameters(ty, runtime, generic_environment)
}
