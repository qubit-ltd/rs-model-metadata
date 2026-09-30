// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deterministic structural resolution diagnostics.
// qubit-style: allow multiple-public-types

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;
use std::any::TypeId;
use std::cmp::Ordering;
use std::error::Error;

use qubit_reflect::identity::FragmentIdentity;
use thiserror::Error as ThisError;

use super::internal::OwnedPropertyPath;
use crate::metadata::DeclarationLocation;
use crate::metadata::ModelMetadataError;
use crate::metadata::ModelRole;
use crate::metadata::ObjectPath;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyResolutionError;
use crate::metadata::TypeMetadata;

/// Machine-readable model resolution error class.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::resolve::ResolveErrorKind;
///
/// let kind = ResolveErrorKind::MissingProperty;
/// assert_eq!(kind, ResolveErrorKind::MissingProperty);
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ResolveErrorKind {
    /// A reflected model capability or its ABI could not be resolved.
    MetadataResolution,
    /// Property capabilities could not be resolved.
    PropertyResolution,
    /// Field and method fragments could not form a valid local property set.
    InvalidProperties,
    /// An Entity embeds another Entity or Projection without a reference.
    InvalidEntityNesting,
    /// An opaque field attempts to hide a registered model role.
    OpaqueModel,
    /// A readable Entity property violates a Projection source contract.
    InvalidProjectionProducer,
    /// The target model ID could not be resolved.
    MissingModelId,
    /// The resolved model has a role incompatible with the declaration.
    WrongModelRole,
    /// A referenced property does not exist.
    MissingProperty,
    /// A referenced property exists but cannot be read.
    UnreadableProperty,
    /// The expected and actual types differ.
    TypeMismatch,
    /// A nested selector type cannot be resolved.
    UnresolvedSelectorType,
    /// A projection source is missing or has the wrong role.
    InvalidProjectionSource,
    /// A value model contains a non-value nested type.
    InvalidValueClosure,
    /// Two query paths flatten to the same external name.
    QueryNameConflict,
}

