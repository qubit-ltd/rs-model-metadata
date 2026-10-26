// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Source representation of a relationship target.

use syn::LitStr;
use syn::Type;

/// Source form used to identify a referenced model.
#[derive(Clone)]
pub(crate) enum ReferenceTargetIr {
    /// Target supplied by a Rust type.
    RustType(Box<Type>),
    /// Target supplied by a stable model ID.
    ModelId(LitStr),
}
