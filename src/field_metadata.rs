// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable domain semantics over a reflected field's static source facts.

use std::any::TypeId;
use std::fmt;

use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::__private::codegen_v4::expression::type_expression_in;
use qubit_reflect::FieldDescriptor;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::descriptor::Visibility;

use self::internal::contains_self_type;
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
use crate::property::OptionalOps;

// Inspects retained source type expressions used by this field overlay.
mod internal;

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
#[derive(Clone, Copy)]
pub struct FieldMetadata {
    /// Concrete declaration identity; absent for generic definitions.
    location: Option<FieldLocation>,
    /// Source position and owning object of this declaration.
    declaration: DeclarationLocation,
    /// Concrete Rust type identity used to resolve the field in a context.
    owner_type_id: Option<TypeId>,
    /// Generic definition path that owns a symbolic field.
    definition_path: Option<&'static str>,
    /// Source field index within its struct or enum variant.
    source_index: usize,
    /// Source enum variant index, if applicable.
    source_variant: Option<usize>,
    /// Source field name; tuple fields have no name.
    source_name: Option<&'static str>,
    /// Visibility recorded from source syntax.
    source_visibility: Visibility,
    /// Static source type-expression facts.
    expression: Option<&'static TypeExpressionData>,
    /// Exact concrete type identity retained for eligible opaque fields.
    opaque_type_id: Option<TypeId>,
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
    /// Exact-type projection operations for an `Option<T>` field.
    optional_ops: Option<OptionalOps>,
}

