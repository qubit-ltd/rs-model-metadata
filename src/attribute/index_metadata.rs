// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::validation::validate_named_fields;
use super::validation::validate_optional_logical_name;

/// A model-level index definition.
#[derive(Clone, Copy, Debug)]
pub struct IndexMetadata {
    /// The optional logical index name.
    name: Option<&'static str>,
    /// The indexed field names in declaration order.
    fields: &'static [&'static str],
}

impl IndexMetadata {
    /// Creates an index definition with an optional logical name and ordered
    /// fields.
    ///
    /// # Parameters
    ///
    /// * `name` - The optional logical name of the index.
    /// * `fields` - The non-empty, distinct field names in declaration order.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty or `fields` is empty.
    ///
    /// # Returns
    ///
    /// An index definition containing the supplied fields.
    #[must_use]
    pub const fn new(
        name: Option<&'static str>,
        fields: &'static [&'static str],
    ) -> Self {
        validate_optional_logical_name(name);
        assert!(!fields.is_empty(), "index requires at least one field");
        validate_named_fields(fields);
        Self { name, fields }
    }

    /// Returns the optional logical index name.
    ///
    /// # Returns
    ///
    /// `Some` with the logical name when one was supplied; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn name(self) -> Option<&'static str> {
        self.name
    }

    /// Returns indexed field names in declaration order.
    ///
    /// # Returns
    ///
    /// The statically allocated indexed field names.
    #[must_use]
    #[inline(always)]
    pub const fn fields(self) -> &'static [&'static str] {
        self.fields
    }

    /// Returns whether this index contains a field with `name`.
    ///
    /// # Parameters
    ///
    /// * `name` - The normalized field name to search for.
    ///
    /// # Returns
    ///
    /// `true` when the index contains `name`; otherwise, `false`.
    #[must_use]
    pub fn contains(self, name: &str) -> bool {
        self.fields.contains(&name)
    }
}
