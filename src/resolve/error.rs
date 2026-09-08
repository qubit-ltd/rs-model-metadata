// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Deterministic structural resolution diagnostics.
// qubit-style: allow multiple-public-types

use std::any::TypeId;

use qubit_reflect::identity::FragmentIdentity;

use super::owned_property_path::OwnedPropertyPath;
use crate::metadata::ModelMetadataError;
use crate::metadata::ModelRole;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyResolutionError;
use crate::metadata::TypeMetadata;
/// Machine-readable model resolution error class.
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
#[derive(Clone, Debug, thiserror::Error)]
pub enum ModelResolutionCause {
    /// Reflected metadata could not be obtained or validated.
    #[error(transparent)]
    Metadata(#[from] ModelMetadataError),
    /// A property set could not be obtained or assembled.
    #[error(transparent)]
    Properties(#[from] PropertyResolutionError),
}

/// One structured deterministic resolution error.
#[must_use]
#[derive(Clone, Debug)]
pub struct ResolveError {
    /// The machine-readable resolution failure class.
    kind: ResolveErrorKind,
    /// The involved property path, when the failure identifies one.
    pub(super) path: Option<OwnedPropertyPath>,
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
            path: path
                .map(|path| OwnedPropertyPath::from_segments(path.segments())),
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
    pub(super) fn resolution(
        root: &'static TypeMetadata,
        path: Option<PropertyPath<'_>>,
        source: &FragmentIdentity,
        cause: impl Into<ModelResolutionCause>,
    ) -> Self {
        let cause = cause.into();
        let kind = match &cause {
            ModelResolutionCause::Metadata(_) => {
                ResolveErrorKind::MetadataResolution
            }
            ModelResolutionCause::Properties(_) => {
                ResolveErrorKind::PropertyResolution
            }
        };
        let mut error = Self::new(
            kind,
            root.model_id().map(|id| id.as_str()),
            path,
            None,
            Some(root.role()),
            Some(source),
        );
        error.cause = Some(cause);
        error
    }

    /// Attaches the original failure to an already classified diagnostic.
    pub(super) fn with_cause(mut self, cause: ModelResolutionCause) -> Self {
        self.cause = Some(cause);
        self
    }

    /// Returns the original structured failure, when present.
    #[must_use]
    pub const fn cause(&self) -> Option<&ModelResolutionCause> {
        self.cause.as_ref()
    }

    /// Adds the expected and observed type identities to this error.
    pub(super) fn with_types(
        mut self,
        expected: TypeId,
        actual: TypeId,
    ) -> Self {
        self.expected_type = Some(expected);
        self.actual_type = Some(actual);
        self
    }

    /// Orders errors by kind, model, path, and source identity.
    pub(super) fn compare(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.kind
            .cmp(&right.kind)
            .then_with(|| left.model_id.cmp(&right.model_id))
            .then_with(|| {
                left.path
                    .as_ref()
                    .map(|path: &OwnedPropertyPath| path.as_path().to_string())
                    .cmp(&right.path.as_ref().map(
                        |path: &OwnedPropertyPath| path.as_path().to_string(),
                    ))
            })
            .then_with(|| left.sources.cmp(&right.sources))
    }

    /// Returns the machine-readable resolution failure class.
    #[must_use]
    pub const fn kind(&self) -> ResolveErrorKind {
        self.kind
    }
    /// Returns the involved path, or `None` when the failure is model-wide.
    #[must_use]
    pub fn path(&self) -> Option<PropertyPath<'_>> {
        self.path.as_ref().map(OwnedPropertyPath::as_path)
    }
    /// Returns the involved stable model ID, when present.
    #[must_use]
    pub const fn model_id(&self) -> Option<&str> {
        self.model_id
    }
    /// Returns the expected role, when role matching was required.
    #[must_use]
    pub const fn expected_role(&self) -> Option<ModelRole> {
        self.expected_role
    }
    /// Returns the actual role, when role matching was required.
    #[must_use]
    pub const fn actual_role(&self) -> Option<ModelRole> {
        self.actual_role
    }
    /// Returns the expected type identity, when type matching was required.
    #[must_use]
    pub const fn expected_type(&self) -> Option<TypeId> {
        self.expected_type
    }
    /// Returns the actual type identity, when type matching was required.
    #[must_use]
    pub const fn actual_type(&self) -> Option<TypeId> {
        self.actual_type
    }
    /// Returns the fragment identities involved in this failure.
    #[must_use]
    pub fn sources(&self) -> &[FragmentIdentity] {
        &self.sources
    }
}

impl core::fmt::Display for ResolveError {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(formatter, "model resolution failed: {:?}", self.kind)
    }
}

/// All errors from one complete failed resolution pass.
#[must_use]
#[derive(Debug)]
pub struct ResolveErrors {
    /// All deterministic failures collected during one resolution pass.
    pub(super) errors: Vec<ResolveError>,
}

impl ResolveErrors {
    /// Returns every collected resolution failure.
    #[must_use = "inspect the resolution failures"]
    #[inline(always)]
    pub fn errors(&self) -> &[ResolveError] {
        &self.errors
    }

    /// Consumes this collection and returns its failures.
    #[must_use]
    pub fn into_vec(self) -> Vec<ResolveError> {
        self.errors
    }
}

impl core::fmt::Display for ResolveErrors {
    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(formatter, "{} model resolution error(s)", self.errors.len())
    }
}

impl std::error::Error for ResolveErrors {}
impl std::error::Error for ResolveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|cause| cause as &dyn std::error::Error)
    }
}
