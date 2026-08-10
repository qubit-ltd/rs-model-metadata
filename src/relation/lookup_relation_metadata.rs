// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for relations resolved through a lookup.

use super::field_path::FieldPath;
use crate::type_metadata::NamedTypeRef;

/// A relation resolved by looking up another model.
#[derive(Clone, Copy, Debug)]
pub struct LookupRelationMetadata {
    /// The named model containing the lookup field.
    target: NamedTypeRef,
    /// The field path used for lookup in the target model.
    target_field: FieldPath,
}

impl LookupRelationMetadata {
    /// Creates lookup-relation metadata for a named target and target field
    /// path.
    ///
    /// # Parameters
    ///
    /// - `target`: The named model containing the lookup field.
    /// - `target_field`: The field path used for lookup in the target model.
    ///
    /// # Returns
    ///
    /// The constructed lookup-relation metadata.
    ///
    /// # Panics
    ///
    /// Panics when `target_field` is empty or contains an empty segment.
    #[must_use]
    #[inline]
    pub const fn new(target: NamedTypeRef, target_field: FieldPath) -> Self {
        validate_target_field_path(target_field);
        Self {
            target,
            target_field,
        }
    }

    /// Returns the named target model.
    ///
    /// # Returns
    ///
    /// The named model containing the lookup field.
    #[inline(always)]
    pub const fn target(self) -> NamedTypeRef {
        self.target
    }

    /// Returns the target field path used for lookup.
    ///
    /// # Returns
    ///
    /// The field path used for lookup in the target model.
    #[must_use]
    #[inline(always)]
    pub const fn target_field(self) -> FieldPath {
        self.target_field
    }
}

/// Validates a lookup relation target field path.
const fn validate_target_field_path(path: FieldPath) {
    if path.is_empty() {
        panic!("lookup relation target field path cannot be empty");
    }
    let segments = path.segments();
    let mut index = 0;
    while index < segments.len() {
        if segments[index].is_empty() {
            panic!(
                "lookup relation target field path cannot contain empty segments"
            );
        }
        index += 1;
    }
}
