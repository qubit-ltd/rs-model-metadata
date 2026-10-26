// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Helpers for safe validation binding diagnostics.

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;

/// Creates a binding error for a property-path validation failure.
///
/// # Parameters
///
/// - `kind`: the binding error category for the property-path failure.
///
/// # Returns
///
/// A binding error carrying the supplied failure category.
#[inline]
pub(crate) fn path_error(kind: BindErrorKind) -> BindError {
    BindError::new(kind)
}
