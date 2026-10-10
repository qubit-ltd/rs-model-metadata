// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Domain semantics attached to reflected model fields.

use core::any::TypeId;
use core::any::type_name;
use core::fmt;
use core::hash::Hash;
use core::hash::Hasher;

use bitflags::bitflags;
use qubit_reflect::descriptor::TypeRef;

use crate::constraint_metadata::ConstraintMetadata;
use crate::metadata::ModelId;
use crate::relation::DeclarationLocation;
use crate::relation::ObjectPath;
use crate::relation::PropertyPath;
use crate::type_metadata::TypeMetadata;

/// A declaration-time reference to a Rust type.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::RustTypeReference;
///
/// let reference = RustTypeReference::of::<String>();
/// assert_eq!(reference.type_id(), core::any::TypeId::of::<String>());
/// assert!(reference.type_name().contains("String"));
/// ```
#[derive(Clone, Copy)]
pub struct RustTypeReference {
    /// Monomorphized identity query; no reflection descriptor is constructed.
    type_id: fn() -> TypeId,
    /// Monomorphized diagnostic name query for the same Rust type.
    type_name: fn() -> &'static str,
}

impl RustTypeReference {
    /// Creates a reference to `T` without constructing a runtime descriptor.
    ///
    /// # Returns
    ///
    /// A compact reference that can later report `T`'s type identity and name.
    #[must_use]
    #[inline]
    pub const fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>,
            type_name: type_name::<T>,
        }
    }

    /// Returns the referenced Rust [`TypeId`].
    ///
    /// # Returns
    ///
    /// The monomorphized `TypeId` for the referenced Rust type.
    #[must_use]
    #[inline]
    pub fn type_id(self) -> TypeId {
        (self.type_id)()
    }

    /// Returns the referenced Rust type name.
    ///
    /// # Returns
    ///
    /// The diagnostic name reported for the referenced Rust type.
    #[must_use]
    #[inline]
    pub fn type_name(self) -> &'static str {
        (self.type_name)()
    }
}

impl fmt::Debug for RustTypeReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("RustTypeReference")
            .field(&self.type_name())
            .finish()
    }
}

impl PartialEq for RustTypeReference {
    fn eq(&self, other: &Self) -> bool {
        self.type_id() == other.type_id()
    }
}

impl Eq for RustTypeReference {}

impl Hash for RustTypeReference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id().hash(state);
    }
}

/// One statically typed validator parameter value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ValidationArgument;
///
/// let argument = ValidationArgument::String("example");
/// assert!(matches!(argument, ValidationArgument::String("example")));
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ValidationArgument<'a> {
    /// A Boolean value.
    Bool(bool),
    /// A signed integer value.
    Integer(i128),
    /// An unsigned integer value.
    Unsigned(u128),
    /// A borrowed string value.
    String(&'a str),
    /// A borrowed Boolean list.
    BoolList(&'a [bool]),
    /// A borrowed signed integer list.
    IntegerList(&'a [i128]),
    /// A borrowed unsigned integer list.
    UnsignedList(&'a [u128]),
    /// A borrowed string list.
    StringList(&'a [&'a str]),
}

impl fmt::Debug for ValidationArgument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(_) => formatter.write_str("Bool(<redacted>)"),
            Self::Integer(_) => formatter.write_str("Integer(<redacted>)"),
            Self::Unsigned(_) => formatter.write_str("Unsigned(<redacted>)"),
            Self::String(_) => formatter.write_str("String(<redacted>)"),
            Self::BoolList(values) => formatter.debug_struct("BoolList").field("len", &values.len()).finish(),
            Self::IntegerList(values) => formatter
                .debug_struct("IntegerList")
                .field("len", &values.len())
                .finish(),
            Self::UnsignedList(values) => formatter
                .debug_struct("UnsignedList")
                .field("len", &values.len())
                .finish(),
            Self::StringList(values) => formatter
                .debug_struct("StringList")
                .field("len", &values.len())
                .finish(),
        }
    }
}

/// One named validator parameter borrowing its name and value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{NamedValidationArgument, ValidationArgument};
///
/// let argument = NamedValidationArgument::new("limit", ValidationArgument::Integer(5));
/// assert_eq!(argument.name(), "limit");
/// assert!(matches!(argument.value(), ValidationArgument::Integer(5)));
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NamedValidationArgument<'a> {
    /// Name used by the validator's parameter schema.
    name: &'a str,
    /// Typed value supplied for the named parameter.
    value: ValidationArgument<'a>,
}

impl<'a> NamedValidationArgument<'a> {
    /// Creates a named parameter; panics if `name` is empty.
    ///
    /// # Parameters
    ///
    /// * `name` - Non-empty key expected by the validator schema.
    /// * `value` - Typed value borrowed by the validator.
    ///
    /// # Returns
    ///
    /// The named parameter borrowing `name` and `value`.
    #[must_use]
    #[inline]
    pub const fn new(name: &'a str, value: ValidationArgument<'a>) -> Self {
        assert!(!name.is_empty(), "validator parameter name cannot be empty");
        Self { name, value }
    }

    /// Returns the borrowed parameter name.
    ///
    /// # Returns
    ///
    /// The schema key borrowed for the lifetime of this argument.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the typed parameter value.
    ///
    /// # Returns
    ///
    /// The copyable value supplied for this named parameter.
    #[must_use]
    #[inline]
    pub const fn value(&self) -> ValidationArgument<'a> {
        self.value
    }
}

