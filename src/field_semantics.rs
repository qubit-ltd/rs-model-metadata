// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Domain semantics attached to reflected model fields.
#![allow(
    missing_docs,
    reason = "the occurrence vocabulary is documented as one cohesive contract in the module guide"
)]

use bitflags::bitflags;
use qubit_codec::ValueCodecDescriptor;
use qubit_redact::Sensitivity;
use qubit_validator::NamedValidationArgument;

use crate::ModelId;
use crate::constraint::ConstraintMetadata;
use crate::relation::PropertyPath;
use crate::type_metadata::TypeMetadata;

/// Selects which layer assigns an entity identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierAssignment {
    /// Application code assigns the value.
    Application,
    /// The backing database assigns the value.
    Database,
}

/// Describes identifier assignment for one field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentifierMetadata {
    /// The layer responsible for assigning the identifier value.
    assigned_by: IdentifierAssignment,
}

impl IdentifierMetadata {
    /// Creates identifier metadata.
    #[must_use]
    pub const fn new(assigned_by: IdentifierAssignment) -> Self {
        Self { assigned_by }
    }

    /// Returns the identifier assignment source.
    #[must_use]
    pub const fn assigned_by(&self) -> IdentifierAssignment {
        self.assigned_by
    }
}

bitflags! {
    /// Explains every declaration fact that makes a field indexed.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct IndexingReasons: u8 {
        /// The source explicitly requested an index.
        const EXPLICIT = 0b0001;
        /// Identifier lookup requires an index.
        const IDENTIFIER = 0b0010;
        /// Uniqueness enforcement requires an index.
        const UNIQUE = 0b0100;
        /// Reference lookup requires an index.
        const REFERENCE = 0b1000;
    }
}

/// Field-local uniqueness semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UniqueMetadata {
    /// Property paths that scope uniqueness with this field.
    respect_to: &'static [PropertyPath<'static>],
    /// Whether textual comparisons use case-insensitive matching.
    ignore_case: bool,
}

impl UniqueMetadata {
    /// Creates uniqueness metadata.
    #[must_use]
    pub const fn new(respect_to: &'static [PropertyPath<'static>], ignore_case: bool) -> Self {
        Self {
            respect_to,
            ignore_case,
        }
    }

    /// Returns scope paths in source order.
    #[must_use]
    pub const fn respect_to(&self) -> &'static [PropertyPath<'static>] {
        self.respect_to
    }

    /// Returns whether text comparison ignores case.
    #[must_use]
    pub const fn ignore_case(&self) -> bool {
        self.ignore_case
    }

    /// Returns whether uniqueness is scoped by another property.
    #[must_use]
    pub const fn is_scoped(&self) -> bool {
        !self.respect_to.is_empty()
    }
}

/// Declares the ordering of one key component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyPartMetadata {
    /// The zero-based position within the composite key.
    order: usize,
}

impl KeyPartMetadata {
    /// Creates key-part metadata.
    #[must_use]
    pub const fn new(order: usize) -> Self {
        Self { order }
    }

    /// Returns the zero-based component order.
    #[must_use]
    pub const fn order(&self) -> usize {
        self.order
    }
}

/// Distinguishes a Rust-type target from a stable model-ID target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclaredEntityTargetKind {
    /// The target is supplied by a static metadata provider.
    RustType,
    /// The target is supplied as a stable model ID.
    ModelId,
}

/// A declaration-time entity target that does not consult the registry.
#[derive(Clone, Copy)]
pub enum DeclaredEntityTarget {
    /// A Rust type metadata provider.
    RustType(fn() -> &'static TypeMetadata),
    /// A stable model ID string.
    ModelId(ModelId),
}

impl DeclaredEntityTarget {
    /// Returns the target representation kind.
    #[must_use]
    pub const fn kind(&self) -> DeclaredEntityTargetKind {
        match self {
            Self::RustType(_) => DeclaredEntityTargetKind::RustType,
            Self::ModelId(_) => DeclaredEntityTargetKind::ModelId,
        }
    }

    /// Returns directly supplied Rust metadata, if any.
    #[must_use]
    pub fn metadata(&self) -> Option<&'static TypeMetadata> {
        match self {
            Self::RustType(provider) => Some(provider()),
            Self::ModelId(_) => None,
        }
    }

