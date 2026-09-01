// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Dispatches crate-root proc-macro entry points to the shared compiler.

use proc_macro::TokenStream;

use crate::ir::MacroKind;

/// Expands one public macro through the shared compiler pipeline.
pub(crate) fn expand(kind: MacroKind, args: TokenStream, input: TokenStream) -> TokenStream {
    crate::expand::metadata::expand(kind, args.into(), input.into()).into()
}
