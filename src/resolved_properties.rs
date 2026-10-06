// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned views of effective properties resolved from a reflection snapshot.

use std::sync::Arc;

use crate::metadata::LocalPropertySet;
use crate::metadata::PropertyMetadata;

/// Properties whose storage remains alive for as long as this view.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
///
/// use qubit_model_metadata::metadata::ResolvedProperties;
///
/// let properties = ResolvedProperties::Merged(Arc::from([]));
/// assert!(properties.properties().is_empty());
/// assert!(properties.property("name").is_none());
/// ```
#[derive(Clone, Debug)]
pub enum ResolvedProperties {
    /// Properties emitted as part of static model metadata.
    Static(&'static LocalPropertySet),
    /// Properties assembled from an explicit reflection snapshot.
    Merged(Arc<[PropertyMetadata]>),
}

impl ResolvedProperties {
    /// Returns the properties retained by this view.
    ///
    /// # Returns
    ///
    /// A borrowed slice of the static or merged properties retained by this
    /// value. The slice is valid for the lifetime of the borrow.
    #[must_use]
    #[inline]
    pub fn properties(&self) -> &[PropertyMetadata] {
        match self {
            Self::Static(properties) => properties.properties(),
            Self::Merged(properties) => properties,
        }
    }

    /// Finds a property by name within this view.
    ///
    /// # Parameters
    ///
    /// - `name`: Property name to look up.
    ///
    /// # Returns
    ///
    /// `Some(property)` when a property with `name` exists, or `None` when the
    /// view contains no property with that name.
    #[must_use]
    #[inline]
    pub fn property(&self, name: &str) -> Option<&PropertyMetadata> {
        self.properties()
            .iter()
            .find(|property| property.name() == name)
    }
}
