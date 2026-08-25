// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// A fieldless enum variant.
pub(crate) struct ModelVariant {
    /// The zero-based declaration ordinal.
    pub(crate) ordinal: usize,
    /// The normalized variant name.
    pub(crate) name: String,
}
