// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural classifications for enum variants.

use crate::field_metadata::FieldMetadata;

/// The structural form of an enum variant.
///
/// Tuple and struct variants retain their fields in declaration order. Tuple
/// field names are decimal declaration ordinals such as `"0"` and `"1"`.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::EnumVariantKind;
///
/// assert!(matches!(EnumVariantKind::Unit, EnumVariantKind::Unit));
/// ```
#[must_use]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum EnumVariantKind {
    /// A variant without a payload.
    Unit,
    /// A variant with positional payload fields.
    Tuple(
        /// The positional fields in declaration order.
        &'static [FieldMetadata],
    ),
    /// A variant with named payload fields.
    Struct(
        /// The named fields in declaration order.
        &'static [FieldMetadata],
    ),
}

/// Validates declaration order and names for one variant's fields.
///
/// # Parameters
///
/// * `fields` - The variant fields to validate in declaration order.
///
/// # Panics
///
/// Panics when a field ordinal is not contiguous, a field name is empty, or
/// two fields have the same name.
pub(super) const fn validate_variant_fields(fields: &'static [FieldMetadata]) {
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
                "variant fields cannot have duplicate names"
            );
            previous += 1;
        }
        index += 1;
    }
}

/// Compares two strings without allocation.
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
