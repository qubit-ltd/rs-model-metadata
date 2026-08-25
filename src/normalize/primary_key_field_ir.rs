// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use proc_macro2::Span;

/// A canonical primary-key field.
pub(crate) struct PrimaryKeyFieldIr {
    /// The normalized field name.
    pub(crate) name: String,
    /// The originating field-name or shorthand span.
    pub(crate) span: Span,
}
