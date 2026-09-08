// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Dispatches crate-root proc-macro entry points to the shared compiler.

use proc_macro::TokenStream;

use crate::ir::MacroKind;

/// Expands one public macro through the shared compiler pipeline.
///
/// `kind` selects the macro contract, while `args` and `input` are its raw
/// token streams. This conversion never panics for invalid user input; errors
/// are emitted as compiler diagnostics in the returned token stream.
pub(crate) fn expand(kind: MacroKind, args: TokenStream, input: TokenStream) -> TokenStream {
    crate::expand::declaration::expand(kind, args.into(), input.into()).into()
}
