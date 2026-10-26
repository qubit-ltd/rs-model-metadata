// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Failures raised while executing resolved Projection producers.

use qubit_reflect::descriptor::FieldAccessError;
use thiserror::Error;

use crate::metadata::PropertyAccessError;

/// Failure while executing an automatic Projection producer.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::resolve::ProjectionExecutionError;
///
/// let error = ProjectionExecutionError::MissingProjector;
/// assert!(matches!(error, ProjectionExecutionError::MissingProjector));
/// ```
#[must_use]
#[derive(Debug, Error)]
pub enum ProjectionExecutionError {
    /// No executable adapter is registered for this producer.
    #[error("projection producer has no executable projector")]
    MissingProjector,
    /// The resolved edge no longer has the required Entity/Projection shape.
    #[error("projection producer metadata is invalid")]
    InvalidProducer,
    /// An identifier field did not contain the exact `qubit_id::Id` type.
    #[error("projection identifier has an invalid Rust type")]
    InvalidIdentifierType,
    /// The produced Projection changed the source Entity identifier.
    #[error("projection identifier differs from its source entity")]
    IdentifierMismatch,
    /// A property getter failed.
    #[error("projection property access failed: {0}")]
    Property(#[from] PropertyAccessError),
    /// A reflected identifier field could not be read.
    #[error("projection identifier field access failed: {0}")]
    Field(#[from] FieldAccessError),
}
