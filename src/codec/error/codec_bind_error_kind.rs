// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Machine-readable codec binding failure class.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::codec::CodecBindErrorKind;
///
/// let kind = CodecBindErrorKind::ValueTypeMismatch;
/// let description = match kind {
///     CodecBindErrorKind::Missing => "no registration matched",
///     CodecBindErrorKind::Ambiguous => "multiple registrations matched",
///     CodecBindErrorKind::ValueTypeMismatch => "the value type did not match",
/// };
/// assert_eq!(description, "the value type did not match");
/// ```
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CodecBindErrorKind {
    /// No registration matches the declaration.
    Missing,
    /// More than one registration matches a Rust codec type.
    Ambiguous,
    /// The codec operates on a different value type.
    ValueTypeMismatch,
}
