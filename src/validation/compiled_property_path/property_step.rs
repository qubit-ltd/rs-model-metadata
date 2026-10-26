// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One validated property access retained for a later executor.

use std::any::TypeId;

use crate::metadata::PropertyMetadata;
use crate::metadata::TypeMetadata;

/// One validated property access retained for a later executor.
///
/// The metadata is copied from the resolved property set or constructed from
/// a transparent value field. Inspecting a step never invokes a getter.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PropertyStep {
    pub(super) property: PropertyMetadata,
    pub(super) owner: Option<&'static TypeMetadata>,
    /// Exact element type expected after projecting a borrowed `Option<T>`.
    pub(super) optional_element: Option<TypeId>,
}

impl PropertyStep {
    /// Returns the metadata for this path segment.
    ///
    /// The returned metadata is a small copy of the validated descriptor and
    /// can be inspected without executing user code.
    ///
    /// # Returns
    ///
    /// The copy of the property metadata retained for this path segment.
    #[must_use]
    #[inline]
    pub(crate) const fn property(self) -> PropertyMetadata {
        self.property
    }

    /// Returns the metadata owner, when this step was compiled from a model.
    ///
    /// # Returns
    ///
    /// The model metadata that owns the property, or `None` for synthetic
    /// steps.
    #[inline]
    pub(crate) const fn owner(self) -> Option<&'static TypeMetadata> {
        self.owner
    }

    /// Returns the optional element type to project for intermediate traversal.
    ///
    /// # Returns
    ///
    /// The reflected `TypeId` expected from the optional value, or `None` when
    /// this step does not project an optional element.
    #[inline]
    pub(crate) const fn optional_element(self) -> Option<TypeId> {
        self.optional_element
    }
}
