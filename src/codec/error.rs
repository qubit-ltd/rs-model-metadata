//! Codec binding diagnostics.
// qubit-style: allow multiple-public-types

use core::any::TypeId;

use qubit_codec::ValueCodecRegistrationSource;

use super::CodecOccurrenceId;
use crate::metadata::CodecReference;

/// Machine-readable codec binding failure class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CodecBindErrorKind {
    /// No registration matches the declaration.
    Missing,
    /// More than one registration matches a Rust codec type.
    Ambiguous,
    /// The codec operates on a different value type.
    ValueTypeMismatch,
}

/// One codec binding failure.
#[derive(Clone, Debug)]
pub struct CodecBindError {
    kind: CodecBindErrorKind,
    occurrence: CodecOccurrenceId,
    declaration: CodecReference,
    expected_type: TypeId,
    actual_type: Option<TypeId>,
    candidate_sources: Box<[ValueCodecRegistrationSource]>,
}

impl CodecBindError {
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
    #[must_use]
    pub const fn kind(&self) -> CodecBindErrorKind {
        self.kind
    }

    /// Returns the stable declaration occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> &CodecOccurrenceId {
        &self.occurrence
    }

    /// Returns the codec declaration.
    #[must_use]
    pub const fn declaration(&self) -> CodecReference {
        self.declaration
    }

    /// Returns the required value type.
    #[must_use]
    pub const fn expected_type(&self) -> TypeId {
        self.expected_type
    }

    /// Returns the registered value type when one was selected.
    #[must_use]
    pub const fn actual_type(&self) -> Option<TypeId> {
        self.actual_type
    }

    /// Returns the source locations of registrations considered for this
    /// occurrence, in deterministic order.
    #[must_use]
    pub const fn candidate_sources(&self) -> &[ValueCodecRegistrationSource] {
        &self.candidate_sources
    }
}

impl core::fmt::Display for CodecBindError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "codec binding failed at {}: {:?}",
            self.occurrence, self.kind
        )
    }
}

impl std::error::Error for CodecBindError {}

/// All deterministically ordered codec binding failures.
#[derive(Debug)]
pub struct CodecBindErrors(Box<[CodecBindError]>);

impl CodecBindErrors {
    pub(crate) fn new(mut errors: Vec<CodecBindError>) -> Self {
        errors.sort_by(|left, right| {
            left.occurrence
                .cmp(&right.occurrence)
                .then_with(|| left.kind.cmp(&right.kind))
        });
        Self(errors.into_boxed_slice())
    }

    /// Returns all failures.
    #[must_use]
    pub fn errors(&self) -> &[CodecBindError] {
        &self.0
    }

    /// Consumes the collection.
    #[must_use]
    pub fn into_vec(self) -> Vec<CodecBindError> {
        self.0.into_vec()
    }
}

impl core::fmt::Display for CodecBindErrors {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{} codec binding error(s)", self.0.len())
    }
}

impl std::error::Error for CodecBindErrors {}
