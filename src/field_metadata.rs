// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable metadata for a declared model field.

use crate::attribute::{
    AttributeMetadata,
    SensitiveMetadata,
    StrategyRef,
};
use crate::constraint::{
    DecimalConstraint,
    MapConstraint,
    SequenceConstraint,
    TemporalConstraint,
    TextConstraint,
};
use crate::relation::{
    LookupRelationMetadata,
    ReferenceMetadata,
};
use crate::type_shape::{
    TypeCapabilities,
    TypeRef,
    TypeShape,
};

/// Immutable metadata for a declared model field.
#[derive(Clone, Copy, Debug)]
pub struct FieldMetadata {
    ordinal: usize,
    name: &'static str,
    rust_type_name: &'static str,
    field_type: TypeRef,
    attributes: &'static [AttributeMetadata],
}

impl FieldMetadata {
    /// Creates field metadata from declaration details and static attributes.
    #[must_use]
    pub const fn new(
        ordinal: usize,
        name: &'static str,
        rust_type_name: &'static str,
        field_type: TypeRef,
        attributes: &'static [AttributeMetadata],
    ) -> Self {
        validate_field_attributes(attributes, field_type.capabilities());
        Self {
            ordinal,
            name,
            rust_type_name,
            field_type,
            attributes,
        }
    }

    /// Returns the field's declaration ordinal.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self.ordinal
    }

    /// Returns the normalized Rust field name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the declared Rust type spelling recorded by the metadata
    /// producer.
    #[must_use]
    pub const fn rust_type_name(self) -> &'static str {
        self.rust_type_name
    }

    /// Returns the recursive structural metadata for the field type.
    #[must_use]
    pub const fn field_type(self) -> TypeRef {
        self.field_type
    }

    /// Returns whether only the field's outermost type layer is `Option`.
    #[must_use]
    pub fn is_nullable(self) -> bool {
        matches!(self.field_type.shape(), TypeShape::Optional(_))
    }

    /// Returns all field-level metadata attributes.
    #[must_use]
    pub const fn attributes(self) -> &'static [AttributeMetadata] {
        self.attributes
    }

    /// Returns the field's text constraints, if present.
    #[must_use]
    pub fn text_constraint(self) -> Option<TextConstraint> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Text(constraint) => Some(*constraint),
                _ => None,
            })
    }

    /// Returns the field's direct reference metadata, if present.
    #[must_use]
    pub fn reference(self) -> Option<ReferenceMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Reference(reference) => Some(*reference),
                _ => None,
            })
    }

    /// Returns the field's sequence constraints, if present.
    #[must_use]
    pub fn sequence_constraint(self) -> Option<SequenceConstraint> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Sequence(constraint) => Some(*constraint),
                _ => None,
            })
    }

    /// Returns the field's map constraints, if present.
    #[must_use]
    pub fn map_constraint(self) -> Option<MapConstraint> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Map(constraint) => Some(*constraint),
                _ => None,
            })
    }

    /// Returns the field's temporal constraints, if present.
    #[must_use]
    pub fn temporal_constraint(self) -> Option<TemporalConstraint> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Temporal(constraint) => Some(*constraint),
                _ => None,
            })
    }

    /// Returns the field's decimal constraints, if present.
    #[must_use]
    pub fn decimal_constraint(self) -> Option<DecimalConstraint> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Decimal(constraint) => Some(*constraint),
                _ => None,
            })
    }

    /// Returns the field's lookup relation, if present.
    #[must_use]
    pub fn lookup_relation(self) -> Option<LookupRelationMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::LookupRelation(relation) => Some(*relation),
                _ => None,
            })
    }

    /// Returns the field's codec strategy reference, if present.
    #[must_use]
    pub fn codec(self) -> Option<StrategyRef> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Codec(strategy) => Some(*strategy),
                _ => None,
            })
    }

    /// Returns the field's generator strategy reference, if present.
    #[must_use]
    pub fn generator(self) -> Option<StrategyRef> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Generator(strategy) => Some(*strategy),
                _ => None,
            })
    }

    /// Returns the field's sensitive-data policy, if present.
    #[must_use]
    pub fn sensitive(self) -> Option<SensitiveMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeMetadata::Sensitive(metadata) => Some(*metadata),
                _ => None,
            })
    }
}

/// Validates the attribute scopes and type capabilities stored on one field.
const fn validate_field_attributes(
    attributes: &'static [AttributeMetadata],
    capabilities: TypeCapabilities,
) {
    let mut index = 0;
    while index < attributes.len() {
        match attributes[index] {
            AttributeMetadata::Text(_) => assert!(
                has_capability(capabilities, TypeCapabilities::TEXT),
                "text attributes require a text-capable field"
            ),
            AttributeMetadata::Sequence(constraint) => {
                assert!(
                    has_capability(capabilities, TypeCapabilities::SEQUENCE),
                    "sequence attributes require a sequence-capable field"
                );
                assert!(
                    !has_capability(capabilities, TypeCapabilities::ARRAY)
                        || (constraint.min_items().is_none()
                            && constraint.max_items().is_none()),
                    "array length is fixed by its type"
                );
            }
            AttributeMetadata::Map(_) => assert!(
                has_capability(capabilities, TypeCapabilities::MAP),
                "map attributes require a map-capable field"
            ),
            AttributeMetadata::Temporal(_) => assert!(
                has_capability(capabilities, TypeCapabilities::TEMPORAL),
                "temporal attributes require a temporal-capable field"
            ),
            AttributeMetadata::Decimal(_) => assert!(
                has_capability(capabilities, TypeCapabilities::DECIMAL),
                "decimal attributes require a decimal-capable field"
            ),
            AttributeMetadata::PrimaryKey(_)
            | AttributeMetadata::Unique(_)
            | AttributeMetadata::Index(_)
            | AttributeMetadata::Key(_)
            | AttributeMetadata::Ownership(_) => {
                panic!("primary-key attributes are only valid at model scope")
            }
            AttributeMetadata::Reference(_)
            | AttributeMetadata::LookupRelation(_)
            | AttributeMetadata::Codec(_)
            | AttributeMetadata::Generator(_)
            | AttributeMetadata::Sensitive(_) => {}
        }
        index += 1;
    }
}

/// Returns whether `capabilities` contains `required`.
const fn has_capability(
    capabilities: TypeCapabilities,
    required: TypeCapabilities,
) -> bool {
    capabilities.bits() & required.bits() == required.bits()
}