/// The original failure retained by a contextual model resolution error.
///
/// # Examples
///
/// ```
/// use std::any::TypeId;
///
/// use qubit_model_metadata::metadata::PropertyResolutionError;
/// use qubit_model_metadata::resolve::ModelResolutionCause;
/// use qubit_reflect::capability::CapabilityAccessError;
/// use qubit_reflect::identity::CapabilityId;
///
/// let property_error = PropertyResolutionError::Capability(CapabilityAccessError::FactOnly {
///     id: CapabilityId::new("example.fact").expect("valid capability ID"),
///     adapter_type: TypeId::of::<u32>(),
/// });
/// let cause = ModelResolutionCause::from(property_error);
/// assert!(matches!(cause, ModelResolutionCause::Properties(_)));
/// ```
#[must_use]
#[derive(Clone, Debug, ThisError)]
pub enum ModelResolutionCause {
    /// Reflected metadata could not be obtained or validated.
    #[error(transparent)]
    Metadata(#[from] ModelMetadataError),
    /// A property set could not be obtained or assembled.
    #[error(transparent)]
    Properties(#[from] PropertyResolutionError),
}

/// One structured deterministic resolution error.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Value;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::{ResolveErrorKind, ResolveInputs, StructureResolver};
/// use qubit_reflect::registry::RegistrySnapshotBuilder;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let reflection = RegistrySnapshotBuilder::new().build()?;
/// let models = ModelRegistry::from_reflect_registry(&reflection)?;
/// let roots = [TypeMetadata::of::<InvalidValue>()];
/// let errors = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve()
///     .unwrap_err();
/// assert!(errors.errors().iter().any(|error| error.kind() == ResolveErrorKind::InvalidValueClosure));
///
/// #[Value]
/// struct InvalidValue { nested: PlainModel }
/// struct PlainModel;
/// # Ok(())
/// # }
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct ResolveError {
    /// The machine-readable resolution failure class.
    kind: ResolveErrorKind,
    /// Concrete owner identity, independent from stable registration.
    owner: Option<(TypeId, &'static str)>,
    /// Exact declaration position, including unnamed Enum payloads.
    declaration: Option<DeclarationLocation>,
    /// The involved property path, when the failure identifies one.
    pub(super) path: Option<OwnedPropertyPath>,
    /// Object navigation remains separate from the selected Property path.
    object_path: Option<ObjectPath>,
    /// The involved stable model ID, when the failure identifies one.
    model_id: Option<&'static str>,
    /// The role expected by the resolution step, when applicable.
    expected_role: Option<ModelRole>,
    /// The role actually observed by the resolution step, when applicable.
    actual_role: Option<ModelRole>,
    /// The type expected by the resolution step, when applicable.
    expected_type: Option<TypeId>,
    /// The type actually observed by the resolution step, when applicable.
    actual_type: Option<TypeId>,
    /// Fragment identities involved in the failure.
    sources: Vec<FragmentIdentity>,
    /// Original structured failure, if this error crosses a fallible boundary.
    cause: Option<ModelResolutionCause>,
}

impl ResolveError {
    /// Creates one resolution error with optional contextual details.
    ///
    /// # Parameters
    ///
    /// - `kind`: stable category for this failure.
    /// - `model_id`: registered model identifier, if available.
    /// - `path`: involved property path, if the failure is property-specific.
    /// - `expected_role`: required model role, when role matching applies.
    /// - `actual_role`: resolved role, when role matching applies.
    /// - `source`: fragment identity that contributed to the failure.
    ///
    /// # Returns
    ///
    /// An error retaining the supplied context and owned property path.
    pub(super) fn new(
        kind: ResolveErrorKind,
        model_id: Option<&'static str>,
        path: Option<PropertyPath<'_>>,
        expected_role: Option<ModelRole>,
        actual_role: Option<ModelRole>,
        source: Option<&FragmentIdentity>,
    ) -> Self {
        Self {
            kind,
            owner: None,
            declaration: None,
            object_path: None,
            path: path.map(|path| OwnedPropertyPath::from_segments(path.segments())),
            model_id,
            expected_role,
            actual_role,
            expected_type: None,
            actual_type: None,
            sources: source.into_iter().cloned().collect(),
            cause: None,
        }
    }

    /// Creates a contextual error without discarding its underlying cause.
    ///
    /// # Parameters
    ///
    /// - `root`: metadata whose resolution failed.
    /// - `path`: property path associated with the failure, if any.
    /// - `source`: contributing fragment identity, if any.
    /// - `cause`: original metadata or property resolution failure.
    ///
    /// # Returns
    ///
    /// A classified error retaining the root identity and original cause.
    pub(super) fn resolution(
        root: &'static TypeMetadata,
        path: Option<PropertyPath<'_>>,
        source: Option<&FragmentIdentity>,
        cause: impl Into<ModelResolutionCause>,
    ) -> Self {
        let cause = cause.into();
        let kind = match &cause {
            ModelResolutionCause::Metadata(_) => ResolveErrorKind::MetadataResolution,
            ModelResolutionCause::Properties(_) => ResolveErrorKind::PropertyResolution,
        };
        let mut error = Self::new(
            kind,
            root.model_id().map(|id| id.as_str()),
            path,
            None,
            Some(root.role()),
            source,
        );
        error.owner = Some((root.type_id(), root.type_name()));
        error.cause = Some(cause);
        error
    }

    /// Returns the declaration's typed object navigation, if applicable.
    ///
    /// # Returns
    ///
    /// `Some` contains the navigation path; `None` means no object path was
    /// attached.
    #[must_use]
    pub const fn object_path(&self) -> Option<&ObjectPath> {
        self.object_path.as_ref()
    }

    /// Returns the concrete owner type, including anonymous models.
    ///
    /// # Returns
    ///
    /// `Some` is the process-local `TypeId`; `None` means the error has no
    /// concrete owner metadata.
    #[must_use]
    pub fn owner_type_id(&self) -> Option<TypeId> {
        self.owner.map(|(id, _)| id)
    }

    /// Returns the diagnostic Rust name of the concrete owner.
    ///
    /// # Returns
    ///
    /// `Some` is the owner's Rust type name; `None` means no owner was
    /// attached.
    #[must_use]
    pub fn owner_type_name(&self) -> Option<&'static str> {
        self.owner.map(|(_, name)| name)
    }

    /// Returns the original field position when the error concerns a field.
    ///
    /// # Returns
    ///
    /// The copied declaration location, or `None` for model-wide errors.
    #[must_use]
    pub const fn declaration(&self) -> Option<DeclarationLocation> {
        self.declaration
    }

    /// Returns the original structured failure, when present.
    ///
    /// # Returns
    ///
    /// `Some` is the retained metadata or property failure; `None` means the
    /// error was classified without an underlying cause.
    #[must_use]
    pub const fn cause(&self) -> Option<&ModelResolutionCause> {
        self.cause.as_ref()
    }

    /// Returns the machine-readable resolution failure class.
    ///
    /// # Returns
    ///
    /// The stable category assigned when this diagnostic was created.
    #[must_use]
    pub const fn kind(&self) -> ResolveErrorKind {
        self.kind
    }

    /// Returns the involved path, or `None` when the failure is model-wide.
    ///
    /// # Returns
    ///
    /// `Some` borrows the owned property path; `None` means the diagnostic
    /// applies to the model as a whole.
    #[must_use]
    pub fn path(&self) -> Option<PropertyPath<'_>> {
        self.path.as_ref().map(OwnedPropertyPath::as_path)
    }

    /// Returns the involved stable model ID, when present.
    ///
    /// # Returns
    ///
    /// `Some` is the registered model ID; `None` means no stable ID is known.
    #[must_use]
    pub const fn model_id(&self) -> Option<&str> {
        self.model_id
    }

    /// Returns the expected role, when role matching was required.
    ///
    /// # Returns
    ///
    /// `Some` is the required role; `None` means this failure did not require
    /// role matching.
    #[must_use]
    pub const fn expected_role(&self) -> Option<ModelRole> {
        self.expected_role
    }

    /// Returns the actual role, when role matching was required.
    ///
    /// # Returns
    ///
    /// `Some` is the resolved role; `None` means no actual role was observed.
    #[must_use]
    pub const fn actual_role(&self) -> Option<ModelRole> {
        self.actual_role
    }

    /// Returns the expected type identity, when type matching was required.
    ///
    /// # Returns
    ///
    /// `Some` is the expected `TypeId`; `None` means no type comparison was
    /// recorded.
    #[must_use]
    pub const fn expected_type(&self) -> Option<TypeId> {
        self.expected_type
    }

    /// Returns the actual type identity, when type matching was required.
    ///
    /// # Returns
    ///
    /// `Some` is the actual `TypeId`; `None` means no type comparison was
    /// recorded.
    #[must_use]
    pub const fn actual_type(&self) -> Option<TypeId> {
        self.actual_type
    }

    /// Returns the fragment identities involved in this failure.
    ///
    /// # Returns
    ///
    /// A borrowed slice of contributing fragment identities; it is empty when
    /// the diagnostic has no fragment source.
    #[must_use]
    pub fn sources(&self) -> &[FragmentIdentity] {
        &self.sources
    }

    /// Attaches object navigation without converting Parent into a property
    /// name.
    ///
    /// # Parameters
    ///
    /// - `path`: typed navigation path associated with the failure.
    ///
    /// # Returns
    ///
    /// This error with the object navigation attached.
    pub(super) fn with_object_path(mut self, path: &ObjectPath) -> Self {
        self.object_path = Some(*path);
        self
    }

    /// Adds concrete owner identity without inventing a stable ID.
    ///
    /// # Parameters
    ///
    /// - `owner`: metadata for the concrete owner type.
    pub(super) fn attach_owner(&mut self, owner: &'static TypeMetadata) {
        self.owner.get_or_insert((owner.type_id(), owner.type_name()));
    }

    /// Associates a field diagnostic with its original declaration.
    ///
    /// # Parameters
    ///
    /// - `declaration`: exact source declaration location.
    ///
    /// # Returns
    ///
    /// This error with the declaration location attached.
    pub(super) fn with_declaration(mut self, declaration: DeclarationLocation) -> Self {
        self.attach_declaration(declaration);
        self
    }

    /// Adds exact declaration source facts.
    ///
    /// # Parameters
    ///
    /// - `declaration`: exact source declaration location.
    pub(super) fn attach_declaration(&mut self, declaration: DeclarationLocation) {
        self.declaration = Some(declaration);
    }

    /// Attaches the original failure to an already classified diagnostic.
    ///
    /// # Parameters
    ///
    /// - `cause`: underlying structured failure to preserve.
    ///
    /// # Returns
    ///
    /// This error retaining the supplied cause.
    pub(super) fn with_cause(mut self, cause: ModelResolutionCause) -> Self {
        self.cause = Some(cause);
        self
    }

    /// Adds the expected and observed type identities to this error.
    ///
    /// # Parameters
    ///
    /// - `expected`: required Rust type identity.
    /// - `actual`: observed Rust type identity.
    ///
    /// # Returns
    ///
    /// This error with both type identities attached.
    pub(super) fn with_types(mut self, expected: TypeId, actual: TypeId) -> Self {
        self.expected_type = Some(expected);
        self.actual_type = Some(actual);
        self
    }

    /// Orders errors by kind, model, path, and source identity.
    ///
    /// # Parameters
    ///
    /// - `left`: first diagnostic to compare.
    /// - `right`: second diagnostic to compare.
    ///
    /// # Returns
    ///
    /// Their deterministic ordering by classification and contextual identity.
    pub(super) fn compare(left: &Self, right: &Self) -> Ordering {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.model_id.cmp(&right.model_id))
            .then_with(|| left.owner_type_name().cmp(&right.owner_type_name()))
            .then_with(|| {
                left.path
                    .as_ref()
                    .map(|path: &OwnedPropertyPath| path.as_path().to_string())
                    .cmp(
                        &right
                            .path
                            .as_ref()
                            .map(|path: &OwnedPropertyPath| path.as_path().to_string()),
                    )
            })
            .then_with(|| left.sources.cmp(&right.sources))
    }
}