impl fmt::Debug for FieldMetadata {
    /// Formats the field's stable identity and source summary for diagnostics.
    ///
    /// # Parameters
    ///
    /// * `formatter` - The formatter receiving the diagnostic representation.
    ///
    /// # Returns
    ///
    /// The result of writing the debug representation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FieldMetadata")
            .field("location", &self.location)
            .field("declaration", &self.declaration)
            .field("owner_type_id", &self.owner_type_id)
            .field("definition_path", &self.definition_path)
            .field("source_index", &self.source_index)
            .field("source_variant", &self.source_variant)
            .field("source_name", &self.source_name)
            .field("source_visibility", &self.source_visibility)
            .field("has_expression", &self.expression.is_some())
            .field("variant_inherited", &self.variant_inherited)
            .finish_non_exhaustive()
    }
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
    pub fn from_reflect(owner: TypeId, reflect: FieldDescriptor<'_>) -> Self {
        Self {
            declaration: DeclarationLocation::unknown(),
            location: Some(FieldLocation::new(
                owner,
                reflect.declaring_variant().map(|value| value.index()),
                reflect.index(),
            )),
            owner_type_id: Some(owner),
            definition_path: None,
            source_index: reflect.index(),
            source_variant: reflect.declaring_variant().map(|value| value.index()),
            source_name: Some(reflect.name()),
            source_visibility: reflect.visibility(),
            expression: None,
            opaque_type_id: None,
            variant_inherited: reflect.declaring_variant().is_some(),
            attributes: &[],
            constraints: &[],
            validators: &[],
            serde: &SerdeFieldMetadata::DEFAULT,
            collection_ops: None,
            optional_ops: None,
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
    /// * `variant` - The source enum variant index, if this is a variant field.
    /// * `index` - The source field index within its struct or enum variant.
    /// * `name` - The source field name, or `None` for a tuple field.
    /// * `visibility` - The visibility recorded from source syntax.
    /// * `expression` - The static source type expression for the field.
    /// * `opaque_type_id` - The concrete type identity retained for eligible
    ///   opaque fields.
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn with_semantics(
        owner: TypeId,
        variant: Option<usize>,
        index: usize,
        name: Option<&'static str>,
        visibility: Visibility,
        expression: &'static TypeExpressionData,
        opaque_type_id: Option<TypeId>,
        attributes: &'static [FieldAttributeMetadata],
        constraints: &'static [ConstraintMetadata],
        validators: &'static [ValidatorMetadata],
        serde: &'static SerdeFieldMetadata,
    ) -> Self {
        Self {
            declaration: DeclarationLocation::unknown(),
            location: Some(FieldLocation::new(owner, variant, index)),
            owner_type_id: Some(owner),
            definition_path: None,
            source_index: index,
            source_variant: variant,
            source_name: name,
            source_visibility: visibility,
            expression: Some(expression),
            opaque_type_id,
            variant_inherited: variant.is_some(),
            attributes,
            constraints,
            validators,
            serde,
            collection_ops: None,
            optional_ops: None,
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
    /// * `definition_path` - The generic source declaration path.
    /// * `index` - The source field index within its struct or enum variant.
    /// * `variant` - The source enum variant index, if this is a variant field.
    /// * `name` - The source field name, or `None` for a tuple field.
    /// * `visibility` - The visibility recorded from source syntax.
    /// * `expression` - The unresolved type expression declared for this field.
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn with_definition_semantics(
        definition_path: &'static str,
        index: usize,
        variant: Option<usize>,
        name: Option<&'static str>,
        visibility: Visibility,
        expression: &'static TypeExpressionData,
        variant_inherited: bool,
        attributes: &'static [FieldAttributeMetadata],
        constraints: &'static [ConstraintMetadata],
        validators: &'static [ValidatorMetadata],
        serde: &'static SerdeFieldMetadata,
    ) -> Self {
        Self {
            declaration: DeclarationLocation::unknown(),
            location: None,
            owner_type_id: None,
            definition_path: Some(definition_path),
            source_index: index,
            source_variant: variant,
            source_name: name,
            source_visibility: visibility,
            expression: Some(expression),
            opaque_type_id: None,
            variant_inherited,
            attributes,
            constraints,
            validators,
            serde,
            collection_ops: None,
            optional_ops: None,
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
    #[inline]
    pub const fn collection_ops(&self) -> Option<&CollectionOps> {
        self.collection_ops.as_ref()
    }

    /// Associates exact-type optional projection operations.
    ///
    /// # Parameters
    ///
    /// * `optional_ops` - The generated exact-type optional projection.
    ///
    /// # Returns
    ///
    /// A copy of this overlay with the projection operations attached.
    #[doc(hidden)]
    #[must_use]
    pub const fn with_optional_ops(mut self, optional_ops: OptionalOps) -> Self {
        self.optional_ops = Some(optional_ops);
        self
    }

    /// Returns exact-type optional projection operations, when generated.
    ///
    /// # Returns
    ///
    /// The attached operations, or `None` when no optional projection was
    /// generated for this field.
    #[inline]
    pub const fn optional_ops(&self) -> Option<OptionalOps> {
        self.optional_ops
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

    /// Resolves this field in the supplied reflection context.
    ///
    /// Generic declaration fields have no concrete reflected field and return
    /// `None`.
    ///
    /// # Type Parameters
    ///
    /// * `'ctx` - The lifetime of the context and returned field descriptor.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context in which to resolve the field.
    ///
    /// # Returns
    ///
    /// The matching reflected field when this is a concrete field and its
    /// owner is present in the context; otherwise `None`.
    pub fn reflect_in<'ctx>(&self, context: &'ctx ReflectionContext) -> Option<FieldDescriptor<'ctx>> {
        let descriptor = context.find_type_by_id(self.owner_type_id?).ok().flatten()?;
        if let Some(variant_index) = self.source_variant {
            let variant = descriptor
                .variants()
                .iter()
                .find(|value| value.index() == variant_index)?;
            return variant.fields().iter().find(|value| value.index() == self.source_index);
        }
        descriptor
            .fields()
            .iter()
            .find(|value| value.index() == self.source_index)
    }

    /// Binds the retained generic declaration expression to a reflection
    /// context.
    ///
    /// # Type Parameters
    ///
    /// * `'ctx` - The lifetime of the context-bound type expression.
    ///
    /// # Parameters
    ///
    /// * `context` - The reflection context used to bind the expression.
    ///
    /// # Returns
    ///
    /// The context-bound expression when retained expression data or a
    /// reflected field is available; otherwise `None`.
    pub fn type_expression_in<'ctx>(&self, context: &'ctx ReflectionContext) -> Option<TypeExpression<'ctx>> {
        if let Some(expression) = self.expression.filter(|expression| contains_self_type(expression)) {
            return Some(type_expression_in(expression, context));
        }
        if let Some(field) = self.reflect_in(context) {
            return Some(field.type_expression());
        }
        self.expression
            .map(|expression| type_expression_in(expression, context))
    }

    /// Returns the retained expression facts used to build context-bound views.
    ///
    /// # Returns
    ///
    /// The retained static expression data, or `None` when none is available.
    #[doc(hidden)]
    #[inline]
    pub const fn type_expression_data(&self) -> Option<&'static TypeExpressionData> {
        self.expression
    }

    /// Returns an exact concrete identity retained for an eligible opaque
    /// field.
    ///
    /// The identity is available without requiring the field type to implement
    /// reflection. `None` means the field has no retained concrete identity,
    /// which includes symbolic generic declarations and opaque types with
    /// non-static lifetimes.
    ///
    /// # Returns
    ///
    /// The retained concrete type identity, or `None` when it is unavailable.
    #[inline]
    pub const fn opaque_type_id(&self) -> Option<TypeId> {
        self.opaque_type_id
    }

    /// Returns the zero-based source field index.
    ///
    /// # Returns
    ///
    /// The zero-based index within the source struct or enum variant.
    #[must_use]
    #[inline]
    pub const fn index(&self) -> usize {
        self.source_index
    }

    /// Returns the field query name, or `None` for an unnamed tuple payload.
    ///
    /// # Returns
    ///
    /// The source name, or `None` for an unnamed tuple field.
    #[inline]
    pub const fn name(&self) -> Option<&'static str> {
        self.source_name
    }

    /// Returns the visibility recorded from source syntax.
    ///
    /// # Returns
    ///
    /// The source visibility for this field.
    #[must_use]
    #[inline]
    pub const fn visibility(&self) -> Visibility {
        self.source_visibility
    }

    /// Returns whether the field belongs to a variant whose visibility is
    /// inherited.
    ///
    /// # Returns
    ///
    /// `true` when the field inherits visibility from its enum variant.
    #[must_use]
    #[inline]
    pub const fn variant_inherited(&self) -> bool {
        self.variant_inherited
    }

    /// Returns the generic definition path for a symbolic declaration field.
    ///
    /// # Returns
    ///
    /// The generic definition path, or `None` for a concrete runtime field.
    #[inline]
    pub const fn definition_path(&self) -> Option<&'static str> {
        self.definition_path
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
    /// The identifier metadata when declared; otherwise `None`.
    #[inline]
    pub fn identifier(&self) -> Option<&'static IdentifierMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
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
        self.attributes
            .iter()
            .fold(IndexingReasons::empty(), |result, attribute| match attribute {
                FieldAttributeMetadata::Indexed(value) => result | *value,
                _ => result,
            })
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
    /// The uniqueness metadata when declared; otherwise `None`.
    #[inline]
    pub fn unique(&self) -> Option<&'static FieldUniqueMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
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
    /// The reference metadata when declared; otherwise `None`.
    #[inline]
    pub fn reference(&self) -> Option<&'static FieldReferenceMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
            FieldAttributeMetadata::Reference(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the ordered composite-key declaration, when present.
    ///
    /// # Returns
    ///
    /// The key position metadata when declared; otherwise `None`.
    #[inline]
    pub fn key_part(&self) -> Option<&'static KeyPartMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
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
    /// The text constraint when declared; otherwise `None`.
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
    /// The decimal constraint when declared; otherwise `None`.
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
    /// The temporal constraint when declared; otherwise `None`.
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
    /// The sequence constraint when declared; otherwise `None`.
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
    /// The map constraint when declared; otherwise `None`.
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
    /// The codec metadata when declared; otherwise `None`.
    #[inline]
    pub fn codec(&self) -> Option<&'static CodecMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
            FieldAttributeMetadata::Codec(value) => Some(*value),
            _ => None,
        })
    }

    /// Returns the field redaction declaration, when present.
    ///
    /// # Returns
    ///
    /// The redaction metadata when declared; otherwise `None`.
    #[inline]
    pub fn redact(&self) -> Option<&'static RedactMetadata> {
        self.attributes.iter().find_map(|attribute| match attribute {
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
