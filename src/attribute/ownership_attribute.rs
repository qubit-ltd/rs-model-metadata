// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;
use syn::TypePath;

/// Parsed ownership syntax.
pub(crate) struct OwnershipAttribute {
    /// Owning-model type occurrences in source order.
    pub(crate) owner: Vec<TypePath>,
    /// The span of the complete attribute item.
    pub(crate) span: Span,
}
