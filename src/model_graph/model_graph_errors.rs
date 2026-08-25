// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Aggregated direct-reference graph validation errors.

use core::fmt;

use super::model_graph_error::ModelGraphError;

/// All independently discoverable graph-validation errors in a registry.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::ModelGraphErrors;
/// use qubit_model_metadata::ModelRegistry;
///
/// let registry = ModelRegistry::from_registrations([]).expect("empty registry is valid");
/// let result: Result<(), ModelGraphErrors> = registry.validate_graph();
/// assert!(result.is_ok());
/// ```
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelGraphErrors {
    /// Errors in deterministic model, field, kind, and target order.
    errors: Vec<ModelGraphError>,
}

impl ModelGraphErrors {
    /// Creates an error aggregation from already-sorted validation errors.
    ///
    /// # Parameters
    ///
    /// - `errors`: The already-sorted graph-validation errors.
    ///
    /// # Returns
    ///
    /// An aggregation wrapping `errors`.
    pub(crate) fn new(errors: Vec<ModelGraphError>) -> Self {
        Self { errors }
    }

    /// Returns all graph-validation errors in deterministic order.
    ///
    /// # Returns
    ///
    /// The aggregated errors in model, field, kind, and target order.
    #[inline(always)]
    pub fn errors(&self) -> &[ModelGraphError] {
        &self.errors
    }
}

impl fmt::Display for ModelGraphErrors {
    /// Formats every graph-validation error in this aggregation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "model graph validation failed")?;
        for error in &self.errors {
            write!(formatter, "; {error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ModelGraphErrors {}
