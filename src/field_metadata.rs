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
    /// The field's zero-based declaration position.
    ordinal: usize,
    /// The normalized Rust field name.
    name: &'static str,
    /// The Rust type spelling recorded by the metadata producer.
    rust_type_name: &'static str,
    /// The recursive structural metadata for the field type.
    field_type: TypeRef,
    /// The field-level attributes in declaration order.
    attributes: &'static [AttributeMetadata],
}

impl FieldMetadata {
    /// Creates field metadata from declaration details and static attributes.
    ///
    /// # Parameters
    ///
    /// - `ordinal`: The zero-based declaration position.
    /// - `name`: The normalized Rust field name.
    /// - `rust_type_name`: The Rust type spelling recorded by the metadata
    ///   producer.
    /// - `field_type`: The recursive structural metadata for the field type.
    /// - `attributes`: The field-level attributes in declaration order.
    ///
    /// # Returns
    ///
    /// The constructed field metadata.
    ///
    /// # Panics
    ///
    /// Panics when an attribute is incompatible with the field type or is
    /// valid only at model scope.
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
    ///
    /// # Returns
    ///
    /// The field's zero-based declaration position.
    #[must_use]
    #[inline(always)]
    pub const fn ordinal(self) -> usize {
        self.ordinal
    }

    /// Returns the normalized Rust field name.
    ///
    /// # Returns
    ///
    /// The field name recorded in the metadata.
    #[must_use]
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Returns the declared Rust type spelling recorded by the metadata
    /// producer.
    ///
    /// # Returns
    ///
    /// The Rust type spelling recorded by the metadata producer.
    #[must_use]
    #[inline(always)]
    pub const fn rust_type_name(self) -> &'static str {
        self.rust_type_name
    }

    /// Returns the recursive structural metadata for the field type.
    ///
    /// # Returns
    ///
    /// The field's recursive type metadata.
    #[inline(always)]
    pub const fn field_type(self) -> TypeRef {
        self.field_type
    }

    /// Returns whether only the field's outermost type layer is `Option`.
    ///
    /// # Returns
    ///
    /// `true` when the outermost type layer is `Option`; otherwise `false`.
    #[must_use]
    #[inline]
    pub fn is_nullable(self) -> bool {
        matches!(self.field_type.shape(), TypeShape::Optional(_))
    }

    /// Returns all field-level metadata attributes.
    ///
    /// # Returns
    ///
    /// The field-level attributes in declaration order.
    #[must_use]
    #[inline(always)]
    pub const fn attributes(self) -> &'static [AttributeMetadata] {
        self.attributes
    }

    /// Returns the field's text constraints, if present.
    ///
    /// # Returns
    ///
    /// `Some` with the text constraint when one is present; otherwise `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the reference metadata when one is present; otherwise
    /// `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the sequence constraint when one is present; otherwise
    /// `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the map constraint when one is present; otherwise `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the temporal constraint when one is present; otherwise
    /// `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the decimal constraint when one is present; otherwise
    /// `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the lookup relation when one is present; otherwise `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the codec strategy when one is present; otherwise `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the generator strategy when one is present; otherwise
    /// `None`.
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
    ///
    /// # Returns
    ///
    /// `Some` with the sensitive-data policy when one is present; otherwise
    /// `None`.
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
///
/// # Parameters
///
/// - `attributes`: The field-level attributes to validate.
/// - `capabilities`: The capabilities exposed by the field's type.
///
/// # Panics
///
/// Panics when an attribute requires an unsupported capability, specifies a
/// length for an array with a fixed type-level length, or is valid only at
/// model scope.
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
///
/// # Parameters
///
/// - `capabilities`: The capabilities exposed by a field type.
/// - `required`: The capability mask to check.
///
/// # Returns
///
/// `true` when every bit in `required` is present in `capabilities`.
#[inline]
const fn has_capability(
    capabilities: TypeCapabilities,
    required: TypeCapabilities,
) -> bool {
    capabilities.bits() & required.bits() == required.bits()
}
