// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Distinguishes lifetime-preserving borrowed output from owned output.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::GetterOutputKind;
///
/// let output = GetterOutputKind::Borrowed;
/// assert_eq!(output, GetterOutputKind::Borrowed);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GetterOutputKind {
    /// The adapter returns an optional borrow of the contained value.
    OptionalBorrowed,
    /// The adapter returns a slice supporting borrowed element access.
    BorrowedSlice,
    /// The output borrows from its target.
    Borrowed,
    /// The output owns its value.
    Owned,
}
