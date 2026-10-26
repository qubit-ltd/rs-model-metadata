// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stopping policies for validation execution.

/// Selects whether validation stops at the first failure.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::validation::{ValidationMode, ValidationOptions};
///
/// let options = ValidationOptions::builder()
///     .mode(ValidationMode::FailFast)
///     .build();
/// assert_ne!(options, ValidationOptions::default());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationMode {
    /// Execute every selected occurrence until a budget is reached.
    CollectAll,
    /// Stop after the first reported violation.
    FailFast,
}