    /// Returns the declared stable model ID, if any.
    #[must_use]
    pub const fn model_id(&self) -> Option<ModelId> {
        match self {
            Self::RustType(_) => None,
            Self::ModelId(id) => Some(*id),
        }
    }
}

impl core::fmt::Debug for DeclaredEntityTarget {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_tuple("DeclaredEntityTarget")
            .field(&self.kind())
            .finish()
    }
}

/// Selects an entity as a whole or one of its properties.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceSelection {
    /// Selects the complete entity.
    Entity,
    /// Selects a property path on the entity.
    Property(PropertyPath<'static>),
}

/// Declaration metadata for one entity reference.
#[derive(Clone, Copy, Debug)]
pub struct ReferenceMetadata {
    /// The entity declaration selected by this reference.
    target: &'static DeclaredEntityTarget,
    /// The entity or property selected from the target.
    selection: &'static ReferenceSelection,
    /// Whether the referenced record must exist before assignment.
    existing: bool,
    /// An equivalent local property path, when declared.
    same_as: Option<&'static PropertyPath<'static>>,
}

impl ReferenceMetadata {
    /// Creates reference metadata.
    #[must_use]
    pub const fn new(
        target: &'static DeclaredEntityTarget,
        selection: &'static ReferenceSelection,
        existing: bool,
        same_as: Option<&'static PropertyPath<'static>>,
    ) -> Self {
        Self {
            target,
            selection,
            existing,
            same_as,
        }
    }

    /// Returns the declared entity target.
    #[must_use]
    pub const fn target(&self) -> &'static DeclaredEntityTarget {
        self.target
    }

    /// Returns the selected entity value or property.
    #[must_use]
    pub const fn selection(&self) -> &'static ReferenceSelection {
        self.selection
    }

    /// Returns whether the referenced record must already exist.
    #[must_use]
    pub const fn existing(&self) -> bool {
        self.existing
    }

    /// Returns an equivalent property path, if declared.
    #[must_use]
    pub const fn same_as(&self) -> Option<&'static PropertyPath<'static>> {
        self.same_as
    }
}

/// One declared validator occurrence.
#[derive(Clone, Copy, Debug)]
pub struct ValidatorMetadata {
    /// The stable identifier used to resolve the validator registration.
    declared_id: &'static str,
    /// Static arguments passed to the resolved validator.
    params: &'static [NamedValidationArgument<'static>],
    /// Property paths that must be available to the validator.
    depends_on: &'static [PropertyPath<'static>],
}

impl ValidatorMetadata {
    /// Creates a validator occurrence without resolving a runtime registry.
    #[must_use]
    pub const fn new(
        declared_id: &'static str,
        params: &'static [NamedValidationArgument<'static>],
        depends_on: &'static [PropertyPath<'static>],
    ) -> Self {
        assert!(!declared_id.is_empty(), "validator ID cannot be empty");
        Self {
            declared_id,
            params,
            depends_on,
        }
    }

    /// Returns the validated declaration ID.
    #[must_use]
    pub const fn declared_id(&self) -> &'static str {
        self.declared_id
    }

    /// Returns parameters in source order.
    #[must_use]
    pub const fn params(&self) -> &'static [NamedValidationArgument<'static>] {
        self.params
    }

    /// Returns dependency paths in source order.
    #[must_use]
    pub const fn depends_on(&self) -> &'static [PropertyPath<'static>] {
        self.depends_on
    }
}

/// A codec declared by Rust type or stable textual ID.
#[derive(Clone, Copy, Debug)]
pub enum CodecReference {
    /// A codec identified by its static reflection descriptor.
    RustType(&'static ValueCodecDescriptor),
    /// A codec identified by its stable registry ID.
    DeclaredId(&'static str),
}

/// Identifies where a codec declaration originated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecSource {
    /// A codec declared directly on a field.
    Field,
    /// A codec declared for a value object's canonical representation.
    CanonicalValue,
    /// A codec declared for a nested selector position.
    Selector(SelectorPosition),
}

/// One codec occurrence.
#[derive(Clone, Copy, Debug)]
pub struct CodecMetadata {
    /// The Rust-type or textual codec declaration.
    codec: &'static CodecReference,
    /// The metadata location that supplied the declaration.
    source: CodecSource,
}

impl CodecMetadata {
    /// Creates codec metadata.
    #[must_use]
    pub const fn new(codec: &'static CodecReference, source: CodecSource) -> Self {
        Self { codec, source }
    }