impl fmt::Debug for NamedValidationArgument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NamedValidationArgument")
            .field("value", &self.value)
            .finish()
    }
}

/// Sensitivity declared for redaction-aware consumers.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::Sensitivity;
///
/// assert_eq!(Sensitivity::Secret.as_str(), "secret");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Sensitivity {
    /// Information intended for unrestricted disclosure.
    Public,
    /// Personal information requiring ordinary access controls.
    Personal,
    /// Confidential information requiring restricted access.
    Confidential,
    /// Secret data.
    Secret,
}

impl Sensitivity {
    /// Returns the stable declaration spelling.
    ///
    /// # Returns
    ///
    /// The lowercase metadata spelling used by reflection consumers.
    #[must_use]
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Personal => "personal",
            Self::Confidential => "confidential",
            Self::Secret => "secret",
        }
    }
}

/// Selects which layer assigns an entity identifier.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{IdentifierAssignment, IdentifierMetadata};
///
/// let identifier = IdentifierMetadata::new(IdentifierAssignment::Database);
/// assert_eq!(identifier.assigned_by(), IdentifierAssignment::Database);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierAssignment {
    /// Application code assigns the value.
    Application,
    /// The backing database assigns the value.
    Database,
}

/// Describes identifier assignment for one field.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{IdentifierAssignment, IdentifierMetadata};
///
/// let identifier = IdentifierMetadata::new(IdentifierAssignment::Database);
/// assert_eq!(identifier.assigned_by(), IdentifierAssignment::Database);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentifierMetadata {
    /// The layer responsible for assigning the identifier value.
    assigned_by: IdentifierAssignment,
}

impl IdentifierMetadata {
    /// Creates identifier metadata.
    ///
    /// # Parameters
    ///
    /// * `assigned_by` - Layer that supplies the identifier value.
    ///
    /// # Returns
    ///
    /// Metadata recording the identifier assignment source.
    #[must_use]
    #[inline]
    pub const fn new(assigned_by: IdentifierAssignment) -> Self {
        Self { assigned_by }
    }

    /// Returns the identifier assignment source.
    ///
    /// # Returns
    ///
    /// The application or database layer responsible for assigning the value.
    #[must_use]
    #[inline]
    pub const fn assigned_by(&self) -> IdentifierAssignment {
        self.assigned_by
    }
}

bitflags! {
    /// Explains every declaration fact that makes a field indexed.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_model_metadata::metadata::IndexingReasons;
    ///
    /// let reasons = IndexingReasons::EXPLICIT | IndexingReasons::IDENTIFIER;
    /// assert!(reasons.contains(IndexingReasons::EXPLICIT));
    /// ```
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
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::FieldUniqueMetadata;
///
/// let unique = FieldUniqueMetadata::for_definition(&[], None);
/// assert_eq!(unique.effective_ignore_case(), None);
/// assert!(!unique.is_scoped());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UniqueMetadata {
    /// Property paths that scope uniqueness with this field.
    respect_to: &'static [PropertyPath<'static>],
    /// Whether textual comparisons use case-insensitive matching.
    ignore_case: bool,
    /// Explicit source option; absent when text capability supplies the
    /// default.
    declared_ignore_case: Option<bool>,
    /// Whether the effective default awaits concrete generic field
    /// capabilities.
    deferred: bool,
}

impl UniqueMetadata {
    /// Creates uniqueness metadata.
    ///
    /// # Parameters
    ///
    /// * `respect_to` - Scope paths that constrain the uniqueness key.
    /// * `ignore_case` - Whether textual comparisons ignore case.
    ///
    /// # Returns
    ///
    /// Concrete uniqueness metadata with the supplied comparison policy.
    #[must_use]
    #[inline]
    pub const fn new(respect_to: &'static [PropertyPath<'static>], ignore_case: bool) -> Self {
        Self {
            respect_to,
            ignore_case,
            declared_ignore_case: Some(ignore_case),
            deferred: false,
        }
    }

    /// Preserves a source-level generic declaration before concrete
    /// substitution.
    ///
    /// # Parameters
    ///
    /// * `respect_to` - Scope paths declared on the generic definition.
    /// * `ignore_case` - Explicit policy, or `None` for a deferred default.
    ///
    /// # Returns
    ///
    /// Unresolved metadata retaining the declaration for later substitution.
    #[must_use]
    #[inline]
    pub const fn for_definition(respect_to: &'static [PropertyPath<'static>], ignore_case: Option<bool>) -> Self {
        Self {
            respect_to,
            ignore_case: false,
            declared_ignore_case: ignore_case,
            deferred: true,
        }
    }

    /// Computes implicit case comparison from the reflected field capability.
    ///
    /// # Parameters
    ///
    /// * `respect_to` - Scope paths that constrain the uniqueness key.
    /// * `ignore_case` - Explicit comparison policy, when provided.
    /// * `ty` - Reflected field type used to infer a textual default.
    ///
    /// # Returns
    ///
    /// Concrete uniqueness metadata with the explicit or inferred policy.
    #[must_use]
    pub fn for_type(respect_to: &'static [PropertyPath<'static>], ignore_case: Option<bool>, ty: &TypeRef) -> Self {
        Self {
            respect_to,
            ignore_case: ignore_case.unwrap_or_else(|| text_reference(ty)),
            declared_ignore_case: ignore_case,
            deferred: false,
        }
    }

    /// Returns the explicit declaration independently of the effective default.
    ///
    /// # Returns
    ///
    /// The declared option, preserving whether the policy was omitted.
    #[inline]
    pub const fn declared_ignore_case(&self) -> Option<bool> {
        self.declared_ignore_case
    }

    /// Returns an explicit policy even before generic substitution. Without
    /// an explicit option, returns None for a definition and Some of the
    /// inferred policy for a concrete field.
    ///
    /// # Returns
    ///
    /// The explicit option for deferred definitions, or the effective policy
    /// for concrete metadata.
    #[inline]
    pub const fn effective_ignore_case(&self) -> Option<bool> {
        if self.deferred {
            self.declared_ignore_case
        } else {
            Some(self.ignore_case)
        }
    }

    /// Returns scope paths in source order.
    ///
    /// # Returns
    ///
    /// The declared uniqueness scope paths without reordering.
    #[must_use]
    #[inline]
    pub const fn respect_to(&self) -> &'static [PropertyPath<'static>] {
        self.respect_to
    }

    /// Returns the stored concrete comparison policy. Generic definitions
    /// retain a false placeholder here; use [`Self::effective_ignore_case`] to
    /// distinguish unresolved defaults and preserve explicit source options.
    ///
    /// # Returns
    ///
    /// The stored case-comparison policy, including a placeholder for deferred
    /// generic defaults.
    #[must_use]
    #[inline]
    pub const fn ignore_case(&self) -> bool {
        self.ignore_case
    }

    /// Returns whether uniqueness is scoped by another property.
    ///
    /// # Returns
    ///
    /// `true` when at least one scope path is present.
    #[must_use]
    #[inline]
    pub const fn is_scoped(&self) -> bool {
        !self.respect_to.is_empty()
    }
}

