// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::PrimaryKeyFieldMetadata;
use super::UniqueFieldMetadata;

/// Validates non-empty, distinct primary-key field names.
///
/// # Parameters
///
/// * `fields` - The primary-key fields to validate.
pub(super) const fn validate_primary_key_fields(
    fields: &'static [PrimaryKeyFieldMetadata],
) {
    let mut index = 0;
    while index < fields.len() {
        assert!(
            !fields[index].name().is_empty(),
            "primary key field names cannot be empty"
        );
        let mut previous = 0;
        while previous < index {
            assert!(
                !str_eq(fields[index].name(), fields[previous].name()),
                "primary key fields cannot contain duplicates"
            );
            previous += 1;
        }
        index += 1;
    }
}

/// Validates non-empty, distinct unique-constraint field names.
///
/// # Parameters
///
/// * `fields` - The unique-constraint fields to validate.
pub(super) const fn validate_unique_fields(
    fields: &'static [UniqueFieldMetadata],
) {
    let mut index = 0;
    while index < fields.len() {
        assert!(
            !fields[index].name().is_empty(),
            "unique field names cannot be empty"
        );
        let mut previous = 0;
        while previous < index {
            assert!(
                !str_eq(fields[index].name(), fields[previous].name()),
                "unique fields cannot contain duplicates"
            );
            previous += 1;
        }
        index += 1;
    }
}

/// Validates non-empty, distinct named fields.
///
/// # Parameters
///
/// * `names` - The field names to validate.
pub(super) const fn validate_named_fields(names: &'static [&'static str]) {
    let mut index = 0;
    while index < names.len() {
        assert!(
            !names[index].is_empty(),
            "constraint field names cannot be empty"
        );
        let mut previous = 0;
        while previous < index {
            assert!(
                !str_eq(names[index], names[previous]),
                "constraint fields cannot contain duplicates"
            );
            previous += 1;
        }
        index += 1;
    }
}

/// Validates an optional logical name when one is supplied.
pub(super) const fn validate_optional_logical_name(name: Option<&'static str>) {
    if let Some(name) = name {
        assert!(!name.is_empty(), "logical constraint names cannot be empty");
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
#[must_use]
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
