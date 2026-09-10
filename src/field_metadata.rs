// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable domain semantics over one reflected field descriptor.

use std::any::TypeId;

use qubit_reflect::FieldDefinitionDescriptor;
use qubit_reflect::FieldDescriptor;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::access::FieldVisibility;
use qubit_reflect::descriptor::TypeRef;

use crate::metadata::CodecMetadata;
use crate::metadata::ConstraintMetadata;
use crate::metadata::DecimalConstraint;
use crate::metadata::DeclarationLocation;
use crate::metadata::FieldAttributeMetadata;
use crate::metadata::FieldLocation;
use crate::metadata::FieldReferenceMetadata;
use crate::metadata::FieldUniqueMetadata;
use crate::metadata::IdentifierMetadata;
use crate::metadata::IndexingReasons;
use crate::metadata::KeyPartMetadata;
use crate::metadata::MapConstraint;
use crate::metadata::RedactMetadata;
use crate::metadata::SequenceConstraint;
use crate::metadata::SerdeFieldMetadata;
use crate::metadata::TextConstraint;
use crate::metadata::TimeConstraint;
use crate::metadata::ValidatorMetadata;

/// Model semantics attached to one reflection-owned structural field.
///
/// Concrete fields have a [`FieldLocation`] that remains unchanged when this
/// value is copied. Uninstantiated generic definition fields have no concrete
/// location. Source coordinates remain available separately through
/// [`Self::declaration`], including unnamed Enum payload positions.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Model]
/// struct Address { city: String }
/// # fn main() {
/// let metadata = TypeMetadata::of::<Address>();
/// let field = metadata.field("city").expect("city field");
/// let copied = *field;
/// let location = copied.location().expect("concrete field");
/// assert_eq!(field.location(), Some(location));
/// assert_eq!(location.owner(), metadata.type_id());
/// assert_eq!(location.index(), 0);
/// assert_eq!(location.variant(), None);
/// # }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct FieldMetadata {
    /// Concrete declaration identity; absent for generic definitions.
    location: Option<FieldLocation>,
    /// Source position and owning object of this declaration.
    declaration: DeclarationLocation,
    /// The concrete reflection descriptor, when this is a runtime overlay.
    reflect: Option<&'static FieldDescriptor>,
    /// The source-level descriptor, when this is a generic declaration
    /// overlay.
    definition: Option<&'static FieldDefinitionDescriptor>,
    /// The symbolic type reference used by a generic declaration overlay.
    symbolic_type: Option<&'static TypeRef>,
    /// Whether a definition field inherits an enum variant's visibility.
    variant_inherited: bool,
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
    ///
    /// # Parameters
    ///
    /// * `owner` - The concrete declaring Rust type identity. It must match
    ///   `reflect`'s owner; the checked generated-code protocol validates this
    ///   relationship when assembling a complete model.
    /// * `reflect` - The permanently retained structural field descriptor.
    ///
    /// # Returns
    ///
    /// An overlay with a concrete location, unknown source coordinates, empty
    /// domain declarations, and default Serde metadata. This constructor does
    /// not infer domain markers from structural reflection facts.
    #[must_use]
    pub const fn from_reflect(owner: TypeId, reflect: &'static FieldDescriptor) -> Self {
        Self {
            declaration: DeclarationLocation::unknown(),
            location: Some(FieldLocation::new(owner, reflect.variant_index(), reflect.index())),
            reflect: Some(reflect),
            definition: None,
            symbolic_type: None,
            variant_inherited: false,
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
        owner: TypeId,
        reflect: &'static FieldDescriptor,
        attributes: &'static [FieldAttributeMetadata],
        constraints: &'static [ConstraintMetadata],
        validators: &'static [ValidatorMetadata],
        serde: &'static SerdeFieldMetadata,
    ) -> Self {
        Self {
            declaration: DeclarationLocation::unknown(),
            location: Some(FieldLocation::new(owner, reflect.variant_index(), reflect.index())),
            reflect: Some(reflect),
            definition: None,
            symbolic_type: None,
            variant_inherited: false,
            attributes,
            constraints,
            validators,
            serde,
        }
    }

    /// Creates a semantic overlay for one generic declaration field.
    #[doc(hidden)]
    #[must_use]
    pub(crate) const fn with_definition_semantics(
        definition: &'static FieldDefinitionDescriptor,
        symbolic_type: &'static TypeRef,
        variant_inherited: bool,
        attributes: &'static [FieldAttributeMetadata],
        constraints: &'static [ConstraintMetadata],
        validators: &'static [ValidatorMetadata],
        serde: &'static SerdeFieldMetadata,
    ) -> Self {
        Self {
            declaration: DeclarationLocation::unknown(),
            location: None,
            reflect: None,
            definition: Some(definition),
            symbolic_type: Some(symbolic_type),
            variant_inherited,
            attributes,
            constraints,
            validators,
            serde,
        }
    }

    /// Associates source coordinates without changing reflection identity.
    #[must_use]
    pub const fn with_declaration(mut self, declaration: DeclarationLocation) -> Self {
        self.declaration = declaration;
        self
    }

    /// Returns this concrete field's process-local identity.
    ///
    /// Generic definition fields return `None` until instantiated. Copies of a
    /// concrete overlay retain the same identity.
    #[must_use]
    #[inline(always)]
    pub const fn location(&self) -> Option<FieldLocation> {
        self.location
    }

    /// Returns the field's source occurrence, which may have unknown
    /// coordinates for a manually constructed structural overlay.
    #[must_use]
    #[inline(always)]
    pub const fn declaration(&self) -> &DeclarationLocation {
        &self.declaration
    }

    /// Returns the concrete reflection field descriptor, or `None` for an
    /// uninstantiated generic definition field.
    #[must_use]
    #[inline(always)]
    pub const fn reflect(&self) -> Option<&'static FieldDescriptor> {
        self.reflect
    }

    /// Returns the source-level field for a generic declaration overlay, or
    /// `None` for a concrete field, including a specialized generic field.
    #[must_use]
    #[inline(always)]
    pub const fn definition(&self) -> Option<&'static FieldDefinitionDescriptor> {
        self.definition
    }

    /// Returns the zero-based source field index, local to its struct or enum
    /// variant. Generic definition and specialization preserve this index.
    #[must_use]
    #[inline(always)]
    pub const fn index(&self) -> usize {
        match (self.reflect, self.definition) {
            (Some(reflect), _) => reflect.index(),
            (_, Some(definition)) => definition.index(),
            _ => unreachable!(),
        }
    }

    /// Returns the field query name, or `None` for an unnamed tuple payload.
    /// This is independent of Serde serialization and deserialization names.
    #[must_use]
    #[inline(always)]
    pub const fn name(&self) -> Option<&'static str> {
        match (self.reflect, self.definition) {
            (Some(reflect), _) => reflect.query_name(),
            (_, Some(definition)) => definition.query_name(),
            _ => unreachable!(),
        }
    }

    /// Returns declared visibility for a struct field, or inherited variant
    /// visibility for an enum payload. This also works for symbolic fields.
    #[must_use]
    #[inline(always)]
    pub const fn visibility(&self) -> FieldVisibility<'_> {
        match (self.reflect, self.definition) {
            (Some(reflect), _) => reflect.visibility(),
            (_, Some(_)) if self.variant_inherited => FieldVisibility::VariantInherited,
            (_, Some(definition)) => FieldVisibility::Declared(definition.visibility()),
            _ => unreachable!(),
        }
    }

    /// Returns the exact resolved, opaque, or symbolic field type reference.
    #[must_use]
    #[inline(always)]
    pub fn type_ref(&self) -> &'static TypeRef {
        match (self.reflect, self.symbolic_type) {
            (Some(reflect), _) => reflect.field_type(),
            (_, Some(symbolic_type)) => symbolic_type,
            _ => unreachable!(),
        }
    }

    /// Returns the resolved field type descriptor, or `None` for opaque and
    /// symbolic type references.
    #[must_use]
    #[inline(always)]
    pub fn descriptor(&self) -> Option<&'static TypeDescriptor> {
        self.type_ref().as_resolved()
    }

    /// Returns all semantic occurrences in source order.
    #[must_use]
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
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
    #[inline(always)]
    pub const fn serde(&self) -> &'static SerdeFieldMetadata {
        self.serde
    }

    /// Returns whether the model declaration contains an explicit opaque
    /// marker. A bare [`Self::from_reflect`] overlay has no domain markers,
    /// even if its structural type reference is opaque.
    #[must_use]
    pub fn is_opaque(&self) -> bool {
        self.attributes
            .iter()
            .any(|attribute| matches!(attribute, FieldAttributeMetadata::Opaque))
    }
}
