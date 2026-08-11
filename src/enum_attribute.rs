// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Public expansion entry point for fieldless enum declarations.

use proc_macro2::TokenStream;

use crate::model_attribute;

/// Expands the `Enum` attribute macro.
pub(crate) fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    model_attribute::expand_enum(args, input)
}
