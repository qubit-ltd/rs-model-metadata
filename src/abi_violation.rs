// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structured failures at the generated-code ABI boundary.

use thiserror::Error;

/// A generated metadata aggregate disagrees with its reflection descriptor.
///
/// `AbiViolation` is returned by metadata validation APIs. Callers can inspect
/// its stable code and explanation when handling the error.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::AbiViolation;
///
/// fn describe(result: Result<(), AbiViolation>) -> Option<String> {
///     result
///         .err()
///         .map(|error| format!("{}: {}", error.code(), error.message()))
/// }
///
/// assert_eq!(describe(Ok(())), None);
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
#[error("{code}: {message}")]
pub struct AbiViolation {
    /// Stable machine-readable ABI diagnostic code.
    code: &'static str,
    /// Stable human-readable explanation.
    message: &'static str,
}

impl AbiViolation {
    /// Creates one ABI violation.
    ///
    /// # Parameters
    ///
    /// * `code` - The stable machine-readable diagnostic code.
    /// * `message` - The stable human-readable explanation.
    ///
    /// # Returns
    ///
    /// An ABI violation containing the supplied code and message.
    pub(crate) const fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    #[inline]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Returns the stable diagnostic message.
    #[must_use]
    #[inline]
    pub const fn message(&self) -> &'static str {
        self.message
    }
}
