// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;
use syn::TypePath;

/// A canonical ownership relation.
pub(crate) struct OwnershipIr {
    /// Owning-model occurrences in source order.
    pub(crate) owner: Vec<TypePath>,
    /// The originating attribute span.
    pub(crate) span: Span,
}
