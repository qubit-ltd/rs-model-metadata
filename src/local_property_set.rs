// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Locally assembled property metadata.

use crate::metadata::PropertyMetadata;

/// A model type's locally validated field/getter/setter properties.
///
/// Each set contains properties assembled from one local metadata source. A
/// static set can be returned by
/// [`crate::metadata::TypeMetadata::try_properties`]; registry snapshot merges
/// that combine multiple providers are represented separately by
/// [`crate::metadata::ResolvedProperties::Merged`].
///
/// # Examples
///
/// ```
/// use std::error::Error;
///
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::LocalPropertySet;
/// use qubit_model_metadata::metadata::ResolvedProperties;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Model]
/// struct Account { name: String }
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let metadata = TypeMetadata::of::<Account>();
/// let resolved = metadata.try_properties()?;
/// let ResolvedProperties::Static(properties) = resolved else {
///     panic!("a model without snapshot adapters uses static properties");
/// };
/// let properties: &LocalPropertySet = properties;
/// assert_eq!(properties.property("name").map(|property| property.name()), Some("name"));
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LocalPropertySet {
    /// Properties ordered by their first declaration fragment.
    properties: &'static [PropertyMetadata],
}

impl LocalPropertySet {
    /// Creates a locally validated property collection.
    ///
    /// # Parameters
    ///
    /// * `properties` - The property metadata in first-declaration order.
    ///
    /// # Returns
    ///
    /// A set that retains the supplied process-lifetime property slice.
    #[must_use]
    #[inline]
    pub(crate) const fn new(properties: &'static [PropertyMetadata]) -> Self {
        Self { properties }
    }

    /// Returns locally assembled properties in deterministic declaration
    /// order.
    ///
    /// # Returns
    ///
    /// The static property slice ordered by each property's first declaration
    /// fragment.
    #[must_use]
    #[inline]
    pub const fn properties(&self) -> &'static [PropertyMetadata] {
        self.properties
    }

    /// Finds a locally assembled property by its canonical public name.
    ///
    /// # Parameters
    ///
    /// * `name` - The canonical property name to look up.
    ///
    /// # Returns
    ///
    /// `Some` with the matching property, or `None` when no property has that
    /// name.
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&'static PropertyMetadata> {
        self.properties.iter().find(|property| property.name() == name)
    }
}
