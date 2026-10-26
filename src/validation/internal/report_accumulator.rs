// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One result contract and stopping policy for every execution position.

use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::RecordedOutcome;
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;

use crate::validation::ValidationMode;
use crate::validation::ValidationOptions;

/// Owns the report and the plan-wide stop state for a single validation call.
#[must_use]
pub(crate) struct ReportAccumulator<'options> {
    /// Caller policy shared by every result producer.
    options: &'options ValidationOptions,
    /// Report collected so far.
    report: ValidationReport,
    /// Whether the caller's violation policy has stopped all subsequent work.
    stopped: bool,
}

impl<'options> ReportAccumulator<'options> {
    /// Starts a fresh report under the caller's policy.
    ///
    /// # Parameters
    ///
    /// * `options` - The validation policy applied to accepted outcomes.
    ///
    /// # Returns
    ///
    /// Returns an empty accumulator whose failure limit reflects the selected
    /// fail-fast or collect-all mode.
    #[inline]
    pub(crate) fn new(options: &'options ValidationOptions) -> Self {
        let max_violations = if options.mode() == ValidationMode::FailFast {
            1
        } else {
            options.max_violations()
        };
        Self {
            options,
            report: ValidationReport::with_limits(ValidationLimits {
                max_violations: Some(max_violations),
                max_skipped: None,
            }),
            stopped: false,
        }
    }

    /// Reports whether execution must return without any further reads or
    /// calls.
    ///
    /// # Returns
    ///
    /// Returns `true` after an incomplete result or a stopping violation limit
    /// has ended further execution.
    #[must_use]
    #[inline]
    pub(crate) const fn stopped(&self) -> bool {
        self.stopped
    }

    /// Checks the outcome contract and appends all report content through one
    /// limit. When this result fills the limit, `has_more_work` determines
    /// whether stopping execution makes the report incomplete.
    ///
    /// # Parameters
    ///
    /// * `occurrence` - Stable execution position of the result.
    /// * `path` - Model location used to prefix relative violation paths.
    /// * `outcome` - Bound result to validate and record.
    /// * `has_more_work` - Whether selected occurrences remain after this one.
    ///
    /// # Returns
    ///
    /// Returns the accepted outcome with report-assigned failure identifiers.
    /// If execution has already stopped, records a valid placeholder outcome.
    ///
    /// # Errors
    ///
    /// Returns an adapter contract violation when the outcome shape is invalid.
    pub(crate) fn accept(
        &mut self,
        occurrence: usize,
        path: &ValidationPath,
        outcome: ValidationOutcome,
        has_more_work: bool,
    ) -> Result<RecordedOutcome, ExecutionError> {
        if self.stopped {
            return self
                .report
                .record_outcome(occurrence, path.clone(), ValidationOutcome::valid())
                .map_err(|_| contract_error());
        }
        let accepted = self
            .report
            .record_outcome(occurrence, path.clone(), outcome)
            .map_err(|_| contract_error())?;
        let added_failures = !accepted.failure_ids().is_empty();
        let at_limit = self.report.failure_count() >= self.options.max_violations();
        if !accepted.complete()
            || (added_failures && has_more_work && (self.options.mode() == ValidationMode::FailFast || at_limit))
        {
            self.stopped = true;
            self.report.mark_truncated();
        }
        Ok(accepted)
    }

    /// Returns the report, including partial results after an execution error.
    ///
    /// # Returns
    ///
    /// Returns the accumulated report, consuming this accumulator.
    #[inline]
    pub(crate) fn into_report(self) -> ValidationReport {
        self.report
    }
}

/// Creates the common invalid-validator-outcome error.
///
/// # Returns
///
/// Returns the standard adapter-contract violation execution error.
#[inline]
fn contract_error() -> ExecutionError {
    ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)
}
