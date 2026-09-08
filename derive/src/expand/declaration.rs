// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compiles model declarations into reflection and metadata tokens.

use proc_macro2::TokenStream;
use syn::Error;

use crate::ir::MacroKind;

/// Expands one declaration and converts all failures to compiler diagnostics.
pub(crate) fn expand(kind: MacroKind, args: TokenStream, input: TokenStream) -> TokenStream {
    crate::expand::pipeline::run(kind, args, input).unwrap_or_else(Error::into_compile_error)
}
