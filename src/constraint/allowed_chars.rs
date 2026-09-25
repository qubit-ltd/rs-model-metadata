// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Character-set policies for text constraints.

/// The character set accepted by a text constraint.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::AllowedChars;
///
/// assert_eq!(AllowedChars::default(), AllowedChars::Unicode);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AllowedChars {
    /// All Unicode scalar values are accepted.
    #[default]
    Unicode,
    /// Unicode letters, marks, numbers, punctuation, symbols, and space
    /// separators.
    ///
    /// Control, format, private-use, unassigned, line-separator, and
    /// paragraph-separator characters are rejected. This also rejects format
    /// characters such as zero-width joiners in emoji sequences.
    PrintableUnicode,
    /// Only ASCII characters are accepted.
    Ascii,
    /// Printable ASCII characters only.
    PrintableAscii,
    /// ASCII letters, digits, period, underscore, and hyphen.
    Code,
}
