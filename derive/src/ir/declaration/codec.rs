// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parsed codec declarations retained in declaration IR.

// qubit-style: allow public-type-layout

use syn::LitStr;
use syn::Type;

/// Codec reference supplied by a model declaration.
#[derive(Clone)]
pub(crate) enum CodecIr {
    /// A Rust type used directly as the codec implementation.
    RustType(Box<Type>),
    /// A declared model identifier resolved to a codec later in compilation.
    DeclaredId(LitStr),
}
