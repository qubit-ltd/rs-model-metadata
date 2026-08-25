// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::PrimaryKeyFieldMetadata;
use super::internal::validate_primary_key_fields;

/// A model-level primary-key definition.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::PrimaryKeyFieldMetadata;
/// use qubit_model_metadata::PrimaryKeyMetadata;
///
/// const FIELDS: [PrimaryKeyFieldMetadata; 1] = [PrimaryKeyFieldMetadata::new("id", true)];
/// let primary_key = PrimaryKeyMetadata::new(&FIELDS);
/// assert!(primary_key.contains("id"));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PrimaryKeyMetadata {
    /// The primary-key fields in declaration order.
    fields: &'static [PrimaryKeyFieldMetadata],
}

impl PrimaryKeyMetadata {
    /// Creates a primary-key definition from its ordered fields.
    ///
    /// # Parameters
    ///
    /// * `fields` - The non-empty, distinct fields in declaration order.
    ///
    /// # Returns
    ///
    /// A primary-key definition containing the supplied fields.
    ///
    /// # Panics
    ///
    /// Panics when `fields` is empty.
    #[must_use]
    pub const fn new(fields: &'static [PrimaryKeyFieldMetadata]) -> Self {
        assert!(!fields.is_empty(), "primary key requires at least one field");
        validate_primary_key_fields(fields);
        Self { fields }
    }

    /// Returns the primary-key fields in declaration order.
    ///
    /// # Returns
    ///
    /// The statically allocated primary-key fields.
    #[must_use]
    #[inline(always)]
    pub const fn fields(self) -> &'static [PrimaryKeyFieldMetadata] {
        self.fields
    }

    /// Returns whether this primary key contains a field with `name`.
    ///
    /// # Parameters
    ///
    /// * `name` - The normalized field name to search for.
    ///
    /// # Returns
    ///
    /// `true` when the primary key contains `name`; otherwise, `false`.
    #[must_use]
    pub fn contains(self, name: &str) -> bool {
        self.fields.iter().any(|field| field.name() == name)
    }
}
