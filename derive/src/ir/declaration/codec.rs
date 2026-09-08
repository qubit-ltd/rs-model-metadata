// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Codec IR.

// qubit-style: allow public-type-layout

use syn::LitStr;
use syn::Type;

#[derive(Clone)]
pub(crate) enum CodecIr {
    RustType(Box<Type>),
    DeclaredId(LitStr),
}
