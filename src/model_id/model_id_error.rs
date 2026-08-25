// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Errors returned when stable model IDs violate the ID protocol.

/// A reason a model ID does not follow the stable-ID protocol.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::ModelId;
/// use qubit_model_metadata::ModelIdError;
///
/// assert_eq!(ModelId::try_new(""), Err(ModelIdError::Empty));
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ModelIdError {
    /// The complete ID is empty.
    Empty,
    /// The ID contains an empty dot-separated segment.
    EmptySegment,
    /// A module segment is not an ASCII snake-case identifier.
    InvalidModuleSegment,
    /// The final type segment is not an ASCII UpperCamelCase identifier.
    InvalidTypeSegment,
    /// A module segment is a Rust keyword.
    KeywordModuleSegment,
}

impl core::fmt::Display for ModelIdError {
    /// Formats a concise explanation of the invalid model-ID component.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "model ID cannot be empty",
            Self::EmptySegment => "model ID cannot contain empty segments",
            Self::InvalidModuleSegment => "model ID has an invalid module segment",
            Self::InvalidTypeSegment => "model ID has an invalid type segment",
            Self::KeywordModuleSegment => "model ID module segments cannot be Rust keywords",
        })
    }
}

impl std::error::Error for ModelIdError {}
