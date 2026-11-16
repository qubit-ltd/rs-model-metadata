// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-private report accumulation contracts.

use std::num::NonZeroUsize;

use qubit_validator::ExecutionErrorKind;
use qubit_validator::SkipReason;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

use crate::validation::ValidationMode;
use crate::validation::ValidationOptions;
use crate::validation::internal::report_accumulator::ReportAccumulator;

#[test]
fn test_prerequisite_references_do_not_consume_the_failure_limit() {
    let options = ValidationOptions::builder()
        .max_violations(NonZeroUsize::new(2).unwrap())
        .build();
    let mut accumulator = ReportAccumulator::new(&options);
    let first = accumulator
        .accept(
            0,
            &ValidationPath::root(),
            ValidationOutcome::Invalid(vec![Violation::new(
                ValidatorId::new("execution.rule"),
                ViolationCode::new("bad"),
            )]),
            true,
        )
        .unwrap();
    let id = first.failure_ids()[0];
    accumulator
        .accept(
            1,
            &ValidationPath::root().with_field("skipped"),
            ValidationOutcome::failed_prerequisite(vec![id]).unwrap(),
            true,
        )
        .unwrap();
    assert!(!accumulator.stopped());
    let report = accumulator.into_report();
    assert_eq!(report.failure_count(), 1);
    assert_eq!(report.failures().count(), 1);
    assert_eq!(report.skipped().len(), 1);
    assert_eq!(report.skipped()[0].prerequisites(), &[id]);
}

#[test]
fn test_fail_fast_stops_on_original_failure() {
    let options = ValidationOptions::builder().mode(ValidationMode::FailFast).build();
    let mut accumulator = ReportAccumulator::new(&options);
    accumulator
        .accept(
            0,
            &ValidationPath::root(),
            ValidationOutcome::Invalid(vec![Violation::new(
                ValidatorId::new("execution.rule"),
                ViolationCode::new("bad"),
            )]),
            true,
        )
        .unwrap();
    assert!(accumulator.stopped());
    accumulator
        .accept(
            1,
            &ValidationPath::root().with_field("later"),
            ValidationOutcome::missing_optional(),
            false,
        )
        .unwrap();
    let report = accumulator.into_report();
    assert_eq!(report.failure_count(), 1);
    assert!(report.is_truncated());
}

#[test]
fn test_invalid_skip_contracts_are_execution_errors_without_report_mutation() {
    let outcome = ValidationOutcome::Skipped {
        reason: SkipReason::FailedPrerequisite,
        prerequisites: vec![],
    };
    let options = ValidationOptions::default();
    let mut accumulator = ReportAccumulator::new(&options);
    let error = accumulator
        .accept(0, &ValidationPath::root(), outcome, false)
        .expect_err("the skip shape violates the result contract");
    assert_eq!(error.kind(), ExecutionErrorKind::AdapterContractViolation);
    assert!(!accumulator.stopped());
    let report = accumulator.into_report();
    assert_eq!(report.failure_count(), 0);
    assert!(report.skipped().is_empty());
    assert!(!report.is_truncated());
}

#[test]
fn test_invalid_outcome_shape_is_reported_as_an_adapter_contract_violation() {
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
