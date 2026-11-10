// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structured detail for a validation-plan build limit.

use super::ValidationBuildLimitKind;

/// The category and configured capacity of an exhausted build budget.
///
/// The value is returned by
/// [`crate::validation::ValidationBuildError::build_limit`].
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::validation::ValidationBuildLimit;
/// use qubit_model_metadata::validation::ValidationBuildLimitKind;
///
/// fn describe(limit: ValidationBuildLimit) -> (ValidationBuildLimitKind, usize) {
///     (limit.kind(), limit.limit())
/// }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationBuildLimit {
    /// The budget category that was exhausted.
    kind: ValidationBuildLimitKind,
    /// The configured maximum accepted for that category.
    limit: usize,
}

impl ValidationBuildLimit {
    /// Creates a build limit detail for internal budget accounting.
    ///
    /// # Parameters
    ///
    /// * `kind` - The exhausted build-budget category.
    /// * `limit` - The configured maximum accepted for that category.
    pub(crate) const fn new(kind: ValidationBuildLimitKind, limit: usize) -> Self {
        Self { kind, limit }
    }

    /// Returns the category of the exhausted budget.
    #[must_use]
    pub const fn kind(&self) -> ValidationBuildLimitKind {
        self.kind
    }

    /// Returns the maximum accepted capacity for this budget.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }
}
