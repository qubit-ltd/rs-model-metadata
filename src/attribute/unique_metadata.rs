// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::UniqueComparison;
use super::UniqueFieldMetadata;
use super::internal::validate_optional_logical_name;
use super::internal::validate_unique_fields;

/// A model-level unique-constraint definition.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::UniqueComparison;
/// use qubit_model_metadata::UniqueFieldMetadata;
/// use qubit_model_metadata::UniqueMetadata;
///
/// const FIELDS: [UniqueFieldMetadata; 1] =
///     [UniqueFieldMetadata::new("email", UniqueComparison::IgnoreCase)];
/// let unique = UniqueMetadata::new(Some("user_email"), &FIELDS);
/// assert_eq!(unique.comparison_of("email"), Some(UniqueComparison::IgnoreCase));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct UniqueMetadata {
    /// The optional logical constraint name.
    name: Option<&'static str>,
    /// The unique fields in declaration order.
    fields: &'static [UniqueFieldMetadata],
}

impl UniqueMetadata {
    /// Creates a unique constraint with an optional logical name and ordered
    /// fields.
    ///
    /// # Parameters
    ///
    /// * `name` - The optional logical name of the constraint.
    /// * `fields` - The non-empty, distinct fields in declaration order.
    ///
    /// # Returns
    ///
    /// A unique-constraint definition containing the supplied fields.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty or `fields` is empty.
    #[must_use]
    pub const fn new(name: Option<&'static str>, fields: &'static [UniqueFieldMetadata]) -> Self {
        validate_optional_logical_name(name);
        assert!(!fields.is_empty(), "unique constraint requires at least one field");
        validate_unique_fields(fields);
        Self { name, fields }
    }

    /// Returns the optional logical constraint name.
    ///
    /// # Returns
    ///
    /// `Some` with the logical name when one was supplied; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn name(self) -> Option<&'static str> {
        self.name
    }

    /// Returns the unique fields in declaration order.
    ///
    /// # Returns
    ///
    /// The statically allocated unique fields.
    #[must_use]
    #[inline(always)]
    pub const fn fields(self) -> &'static [UniqueFieldMetadata] {
        self.fields
    }

    /// Returns whether this unique constraint contains a field with `name`.
    ///
    /// # Parameters
    ///
    /// * `name` - The normalized field name to search for.
    ///
    /// # Returns
    ///
    /// `true` when the constraint contains `name`; otherwise, `false`.
    #[must_use]
    pub fn contains(self, name: &str) -> bool {
        self.fields.iter().any(|field| field.name() == name)
    }

    /// Returns the comparison semantics for `name`, or `None` when it is
    /// absent.
    ///
    /// # Parameters
    ///
    /// * `name` - The normalized field name to search for.
    ///
    /// # Returns
    ///
    /// `Some` with the field's comparison semantics when `name` is present;
    /// otherwise, `None`.
    #[must_use]
    pub fn comparison_of(self, name: &str) -> Option<UniqueComparison> {
        self.fields
            .iter()
            .find(|field| field.name() == name)
            .map(|field| field.comparison())
    }
}