    /// Returns the codec declaration.
    #[must_use]
    pub const fn codec(&self) -> &'static CodecReference {
        self.codec
    }

    /// Returns the declaration source.
    #[must_use]
    pub const fn source(&self) -> CodecSource {
        self.source
    }
}

/// The structural position selected by nested field semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorPosition {
    /// The element type of a sequence.
    Element,
    /// The key type of a map.
    MapKey,
    /// The value type of a map.
    MapValue,
}

/// Narrow declaration modes delegated to `qubit-redact` capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactModeMetadata {
    /// Apply the configured sensitivity level.
    Level,
    /// Omit the selected value.
    Skip,
    /// Recurse into nested metadata.
    Nested,
    /// Apply map-specific redaction.
    Map,
    /// Select a redaction policy by stable key.
    KeyedBy(&'static str),
    /// Use JSON redaction semantics.
    Json,
}

/// The value position affected by a redact declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactPosition {
    /// The field value itself.
    Field,
    /// A sequence element.
    Element,
    /// A map key.
    MapKey,
    /// A map value.
    MapValue,
}

/// One redact declaration using the upstream sensitivity vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedactMetadata {
    /// The optional sensitivity level applied by the redact capability.
    sensitivity: Option<Sensitivity>,
    /// The redact operation to perform.
    mode: RedactModeMetadata,
    /// The structural value position to redact.
    position: RedactPosition,
}

impl RedactMetadata {
    /// Creates redact declaration metadata.
    #[must_use]
    pub const fn new(sensitivity: Option<Sensitivity>, mode: RedactModeMetadata, position: RedactPosition) -> Self {
        Self {
            sensitivity,
            mode,
            position,
        }
    }

    /// Returns the configured sensitivity, if applicable.
    #[must_use]
    pub const fn sensitivity(&self) -> Option<Sensitivity> {
        self.sensitivity
    }

    /// Returns the declaration mode.
    #[must_use]
    pub const fn mode(&self) -> RedactModeMetadata {
        self.mode
    }

    /// Returns the affected value position.
    #[must_use]
    pub const fn position(&self) -> RedactPosition {
        self.position
    }
}

/// Final Serde behavior for one field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SerdeBehaviorSource {
    /// The behavior is absent.
    None,
    /// The behavior came from an explicit Serde field attribute.
    Explicit,
    /// The model macro supplied the behavior.
    ModelDefault,
    /// A model marker explicitly suppressed the model default.
    Suppressed,
}

/// Final Serde behavior for one field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SerdeFieldMetadata {
    /// The explicit serialization name, when one is configured.
    serialize_name: Option<&'static str>,
    /// The explicit deserialization name, when one is configured.
    deserialize_name: Option<&'static str>,
    /// Whether Serde omits this field while serializing.
    skip_serializing: bool,
    /// Whether Serde ignores this field while deserializing.
    skip_deserializing: bool,
    /// Whether Serde flattens this field into its parent representation.
    flatten: bool,
    /// The Serde conversion module or function path, when configured.
    with: Option<&'static str>,
    /// Whether Serde supplies a missing value through its default behavior.
    default: bool,
    /// The origin of the missing-value default behavior.
    default_source: SerdeBehaviorSource,
    /// The origin of empty-value omission behavior.
    omit_source: SerdeBehaviorSource,
}

impl SerdeFieldMetadata {
    /// Empty Serde behavior used when no configuration applies.
    pub const DEFAULT: Self = Self::new(None, None, false, false, false, None, false);

    /// Creates final Serde field behavior.
    #[must_use]
    pub const fn new(
        serialize_name: Option<&'static str>,
        deserialize_name: Option<&'static str>,
        skip_serializing: bool,
        skip_deserializing: bool,
        flatten: bool,
        with: Option<&'static str>,
        default: bool,
    ) -> Self {
        Self {
            serialize_name,
            deserialize_name,
            skip_serializing,
            skip_deserializing,
            flatten,
            with,
            default,
            default_source: if default {
                SerdeBehaviorSource::Explicit
            } else {
                SerdeBehaviorSource::None
            },
            omit_source: SerdeBehaviorSource::None,
        }
    }

    /// Records whether missing-value defaults and empty-value omission were
    /// explicit, generated, or suppressed.
    #[must_use]
    pub const fn with_sources(mut self, default_source: SerdeBehaviorSource, omit_source: SerdeBehaviorSource) -> Self {
        self.default_source = default_source;
        self.omit_source = omit_source;
        self
    }