/// Declares the ordering of one key component.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::KeyPartMetadata;
///
/// let part = KeyPartMetadata::new(0);
/// assert_eq!(part.order(), 0);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyPartMetadata {
    /// The zero-based position within the composite key.
    order: usize,
}

impl KeyPartMetadata {
    /// Creates key-part metadata.
    ///
    /// # Parameters
    ///
    /// * `order` - Zero-based position within the composite key.
    ///
    /// # Returns
    ///
    /// Metadata for that component position.
    #[must_use]
    #[inline]
    pub const fn new(order: usize) -> Self {
        Self { order }
    }

    /// Returns the zero-based component order.
    ///
    /// # Returns
    ///
    /// The component's position in the composite key.
    #[must_use]
    #[inline]
    pub const fn order(&self) -> usize {
        self.order
    }
}

/// Distinguishes a Rust-type target from a stable model-ID target.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{DeclaredEntityTarget, DeclaredEntityTargetKind, ModelId};
///
/// let target = DeclaredEntityTarget::ModelId(ModelId::new("example.Customer"));
/// assert_eq!(target.kind(), DeclaredEntityTargetKind::ModelId);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclaredEntityTargetKind {
    /// The target is supplied by a static metadata provider.
    RustType,
    /// The target is supplied as a stable model ID.
    ModelId,
}

/// A declaration-time entity target that does not consult the registry.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{DeclaredEntityTarget, DeclaredEntityTargetKind, ModelId};
///
/// let target = DeclaredEntityTarget::ModelId(ModelId::new("example.Customer"));
/// assert_eq!(target.kind(), DeclaredEntityTargetKind::ModelId);
/// assert_eq!(target.model_id().unwrap().as_str(), "example.Customer");
/// ```
#[derive(Clone, Copy)]
pub enum DeclaredEntityTarget {
    /// A Rust type metadata provider.
    RustType(fn() -> &'static TypeMetadata),
    /// A stable model ID string.
    ModelId(ModelId),
}

impl DeclaredEntityTarget {
    /// Returns the target representation kind.
    ///
    /// # Returns
    ///
    /// Whether this declaration names a Rust type provider or stable model ID.
    #[must_use]
    #[inline]
    pub const fn kind(&self) -> DeclaredEntityTargetKind {
        match self {
            Self::RustType(_) => DeclaredEntityTargetKind::RustType,
            Self::ModelId(_) => DeclaredEntityTargetKind::ModelId,
        }
    }

    /// Calls the Rust-type provider and returns its metadata. Returns None for
    /// a stable-ID target; this method never resolves that ID in a registry.
    /// The provider can initialize caches or perform its own side effects.
    ///
    /// # Returns
    ///
    /// Provider metadata for Rust-type targets, or `None` for model-ID targets.
    ///
    /// # Panics
    /// Propagates any panic raised by the supplied provider.
    pub fn metadata(&self) -> Option<&'static TypeMetadata> {
        match self {
            Self::RustType(provider) => Some(provider()),
            Self::ModelId(_) => None,
        }
    }

    /// Returns the declared stable model ID, if any.
    ///
    /// # Returns
    ///
    /// The stable model ID for that target kind, or `None` for Rust types.
    #[inline]
    pub const fn model_id(&self) -> Option<ModelId> {
        match self {
            Self::RustType(_) => None,
            Self::ModelId(id) => Some(*id),
        }
    }
}

impl fmt::Debug for DeclaredEntityTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("DeclaredEntityTarget")
            .field(&self.kind())
            .finish()
    }
}

/// Selects an entity as a whole or one of its properties.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyPath;
/// use qubit_model_metadata::metadata::ReferenceSelection;
///
/// let selection = ReferenceSelection::Property(PropertyPath::new(&["owner", "id"]));
/// assert!(matches!(selection, ReferenceSelection::Property(path) if path.to_string() == "owner.id"));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceSelection {
    /// Selects the complete entity.
    Entity,
    /// Selects a property path on the entity.
    Property(PropertyPath<'static>),
}

