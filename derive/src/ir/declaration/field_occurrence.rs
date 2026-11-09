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
    Identifier(
        /// Assignment mode parsed from the field's identifier annotation.
        IdentifierAssignmentIr,
    ),
    /// Marks a field whose position contributes to tuple layout.
    Indexed,
    /// Marks the field as participating in a uniqueness constraint.
    Unique(
        /// Normalized uniqueness options declared for the field.
        UniqueIr,
    ),
    /// Declares a relationship to another model.
    Reference(
        /// Normalized target, selection, and existence rules for the reference.
        ReferenceIr,
    ),
    /// Position of this field in a compound key.
    KeyPart(
        /// Zero-based order of the field in the compound key.
        usize,
    ),
    /// Validation constraints attached to this field.
    Constraint(
        /// Parsed constraint kind and its normalized bounds.
        ConstraintIr,
    ),
    /// Selector declaration used to bind validator dependencies.
    Selector(
        /// Normalized selector position, constraints, and related options.
        SelectorIr,
    ),
    /// Validator declaration attached to this field.
    Validator(
        /// Normalized validator function and argument declaration.
        ValidatorIr,
    ),
    /// Codec declaration for this field's value.
    Codec(
        /// Normalized codec type or declared codec identity.
        CodecIr,
    ),
    /// Redaction policy applied to this field.
    Redact(
        /// Normalized redaction mode and its optional level or selector.
        RedactIr,
    ),
    /// Serialization behavior declared for this field.
    Serde(
        /// Normalized serialization and deserialization options.
        SerdeIr,
    ),
    /// An unrecognized or otherwise opaque field annotation.
    Opaque,
}
