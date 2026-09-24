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
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;

use crate::validation::ValidationMode;
use crate::validation::ValidationOptions;

/// Owns the report and the plan-wide stop state for a single validation call.
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
    /// # Errors
    ///
    /// Returns an adapter contract violation when the outcome shape is invalid.
    pub(crate) fn accept(
        &mut self,
        occurrence: usize,
        path: &ValidationPath,
        outcome: ValidationOutcome,
        has_more_work: bool,
    ) -> Result<(), ExecutionError> {
        if self.stopped {
            return Ok(());
        }
        let previous_failures = self.report.failure_count();
        let accepted = self
            .report
            .record_outcome(occurrence, path.clone(), outcome)
            .map_err(|_| contract_error())?;
        let added_failures = self.report.failure_count() > previous_failures;
        let at_limit = self.report.failure_count() >= self.options.max_violations();
        if !accepted
            || (added_failures && has_more_work && (self.options.mode() == ValidationMode::FailFast || at_limit))
        {
            self.stopped = true;
            self.report.mark_truncated();
        }
        Ok(())
    }

    /// Returns the report, including partial results after an execution error.
    pub(crate) fn into_report(self) -> ValidationReport {
        self.report
    }
}

/// Creates the common invalid-validator-outcome error.
fn contract_error() -> ExecutionError {
    ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)
}

#[cfg(test)]
mod tests {
    use qubit_validator::ExecutionErrorKind;
    use qubit_validator::ValidationOutcome;
    use qubit_validator::ValidationPath;

    use super::ReportAccumulator;
    use crate::validation::ValidationOptions;

    #[test]
    fn invalid_outcome_shape_is_reported_as_an_adapter_contract_violation() {
        let options = ValidationOptions::default();
        let mut report = ReportAccumulator::new(&options);

        let error = report
            .accept(
                0,
                &ValidationPath::root(),
                ValidationOutcome::Invalid(Vec::new()),
                false,
            )
            .expect_err("an empty invalid outcome violates the result contract");

        assert_eq!(error.kind(), ExecutionErrorKind::AdapterContractViolation);
        assert!(!report.stopped());
    }
}