/// Declaration metadata for one entity reference.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{DeclaredEntityTarget, FieldReferenceMetadata, ModelId, ReferenceSelection};
///
/// static TARGET: DeclaredEntityTarget = DeclaredEntityTarget::ModelId(ModelId::new("example.Customer"));
/// static SELECTION: ReferenceSelection = ReferenceSelection::Entity;
/// let reference = FieldReferenceMetadata::new(&TARGET, &SELECTION, true, None);
/// assert!(reference.existing());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ReferenceMetadata {
    /// The entity declaration selected by this reference.
    target: &'static DeclaredEntityTarget,
    /// The entity or property selected from the target.
    selection: &'static ReferenceSelection,
    /// Whether the referenced record must exist before assignment.
    existing: bool,
    /// Object navigation from the declaration to the target context, if
    /// supplied.
    path: Option<&'static ObjectPath>,
}

impl ReferenceMetadata {
    /// Records the target, selected value, existence policy and optional
    /// object navigation without resolving a registry or invoking a provider.
    ///
    /// # Parameters
    ///
    /// * `target` - Static declaration of the referenced entity.
    /// * `selection` - Entity value or property selected from that target.
    /// * `existing` - Whether the referenced record must already exist.
    /// * `path` - Optional object navigation to the target context.
    ///
    /// # Returns
    ///
    /// Reference metadata preserving the declaration without resolving it.
    #[must_use]
    #[inline]
    pub const fn new(
        target: &'static DeclaredEntityTarget,
        selection: &'static ReferenceSelection,
        existing: bool,
        path: Option<&'static ObjectPath>,
    ) -> Self {
        Self {
            target,
            selection,
            existing,
            path,
        }
    }

    /// Returns the declared entity target.
    ///
    /// # Returns
    ///
    /// The static target declaration selected by this reference.
    #[must_use]
    #[inline]
    pub const fn target(&self) -> &'static DeclaredEntityTarget {
        self.target
    }

    /// Returns the selected entity value or property.
    ///
    /// # Returns
    ///
    /// The static selection describing which target value is referenced.
    #[must_use]
    #[inline]
    pub const fn selection(&self) -> &'static ReferenceSelection {
        self.selection
    }

    /// Returns whether the referenced record must already exist.
    ///
    /// # Returns
    ///
    /// `true` when existence is required before assignment.
    #[must_use]
    #[inline]
    pub const fn existing(&self) -> bool {
        self.existing
    }

    /// Returns explicit object navigation, or None when no path was declared.
    /// Property selection is retained separately by [`Self::selection`].
    ///
    /// # Returns
    ///
    /// The declared navigation path, or `None` when the reference is local.
    #[inline]
    pub const fn path(&self) -> Option<&'static ObjectPath> {
        self.path
    }
}

/// Names one validator dependency slot and the property path supplying it.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::DependencyBindingMetadata;
/// use qubit_model_metadata::metadata::PropertyPath;
///
/// let binding = DependencyBindingMetadata::new("owner", PropertyPath::new(&["owner"]));
/// assert_eq!(binding.name(), "owner");
/// assert_eq!(binding.path().to_string(), "owner");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DependencyBindingMetadata {
    /// The name used by the validator signature.
    name: &'static str,
    /// The model property path supplying this dependency.
    path: PropertyPath<'static>,
    /// Navigation from the owning object before selecting the property.
    object_path: ObjectPath,
    /// Source occurrence anchoring navigation to its owning object.
    declaration: &'static DeclarationLocation,
}

impl DependencyBindingMetadata {
    /// Creates a named dependency binding.
    ///
    /// # Parameters
    ///
    /// * `name` - Non-empty validator signature slot name.
    /// * `path` - Non-empty model property path supplying the slot.
    ///
    /// # Returns
    ///
    /// A dependency binding with local object navigation by default.
    ///
    /// # Panics
    /// Panics if the slot name or property path is empty.
    #[must_use]
    #[inline]
    pub const fn new(name: &'static str, path: PropertyPath<'static>) -> Self {
        static UNKNOWN: DeclarationLocation = DeclarationLocation::unknown();
        assert!(!name.is_empty(), "validator dependency name cannot be empty");
        assert!(!path.is_empty(), "validator dependency path cannot be empty");
        Self {
            name,
            path,
            object_path: ObjectPath::current(),
            declaration: &UNKNOWN,
        }
    }

    /// Returns navigation relative to the declaration's owning object.
    ///
    /// # Returns
    ///
    /// The object path used before selecting the dependency property.
    #[must_use]
    #[inline]
    pub const fn object_path(&self) -> ObjectPath {
        self.object_path
    }

