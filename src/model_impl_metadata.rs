// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
//
// =============================================================================

//! Generated metadata for one model-aware inherent implementation block.

use crate::LocalPropertySet;
use crate::PropertyBuildErrors;
use crate::PropertyFragment;

/// Stores raw implementation fragments and their fallible local merge.
#[derive(Clone, Copy, Debug)]
pub struct ModelImplMetadata {
    /// Field/getter/setter declarations in deterministic source order.
    fragments: &'static [PropertyFragment],
    /// Cached local merge result.
    properties: Result<&'static LocalPropertySet, &'static PropertyBuildErrors>,
}

impl ModelImplMetadata {
    /// Creates generated implementation metadata.
    #[must_use]
    pub(crate) const fn new(
        fragments: &'static [PropertyFragment],
        properties: Result<&'static LocalPropertySet, &'static PropertyBuildErrors>,
    ) -> Self {
        Self { fragments, properties }
    }

    /// Returns every unmerged field/getter/setter source fact.
    #[must_use]
    #[inline(always)]
    pub const fn fragments(&self) -> &'static [PropertyFragment] {
        self.fragments
    }

    /// Returns locally merged properties or deterministic compatibility errors.
    ///
    /// # Errors
    ///
    /// Returns errors when field, getter, or setter fragments with the same
    /// name do not share a compatible value type.
    pub const fn try_properties(&self) -> Result<&'static LocalPropertySet, &'static PropertyBuildErrors> {
        self.properties
    }
}
