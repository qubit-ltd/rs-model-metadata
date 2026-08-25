// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for structs with named fields.

use crate::field_metadata::FieldMetadata;

/// Metadata for a struct with named fields.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::FieldMetadata;
/// use qubit_model_metadata::StructMetadata;
/// use qubit_model_metadata::TypeRef;
///
/// const FIELDS: [FieldMetadata; 1] =
///     [FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &[])];
/// let metadata = StructMetadata::new(&FIELDS);
/// assert_eq!(metadata.fields()[0].name(), "id");
/// ```
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct StructMetadata {
    /// The fields in declaration order.
    fields: &'static [FieldMetadata],
}

impl StructMetadata {
    /// Creates struct metadata from fields in declaration order.
    ///
    /// # Parameters
    ///
    /// * `fields` - The named fields in declaration order.
    ///
    /// # Returns
    ///
    /// Validated immutable struct metadata.
    ///
    /// # Panics
    ///
    /// Panics when a field ordinal is not contiguous, a field name is empty,
    /// or two fields have the same name.
    #[inline]
    pub const fn new(fields: &'static [FieldMetadata]) -> Self {
        validate_struct_fields(fields);
        Self { fields }
    }

    /// Returns fields in declaration order.
    ///
    /// # Returns
    ///
    /// The named fields in declaration order.
    #[must_use]
    #[inline(always)]
    pub const fn fields(self) -> &'static [FieldMetadata] {
        self.fields
    }
}

/// Validates declaration order and names for a struct's fields.
///
/// # Parameters
///
/// * `fields` - The fields to validate in declaration order.
///
/// # Panics
///
/// Panics when a field ordinal is not contiguous, a field name is empty, or
/// two fields have the same name.
const fn validate_struct_fields(fields: &'static [FieldMetadata]) {
    let mut index = 0;
    while index < fields.len() {
        assert!(
            fields[index].ordinal() == index,
            "field ordinals must match declaration order"
        );
        assert!(!fields[index].name().is_empty(), "field names cannot be empty");
        let mut previous = 0;
        while previous < index {
            assert!(
                !str_eq(fields[index].name(), fields[previous].name()),
                "struct fields cannot have duplicate names"
            );
            previous += 1;
        }
        index += 1;
    }
}

/// Compares two static strings without allocating.
///
/// # Parameters
///
/// * `left` - The first string to compare.
/// * `right` - The second string to compare.
///
/// # Returns
///
/// `true` when both strings contain the same bytes; otherwise, `false`.
const fn str_eq(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}
