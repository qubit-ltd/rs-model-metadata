// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Invalid object navigation metadata.

use thiserror::Error;

/// A property step contains an empty or ambiguous name.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::relation::ObjectPathError;
///
/// let error = ObjectPathError { index: 2, name: "" };
/// assert_eq!(error.index, 2);
/// assert_eq!(error.name, "");
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
#[error("invalid object navigation property at step {index}: {name}")]
pub struct ObjectPathError {
    /// The zero-based invalid navigation step.
    pub index: usize,
    /// The rejected property name.
    pub name: &'static str,
}
