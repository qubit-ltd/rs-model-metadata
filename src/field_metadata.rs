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
use crate::metadata::CollectionOps;
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
    /// Exact-type read-only operations for supported collections.
    collection_ops: Option<CollectionOps>,
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
            location: Some(FieldLocation::new(
                owner,
                reflect.variant_index(),
                reflect.index(),
            )),
            reflect: Some(reflect),
            definition: None,
            symbolic_type: None,
            variant_inherited: false,
            attributes: &[],
            constraints: &[],
            validators: &[],
            serde: &SerdeFieldMetadata::DEFAULT,
            collection_ops: None,
        }
    }

    /// Creates the runtime overlay generated for a concrete reflected field.
    ///
    /// The overlay retains the reflection descriptor and concrete field
    /// identity while attaching declaration metadata collected by the derive
    /// macro. The attached Serde value contains the field's effective Serde
    /// behavior; this constructor does not reinterpret it.
    ///
    /// # Parameters
    ///
    /// * `owner` - The concrete Rust type identity that owns `reflect`.
    /// * `reflect` - The structural descriptor for the concrete field.
    /// * `attributes` - The field's source-ordered domain attributes.
    /// * `constraints` - The field's standard validation constraints.
    /// * `validators` - The field's custom validator declarations.
    /// * `serde` - The effective Serde behavior generated for the field.
    ///
    /// # Returns
    ///
    /// A concrete overlay with its reflection location and supplied semantic
    /// declarations. The source declaration location remains unknown until a
    /// caller associates it separately.
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
            location: Some(FieldLocation::new(
                owner,
                reflect.variant_index(),
                reflect.index(),
            )),
            reflect: Some(reflect),
            definition: None,
            symbolic_type: None,
            variant_inherited: false,
            attributes,
            constraints,
            validators,
            serde,
            collection_ops: None,
        }
    }

    /// Creates a symbolic semantic overlay for one generic declaration field.
    ///
    /// Unlike a runtime overlay, this value refers to the declaration's
    /// source descriptor and symbolic type without a concrete reflection
    /// descriptor or runtime field location. `variant_inherited` records
    /// whether visibility comes from the enclosing enum variant, preserving
    /// the distinction between the generic definition and a later
    /// instantiation.
    ///
    /// # Parameters
    ///
    /// * `definition` - The structural descriptor from the generic source
    ///   declaration.
    /// * `symbolic_type` - The unresolved type expression declared for this
    ///   field.
    /// * `variant_inherited` - Whether the field inherits its visibility from
    ///   the containing enum variant.
    /// * `attributes` - The field's source-ordered domain attributes.
    /// * `constraints` - The field's standard validation constraints.
    /// * `validators` - The field's custom validator declarations.
    /// * `serde` - The effective Serde behavior generated for the declaration.
    ///
    /// # Returns
    ///
    /// A generic definition overlay with no concrete field location or
    /// reflection descriptor. Callers may associate source coordinates
    /// separately.
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
            collection_ops: None,
        }
    }

    /// Associates source coordinates without changing reflection identity.
    ///
    /// # Parameters
    ///
    /// * `declaration` - The source occurrence and its available coordinates.
    ///
    /// # Returns
    ///
    /// A copy of this overlay with the supplied source occurrence attached.
    #[must_use]
    pub const fn with_declaration(mut self, declaration: DeclarationLocation) -> Self {
        self.declaration = declaration;
        self
    }

    /// Associates generated, read-only collection operations.
    ///
    /// # Parameters
    ///
    /// * `collection_ops` - The generated exact-type collection operations.
    ///
    /// # Returns
    ///
    /// A copy of this overlay with the operations available through
    /// [`Self::collection_ops`].
    #[must_use]
    pub const fn with_collection_ops(mut self, collection_ops: CollectionOps) -> Self {
        self.collection_ops = Some(collection_ops);
        self
    }

    /// Returns exact-type collection operations, if generated for this field.
    ///
    /// # Returns
    ///
    /// `Some` when collection operations were attached; otherwise `None`.
    #[must_use]
    #[inline]
    pub const fn collection_ops(&self) -> Option<&CollectionOps> {
        self.collection_ops.as_ref()
    }

    /// Returns this concrete field's process-local identity.
    ///
    /// Generic definition fields return `None` until instantiated. Copies of a
    /// concrete overlay retain the same identity.
    ///
    /// # Returns
    ///
    /// `Some` with the concrete field identity, or `None` for a generic
    /// definition field.
    #[must_use]
    #[inline]
    pub const fn location(&self) -> Option<FieldLocation> {
        self.location
    }

    /// Returns the field's source occurrence, which may have unknown
    /// coordinates for a manually constructed structural overlay.
    ///
    /// # Returns
    ///
    /// The source occurrence and any known declaration coordinates.
    #[must_use]
    #[inline]
    pub const fn declaration(&self) -> &DeclarationLocation {
        &self.declaration
    }

    /// Returns the concrete reflection field descriptor, or `None` for an
    /// uninstantiated generic definition field.
    ///
    /// # Returns
    ///
    /// `Some` for a concrete reflected field, or `None` for a generic
    /// definition field.
    #[must_use]
    #[inline]
    pub const fn reflect(&self) -> Option<&'static FieldDescriptor> {
        self.reflect
    }

    /// Returns the source-level field for a generic declaration overlay, or
    /// `None` for a concrete field, including a specialized generic field.
    ///
    /// # Returns
    ///
    /// `Some` for a generic declaration field, or `None` for a concrete
    /// runtime field.
    #[must_use]
    #[inline]
    pub const fn definition(&self) -> Option<&'static FieldDefinitionDescriptor> {
        self.definition
    }

    /// Returns the zero-based source field index, local to its struct or enum
    /// variant. Generic definition and specialization preserve this index.
    ///
    /// # Returns
    ///
    /// The zero-based index within the containing struct or enum variant.
    #[must_use]
    #[inline]
    pub const fn index(&self) -> usize {
        match (self.reflect, self.definition) {
            (Some(reflect), _) => reflect.index(),
            (_, Some(definition)) => definition.index(),
            _ => unreachable!(),
        }
    }

    /// Returns the field query name, or `None` for an unnamed tuple payload.
    /// This is independent of Serde serialization and deserialization names.
    ///
    /// # Returns
    ///
    /// `Some` with the query name, or `None` when the declaration is an
    /// unnamed tuple payload.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> Option<&'static str> {
        match (self.reflect, self.definition) {
            (Some(reflect), _) => reflect.query_name(),
            (_, Some(definition)) => definition.query_name(),
            _ => unreachable!(),
        }
    }

    /// Returns declared visibility for a struct field, or inherited variant
    /// visibility for an enum payload. This also works for symbolic fields.
    ///
    /// # Returns
    ///
    /// The declared field visibility or the inherited variant visibility.
    #[must_use]
    #[inline]
    pub const fn visibility(&self) -> FieldVisibility<'_> {
        match (self.reflect, self.definition) {
            (Some(reflect), _) => reflect.visibility(),
            (_, Some(_)) if self.variant_inherited => FieldVisibility::VariantInherited,
            (_, Some(definition)) => FieldVisibility::Declared(definition.visibility()),
            _ => unreachable!(),
        }
    }

    /// Returns the exact resolved, opaque, or symbolic field type reference.
    ///
    /// # Returns
    ///
    /// The type reference retained by this concrete or generic overlay.
    #[must_use]
    #[inline]
    pub fn type_ref(&self) -> &'static TypeRef {
        match (self.reflect, self.symbolic_type) {
            (Some(reflect), _) => reflect.field_type(),
            (_, Some(symbolic_type)) => symbolic_type,
            _ => unreachable!(),
        }
    }

    /// Returns the resolved field type descriptor, or `None` for opaque and
    /// symbolic type references.
    ///
    /// # Returns
    ///
    /// `Some` when the field type is resolved, or `None` when it is opaque or
    /// symbolic.
    #[must_use]
    #[inline]
    pub fn descriptor(&self) -> Option<&'static TypeDescriptor> {
        self.type_ref().as_resolved()
    }

    /// Returns all semantic occurrences in source order.
    ///
    /// # Returns
    ///
    /// The field attributes in declaration order.
    #[must_use]
    #[inline]
    pub const fn attributes(&self) -> &'static [FieldAttributeMetadata] {
        self.attributes
    }

    /// Returns the identifier declaration, when present.
    ///
    /// # Returns
    ///
    /// `Some` with the identifier metadata, or `None` when no identifier is
    /// declared.
    #[must_use]
    pub fn identifier(&self) -> Option<&'static IdentifierMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                FieldAttributeMetadata::Identifier(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns whether this field is the model identifier.
    ///
    /// # Returns
    ///
    /// `true` when the field has an identifier declaration.
    #[must_use]
    #[inline]
    pub fn is_identifier(&self) -> bool {
        self.identifier().is_some()
    }

    /// Returns every reason this field participates in an index.
    ///
    /// # Returns
    ///
    /// The combined indexing reasons, empty when the field is not indexed.
    #[must_use]
    pub fn indexing_reasons(&self) -> IndexingReasons {
        self.attributes.iter().fold(
            IndexingReasons::empty(),
            |result, attribute| match attribute {
                FieldAttributeMetadata::Indexed(value) => result | *value,
                _ => result,
            },
        )
    }

    /// Returns whether this field participates in any index.
    ///
    /// # Returns
    ///
    /// `true` when at least one indexing reason is present.
    #[must_use]
    pub fn is_indexed(&self) -> bool {
        !self.indexing_reasons().is_empty()
    }

    /// Returns the uniqueness declaration, when present.
    ///
    /// # Returns
    ///
    /// `Some` with the uniqueness metadata, or `None` when uniqueness is not
    /// declared.
    #[must_use]
    pub fn unique(&self) -> Option<&'static FieldUniqueMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                FieldAttributeMetadata::Unique(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns whether this field declares uniqueness.
    ///
    /// # Returns
    ///
    /// `true` when a uniqueness declaration is present.
    #[must_use]
    #[inline]
    pub fn is_unique(&self) -> bool {
        self.unique().is_some()
    }

    /// Returns the entity reference declaration, when present.
    ///
    /// # Returns
    ///
    /// `Some` with the reference metadata, or `None` when the field is not an
    /// entity reference.
    #[must_use]
    pub fn reference(&self) -> Option<&'static FieldReferenceMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                FieldAttributeMetadata::Reference(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns the ordered composite-key declaration, when present.
    ///
    /// # Returns
    ///
    /// `Some` with the key position metadata, or `None` when the field is not
    /// part of a composite key.
    #[must_use]
    pub fn key_part(&self) -> Option<&'static KeyPartMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                FieldAttributeMetadata::KeyPart(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns all standard field constraints.
    ///
    /// # Returns
    ///
    /// The field's validation constraints in declaration order.
    #[must_use]
    #[inline]
    pub const fn constraints(&self) -> &'static [ConstraintMetadata] {
        self.constraints
    }

    /// Returns the text constraint, when declared.
    ///
    /// # Returns
    ///
    /// `Some` with the text constraint, or `None` when none is declared.
    #[must_use]
    pub fn text_constraint(&self) -> Option<TextConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Text(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the decimal or money constraint, when declared.
    ///
    /// # Returns
    ///
    /// `Some` with the decimal constraint, or `None` when none is declared.
    #[must_use]
    pub fn decimal_constraint(&self) -> Option<DecimalConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Decimal(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the temporal constraint, when declared.
    ///
    /// # Returns
    ///
    /// `Some` with the temporal constraint, or `None` when none is declared.
    #[must_use]
    pub fn time_constraint(&self) -> Option<TimeConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Time(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the sequence constraint, when declared.
    ///
    /// # Returns
    ///
    /// `Some` with the sequence constraint, or `None` when none is declared.
    #[must_use]
    pub fn sequence_constraint(&self) -> Option<SequenceConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Sequence(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the map constraint, when declared.
    ///
    /// # Returns
    ///
    /// `Some` with the map constraint, or `None` when none is declared.
    #[must_use]
    pub fn map_constraint(&self) -> Option<MapConstraint> {
        self.constraints.iter().find_map(|value| match value {
            ConstraintMetadata::Map(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns validator declarations in source order.
    ///
    /// # Returns
    ///
    /// The custom validator declarations in source order.
    #[must_use]
    #[inline]
    pub const fn validators(&self) -> &'static [ValidatorMetadata] {
        self.validators
    }

    /// Returns the field codec declaration, when present.
    ///
    /// # Returns
    ///
    /// `Some` with the codec metadata, or `None` when no codec is declared.
    #[must_use]
    pub fn codec(&self) -> Option<&'static CodecMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                FieldAttributeMetadata::Codec(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns the field redaction declaration, when present.
    ///
    /// # Returns
    ///
    /// `Some` with the redaction metadata, or `None` when no redaction is
    /// declared.
    #[must_use]
    pub fn redact(&self) -> Option<&'static RedactMetadata> {
        self.attributes
            .iter()
            .find_map(|attribute| match attribute {
                FieldAttributeMetadata::Redact(value) => Some(*value),
                _ => None,
            })
    }

    /// Returns the effective Serde behavior.
    ///
    /// # Returns
    ///
    /// The Serde behavior attached to this field overlay.
    #[must_use]
    #[inline]
    pub const fn serde(&self) -> &'static SerdeFieldMetadata {
        self.serde
    }

    /// Returns whether the model declaration contains an explicit opaque
    /// marker. A bare [`Self::from_reflect`] overlay has no domain markers,
    /// even if its structural type reference is opaque.
    ///
    /// # Returns
    ///
    /// `true` only when an explicit opaque marker is attached to the field.
    #[must_use]
    pub fn is_opaque(&self) -> bool {
        self.attributes
            .iter()
            .any(|attribute| matches!(attribute, FieldAttributeMetadata::Opaque))
    }
}
