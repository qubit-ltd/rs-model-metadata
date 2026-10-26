// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Execution failure context before it is attached to a validation report.

use qubit_validator::ExecutionError;

use crate::metadata::DependencyBindingMetadata;

/// An execution error with an optional original dependency navigation
/// declaration.
pub(crate) struct ExecutionFailure {
    /// Original execution error, including any underlying adapter cause.
    pub(crate) error: Box<ExecutionError>,
    /// Separate object navigation and property selection of a failed
    /// dependency.
    pub(crate) dependency: Option<DependencyBindingMetadata>,
}

/// Wraps an execution error without dependency navigation metadata.
///
/// The error is boxed, and `dependency` is set to `None` because this
/// conversion carries no information identifying a failed dependency.
impl From<ExecutionError> for ExecutionFailure {
    fn from(error: ExecutionError) -> Self {
        Self {
            error: Box::new(error),
            dependency: None,
        }
    }
}
