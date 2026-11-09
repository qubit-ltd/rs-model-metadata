// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::super::identifier_metadata::IdentifierMetadata;
use super::super::indexing_reasons::IndexingReasons;
use super::super::key_part_metadata::KeyPartMetadata;
use super::super::redact_metadata::RedactMetadata;
use super::super::reference_metadata::ReferenceMetadata;
use super::super::serde_field_metadata::SerdeFieldMetadata;
use super::super::unique_metadata::UniqueMetadata;
use super::super::validator_metadata::ValidatorMetadata;
use crate::constraint_metadata::ConstraintMetadata;
use crate::metadata::CodecMetadata;

/// Source-order view over the same strongly typed field objects.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::FieldAttributeMetadata;
/// use qubit_model_metadata::metadata::IdentifierAssignment;
/// use qubit_model_metadata::metadata::IdentifierMetadata;
///
/// static IDENTIFIER: IdentifierMetadata = IdentifierMetadata::new(IdentifierAssignment::Database);
/// let attribute = FieldAttributeMetadata::Identifier(&IDENTIFIER);
/// assert!(matches!(
///     attribute,
///     FieldAttributeMetadata::Identifier(value)
///         if value.assigned_by() == IdentifierAssignment::Database
/// ));
/// ```
#[derive(Clone, Copy, Debug)]
pub enum FieldAttributeMetadata {
    /// Identifier declaration metadata.
    Identifier(
        /// Assignment metadata for the field's identifier.
        &'static IdentifierMetadata,
    ),
    /// Reasons the field participates in an index.
    Indexed(
        /// Bit flags describing the indexing reasons attached to the field.
        IndexingReasons,
    ),
    /// Uniqueness declaration metadata.
    Unique(
        /// Uniqueness constraint metadata for the field.
        &'static UniqueMetadata,
    ),
    /// Entity-reference declaration metadata.
    Reference(
        /// Target and selection metadata for the declared entity reference.
        &'static ReferenceMetadata,
    ),
    /// Composite-key position metadata.
    KeyPart(
        /// Ordered position of the field in its composite key.
        &'static KeyPartMetadata,
    ),
    /// A standard validation constraint occurrence.
    Constraint(
        /// Constraint kind and limits associated with this field occurrence.
        &'static ConstraintMetadata,
    ),
    /// A custom validator occurrence.
    Validator(
        /// Validator function and argument metadata for this occurrence.
        &'static ValidatorMetadata,
    ),
    /// A value codec occurrence.
    Codec(
        /// Codec reference and source metadata for this field occurrence.
        &'static CodecMetadata,
    ),
    /// A redaction declaration occurrence.
    Redact(
        /// Redaction policy associated with this field.
        &'static RedactMetadata,
    ),
    /// Effective Serde behavior.
    Serde(
        /// Effective serialization and deserialization behavior for the field.
        &'static SerdeFieldMetadata,
    ),
    /// An explicit opaque-type marker.
    Opaque,
}
