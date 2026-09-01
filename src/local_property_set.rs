// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Successfully merged local property metadata.

use crate::PropertyMetadata;

/// A model type's locally validated field/getter/setter properties.
#[derive(Clone, Copy, Debug)]
pub struct LocalPropertySet {
    /// Properties ordered by their first declaration fragment.
    properties: &'static [PropertyMetadata],
}

impl LocalPropertySet {
    /// Creates a locally validated property collection.
    #[must_use]
    pub(crate) const fn new(properties: &'static [PropertyMetadata]) -> Self {
        Self { properties }
    }

    /// Returns merged properties in deterministic declaration order.
    #[must_use]
    pub const fn properties(&self) -> &'static [PropertyMetadata] {
        self.properties
    }

    /// Finds a merged property by its canonical public name.
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&'static PropertyMetadata> {
        self.properties.iter().find(|property| property.name() == name)
    }
}
