// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;

/// Exact Rust identities involved in a rejected property operation.
///
/// # Examples
///
/// ```
/// use std::any::TypeId;
///
/// use qubit_model_metadata::metadata::PropertyTypeMismatch;
///
/// let mismatch = PropertyTypeMismatch {
///     expected: TypeId::of::<String>(),
///     actual: TypeId::of::<u32>(),
/// };
/// assert_ne!(mismatch.expected, mismatch.actual);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PropertyTypeMismatch {
    /// Rust type identity required by the property operation.
    pub expected: TypeId,
    /// Rust type identity supplied to the property operation.
    pub actual: TypeId,
}
