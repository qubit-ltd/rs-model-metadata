// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Source locations associated with static model registrations.

use std::fmt;

/// The source location where a model registration was declared.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SourceLocation {
    /// The source file containing the registration.
    file: &'static str,
    /// The one-based source line containing the registration.
    line: u32,
    /// The one-based source column containing the registration.
    column: u32,
}

impl SourceLocation {
    /// Creates a source location from its file, line, and column.
    ///
    /// # Parameters
    ///
    /// * `file` - The source file containing the registration.
    /// * `line` - The one-based source line.
    /// * `column` - The one-based source column.
    ///
    /// # Returns
    ///
    /// An immutable source location.
    #[must_use]
    #[inline(always)]
    pub const fn new(file: &'static str, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }

    /// Returns the source file containing the registration.
    #[must_use]
    #[inline(always)]
    pub const fn file(self) -> &'static str {
        self.file
    }

    /// Returns the one-based source line containing the registration.
    #[must_use]
    #[inline(always)]
    pub const fn line(self) -> u32 {
        self.line
    }

    /// Returns the one-based source column containing the registration.
    #[must_use]
    #[inline(always)]
    pub const fn column(self) -> u32 {
        self.column
    }
}

impl fmt::Display for SourceLocation {
    /// Formats this location as `file:line:column`.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}:{}", self.file, self.line, self.column)
    }
}
