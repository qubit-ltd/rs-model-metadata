// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::declared_entity_target::DeclaredEntityTarget;
use super::reference_selection::ReferenceSelection;
use crate::relation::ObjectPath;

/// Declaration metadata for one entity reference.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{DeclaredEntityTarget, FieldReferenceMetadata as ReferenceMetadata, ModelId, ReferenceSelection};
///
/// static TARGET: DeclaredEntityTarget = DeclaredEntityTarget::ModelId(ModelId::new("example.Customer"));
/// static SELECTION: ReferenceSelection = ReferenceSelection::Entity;
/// let reference = ReferenceMetadata::new(&TARGET, &SELECTION, true, None);
/// assert!(reference.existing());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ReferenceMetadata {
    /// The entity declaration selected by this reference.
    target: &'static DeclaredEntityTarget,
    /// The entity or property selected from the target.
    selection: &'static ReferenceSelection,
    /// Whether the referenced record must exist before assignment.
    existing: bool,
    /// Object navigation from the declaration to the target context, if
    /// supplied.
    path: Option<&'static ObjectPath>,
}

impl ReferenceMetadata {
    /// Records the target, selected value, existence policy and optional
    /// object navigation without resolving a registry or invoking a provider.
    ///
    /// # Parameters
    ///
    /// * `target` - Static declaration of the referenced entity.
    /// * `selection` - Entity value or property selected from that target.
    /// * `existing` - Whether the referenced record must already exist.
    /// * `path` - Optional object navigation to the target context.
    ///
    /// # Returns
    ///
    /// Reference metadata preserving the declaration without resolving it.
    #[must_use]
    #[inline]
    pub const fn new(
        target: &'static DeclaredEntityTarget,
        selection: &'static ReferenceSelection,
        existing: bool,
        path: Option<&'static ObjectPath>,
    ) -> Self {
        Self {
            target,
            selection,
            existing,
            path,
        }
    }

    /// Returns the declared entity target.
    ///
    /// # Returns
    ///
    /// The static target declaration selected by this reference.
    #[must_use]
    #[inline]
    pub const fn target(&self) -> &'static DeclaredEntityTarget {
        self.target
    }

    /// Returns the selected entity value or property.
    ///
    /// # Returns
    ///
    /// The static selection describing which target value is referenced.
    #[must_use]
    #[inline]
    pub const fn selection(&self) -> &'static ReferenceSelection {
        self.selection
    }

    /// Returns whether the referenced record must already exist.
    ///
    /// # Returns
    ///
    /// `true` when existence is required before assignment.
    #[must_use]
    #[inline]
    pub const fn existing(&self) -> bool {
        self.existing
    }

    /// Returns explicit object navigation, or None when no path was declared.
    /// Property selection is retained separately by [`Self::selection`].
    ///
    /// # Returns
    ///
    /// The declared navigation path, or `None` when the reference is local.
    #[inline]
    pub const fn path(&self) -> Option<&'static ObjectPath> {
        self.path
    }
}
