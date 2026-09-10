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

use qubit_validator::InputType;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidatorId;

/// A typed model-level validator prepared for execution against the plan root.
///
/// Cloning shares the supplied prepared validator through its `Arc`; it does
/// not prepare a new rule. Debug output contains the rule identity without
/// inspecting the validator, which need not implement `Debug`.
#[derive(Clone)]
pub struct ModelRuleBinding {
    /// Rule identity attached to execution failures and violations.
    rule_id: ValidatorId,
    /// Exact root type accepted by this prepared validator.
    input_type: InputType,
    /// Shared immutable rule implementation, without an instance borrow.
    validator: Arc<dyn PreparedValidator>,
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
            .field("rule_id", &self.rule_id)
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
    /// Returns a binding which can be appended to a plan. A mismatched plan
    /// root produces `InputTypeMismatch` at execution before invoking the rule.
    #[must_use]
    #[inline]
    pub fn from_prepared<T: 'static>(rule_id: ValidatorId, validator: Arc<dyn PreparedValidator>) -> Self {
        Self {
            rule_id,
            input_type: InputType::of::<T>(),
            validator,
        }
    }
    /// Returns the bound rule identifier.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn rule_id(&self) -> ValidatorId {
        self.rule_id
    }
    /// Returns the input type accepted by the rule.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn input_type(&self) -> InputType {
        self.input_type
    }
    /// Returns the prepared validator implementation.
    #[must_use]
    #[inline(always)]
    pub(crate) fn validator(&self) -> &dyn PreparedValidator {
        self.validator.as_ref()
    }
}