    /// Returns the property selected on the navigated object.
    ///
    /// # Returns
    ///
    /// The model property path supplying this dependency slot.
    #[must_use]
    #[inline]
    pub const fn property(&self) -> PropertyPath<'static> {
        self.path
    }

    /// Returns source and owning-object coordinates for this dependency.
    ///
    /// # Returns
    ///
    /// The declaration location associated with this binding.
    #[must_use]
    #[inline]
    pub const fn declaration(&self) -> &'static DeclarationLocation {
        self.declaration
    }

    /// Returns the validator signature slot name.
    ///
    /// # Returns
    ///
    /// The stable parameter name borrowed from static metadata.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the model property path supplying the slot.
    ///
    /// # Returns
    ///
    /// The static path selected for this dependency slot.
    #[must_use]
    #[inline]
    pub const fn path(&self) -> PropertyPath<'static> {
        self.path
    }

    /// Adds object navigation without changing the selected property.
    ///
    /// # Parameters
    ///
    /// * `path` - Navigation from the declaration owner to the property owner.
    ///
    /// # Returns
    ///
    /// This binding with the supplied navigation path.
    #[must_use]
    #[inline]
    pub const fn with_object_path(mut self, path: ObjectPath) -> Self {
        self.object_path = path;
        self
    }

    /// Associates an exact field or selector occurrence with this binding.
    ///
    /// # Parameters
    ///
    /// * `declaration` - Source location identifying the owning occurrence.
    ///
    /// # Returns
    ///
    /// This binding associated with the supplied declaration location.
    #[must_use]
    #[inline]
    pub const fn with_declaration(mut self, declaration: &'static DeclarationLocation) -> Self {
        self.declaration = declaration;
        self
    }
}

/// Selects the value shape supplied to a validator occurrence.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{OnNone, TargetMode, ValidatorMetadata};
///
/// let validator = ValidatorMetadata::new_bound(
///     "example.container",
///     &[],
///     &[],
///     &[],
///     TargetMode::Container,
///     OnNone::Skip,
/// );
/// assert_eq!(validator.target(), TargetMode::Container);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetMode {
    /// Expand supported optional and transparent smart-pointer wrappers.
    Value,
    /// Preserve the declared container type.
    Container,
}

/// Compatibility name for [`TargetMode`].
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{OnNone, ValidationTarget, ValidatorMetadata};
///
/// let target: ValidationTarget = ValidationTarget::Container;
/// let validator = ValidatorMetadata::new_bound("example.container", &[], &[], &[], target, OnNone::Skip);
/// assert_eq!(validator.target(), ValidationTarget::Container);
/// ```
pub type ValidationTarget = TargetMode;

/// Selects the behavior for an absent expanded optional value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{OnNone, TargetMode, ValidatorMetadata};
///
/// let validator = ValidatorMetadata::new_bound("example.required", &[], &[], &[], TargetMode::Value, OnNone::Reject);
/// assert_eq!(validator.on_none(), OnNone::Reject);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnNone {
    /// Skip the validator occurrence.
    Skip,
    /// Reject an absent value before invoking the validator.
    Reject,
}

/// One declared validator occurrence.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{OnNone, TargetMode, ValidatorMetadata};
///
/// let validator = ValidatorMetadata::new("example.required", &[], &[]);
/// assert_eq!(validator.target(), TargetMode::Value);
/// assert_eq!(validator.on_none(), OnNone::Skip);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ValidatorMetadata {
    /// The stable identifier used to resolve the validator registration.
    declared_id: &'static str,
    /// Static arguments passed to the resolved validator.
    params: &'static [NamedValidationArgument<'static>],
    /// Property paths that must be available to the validator.
    depends_on: &'static [PropertyPath<'static>],
    /// Named dependency slots for the typed validator contract.
    dependency_bindings: &'static [DependencyBindingMetadata],
    /// Value shape selected for the occurrence.
    target: TargetMode,
    /// Behavior for an absent expanded value.
    on_none: OnNone,
}

impl ValidatorMetadata {
    /// Creates a validator occurrence without resolving a runtime registry.
    /// Uses value expansion and skips absent optional values.
    ///
    /// # Parameters
    ///
    /// * `declared_id` - Non-empty stable validator registration ID.
    /// * `params` - Static typed arguments in declaration order.
    /// * `depends_on` - Static property dependencies in declaration order.
    ///
    /// # Returns
    ///
    /// A value-target validator occurrence that skips absent optional values.
    ///
    /// # Panics
    /// Panics if `declared_id` is empty.
    #[inline]
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
            dependency_bindings: &[],
            target: TargetMode::Value,
            on_none: OnNone::Skip,
        }
    }

    /// Creates a validator occurrence with named dependency bindings.
    ///
    /// # Parameters
    ///
    /// * `declared_id` - Non-empty stable validator registration ID.
    /// * `params` - Static typed arguments in declaration order.
    /// * `depends_on` - Static property dependencies in declaration order.
    /// * `dependency_bindings` - Named slots mapped to property paths.
    /// * `target` - Whether validation receives expanded values or containers.
    /// * `on_none` - Policy for absent expanded optional values.
    ///
    /// # Returns
    ///
    /// A validator occurrence with its explicit dependency and optional-value
    /// policies.
    ///
    /// # Panics
    /// Panics if the ID is empty, dependency slot names repeat, or container
    /// input is combined with rejection of absent expanded values. Individual
    /// bindings already require nonempty slot names and property paths.
    #[must_use]
    pub const fn new_bound(
        declared_id: &'static str,
        params: &'static [NamedValidationArgument<'static>],
        depends_on: &'static [PropertyPath<'static>],
        dependency_bindings: &'static [DependencyBindingMetadata],
        target: TargetMode,
        on_none: OnNone,
    ) -> Self {
        assert!(!declared_id.is_empty(), "validator ID cannot be empty");
        assert!(
            !(matches!(target, TargetMode::Container) && matches!(on_none, OnNone::Reject)),
            "container validators cannot reject missing expanded values",
        );
        let mut index = 0;
        while index < dependency_bindings.len() {
            let binding = dependency_bindings[index];
            assert!(
                !contains_dependency_name(dependency_bindings, index, binding.name()),
                "validator dependency names must be unique",
            );
            index += 1;
        }
        Self {
            declared_id,
            params,
            depends_on,
            dependency_bindings,
            target,
            on_none,
        }
    }

    /// Returns the nonempty source ID. Full identifier grammar validation
    /// belongs to the declaration producer or consuming registry.
    ///
    /// # Returns
    ///
    /// The static validator registration ID.
    #[must_use]
    #[inline]
    pub const fn declared_id(&self) -> &'static str {
        self.declared_id
    }

    /// Returns parameters in source order.
    ///
    /// # Returns
    ///
    /// Static named arguments in the order declared by the model.
    #[must_use]
    #[inline]
    pub const fn params(&self) -> &'static [NamedValidationArgument<'static>] {
        self.params
    }

    /// Returns dependency paths in source order.
    ///
    /// # Returns
    ///
    /// Static property dependencies in declaration order.
    #[must_use]
    #[inline]
    pub const fn depends_on(&self) -> &'static [PropertyPath<'static>] {
        self.depends_on
    }

    /// Returns named dependency bindings in declaration order.
    ///
    /// # Returns
    ///
    /// Static named slots and their source property bindings.
    #[must_use]
    #[inline]
    pub const fn dependency_bindings(&self) -> &'static [DependencyBindingMetadata] {
        self.dependency_bindings
    }

    /// Returns the selected validator value shape.
    ///
    /// # Returns
    ///
    /// Whether the validator receives expanded values or declared containers.
    #[must_use]
    #[inline]
    pub const fn target(&self) -> TargetMode {
        self.target
    }

    /// Returns the behavior for an absent expanded value.
    ///
    /// # Returns
    ///
    /// Whether the occurrence is skipped or rejected when the value is absent.
    #[must_use]
    #[inline]
    pub const fn on_none(&self) -> OnNone {
        self.on_none
    }
}

