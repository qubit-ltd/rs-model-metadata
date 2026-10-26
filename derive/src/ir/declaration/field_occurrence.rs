// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized field annotation and structural-property occurrences.

use super::super::CodecIr;
use super::super::ConstraintIr;
use super::super::RedactIr;
use super::super::ReferenceIr;
use super::super::SelectorIr;
use super::super::SerdeIr;
use super::super::UniqueIr;
use super::super::ValidatorIr;
use super::IdentifierAssignmentIr;

/// One normalized annotation or structural property attached to a model field.
#[derive(Clone)]
pub(crate) enum FieldOccurrence {
    /// Identity storage strategy selected for the field.
    Identifier(IdentifierAssignmentIr),
    /// Marks a field whose position contributes to tuple layout.
    Indexed,
    /// Marks the field as participating in a uniqueness constraint.
    Unique(UniqueIr),
    /// Declares a relationship to another model.
    Reference(ReferenceIr),
    /// Position of this field in a compound key.
    KeyPart(usize),
    /// Validation constraints attached to this field.
    Constraint(ConstraintIr),
    /// Selector declaration used to bind validator dependencies.
    Selector(SelectorIr),
    /// Validator declaration attached to this field.
    Validator(ValidatorIr),
    /// Codec declaration for this field's value.
    Codec(CodecIr),
    /// Redaction policy applied to this field.
    Redact(RedactIr),
    /// Serialization behavior declared for this field.
    Serde(SerdeIr),
    /// An unrecognized or otherwise opaque field annotation.
    Opaque,
}
