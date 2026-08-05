// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Strongly typed type-level and field-level metadata attributes.

use crate::constraint::{
    DecimalConstraint,
    MapConstraint,
    SequenceConstraint,
    TemporalConstraint,
    TextConstraint,
};
use crate::relation::{
    LookupRelationMetadata,
    OwnershipMetadata,
    ReferenceMetadata,
};

/// A strongly typed metadata attribute.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum AttributeMetadata {
    /// Text constraints for a field.
    Text(TextConstraint),
    /// Sequence constraints for a field.
    Sequence(SequenceConstraint),
    /// Map constraints for a field.
    Map(MapConstraint),
    /// Temporal constraints for a field.
    Temporal(TemporalConstraint),
    /// Decimal constraints for a field.
    Decimal(DecimalConstraint),
    /// A model-level primary-key definition.
    PrimaryKey(PrimaryKeyMetadata),
    /// A model-level unique-constraint definition.
    Unique(UniqueMetadata),
    /// A model-level index definition.
    Index(IndexMetadata),
    /// A model-level logical-key definition.
    Key(KeyMetadata),
    /// A direct reference to another model.
    Reference(ReferenceMetadata),
    /// A lookup relation to another model.
    LookupRelation(LookupRelationMetadata),
    /// An ownership relation to another model.
    Ownership(OwnershipMetadata),
    /// A codec strategy name.
    Codec(StrategyRef),
    /// A generator strategy name.
    Generator(StrategyRef),
    /// Sensitive-data handling metadata.
    Sensitive(SensitiveMetadata),
}

impl AttributeMetadata {
    /// Returns the discriminant used by generic attribute queries.
    #[must_use]
    pub const fn kind(self) -> AttributeKind {
        match self {
            Self::Text(_) => AttributeKind::Text,
            Self::Sequence(_) => AttributeKind::Sequence,
            Self::Map(_) => AttributeKind::Map,
            Self::Temporal(_) => AttributeKind::Temporal,
            Self::Decimal(_) => AttributeKind::Decimal,
            Self::PrimaryKey(_) => AttributeKind::PrimaryKey,
            Self::Unique(_) => AttributeKind::Unique,
            Self::Index(_) => AttributeKind::Index,
            Self::Key(_) => AttributeKind::Key,
            Self::Reference(_) => AttributeKind::Reference,
            Self::LookupRelation(_) => AttributeKind::LookupRelation,
            Self::Ownership(_) => AttributeKind::Ownership,
            Self::Codec(_) => AttributeKind::Codec,
            Self::Generator(_) => AttributeKind::Generator,
            Self::Sensitive(_) => AttributeKind::Sensitive,
        }
    }
}

/// A discriminant for generic attribute queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeKind {
    /// Text constraints.
    Text,
    /// Sequence constraints.
    Sequence,
    /// Map constraints.
    Map,
    /// Temporal constraints.
    Temporal,
    /// Decimal constraints.
    Decimal,
    /// Primary-key definitions.
    PrimaryKey,
    /// Unique-constraint definitions.
    Unique,
    /// Index definitions.
    Index,
    /// Logical-key definitions.
    Key,
    /// Direct references.
    Reference,
    /// Lookup relations.
    LookupRelation,
    /// Ownership relations.
    Ownership,
    /// Codec strategies.
    Codec,
    /// Generator strategies.
    Generator,
    /// Sensitive-data metadata.
    Sensitive,
}

/// A model-level primary-key definition.
#[derive(Clone, Copy, Debug)]
pub struct PrimaryKeyMetadata {
    fields: &'static [PrimaryKeyFieldMetadata],
}

impl PrimaryKeyMetadata {
    /// Creates a primary-key definition from its ordered fields.
    ///
    /// # Panics
    ///
    /// Panics when `fields` is empty.
    #[must_use]
    pub const fn new(fields: &'static [PrimaryKeyFieldMetadata]) -> Self {
        assert!(
            !fields.is_empty(),
            "primary key requires at least one field"
        );
        validate_primary_key_fields(fields);
        Self { fields }
    }

    /// Returns the primary-key fields in declaration order.
    #[must_use]
    pub const fn fields(self) -> &'static [PrimaryKeyFieldMetadata] {
        self.fields
    }

    /// Returns whether this primary key contains a field with `name`.
    #[must_use]
    pub fn contains(self, name: &str) -> bool {
        self.fields.iter().any(|field| field.name == name)
    }
}

/// A field participating in a primary key.
#[derive(Clone, Copy, Debug)]
pub struct PrimaryKeyFieldMetadata {
    name: &'static str,
    generated: bool,
}

impl PrimaryKeyFieldMetadata {
    /// Creates primary-key field metadata for a normalized field name.
    #[must_use]
    pub const fn new(name: &'static str, generated: bool) -> Self {
        Self { name, generated }
    }

    /// Returns the normalized field name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns whether this key field is generated by its persistence boundary.
    #[must_use]
    pub const fn is_generated(self) -> bool {
        self.generated
    }
}

/// A model-level unique-constraint definition.
#[derive(Clone, Copy, Debug)]
pub struct UniqueMetadata {
    name: Option<&'static str>,
    fields: &'static [UniqueFieldMetadata],
}