/// Reports whether an earlier dependency binding already uses `name`.
///
/// # Parameters
///
/// * `bindings` - Dependency bindings already declared for the validator.
/// * `end` - Exclusive prefix length to inspect.
/// * `name` - Candidate name to compare with that prefix.
///
/// # Returns
///
/// `true` when a binding in the prefix already uses `name`.
#[must_use]
const fn contains_dependency_name(bindings: &[DependencyBindingMetadata], end: usize, name: &str) -> bool {
    let mut index = 0;
    while index < end {
        if same_str(bindings[index].name(), name) {
            return true;
        }
        index += 1;
    }
    false
}

/// Compares two string slices without relying on non-const string equality.
///
/// # Parameters
///
/// * `left` - First byte sequence to compare.
/// * `right` - Second byte sequence to compare.
///
/// # Returns
///
/// `true` when both strings contain the same bytes in the same order.
#[must_use]
const fn same_str(left: &str, right: &str) -> bool {
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

/// A codec declared by Rust type or stable textual ID.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::CodecReference;
///
/// let reference = CodecReference::DeclaredId("example.codec");
/// assert_eq!(reference.declared_id(), Some("example.codec"));
/// ```
#[derive(Clone, Copy, Debug)]
pub enum CodecReference {
    /// A codec identified by its Rust type.
    RustType(RustTypeReference),
    /// A codec identified by its stable registry ID.
    DeclaredId(&'static str),
}

impl CodecReference {
    /// Returns the referenced Rust type, if the declaration uses one.
    ///
    /// # Returns
    ///
    /// The Rust codec type reference, or `None` for a textual ID declaration.
    #[inline]
    pub const fn rust_type(self) -> Option<RustTypeReference> {
        match self {
            Self::RustType(reference) => Some(reference),
            Self::DeclaredId(_) => None,
        }
    }

    /// Returns the declared registry ID, if the declaration uses one.
    ///
    /// # Returns
    ///
    /// The stable codec ID, or `None` when a Rust type identifies the codec.
    #[inline]
    pub const fn declared_id(self) -> Option<&'static str> {
        match self {
            Self::RustType(_) => None,
            Self::DeclaredId(id) => Some(id),
        }
    }
}

/// Identifies where a codec declaration originated.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{CodecMetadata, CodecReference, CodecSource};
///
/// static REFERENCE: CodecReference = CodecReference::DeclaredId("example.codec");
/// let codec = CodecMetadata::new(&REFERENCE, CodecSource::Field);
/// assert_eq!(codec.source(), CodecSource::Field);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CodecSource {
    /// A codec declared directly on a field.
    Field,
    /// A codec declared for a value object's canonical representation.
    CanonicalValue,
    /// A codec declared for a nested selector position.
    Selector(SelectorPosition),
}

/// One codec occurrence.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{CodecMetadata, CodecReference, CodecSource};
///
/// static REFERENCE: CodecReference = CodecReference::DeclaredId("example.codec");
/// let codec = CodecMetadata::new(&REFERENCE, CodecSource::Field);
/// assert_eq!(codec.source(), CodecSource::Field);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CodecMetadata {
    /// The Rust-type or textual codec declaration.
    codec: &'static CodecReference,
    /// The metadata location that supplied the declaration.
    source: CodecSource,
}

