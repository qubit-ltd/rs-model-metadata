// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Declaration-level options.

// qubit-style: allow public-type-layout

use syn::LitStr;
use syn::Type;

#[derive(Clone)]
pub(crate) struct DeclarationOptions {
    /// Explicit stable model ID.
    pub(crate) id: Option<LitStr>,
    /// Entity source type, when declared.
    pub(crate) source: Option<Type>,
    /// Explicit source model ID.
    pub(crate) source_id: Option<LitStr>,
    /// Whether undeclared projection fields remain open.
    pub(crate) open: bool,
    /// Whether a value transparently wraps one field.
    pub(crate) transparent: bool,
    /// Canonical codec type, when declared.
    pub(crate) codec: Option<Type>,
}
