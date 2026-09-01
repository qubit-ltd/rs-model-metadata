// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Structured failures at the generated-code ABI boundary.

/// A generated metadata aggregate disagrees with its reflection descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{code}: {message}")]
pub struct AbiViolation {
    /// Stable machine-readable ABI diagnostic code.
    code: &'static str,
    /// Stable human-readable explanation.
    message: &'static str,
}

impl AbiViolation {
    /// Creates one ABI violation.
    #[must_use]
    pub(crate) const fn new(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }

    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Returns the stable diagnostic message.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.message
    }
}