impl UniqueMetadata {
    /// Creates a unique constraint with an optional logical name and ordered
    /// fields.
    ///
    /// # Panics
    ///
    /// Panics when `fields` is empty.
    #[must_use]
    pub const fn new(
        name: Option<&'static str>,
        fields: &'static [UniqueFieldMetadata],
    ) -> Self {
        assert!(
            !fields.is_empty(),
            "unique constraint requires at least one field"
        );
        validate_unique_fields(fields);
        Self { name, fields }
    }

    /// Returns the optional logical constraint name.
    #[must_use]
    pub const fn name(self) -> Option<&'static str> {
        self.name
    }

    /// Returns the unique fields in declaration order.
    #[must_use]
    pub const fn fields(self) -> &'static [UniqueFieldMetadata] {
        self.fields
    }

    /// Returns whether this unique constraint contains a field with `name`.
    #[must_use]
    pub fn contains(self, name: &str) -> bool {
        self.fields.iter().any(|field| field.name == name)
    }

    /// Returns the comparison semantics for `name`, or `None` when it is
    /// absent.
    #[must_use]
    pub fn comparison_of(self, name: &str) -> Option<UniqueComparison> {
        self.fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.comparison)
    }
}

/// A field participating in a unique constraint.
#[derive(Clone, Copy, Debug)]
pub struct UniqueFieldMetadata {
    name: &'static str,
    comparison: UniqueComparison,
}

impl UniqueFieldMetadata {
    /// Creates unique-field metadata for a normalized name and comparison
    /// semantics.
    #[must_use]
    pub const fn new(name: &'static str, comparison: UniqueComparison) -> Self {
        Self { name, comparison }
    }

    /// Returns the normalized field name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the comparison semantics for this field.
    #[must_use]
    pub const fn comparison(self) -> UniqueComparison {
        self.comparison
    }
}

/// Comparison semantics for a field within a unique constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniqueComparison {
    /// Compare values exactly.
    Exact,
    /// Compare text values without case sensitivity.
    IgnoreCase,
}

/// A model-level index definition.
#[derive(Clone, Copy, Debug)]
pub struct IndexMetadata {
    name: Option<&'static str>,
    fields: &'static [&'static str],
}

impl IndexMetadata {
    /// Creates an index definition with an optional logical name and ordered
    /// fields.
    ///
    /// # Panics
    ///
    /// Panics when `fields` is empty.
    #[must_use]
    pub const fn new(
        name: Option<&'static str>,
        fields: &'static [&'static str],
    ) -> Self {
        assert!(!fields.is_empty(), "index requires at least one field");
        validate_named_fields(fields);
        Self { name, fields }
    }

    /// Returns the optional logical index name.
    #[must_use]
    pub const fn name(self) -> Option<&'static str> {
        self.name
    }

    /// Returns indexed field names in declaration order.
    #[must_use]
    pub const fn fields(self) -> &'static [&'static str] {
        self.fields
    }

    /// Returns whether this index contains a field with `name`.
    #[must_use]
    pub fn contains(self, name: &str) -> bool {
        self.fields.contains(&name)
    }
}

/// A model-level logical-key definition.
#[derive(Clone, Copy, Debug)]
pub struct KeyMetadata {
    name: Option<&'static str>,
    fields: &'static [&'static str],
}

impl KeyMetadata {
    /// Creates a logical-key definition with an optional logical name and
    /// ordered fields.
    ///
    /// # Panics
    ///
    /// Panics when `fields` is empty.
    #[must_use]
    pub const fn new(
        name: Option<&'static str>,
        fields: &'static [&'static str],
    ) -> Self {
        assert!(
            !fields.is_empty(),
            "logical key requires at least one field"
        );
        validate_named_fields(fields);
        Self { name, fields }
    }

    /// Returns the optional logical key name.
    #[must_use]
    pub const fn name(self) -> Option<&'static str> {
        self.name
    }

    /// Returns logical-key field names in declaration order.
    #[must_use]
    pub const fn fields(self) -> &'static [&'static str] {
        self.fields
    }

    /// Returns whether this logical key contains a field with `name`.
    #[must_use]
    pub fn contains(self, name: &str) -> bool {
        self.fields.contains(&name)
    }
}

/// A static identifier for an external strategy implemented by another crate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrategyRef {
    name: &'static str,
}

impl StrategyRef {
    /// Creates a strategy reference with a stable logical name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self { name }
    }

    /// Returns the stable logical strategy name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }
}

/// Metadata describing how a sensitive value should be handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SensitiveMetadata {
    handling: SensitiveHandling,
}

impl SensitiveMetadata {
    /// Creates sensitive-data metadata with the supplied handling policy.
    #[must_use]
    pub const fn new(handling: SensitiveHandling) -> Self {
        Self { handling }
    }

    /// Returns the handling policy for this sensitive value.
    #[must_use]
    pub const fn handling(self) -> SensitiveHandling {
        self.handling
    }
}

/// A policy for handling sensitive values in downstream consumers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensitiveHandling {
    /// Redact the complete value.
    Redact,
    /// Mask part of the value while retaining contextual information.
    Mask,
}

/// Validates non-empty, distinct primary-key field names.
const fn validate_primary_key_fields(
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
const fn validate_unique_fields(fields: &'static [UniqueFieldMetadata]) {
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
const fn validate_named_fields(names: &'static [&'static str]) {
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

/// Compares two static strings without allocating.
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
