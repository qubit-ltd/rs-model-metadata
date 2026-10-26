// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::any::TypeId;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use std::error::Error;

use qubit_codec::ValueCodecRegistrationSource;

use super::codec_bind_error_kind::CodecBindErrorKind;
use crate::codec::CodecOccurrenceId;
use crate::metadata::CodecReference;

/// One codec binding failure.
///
/// The binding process returns this diagnostic inside
/// [`CodecBindErrors`](crate::codec::CodecBindErrors) when
/// [`bind_codecs`](crate::codec::bind_codecs) cannot select a registration.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::codec::CodecBindError;
///
/// fn inspect(error: &CodecBindError) {
///     let _ = error.kind();
///     let _ = error.occurrence();
///     let _ = error.declaration();
///     let _ = error.expected_type();
///     let _ = error.actual_type();
///     let _ = error.candidate_sources();
/// }
/// # let _ = inspect;
/// ```
#[derive(Clone, Debug)]
pub struct CodecBindError {
    /// Failure category reported to the caller.
    kind: CodecBindErrorKind,
    /// Stable identity of the model declaration that failed to bind.
    occurrence: CodecOccurrenceId,
    /// Codec reference declared by the model metadata.
    declaration: CodecReference,
    /// Runtime value type required by the declaration.
    expected_type: TypeId,
    /// Registered value type, when a unique candidate was selected.
    actual_type: Option<TypeId>,
    /// Registration sources considered during binding, sorted
    /// deterministically.
    candidate_sources: Box<[ValueCodecRegistrationSource]>,
}

impl CodecBindError {
    /// Creates a codec binding diagnostic and normalizes candidate ordering.
    ///
    /// # Parameters
    ///
    /// * `kind` - Failure category reported to the caller.
    /// * `occurrence` - Model declaration occurrence that could not be bound.
    /// * `declaration` - Codec reference from the model metadata.
    /// * `expected_type` - Runtime value type required by the declaration.
    /// * `actual_type` - Registered value type when a candidate was selected.
    /// * `candidate_sources` - Registration sources considered during binding.
    ///
    /// # Returns
    ///
    /// A diagnostic with candidate sources sorted for deterministic reporting.
    pub(crate) fn new(
        kind: CodecBindErrorKind,
        occurrence: CodecOccurrenceId,
        declaration: CodecReference,
        expected_type: TypeId,
        actual_type: Option<TypeId>,
        mut candidate_sources: Vec<ValueCodecRegistrationSource>,
    ) -> Self {
        candidate_sources.sort_unstable();
        Self {
            kind,
            occurrence,
            declaration,
            expected_type,
            actual_type,
            candidate_sources: candidate_sources.into_boxed_slice(),
        }
    }

    /// Returns the failure class.
    ///
    /// # Returns
    ///
    /// The missing, ambiguous, or value-type mismatch category.
    #[must_use]
    #[inline]
    pub const fn kind(&self) -> CodecBindErrorKind {
        self.kind
    }

    /// Returns the stable declaration occurrence.
    ///
    /// # Returns
    ///
    /// The identity of the model declaration that failed to bind.
    #[must_use]
    #[inline]
    pub const fn occurrence(&self) -> &CodecOccurrenceId {
        &self.occurrence
    }

    /// Returns the codec declaration.
    ///
    /// # Returns
    ///
    /// The declared codec reference associated with the failure.
    #[must_use]
    #[inline]
    pub const fn declaration(&self) -> CodecReference {
        self.declaration
    }

    /// Returns the required value type.
    ///
    /// # Returns
    ///
    /// The runtime value type expected by the model declaration.
    #[must_use]
    #[inline]
    pub const fn expected_type(&self) -> TypeId {
        self.expected_type
    }

    /// Returns the registered value type when one was selected.
    ///
    /// # Returns
    ///
    /// The selected registration's value type, or `None` when no unique
    /// registration was selected.
    #[inline]
    pub const fn actual_type(&self) -> Option<TypeId> {
        self.actual_type
    }

    /// Returns the source locations of registrations considered for this
    /// occurrence, in deterministic order.
    ///
    /// # Returns
    ///
    /// Registration sources considered for the occurrence, sorted by source.
    #[must_use]
    #[inline]
    pub const fn candidate_sources(&self) -> &[ValueCodecRegistrationSource] {
        &self.candidate_sources
    }
}

impl Display for CodecBindError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "codec binding failed at {}: {:?}",
            self.occurrence, self.kind
        )
    }
}

impl Error for CodecBindError {}
