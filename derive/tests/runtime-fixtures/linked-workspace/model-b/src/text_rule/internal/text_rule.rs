// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Implements the private validator used by the linked-workspace fixture.

use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationValue;

/// Checks that the linked-workspace fixture passes the expected text value.
pub(super) struct TextRule;

impl PreparedValidator for TextRule {
    /// Identifies the input shape required by this validator.
    ///
    /// # Returns
    ///
    /// Returns the text input shape.
    #[inline]
    fn input_type(&self) -> InputType {
        InputType::Text
    }

    /// Returns the prerequisites needed before this validator can run.
    ///
    /// # Returns
    ///
    /// Returns an empty slice because this fixture has no prerequisites.
    #[inline]
    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }

    /// Verifies the expected fixture value and returns a successful outcome.
    ///
    /// # Parameters
    ///
    /// - `value`: Bound input value supplied by the validator execution flow.
    /// - `_`: Validation context supplied by that same flow.
    ///
    /// # Returns
    ///
    /// Returns a valid outcome when the value is exactly `cross-crate`.
    ///
    /// # Errors
    ///
    /// This fixture does not return an execution error.
    ///
    /// # Panics
    ///
    /// Panics if the framework supplies text other than `cross-crate`.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        assert_eq!(value.as_text(), Some("cross-crate"));
        Ok(PreparedOutcome::Valid)
    }
}
