// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

// qubit-style: allow multiple-public-types
//! Domain semantics attached to reflected model fields.
#![allow(
    missing_docs,
    reason = "the occurrence vocabulary is documented as one cohesive contract in the module guide"
)]

use std::any::TypeId;

use bitflags::bitflags;
use qubit_redact::Sensitivity;

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
    respect_to: &'static [PropertyPath],
    ignore_case: bool,
}

impl UniqueMetadata {
    /// Creates uniqueness metadata.
    #[must_use]
    pub const fn new(respect_to: &'static [PropertyPath], ignore_case: bool) -> Self {
        Self {
            respect_to,
            ignore_case,
        }
    }

    /// Returns scope paths in source order.
    #[must_use]
    pub const fn respect_to(&self) -> &'static [PropertyPath] {
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
    Property(PropertyPath),
}

/// Declaration metadata for one entity reference.
#[derive(Clone, Copy, Debug)]
pub struct ReferenceMetadata {
    target: &'static DeclaredEntityTarget,
    selection: &'static ReferenceSelection,
    existing: bool,
    same_as: Option<&'static PropertyPath>,
}

impl ReferenceMetadata {
    /// Creates reference metadata.
    #[must_use]
    pub const fn new(
        target: &'static DeclaredEntityTarget,
        selection: &'static ReferenceSelection,
        existing: bool,
        same_as: Option<&'static PropertyPath>,
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
    pub const fn same_as(&self) -> Option<&'static PropertyPath> {
        self.same_as
    }
}

/// One statically typed strategy parameter value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StrategyArgument {
    Bool(bool),
    Integer(i128),
    Unsigned(u128),
    String(&'static str),
    BoolList(&'static [bool]),
    IntegerList(&'static [i128]),
    UnsignedList(&'static [u128]),
    StringList(&'static [&'static str]),
}

/// A named strategy parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamedStrategyArgument {
    name: &'static str,
    value: StrategyArgument,
}

impl NamedStrategyArgument {
    /// Creates a named strategy parameter.
    #[must_use]
    pub const fn new(name: &'static str, value: StrategyArgument) -> Self {
        assert!(!name.is_empty(), "strategy parameter name cannot be empty");
        Self { name, value }
    }

    /// Returns the parameter name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the parameter value.
    #[must_use]
    pub const fn value(&self) -> StrategyArgument {
        self.value
    }
}

/// One declared validator occurrence.
#[derive(Clone, Copy, Debug)]
pub struct ValidatorMetadata {
    declared_id: &'static str,
    params: &'static [NamedStrategyArgument],
    depends_on: &'static [PropertyPath],
}

impl ValidatorMetadata {
    /// Creates a validator occurrence without resolving a runtime registry.
    #[must_use]
    pub const fn new(
        declared_id: &'static str,
        params: &'static [NamedStrategyArgument],
        depends_on: &'static [PropertyPath],
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
    pub const fn params(&self) -> &'static [NamedStrategyArgument] {
        self.params
    }

    /// Returns dependency paths in source order.
    #[must_use]
    pub const fn depends_on(&self) -> &'static [PropertyPath] {
        self.depends_on
    }
}

/// Exact process-local identity for a Rust strategy type.
#[derive(Clone, Copy)]
pub struct StrategyTypeIdentity {
    type_id: fn() -> TypeId,
    type_name: fn() -> &'static str,
}

impl StrategyTypeIdentity {
    /// Creates identity providers for `T`.
    #[must_use]
    pub const fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>,
            type_name: core::any::type_name::<T>,
        }
    }

    /// Returns the exact process-local type ID.
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        (self.type_id)()
    }

    /// Returns the compiler-provided diagnostic type name.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        (self.type_name)()
    }
}

impl core::fmt::Debug for StrategyTypeIdentity {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_tuple("StrategyTypeIdentity")
            .field(&self.type_name())
            .finish()
    }
}

/// A codec declared by Rust type or stable textual ID.
#[derive(Clone, Copy, Debug)]
pub enum CodecReference {
    RustType(StrategyTypeIdentity),
    DeclaredId(&'static str),
}

/// Identifies where a codec declaration originated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CodecSource {
    Field,
    CanonicalValue,
    Selector(SelectorPosition),
}

/// One codec occurrence.
#[derive(Clone, Copy, Debug)]
pub struct CodecMetadata {
    codec: &'static CodecReference,
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
    Element,
    MapKey,
    MapValue,
}

/// Narrow declaration modes delegated to `qubit-redact` capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactModeMetadata {
    Level,
    Skip,
    Nested,
    Map,
    KeyedBy(&'static str),
    Json,
}

/// The value position affected by a redact declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedactPosition {
    Field,
    Element,
    MapKey,
    MapValue,
}

/// One redact declaration using the upstream sensitivity vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedactMetadata {
    sensitivity: Option<Sensitivity>,
    mode: RedactModeMetadata,
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
    serialize_name: Option<&'static str>,
    deserialize_name: Option<&'static str>,
    skip_serializing: bool,
    skip_deserializing: bool,
    flatten: bool,
    with: Option<&'static str>,
    default: bool,
    default_source: SerdeBehaviorSource,
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

    pub const fn serialize_name(&self) -> Option<&'static str> {
        self.serialize_name
    }
    pub const fn deserialize_name(&self) -> Option<&'static str> {
        self.deserialize_name
    }
    pub const fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }
    pub const fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }
    pub const fn flatten(&self) -> bool {
        self.flatten
    }
    pub const fn with(&self) -> Option<&'static str> {
        self.with
    }
    pub const fn default(&self) -> bool {
        self.default
    }
    pub const fn default_source(&self) -> SerdeBehaviorSource {
        self.default_source
    }
    pub const fn omit_source(&self) -> SerdeBehaviorSource {
        self.omit_source
    }
}

/// Non-recursive semantics applied to a collection position.
#[derive(Clone, Copy, Debug)]
pub struct SelectorMetadata {
    position: SelectorPosition,
    constraints: &'static [ConstraintMetadata],
    validators: &'static [ValidatorMetadata],
    codec: Option<&'static CodecMetadata>,
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

    pub const fn position(&self) -> SelectorPosition {
        self.position
    }
    pub const fn constraints(&self) -> &'static [ConstraintMetadata] {
        self.constraints
    }
    pub const fn validators(&self) -> &'static [ValidatorMetadata] {
        self.validators
    }
    pub const fn codec(&self) -> Option<&'static CodecMetadata> {
        self.codec
    }
    pub const fn redact(&self) -> Option<&'static RedactMetadata> {
        self.redact
    }
}

/// Source-order view over the same strongly typed field objects.
#[derive(Clone, Copy, Debug)]
pub enum FieldAttributeMetadata {
    Identifier(&'static IdentifierMetadata),
    Indexed(IndexingReasons),
    Unique(&'static UniqueMetadata),
    Reference(&'static ReferenceMetadata),
    KeyPart(&'static KeyPartMetadata),
    Constraint(&'static ConstraintMetadata),
    Validator(&'static ValidatorMetadata),
    Codec(&'static CodecMetadata),
    Redact(&'static RedactMetadata),
    Serde(&'static SerdeFieldMetadata),
    Opaque,
}