    /// Returns the configured serialization name, or `None` when Serde uses
    /// the field name.
    #[must_use]
    pub const fn serialize_name(&self) -> Option<&'static str> {
        self.serialize_name
    }
    /// Returns the configured deserialization name, or `None` when Serde uses
    /// the field name.
    #[must_use]
    pub const fn deserialize_name(&self) -> Option<&'static str> {
        self.deserialize_name
    }
    /// Returns whether Serde omits this field during serialization.
    #[must_use]
    pub const fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }
    /// Returns whether Serde ignores this field during deserialization.
    #[must_use]
    pub const fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }
    /// Returns whether Serde flattens this field into its parent.
    #[must_use]
    pub const fn flatten(&self) -> bool {
        self.flatten
    }
    /// Returns the configured Serde conversion path, if present.
    #[must_use]
    pub const fn with(&self) -> Option<&'static str> {
        self.with
    }
    /// Returns whether a missing value uses Serde's default behavior.
    #[must_use]
    pub const fn default(&self) -> bool {
        self.default
    }
    /// Returns the declaration source for missing-value defaults.
    #[must_use]
    pub const fn default_source(&self) -> SerdeBehaviorSource {
        self.default_source
    }
    /// Returns the declaration source for empty-value omission.
    #[must_use]
    pub const fn omit_source(&self) -> SerdeBehaviorSource {
        self.omit_source
    }
}

/// Non-recursive semantics applied to a collection position.
#[derive(Clone, Copy, Debug)]
pub struct SelectorMetadata {
    /// The nested collection position described by this metadata.
    position: SelectorPosition,
    /// Constraints applied at the selected position.
    constraints: &'static [ConstraintMetadata],
    /// Validators applied at the selected position.
    validators: &'static [ValidatorMetadata],
    /// The optional codec declaration for the selected position.
    codec: Option<&'static CodecMetadata>,
    /// The optional redaction declaration for the selected position.
    redact: Option<&'static RedactMetadata>,
}

impl SelectorMetadata {
    /// Creates selector metadata.
    #[must_use]
    pub const fn new(
        position: SelectorPosition,
        constraints: &'static [ConstraintMetadata],
        validators: &'static [ValidatorMetadata],
        codec: Option<&'static CodecMetadata>,
        redact: Option<&'static RedactMetadata>,
    ) -> Self {
        Self {
            position,
            constraints,
            validators,
            codec,
            redact,
        }
    }

    /// Returns the nested collection position described by this metadata.
    #[must_use]
    pub const fn position(&self) -> SelectorPosition {
        self.position
    }
    /// Returns constraints in source order.
    #[must_use]
    pub const fn constraints(&self) -> &'static [ConstraintMetadata] {
        self.constraints
    }
    /// Returns validators in source order.
    #[must_use]
    pub const fn validators(&self) -> &'static [ValidatorMetadata] {
        self.validators
    }
    /// Returns the codec declaration, or `None` when absent.
    #[must_use]
    pub const fn codec(&self) -> Option<&'static CodecMetadata> {
        self.codec
    }
    /// Returns the redaction declaration, or `None` when absent.
    #[must_use]
    pub const fn redact(&self) -> Option<&'static RedactMetadata> {
        self.redact
    }
}

/// Source-order view over the same strongly typed field objects.
#[derive(Clone, Copy, Debug)]
pub enum FieldAttributeMetadata {
    /// Identifier declaration metadata.
    Identifier(&'static IdentifierMetadata),
    /// Reasons the field participates in an index.
    Indexed(IndexingReasons),
    /// Uniqueness declaration metadata.
    Unique(&'static UniqueMetadata),
    /// Entity-reference declaration metadata.
    Reference(&'static ReferenceMetadata),
    /// Composite-key position metadata.
    KeyPart(&'static KeyPartMetadata),
    /// A standard validation constraint occurrence.
    Constraint(&'static ConstraintMetadata),
    /// A custom validator occurrence.
    Validator(&'static ValidatorMetadata),
    /// A value codec occurrence.
    Codec(&'static CodecMetadata),
    /// A redaction declaration occurrence.
    Redact(&'static RedactMetadata),
    /// Effective Serde behavior.
    Serde(&'static SerdeFieldMetadata),
    /// An explicit opaque-type marker.
    Opaque,
}