impl Display for ResolveError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "model resolution failed: {:?}", self.kind)
    }
}

/// All errors from one complete failed resolution pass.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Value;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::{ResolveErrorKind, ResolveInputs, StructureResolver};
/// use qubit_reflect::registry::RegistrySnapshotBuilder;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let reflection = RegistrySnapshotBuilder::new().build()?;
/// let models = ModelRegistry::from_reflect_registry(&reflection)?;
/// let roots = [TypeMetadata::of::<InvalidValue>()];
/// let errors = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve()
///     .unwrap_err();
/// assert_eq!(errors.errors()[0].kind(), ResolveErrorKind::InvalidValueClosure);
///
/// #[Value]
/// struct InvalidValue { nested: PlainModel }
/// struct PlainModel;
/// # Ok(())
/// # }
/// ```
#[must_use]
#[derive(Debug)]
pub struct ResolveErrors {
    /// All deterministic failures collected during one resolution pass.
    pub(super) errors: Vec<ResolveError>,
}

impl ResolveErrors {
    /// Returns every collected resolution failure.
    ///
    /// # Returns
    ///
    /// A borrowed slice in deterministic diagnostic order; it is empty only
    /// when no failures were collected.
    #[must_use = "inspect the resolution failures"]
    #[inline]
    pub fn errors(&self) -> &[ResolveError] {
        &self.errors
    }

    /// Consumes this collection and returns its failures.
    ///
    /// # Returns
    ///
    /// The owned diagnostics in deterministic order.
    #[must_use]
    pub fn into_vec(self) -> Vec<ResolveError> {
        self.errors
    }
}

impl Display for ResolveErrors {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{} model resolution error(s)", self.errors.len())
    }
}

impl Error for ResolveErrors {}
impl Error for ResolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(|cause| cause as &dyn Error)
    }
}
