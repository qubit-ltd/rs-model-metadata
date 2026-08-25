// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::internal::validate_named_fields;
use super::internal::validate_optional_logical_name;

/// A model-level logical-key definition.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::KeyMetadata;
///
/// const FIELDS: [&str; 1] = ["username"];
/// let key = KeyMetadata::new(Some("user"), &FIELDS);
/// assert_eq!(key.name(), Some("user"));
/// assert!(key.fields().contains(&"username"));
/// ```
#[derive(Clone, Copy, Debug)]
pub struct KeyMetadata {
    /// The optional logical key name.
    name: Option<&'static str>,
    /// The logical-key field names in declaration order.
    fields: &'static [&'static str],
}

impl KeyMetadata {
    /// Creates a logical-key definition with an optional logical name and
    /// ordered fields.
    ///
    /// # Parameters
    ///
    /// * `name` - The optional logical name of the key.
    /// * `fields` - The non-empty, distinct field names in declaration order.
    ///
    /// # Returns
    ///
    /// A logical-key definition containing the supplied fields.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty or `fields` is empty.
    #[must_use]
    pub const fn new(name: Option<&'static str>, fields: &'static [&'static str]) -> Self {
        validate_optional_logical_name(name);
        assert!(!fields.is_empty(), "logical key requires at least one field");
        validate_named_fields(fields);
        Self { name, fields }
    }

    /// Returns the optional logical key name.
    ///
    /// # Returns
    ///
    /// `Some` with the logical name when one was supplied; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn name(self) -> Option<&'static str> {
        self.name
    }

    /// Returns logical-key field names in declaration order.
    ///
    /// # Returns
    ///
    /// The statically allocated logical-key field names.
    #[must_use]
    #[inline(always)]
    pub const fn fields(self) -> &'static [&'static str] {
        self.fields
    }

    /// Returns whether this logical key contains a field with `name`.
    ///
    /// # Parameters
    ///
    /// * `name` - The normalized field name to search for.
    ///
    /// # Returns
    ///
    /// `true` when the logical key contains `name`; otherwise, `false`.
    #[must_use]
    pub fn contains(self, name: &str) -> bool {
        self.fields.contains(&name)
    }
}
