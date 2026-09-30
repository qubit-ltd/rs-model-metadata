// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field and enum-variant IR.

// qubit-style: allow multiple-public-types

use syn::Type;

use super::super::Located;
use super::CodecIr;
use super::ConstraintIr;
use super::RedactIr;
use super::ReferenceIr;
use super::SelectorIr;
use super::SerdeIr;
use super::ValidatorIr;

/// One normalized annotation or structural property attached to a model field.
#[derive(Clone)]
pub(crate) enum FieldOccurrence {
    /// Identity storage strategy selected for the field.
    Identifier(IdentifierAssignmentIr),
    /// Marks a field whose position contributes to tuple layout.
    Indexed,
    /// Marks the field as participating in a uniqueness constraint.
    Unique(super::UniqueIr),
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

/// Strategy for assigning an identifier from application or database state.
#[derive(Clone, Copy)]
pub(crate) enum IdentifierAssignmentIr {
    /// The application assigns the identifier before persistence.
    Application,
    /// The database assigns the identifier during persistence.
    Database,
}

/// Normalized field declaration metadata consumed by model expansion.
#[derive(Clone)]
pub(crate) struct FieldIr {
    /// Enum variant containing this field, when applicable.
    pub(crate) variant_index: Option<usize>,
    /// Reflection declaration index.
    pub(crate) index: Located<usize>,
    /// Declared Rust field type.
    pub(crate) ty: Type,
    /// Normalized field attribute occurrences.
    pub(crate) occurrences: Vec<FieldOccurrence>,
    /// Whether Serde serialization remains enabled.
    pub(crate) keep_serializing: bool,
    /// Whether the source field has a name.
    pub(crate) named: bool,
}

/// Normalized enum variant names and its declared fields.
#[derive(Clone)]
pub(crate) struct VariantIr {
    /// Rust source variant name.
    pub(crate) rust_name: String,
    /// Canonical metadata variant name.
    pub(crate) canonical_name: String,
    /// Serialized variant name.
    pub(crate) serialized_name: String,
    /// Deserialized variant name.
    pub(crate) deserialized_name: String,
    /// Whether this is the default variant.
    pub(crate) default: bool,
    /// Normalized variant fields.
    pub(crate) fields: Vec<FieldIr>,
}