impl CodecMetadata {
    /// Creates codec metadata.
    ///
    /// # Parameters
    ///
    /// * `codec` - Static Rust-type or registry-ID declaration.
    /// * `source` - Metadata location that supplied the declaration.
    ///
    /// # Returns
    ///
    /// Codec metadata preserving the declaration and its source.
    #[must_use]
    #[inline]
    pub const fn new(codec: &'static CodecReference, source: CodecSource) -> Self {
        Self { codec, source }
    }

    /// Returns the codec declaration.
    ///
    /// # Returns
    ///
    /// The static codec reference attached to this occurrence.
    #[must_use]
    #[inline]
    pub const fn codec(&self) -> &'static CodecReference {
        self.codec
    }

    /// Returns the declaration source.
    ///
    /// # Returns
    ///
    /// The field, canonical value, or selector source for this declaration.
    #[must_use]
    #[inline]
    pub const fn source(&self) -> CodecSource {
        self.source
    }
}

/// The structural position selected by nested field semantics.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{SelectorMetadata, SelectorPosition};
///
/// let selector = SelectorMetadata::new(SelectorPosition::MapKey, &[], &[], None, None);
/// assert_eq!(selector.position(), SelectorPosition::MapKey);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorPosition {
    /// The element type of a sequence.
    Element,
    /// The key type of a map.
    MapKey,
    /// The value type of a map.
    MapValue,
}

/// Narrow declaration modes delegated to `qubit-redact` capabilities.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::RedactModeMetadata;
///
/// let mode = RedactModeMetadata::KeyedBy("account");
/// assert!(matches!(mode, RedactModeMetadata::KeyedBy("account")));
/// ```
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
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{RedactMetadata, RedactModeMetadata, RedactPosition};
///
/// let redact = RedactMetadata::new(None, RedactModeMetadata::Skip, RedactPosition::MapKey);
/// assert_eq!(redact.position(), RedactPosition::MapKey);
/// ```
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
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{RedactMetadata, RedactModeMetadata, RedactPosition, Sensitivity};
///
/// let redact = RedactMetadata::new(Some(Sensitivity::Personal), RedactModeMetadata::Level, RedactPosition::Field);
/// assert_eq!(redact.sensitivity(), Some(Sensitivity::Personal));
/// ```
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
    ///
    /// # Parameters
    ///
    /// * `sensitivity` - Optional disclosure level required for redaction.
    /// * `mode` - Redaction operation selected by the declaration.
    /// * `position` - Field, sequence element, map key, or map value affected.
    ///
    /// # Returns
    ///
    /// Metadata describing the redaction policy at the selected position.
    #[must_use]
    #[inline]
    pub const fn new(sensitivity: Option<Sensitivity>, mode: RedactModeMetadata, position: RedactPosition) -> Self {
        Self {
            sensitivity,
            mode,
            position,
        }
    }

    /// Returns the configured sensitivity, if applicable.
    ///
    /// # Returns
    ///
    /// The disclosure level, or `None` when the mode does not use one.
    #[inline]
    pub const fn sensitivity(&self) -> Option<Sensitivity> {
        self.sensitivity
    }

    /// Returns the declaration mode.
    ///
    /// # Returns
    ///
    /// The redaction operation supplied by the declaration.
    #[must_use]
    #[inline]
    pub const fn mode(&self) -> RedactModeMetadata {
        self.mode
    }

    /// Returns the affected value position.
    ///
    /// # Returns
    ///
    /// The structural field or collection position affected by redaction.
    #[must_use]
    #[inline]
    pub const fn position(&self) -> RedactPosition {
        self.position
    }
}

