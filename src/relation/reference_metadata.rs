// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Metadata for direct field references.

use super::field_path::FieldPath;
use crate::model_id::ModelId;

/// A direct reference from a field to a target model field.
#[derive(Clone, Copy, Debug)]
pub struct ReferenceMetadata {
    /// The stable ID of the model containing the referenced field.
    target: ModelId,
    /// The field path within the target model.
    target_field: FieldPath,
    /// Whether the referenced record must exist.
    must_exist: bool,
    /// An optional field path that must have the same value.
    same_as: Option<FieldPath>,
}

impl ReferenceMetadata {
    /// Creates direct-reference metadata for a target model ID and field path.
    ///
    /// # Parameters
    ///
    /// - `target`: The stable ID of the model containing the referenced field.
    /// - `target_field`: The field path within the target model.
    /// - `must_exist`: Whether the referenced record must exist.
    /// - `same_as`: An optional field path that must have the same value.
    ///
    /// # Returns
    ///
    /// The constructed direct-reference metadata.
    ///
    /// # Panics
    ///
    /// Panics when `target_field` is empty or contains an empty segment, or
    /// when `same_as` is empty or contains an empty segment.
    #[must_use]
    #[inline]
    pub const fn new(target: ModelId, target_field: FieldPath, must_exist: bool, same_as: Option<FieldPath>) -> Self {
        validate_target_field_path(target_field);
        if let Some(same_as) = same_as {
            validate_same_as_path(same_as);
        }
        Self {
            target,
            target_field,
            must_exist,
            same_as,
        }
    }

    /// Returns the stable ID of the target model.
    ///
    /// # Returns
    ///
    /// The stable ID of the model containing the referenced field.
    #[inline(always)]
    pub const fn target(self) -> ModelId {
        self.target
    }

    /// Returns the field path in the target model.
    ///
    /// # Returns
    ///
    /// The field path within the target model.
    #[must_use]
    #[inline(always)]
    pub const fn target_field(self) -> FieldPath {
        self.target_field
    }

    /// Returns whether the target record must exist.
    ///
    /// # Returns
    ///
    /// `true` when the referenced record must exist; otherwise `false`.
    #[must_use]
    #[inline(always)]
    pub const fn must_exist(self) -> bool {
        self.must_exist
    }

    /// Returns the same-as field path, if this reference is constrained to one.
    ///
    /// # Returns
    ///
    /// `Some` with the constrained field path, or `None` when no same-as path
    /// is configured.
    #[must_use]
    #[inline(always)]
    pub const fn same_as(self) -> Option<FieldPath> {
        self.same_as
    }
}

/// Validates a reference target field path.
const fn validate_target_field_path(path: FieldPath) {
    if path.is_empty() {
        panic!("reference target field path cannot be empty");
    }
    let segments = path.segments();
    let mut index = 0;
    while index < segments.len() {
        if segments[index].is_empty() {
            panic!("reference target field path cannot contain empty segments");
        }
        index += 1;
    }
}

/// Validates a same-as field path used by a reference constructor.
const fn validate_same_as_path(path: FieldPath) {
    if path.is_empty() {
        panic!("reference same-as path cannot be empty");
    }
    let segments = path.segments();
    let mut index = 0;
    while index < segments.len() {
        if segments[index].is_empty() {
            panic!("reference same-as path cannot contain empty segments");
        }
        index += 1;
    }
}
