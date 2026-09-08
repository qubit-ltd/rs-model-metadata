// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolver-specific access to declared reference selections.

use crate::metadata::PropertyPath;
use crate::metadata::ReferenceSelection;

/// Provides resolver-specific access to a selected property path.
pub(super) trait ReferenceSelectionExt {
    /// Returns the selected property path for property references.
    fn property_path(&self) -> Option<&PropertyPath<'static>>;
}

impl ReferenceSelectionExt for ReferenceSelection {
    /// Returns the selected property path, if this selection is property-based.
    fn property_path(&self) -> Option<&PropertyPath<'static>> {
        match self {
            Self::Entity => None,
            Self::Property(path) => Some(path),
        }
    }
}
