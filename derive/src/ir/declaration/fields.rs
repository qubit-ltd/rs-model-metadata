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

#[derive(Clone)]
pub(crate) enum FieldOccurrence {
    Identifier(IdentifierAssignmentIr),
    Indexed,
    Unique(super::UniqueIr),
    Reference(ReferenceIr),
    KeyPart(usize),
    Constraint(ConstraintIr),
    Selector(SelectorIr),
    Validator(ValidatorIr),
    Codec(CodecIr),
    Redact(RedactIr),
    Serde(SerdeIr),
    Opaque,
}

#[derive(Clone, Copy)]
pub(crate) enum IdentifierAssignmentIr {
    Application,
    Database,
}

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