/// Final Serde behavior for one field.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{SerdeBehaviorSource, SerdeFieldMetadata};
///
/// let serde = SerdeFieldMetadata::DEFAULT.with_sources(
///     SerdeBehaviorSource::Explicit,
///     SerdeBehaviorSource::Suppressed,
/// );
/// assert_eq!(serde.default_source(), SerdeBehaviorSource::Explicit);
/// ```
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
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::SerdeFieldMetadata;
///
/// let serde = SerdeFieldMetadata::DEFAULT;
/// assert_eq!(serde.serialize_name(), None);
/// assert!(!serde.skip_serializing());
/// ```
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
    ///
    /// # Parameters
    ///
    /// * `serialize_name` - Optional serialized field name.
    /// * `deserialize_name` - Optional deserialized field name.
    /// * `skip_serializing` - Whether serialization omits the field.
    /// * `skip_deserializing` - Whether deserialization ignores the field.
    /// * `flatten` - Whether Serde merges this field into its parent.
    /// * `with` - Optional Serde conversion path.
    /// * `default` - Whether missing values use Serde default behavior.
    ///
    /// # Returns
    ///
    /// Effective field behavior with provenance inferred from explicit flags.
    #[must_use]
    #[inline]
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

    /// Returns the configured serialization name, or `None` when Serde uses
    /// the field name.
    ///
    /// # Returns
    ///
    /// The serialization name override, or `None` to retain the field name.
    #[inline]
    pub const fn serialize_name(&self) -> Option<&'static str> {
        self.serialize_name
    }
    /// Returns the configured deserialization name, or `None` when Serde uses
    /// the field name.
    ///
    /// # Returns
    ///
    /// The deserialization name override, or `None` to retain the field name.
    #[inline]
    pub const fn deserialize_name(&self) -> Option<&'static str> {
        self.deserialize_name
    }
    /// Returns whether Serde omits this field during serialization.
    ///
    /// # Returns
    ///
    /// `true` when serialization skips this field.
    #[must_use]
    #[inline]
    pub const fn skip_serializing(&self) -> bool {
        self.skip_serializing
    }
    /// Returns whether Serde ignores this field during deserialization.
    ///
    /// # Returns
    ///
    /// `true` when deserialization skips this field.
    #[must_use]
    #[inline]
    pub const fn skip_deserializing(&self) -> bool {
        self.skip_deserializing
    }
    /// Returns whether Serde flattens this field into its parent.
    ///
    /// # Returns
    ///
    /// `true` when this field is merged into its parent representation.
    #[must_use]
    #[inline]
    pub const fn flatten(&self) -> bool {
        self.flatten
    }
    /// Returns the configured Serde conversion path, if present.
    ///
    /// # Returns
    ///
    /// The configured conversion path, or `None` when absent.
    #[inline]
    pub const fn with(&self) -> Option<&'static str> {
        self.with
    }
    /// Returns whether a missing value uses Serde's default behavior.
    ///
    /// # Returns
    ///
    /// `true` when missing values are supplied by default behavior.
    #[must_use]
    #[inline]
    pub const fn default(&self) -> bool {
        self.default
    }
    /// Returns the declaration source for missing-value defaults.
    ///
    /// # Returns
    ///
    /// The origin of the missing-value default behavior.
    #[must_use]
    #[inline]
    pub const fn default_source(&self) -> SerdeBehaviorSource {
        self.default_source
    }
    /// Returns the declaration source for empty-value omission.
    ///
    /// # Returns
    ///
    /// The origin of empty-value omission behavior.
    #[must_use]
    #[inline]
    pub const fn omit_source(&self) -> SerdeBehaviorSource {
        self.omit_source
    }

    /// Records whether missing-value defaults and empty-value omission were
    /// explicit, generated, or suppressed.
    ///
    /// # Parameters
    ///
    /// * `default_source` - Provenance of missing-value default behavior.
    /// * `omit_source` - Provenance of empty-value omission behavior.
    ///
    /// # Returns
    ///
    /// This metadata with both behavior provenance values recorded.
    #[must_use]
    #[inline]
    pub const fn with_sources(mut self, default_source: SerdeBehaviorSource, omit_source: SerdeBehaviorSource) -> Self {
        self.default_source = default_source;
        self.omit_source = omit_source;
        self
    }
}

/// Non-recursive semantics applied to a collection position.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{SelectorMetadata, SelectorPosition};
///
/// let selector = SelectorMetadata::new(SelectorPosition::Element, &[], &[], None, None);
/// assert_eq!(selector.position(), SelectorPosition::Element);
/// assert!(selector.codec().is_none());
/// ```
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
    ///
    /// # Parameters
    ///
    /// * `position` - Collection position described by this selector.
    /// * `constraints` - Constraints applied at that position.
    /// * `validators` - Validators applied at that position.
    /// * `codec` - Optional codec declaration for the selected value.
    /// * `redact` - Optional redaction policy for the selected value.
    ///
    /// # Returns
    ///
    /// Metadata for the selected collection position.
    #[must_use]
    #[inline]
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
    ///
    /// # Returns
    ///
    /// The element, map-key, or map-value position represented here.
    #[must_use]
    #[inline]
    pub const fn position(&self) -> SelectorPosition {
        self.position
    }
    /// Returns constraints in source order.
    ///
    /// # Returns
    ///
    /// Constraints declared for this position without reordering.
    #[must_use]
    #[inline]
    pub const fn constraints(&self) -> &'static [ConstraintMetadata] {
        self.constraints
    }
    /// Returns validators in source order.
    ///
    /// # Returns
    ///
    /// Validators declared for this position without reordering.
    #[must_use]
    #[inline]
    pub const fn validators(&self) -> &'static [ValidatorMetadata] {
        self.validators
    }
    /// Returns the codec declaration, or `None` when absent.
    ///
    /// # Returns
    ///
    /// The selector codec metadata, or `None` when no codec was declared.
    #[inline]
    pub const fn codec(&self) -> Option<&'static CodecMetadata> {
        self.codec
    }
    /// Returns the redaction declaration, or `None` when absent.
    ///
    /// # Returns
    ///
    /// The selector redaction metadata, or `None` when absent.
    #[inline]
    pub const fn redact(&self) -> Option<&'static RedactMetadata> {
        self.redact
    }
}

/// Source-order view over the same strongly typed field objects.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{FieldAttributeMetadata, IdentifierAssignment, IdentifierMetadata};
///
/// static IDENTIFIER: IdentifierMetadata = IdentifierMetadata::new(IdentifierAssignment::Database);
/// let attribute = FieldAttributeMetadata::Identifier(&IDENTIFIER);
/// assert!(matches!(
///     attribute,
///     FieldAttributeMetadata::Identifier(value)
///         if value.assigned_by() == IdentifierAssignment::Database
/// ));
/// ```
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

/// Recognizes text through supported optional and smart-pointer wrappers.
///
/// # Parameters
///
/// * `ty` - Reflected type whose resolved descriptor is inspected.
///
/// # Returns
///
/// `true` when the type is text or wraps a supported text type.
#[must_use]
pub(crate) fn text_reference(ty: &TypeRef) -> bool {
    let Some(descriptor) = ty.as_resolved() else {
        return false;
    };
    if descriptor.as_text().is_some() {
        return true;
    }
    descriptor
        .as_optional()
        .map(|value| value.element_type())
        .or_else(|| descriptor.as_smart_pointer().map(|value| value.pointee_type()))
        .is_some_and(text_reference)
}
