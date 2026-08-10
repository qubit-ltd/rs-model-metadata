// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::UniqueComparison;

/// A field participating in a unique constraint.
#[derive(Clone, Copy, Debug)]
pub struct UniqueFieldMetadata {
    /// The normalized field name.
    name: &'static str,
    /// The comparison semantics for this field.
    comparison: UniqueComparison,
}

impl UniqueFieldMetadata {
    /// Creates unique-field metadata for a normalized name and comparison
    /// semantics.
    ///
    /// # Parameters
    ///
    /// * `name` - The normalized field name.
    /// * `comparison` - The comparison semantics for the field.
    ///
    /// # Returns
    ///
    /// Unique-field metadata containing the supplied name and semantics.
    #[must_use]
    #[inline(always)]
    pub const fn new(name: &'static str, comparison: UniqueComparison) -> Self {
        Self { name, comparison }
    }

    /// Returns the normalized field name.
    ///
    /// # Returns
    ///
    /// The normalized field name.
    #[must_use]
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the comparison semantics for this field.
    ///
    /// # Returns
    ///
    /// The comparison semantics for this field.
    #[must_use]
    #[inline(always)]
    pub const fn comparison(self) -> UniqueComparison {
        self.comparison
    }
}
