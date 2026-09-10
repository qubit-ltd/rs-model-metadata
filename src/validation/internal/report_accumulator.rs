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
use qubit_validator::RuleOutcome;
use qubit_validator::SkipReason;
use qubit_validator::SkippedValidation;
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
}

impl<'options> ReportAccumulator<'options> {
    /// Starts a fresh report under the caller's policy.
    pub(crate) fn new(options: &'options ValidationOptions) -> Self {
        Self {
            options,
            report: ValidationReport::new(),
            stopped: false,
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
        outcome: RuleOutcome,
    ) -> Result<(), ExecutionError> {
        if self.stopped {
            return Ok(());
        }
        match outcome {
            RuleOutcome::Valid => {}
            RuleOutcome::Invalid(violations) => {
                if violations.is_empty() {
                    return Err(contract_error());
                }
                self.append(violations, path);
            }
            RuleOutcome::Skipped { reason, prerequisites } => {
                if reason == SkipReason::MissingOptional && !prerequisites.is_empty()
                    || reason == SkipReason::FailedPrerequisite && prerequisites.is_empty()
                {
                    return Err(contract_error());
                }
                self.append(prerequisites, path);
                self.report
                    .record_skip(SkippedValidation::new(occurrence, path.clone(), reason));
            }
        }
        Ok(())
    }

    /// Returns the report, including partial results after an execution error.
    pub(crate) fn into_report(self) -> ValidationReport {
        self.report
    }

    /// Appends violations up to the hard limit and marks global stopping
    /// immediately.
    fn append(&mut self, violations: Vec<Violation>, path: &ValidationPath) {
        for violation in violations {
            if self.stopped {
                break;
            }
            self.report.push(prefix_violation(violation, path));
            if self.options.mode() == ValidationMode::FailFast
                || self.report.violations().len() >= self.options.max_violations()
            {
                self.stopped = true;
                self.report.mark_truncated();
            }
        }
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
        },
    );
    violation.with_path(path)
}
