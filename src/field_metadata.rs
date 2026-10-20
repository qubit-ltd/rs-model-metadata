// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable domain semantics over a reflected field's static source facts.

use std::any::TypeId;

use qubit_reflect::__private::codegen_v4::expression::GenericExpressionArgument;
use qubit_reflect::__private::codegen_v4::expression::TypeExpressionData;
use qubit_reflect::FieldDescriptor;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::TypeExpression;
use qubit_reflect::descriptor::Visibility;

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

impl core::fmt::Debug for FieldMetadata {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    #[doc(hidden)]
    #[must_use]
    pub const fn with_optional_ops(mut self, optional_ops: OptionalOps) -> Self {
        self.optional_ops = Some(optional_ops);
        self
    }

    /// Returns exact-type optional projection operations, when generated.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn type_expression_in<'ctx>(&self, context: &'ctx ReflectionContext) -> Option<TypeExpression<'ctx>> {
        if let Some(expression) = self.expression.filter(|expression| contains_self_type(expression)) {
            return Some(qubit_reflect::__private::codegen_v4::expression::type_expression_in(
                expression, context,
            ));
        }
        if let Some(field) = self.reflect_in(context) {
            return Some(field.type_expression());
        }
        self.expression
            .map(|expression| qubit_reflect::__private::codegen_v4::expression::type_expression_in(expression, context))
    }

    /// Returns the retained expression facts used to build context-bound views.
    #[doc(hidden)]
    #[must_use]
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
    #[must_use]
    #[inline]
    pub const fn opaque_type_id(&self) -> Option<TypeId> {
        self.opaque_type_id
    }

    /// Returns the zero-based source field index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.source_index
    }

    /// Returns the field query name, or `None` for an unnamed tuple payload.
    #[must_use]
    pub const fn name(&self) -> Option<&'static str> {
        self.source_name
    }

    /// Returns the visibility recorded from source syntax.
    #[must_use]
    pub const fn visibility(&self) -> Visibility {
        self.source_visibility
    }

    /// Returns whether the field belongs to a variant whose visibility is
    /// inherited.
    #[must_use]
    pub const fn variant_inherited(&self) -> bool {
        self.variant_inherited
    }

    /// Returns the generic definition path for a symbolic declaration field.
    #[must_use]
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
    /// `Some` with the identifier metadata, or `None` when no identifier is
    /// declared.
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
    /// `Some` with the uniqueness metadata, or `None` when uniqueness is not
    /// declared.
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
    /// `Some` with the reference metadata, or `None` when the field is not an
    /// entity reference.
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
    /// `Some` with the key position metadata, or `None` when the field is not
    /// part of a composite key.
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
    /// `Some` with the text constraint, or `None` when none is declared.
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
    /// `Some` with the redaction metadata, or `None` when no redaction is
    /// declared.
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

fn contains_self_type(expression: &TypeExpressionData) -> bool {
    use qubit_reflect::__private::codegen_v4::expression::ExpressionData;

    fn argument_contains_self(argument: &GenericExpressionArgument) -> bool {
        match argument {
            GenericExpressionArgument::Type(expression)
            | GenericExpressionArgument::AssociatedType { value: expression, .. } => contains_self_type(expression),
            GenericExpressionArgument::Lifetime(_)
            | GenericExpressionArgument::Const(_)
            | GenericExpressionArgument::AssociatedTypeBound { .. } => false,
        }
    }

    match expression {
        ExpressionData::SelfType => true,
        ExpressionData::Named { arguments, .. } => arguments.iter().any(argument_contains_self),
        ExpressionData::AssociatedType {
            self_type, arguments, ..
        } => contains_self_type(self_type) || arguments.iter().any(argument_contains_self),
        ExpressionData::Reference { target, .. }
        | ExpressionData::RawPointer { target, .. }
        | ExpressionData::Slice(target) => contains_self_type(target),
        ExpressionData::Array { element, .. } => contains_self_type(element),
        ExpressionData::Tuple(elements) => elements.iter().any(|element| contains_self_type(element)),
        ExpressionData::FunctionPointer(function) => {
            function.arguments.iter().any(|argument| contains_self_type(argument))
                || function.result.is_some_and(contains_self_type)
        }
        ExpressionData::Parameter(_)
        | ExpressionData::ParameterName(_)
        | ExpressionData::TraitObject(_)
        | ExpressionData::Opaque(_)
        | ExpressionData::Never => false,
    }
}
