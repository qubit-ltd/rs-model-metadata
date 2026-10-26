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
    Identifier(&'static IdentifierMetadata),
    /// Reasons the field participates in an index.
    Indexed(IndexingReasons),
    /// Uniqueness declaration metadata.
    Unique(&'static UniqueMetadata),
    /// Entity-reference declaration metadata.
    Reference(&'static ReferenceMetadata),
    /// Composite-key position metadata.
    KeyPart(&'static KeyPartMetadata),
    /// A standard validation constraint occurrence.
    Constraint(&'static ConstraintMetadata),
    /// A custom validator occurrence.
    Validator(&'static ValidatorMetadata),
    /// A value codec occurrence.
    Codec(&'static CodecMetadata),
    /// A redaction declaration occurrence.
    Redact(&'static RedactMetadata),
    /// Effective Serde behavior.
    Serde(&'static SerdeFieldMetadata),
    /// An explicit opaque-type marker.
    Opaque,
}
