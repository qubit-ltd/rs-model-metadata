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
use qubit_validator::PathSegment;
use qubit_validator::SkipReason;
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::Violation;

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
    /// Top-level and prerequisite violations accepted under the caller limit.
    violation_count: usize,
}

impl<'options> ReportAccumulator<'options> {
    /// Starts a fresh report under the caller's policy.
    pub(crate) fn new(options: &'options ValidationOptions) -> Self {
        Self {
            options,
            report: ValidationReport::with_limits(ValidationLimits {
                max_violations: Some(options.max_violations()),
                max_skipped: None,
            }),
            stopped: false,
            violation_count: 0,
        }
    }

    /// Reports whether execution must return without any further reads or
    /// calls.
    pub(crate) const fn stopped(&self) -> bool {
        self.stopped
    }

    /// Checks the outcome contract and appends all report content through one
    /// limit.
    pub(crate) fn accept(
        &mut self,
        occurrence: usize,
        path: &ValidationPath,
        outcome: ValidationOutcome,
    ) -> Result<(), ExecutionError> {
        if self.stopped {
            return Ok(());
        }
        let violation_count = match &outcome {
            ValidationOutcome::Valid => 0,
            ValidationOutcome::Invalid(violations) => {
                if violations.is_empty() {
                    return Err(contract_error());
                }
                violations.len()
            }
            ValidationOutcome::Skipped { reason, prerequisites } => {
                if (*reason == SkipReason::MissingOptional && !prerequisites.is_empty())
                    || (*reason == SkipReason::FailedPrerequisite && prerequisites.is_empty())
                {
                    return Err(contract_error());
                }
                prerequisites.len()
            }
            _ => return Err(contract_error()),
        };

        let fail_fast = self.options.mode() == ValidationMode::FailFast && violation_count > 0;
        let remaining = self.options.max_violations() - self.violation_count;
        let accepted_limit = if fail_fast { 1 } else { remaining };
        let truncated = violation_count > accepted_limit;
        let outcome = prefix_and_limit_outcome(outcome, path, accepted_limit)?;
        self.report
            .record_outcome(occurrence, path.clone(), outcome)
            .map_err(|_| contract_error())?;
        self.violation_count += violation_count.min(accepted_limit);

        if violation_count > 0
            && (truncated || fail_fast || self.violation_count >= self.options.max_violations())
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

/// Prefixes nested violations and retains only the part allowed by execution
/// policy before passing the result to the shared report type.
fn prefix_and_limit_outcome(
    outcome: ValidationOutcome,
    path: &ValidationPath,
    limit: usize,
) -> Result<ValidationOutcome, ExecutionError> {
    match outcome {
        ValidationOutcome::Valid => Ok(ValidationOutcome::Valid),
        ValidationOutcome::Invalid(violations) => {
            let violations: Vec<_> = violations
                .into_iter()
                .take(limit)
                .map(|violation| prefix_violation(violation, path))
                .collect();
            if violations.is_empty() {
                return Err(contract_error());
            }
            Ok(ValidationOutcome::Invalid(violations))
        }
        ValidationOutcome::Skipped { reason, prerequisites } => {
            let prerequisites = prerequisites
                .into_iter()
                .take(limit)
                .map(|violation| prefix_violation(violation, path))
                .collect::<Vec<_>>();
            if reason == SkipReason::FailedPrerequisite && prerequisites.is_empty() {
                return Err(contract_error());
            }
            Ok(ValidationOutcome::Skipped { reason, prerequisites })
        }
        _ => Err(contract_error()),
    }
}

/// Creates the common invalid-validator-outcome error.
fn contract_error() -> ExecutionError {
    ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)
}

/// Prefixes a nested violation with its containing path.
fn prefix_violation(violation: Violation, prefix: &ValidationPath) -> Violation {
    let path = prefix.as_segments().iter().chain(violation.path().as_segments()).fold(
        ValidationPath::root(),
        |path, segment| match segment {
            PathSegment::Field(field) => path.with_field(field.clone()),
            PathSegment::Index(index) => path.with_index(*index),
            PathSegment::MapEntry(index) => path.with_map_entry(*index),
            PathSegment::MapKey => path.with_map_key(),
            PathSegment::MapValue => path.with_map_value(),
            _ => path,
        },
    );
    violation.with_path(path)
}
