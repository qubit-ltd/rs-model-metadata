// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed model-level validator bindings.

use std::fmt;
use std::sync::Arc;

use qubit_validator::BindError;
use qubit_validator::BoundValidationContext;
use qubit_validator::BoundValidator;
use qubit_validator::ExecutionError;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorId;

/// A typed model-level validator prepared for execution against the plan root.
///
/// Cloning shares the supplied prepared validator through its `Arc`; it does
/// not prepare a new rule. Debug output contains the rule identity without
/// inspecting the validator, which need not implement `Debug`.
#[derive(Clone)]
pub struct ModelRuleBinding {
    /// Prepared implementation with checked input and dependency metadata.
    validator: BoundValidator,
}

impl fmt::Debug for ModelRuleBinding {
    /// Formats the rule identity without invoking or inspecting its validator.
    ///
    /// # Errors
    ///
    /// Returns a formatter error if writing the diagnostic fails.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModelRuleBinding")
            .field("rule_id", &self.rule_id())
            .finish()
    }
}

impl ModelRuleBinding {
    /// Creates a model-level binding for a prepared validator accepting `T`.
    ///
    /// # Type Parameters
    ///
    /// - `T`: concrete root type that the prepared validator accepts.
    ///
    /// # Parameters
    ///
    /// - `rule_id`: identity retained for execution diagnostics.
    /// - `validator`: already prepared rule; registration and preparation
    ///   remain the caller's responsibility. It is not invoked by this
    ///   constructor.
    ///
    /// # Returns
    ///
    /// Returns a binding which can be appended to a plan when its prepared
    /// validator accepts exactly `T` and declares no dependencies.
    ///
    /// # Errors
    ///
    /// Returns `PreparedSignatureMismatch` with the rule ID when the prepared
    /// validator shape does not match the requested root type.
    #[must_use = "handle model rule binding errors"]
    #[inline]
    pub fn from_prepared<T: 'static>(
        rule_id: ValidatorId,
        validator: Arc<dyn PreparedValidator>,
    ) -> Result<Self, BindError> {
        Ok(Self {
            validator: BoundValidator::try_from_prepared::<T>(rule_id, validator)?,
        })
    }
    /// Returns the bound rule identifier.
    #[must_use]
    #[inline]
    pub(crate) const fn rule_id(&self) -> ValidatorId {
        self.validator.rule_id()
    }

    /// Validates one model value through the common bound-validator checks.
    ///
    /// # Errors
    ///
    /// Returns an input, context, adapter, or rule execution error associated
    /// with this binding's rule ID.
    #[inline]
    pub(crate) fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &BoundValidationContext<'_>,
    ) -> Result<ValidationOutcome, ExecutionError> {
        self.validator.validate(value, context)
    }
}
