// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable domain semantics over one reflected field descriptor.

use qubit_reflect::FieldDescriptor;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::access::FieldVisibility;
use qubit_reflect::descriptor::TypeRef;

use crate::CodecMetadata;
use crate::ConstraintMetadata;
use crate::DecimalConstraint;
use crate::FieldAttributeMetadata;
use crate::FieldReferenceMetadata;
use crate::FieldUniqueMetadata;
use crate::IdentifierMetadata;
use crate::IndexingReasons;
use crate::KeyPartMetadata;
use crate::MapConstraint;
use crate::RedactMetadata;
use crate::SequenceConstraint;
use crate::SerdeFieldMetadata;
use crate::TextConstraint;
use crate::TimeConstraint;
use crate::ValidatorMetadata;

/// Model semantics attached to one reflection-owned structural field.
#[derive(Clone, Copy, Debug)]
pub struct FieldMetadata {
    /// The reflection descriptor that defines the structural field.
    reflect: &'static FieldDescriptor,
    /// Source-order declarations attached to the field.
    attributes: &'static [FieldAttributeMetadata],
    /// Standard validation constraints attached to the field.
    constraints: &'static [ConstraintMetadata],
    /// Custom validator declarations attached to the field.
    validators: &'static [ValidatorMetadata],
    /// The effective Serde behavior for the field.
    serde: &'static SerdeFieldMetadata,
}

impl FieldMetadata {
    /// Creates an overlay with no domain-specific declarations.
    #[must_use]
    pub const fn from_reflect(reflect: &'static FieldDescriptor) -> Self {
        Self {
            reflect,
            attributes: &[],
            constraints: &[],
            validators: &[],
            serde: &SerdeFieldMetadata::DEFAULT,
        }
    }

    /// Creates a complete generated overlay.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn with_semantics(
        reflect: &'static FieldDescriptor,
        attributes: &'static [FieldAttributeMetadata],
        constraints: &'static [ConstraintMetadata],
        validators: &'static [ValidatorMetadata],
        serde: &'static SerdeFieldMetadata,
    ) -> Self {
        Self {
            reflect,
            attributes,
            constraints,
            validators,
            serde,
        }
    }

    /// Returns the underlying reflection field descriptor.
    #[must_use]
    pub const fn reflect(&self) -> &'static FieldDescriptor {
        self.reflect
    }

    /// Returns the source field index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.reflect.index()
    }

    /// Returns the field query name, when it has one.
    #[must_use]
    pub const fn name(&self) -> Option<&'static str> {
        self.reflect.query_name()
    }

    /// Returns the reflected field visibility.
    #[must_use]
    pub const fn visibility(&self) -> FieldVisibility<'static> {
        self.reflect.visibility()
    }

    /// Returns the exact resolved, opaque, or symbolic field type reference.
    #[must_use]
    pub fn type_ref(&self) -> &'static TypeRef {
        self.reflect.field_type()
    }

    /// Returns the resolved field type descriptor, when available.
    #[must_use]
    pub fn descriptor(&self) -> Option<&'static TypeDescriptor> {
        self.type_ref().as_resolved()
    }

    /// Returns all semantic occurrences in source order.
    #[must_use]
    pub const fn attributes(&self) -> &'static [FieldAttributeMetadata] {
        self.attributes
    }

    /// Returns the identifier declaration, when present.
    #[must_use]
    pub fn identifier(&self) -> Option<&'static IdentifierMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
            FieldAttributeMetadata::Identifier(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns whether this field is the model identifier.
    #[must_use]
    pub fn is_identifier(&self) -> bool {
        self.identifier().is_some()
    }

    /// Returns every reason this field participates in an index.
    #[must_use]
    pub fn indexing_reasons(&self) -> IndexingReasons {
        self.attributes
            .iter()
            .fold(IndexingReasons::empty(), |result, attribute| match attribute {
                FieldAttributeMetadata::Indexed(value) => result | *value,
                _ => result,
            })
    }

    /// Returns whether this field participates in any index.
    #[must_use]
    pub fn is_indexed(&self) -> bool {
        !self.indexing_reasons().is_empty()
    }

    /// Returns the uniqueness declaration, when present.
    #[must_use]
    pub fn unique(&self) -> Option<&'static FieldUniqueMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
            FieldAttributeMetadata::Unique(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns whether this field declares uniqueness.
    #[must_use]
    pub fn is_unique(&self) -> bool {
        self.unique().is_some()
    }

    /// Returns the entity reference declaration, when present.
    #[must_use]
    pub fn reference(&self) -> Option<&'static FieldReferenceMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
            FieldAttributeMetadata::Reference(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the ordered composite-key declaration, when present.
    #[must_use]
    pub fn key_part(&self) -> Option<&'static KeyPartMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
            FieldAttributeMetadata::KeyPart(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns all standard field constraints.
    #[must_use]
    pub const fn constraints(&self) -> &'static [ConstraintMetadata] {
        self.constraints
    }

    /// Returns the text constraint, when declared.
    #[must_use]
    pub fn text_constraint(&self) -> Option<TextConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Text(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the decimal or money constraint, when declared.
    #[must_use]
    pub fn decimal_constraint(&self) -> Option<DecimalConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Decimal(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the temporal constraint, when declared.
    #[must_use]
    pub fn time_constraint(&self) -> Option<TimeConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Time(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the sequence constraint, when declared.
    #[must_use]
    pub fn sequence_constraint(&self) -> Option<SequenceConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Sequence(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the map constraint, when declared.
    #[must_use]
    pub fn map_constraint(&self) -> Option<MapConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Map(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns validator declarations in source order.
    #[must_use]
    pub const fn validators(&self) -> &'static [ValidatorMetadata] {
        self.validators
    }

    /// Returns the field codec declaration, when present.
    #[must_use]
    pub fn codec(&self) -> Option<&'static CodecMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
            FieldAttributeMetadata::Codec(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the field redaction declaration, when present.
    #[must_use]
    pub fn redact(&self) -> Option<&'static RedactMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
            FieldAttributeMetadata::Redact(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the effective Serde behavior.
    #[must_use]
    pub const fn serde(&self) -> &'static SerdeFieldMetadata {
        self.serde
    }

    /// Returns whether reflection treats the field type as opaque.
    #[must_use]
    pub fn is_opaque(&self) -> bool {
        self.attributes
            .iter()
            .any(|attribute| matches!(attribute, FieldAttributeMetadata::Opaque))
    }
}
