// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Execution APIs sharing one report accumulator and one work budget.

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Utc;
use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::PathSegment;
use qubit_validator::SkipReason;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationParam;

use super::ValidationPlan;
use crate::metadata::OnNone;
use crate::metadata::PropertyValue;
use crate::metadata::SelectorPosition;
use crate::resolve::ModelGraph;
use crate::validation::ModelValidationError;
use crate::validation::ValidationOptions;
use crate::validation::internal::execution_budget::ExecutionBudget;
use crate::validation::internal::execution_failure::ExecutionFailure;
use crate::validation::internal::field_rule_binding::FieldExecution;
use crate::validation::internal::field_rule_binding::FieldRuleBinding;
use crate::validation::internal::path_reader;
use crate::validation::internal::path_reader::path_for;
use crate::validation::internal::report_accumulator::ReportAccumulator;
use crate::validation::standard_constraints::StandardTarget;
use crate::validation::validation_options::FieldPath;
use crate::validation::validation_options::ValidationSelection;

impl<'a> ValidationPlan<'a> {
    /// Executes selected occurrences under one stopping policy and work budget.
    ///
    /// # Errors
    /// Returns the original infrastructure failure and the partial report.
    pub fn validate(
        &self,
        value: ReflectedRef<'_>,
        options: &ValidationOptions,
    ) -> Result<ValidationReport, ModelValidationError> {
        self.validate_with_context(value, &[], options)
    }

    /// Executes with nearest-parent-first containing-object borrows.
    ///
    /// # Errors
    /// Returns typed source context for failed reads or rule invocations, or a
    /// traversal-limit error before the operation that would exceed its budget.
    pub fn validate_with_context<'value>(
        &self,
        value: ReflectedRef<'value>,
        ancestors: &[ReflectedRef<'value>],
        options: &ValidationOptions,
    ) -> Result<ValidationReport, ModelValidationError> {
        let mut report = ReportAccumulator::new(options);
        if value.value_type_id() != self.root().type_id() {
            return Err(ModelValidationError::new(
                ExecutionError::new(ExecutionErrorKind::InputTypeMismatch),
                report.into_report(),
            )
            .at_model(self.root(), None));
        }
        let mut budget = ExecutionBudget::new(options);
        for (occurrence, binding) in self.model_rules().iter().enumerate() {
            if report.stopped() {
                return Ok(report.into_report());
            }
            let path = ValidationPath::root();
            if !selected(options.selection(), &path) {
                continue;
            }
            let result = (|| {
                let input = reflected_value(&value);
                let context = BoundValidationContext::new_with_paths(&[], &[])?;
                budget.invoke(0, false)?;
                let outcome = binding.validate(input, &context)?;
                let has_more_work = self.model_rules()[occurrence + 1..]
                    .iter()
                    .any(|_| selected(options.selection(), &path))
                    || self
                        .bindings()
                        .iter()
                        .any(|field| selected(options.selection(), &path_for(field.value())));
                report.accept(occurrence, &path, outcome, has_more_work).map(|_| ())
            })();
            if let Err(error) = result {
                return Err(
                    ModelValidationError::new(error.with_rule(binding.rule_id()), report.into_report())
                        .at_model(self.root(), Some(occurrence)),
                );
            }
        }
        for (binding_index, binding) in self.bindings().iter().enumerate() {
            if report.stopped() {
                break;
            }
            let path = path_for(binding.value());
            if !selected(options.selection(), &path) {
                continue;
            }
            let occurrence = self.model_rules().len() + binding.occurrence();
            let has_more_work = self.bindings()[binding_index + 1..]
                .iter()
                .any(|field| selected(options.selection(), &path_for(field.value())));
            if let Err(failure) = execute_field(
                binding,
                occurrence,
                value.clone(),
                ancestors,
                self.graph(),
                &mut budget,
                &mut report,
                has_more_work,
            ) {
                let mut error = (*failure.error).with_rule(binding.rule_id());
                if error.path().as_segments().is_empty() {
                    error = error.with_path(path);
                }
                return Err(ModelValidationError::new(error, report.into_report()).at_field(
                    &binding.context,
                    occurrence,
                    failure.dependency,
                ));
            }
        }
        Ok(report.into_report())
    }
}

/// Reads a value before its dependencies and executes one compiled field rule.
#[allow(clippy::too_many_arguments)]
fn execute_field<'value>(
    binding: &FieldRuleBinding,
    occurrence: usize,
    root: ReflectedRef<'value>,
    ancestors: &[ReflectedRef<'value>],
    graph: &ModelGraph<'_>,
    budget: &mut ExecutionBudget<'_>,
    report: &mut ReportAccumulator<'_>,
    has_more_work: bool,
) -> Result<(), ExecutionFailure> {
    let path = path_for(binding.value());
    let value = path_reader::read(binding.value(), root.clone(), 0, budget)?;
    let direct_optional_scalar = binding.value().steps().last().is_some_and(|step| {
        let property = step.property();
        property.getter().is_none()
            && property
                .descriptor()
                .is_some_and(|descriptor| descriptor.as_optional().is_some())
    }) && matches!(
        binding.execution(),
        FieldExecution::Registry {
            target: Some(StandardTarget::Value),
            ..
        }
    );
    let projected_scalar = if direct_optional_scalar {
        let Some(scalar) = optional_scalar_value(&value) else {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)
                .with_path(path.clone())
                .into());
        };
        Some(scalar)
    } else {
        None
    };
    if matches!(value, PropertyValue::OptionalBorrowed(None)) || projected_scalar.as_ref().is_some_and(Option::is_none)
    {
        if !binding.value().is_optional() {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
        }
        let outcome = if binding.on_none() == OnNone::Reject {
            ValidationOutcome::Invalid(vec![Violation::new(
                binding.rule_id(),
                ViolationCode::new("value.required"),
            )])
        } else {
            ValidationOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites: Vec::new(),
            }
        };
        report.accept(occurrence, &path, outcome, has_more_work).map(|_| ())?;
        return Ok(());
    }
    if let FieldExecution::SequenceUnique { item_eq } = binding.execution() {
        return execute_unique(
            binding,
            occurrence,
            value,
            &path,
            *item_eq,
            budget,
            report,
            has_more_work,
        )
        .map_err(Into::into);
    }
    if let Some(selector) = binding.selector() {
        if selector.position() != SelectorPosition::Element {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
        }
        return execute_elements(binding, occurrence, value, &path, budget, report, has_more_work).map_err(Into::into);
    }
    let FieldExecution::Registry {
        validator,
        target,
        map_len,
    } = binding.execution()
    else {
        return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
    };
    let (dependencies, paths) = path_reader::dependencies(binding.dependencies(), root, ancestors, graph, budget)?;
    let values: Vec<_> = dependencies.iter().map(property_value).collect();
    // The plan binder matched dependencies by signature name, so this ordered
    // fast path receives values in the validator's declared order.
    let context = BoundValidationContext::new_with_paths(&values, &paths)?;
    let count = match &value {
        PropertyValue::BorrowedSlice(values) => Some(values.len()),
        _ => None,
    };
    let map_count = if *target == Some(StandardTarget::MapCount) {
        let Some(map_len) = map_len else {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
        };
        let count = map_len(&value).map_err(|error| {
            ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                .with_trusted_source(error)
                .with_path(path.clone())
        })?;
        let Some(count) = count else {
            let declared_optional = binding.value().is_optional()
                || binding
                    .value()
                    .steps()
                    .last()
                    .and_then(|step| step.property().descriptor())
                    .is_some_and(|descriptor| descriptor.as_optional().is_some());
            if !declared_optional {
                return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)
                    .with_path(path)
                    .into());
            }
            report
                .accept(
                    occurrence,
                    &path,
                    ValidationOutcome::Skipped {
                        reason: SkipReason::MissingOptional,
                        prerequisites: Vec::new(),
                    },
                    has_more_work,
                )
                .map(|_| ())?;
            return Ok(());
        };
        Some(count)
    } else {
        None
    };
    let input = match (*target, &value) {
        (Some(StandardTarget::SequenceCount), PropertyValue::BorrowedSlice(_)) => {
            ValidationValue::Typed(count.as_ref().expect("slice count"))
        }
        (Some(StandardTarget::MapCount), PropertyValue::Borrowed(_) | PropertyValue::OptionalBorrowed(Some(_))) => {
            ValidationValue::Typed(map_count.as_ref().expect("map count"))
        }
        (Some(StandardTarget::MapCount), _) => {
            return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                .with_path(path)
                .into());
        }
        (_, PropertyValue::BorrowedSlice(_)) => {
            return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).into());
        }
        _ => projected_scalar.flatten().unwrap_or_else(|| property_value(&value)),
    };
    budget.invoke(path.as_segments().len(), false)?;
    let outcome = validator
        .validate(input, &context)
        .map_err(|error| prefix_error(error, &path))?;
    report.accept(occurrence, &path, outcome, has_more_work).map(|_| ())?;
    Ok(())
}

/// Compares borrowed elements in deterministic pair order, reserving every
/// read and comparison before calling the checked adapter.
#[allow(clippy::too_many_arguments)]
fn execute_unique(
    binding: &FieldRuleBinding,
    occurrence: usize,
    value: PropertyValue<'_>,
    path: &ValidationPath,
    item_eq: crate::property::ItemEqAdapter,
    budget: &mut ExecutionBudget<'_>,
    report: &mut ReportAccumulator<'_>,
    has_more_work: bool,
) -> Result<(), ExecutionError> {
    let PropertyValue::BorrowedSlice(values) = value else {
        return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path.clone()));
    };
    budget
        .invoke(path.as_segments().len(), false)
        .map_err(|error| error.with_path(path.clone()))?;
    for second in 1..values.len() {
        let second_path = path.clone().with_index(second);
        for first in 0..second {
            let first_path = path.clone().with_index(first);
            budget
                .read(first_path.as_segments().len())
                .map_err(|error| error.with_path(first_path.clone()))?;
            let Some(first_value) = values.get(first) else {
                return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(first_path));
            };
            budget
                .read(second_path.as_segments().len())
                .map_err(|error| error.with_path(second_path.clone()))?;
            let Some(second_value) = values.get(second) else {
                return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(second_path.clone()));
            };
            budget
                .compare(second_path.as_segments().len())
                .map_err(|error| error.with_path(second_path.clone()))?;
            let equal = item_eq(first_value, second_value).map_err(|error| {
                ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                    .with_trusted_source(error)
                    .with_path(second_path.clone())
            })?;
            if equal {
                let violation = Violation::new(binding.rule_id(), ViolationCode::new("collection.duplicate_item"))
                    .with_path(ValidationPath::root().with_index(second))
                    .with_param("first_index", ViolationParam::Unsigned(first as u128));
                report.accept(
                    occurrence,
                    path,
                    ValidationOutcome::Invalid(vec![violation]),
                    has_more_work,
                )?;
                return Ok(());
            }
        }
    }
    report.accept(occurrence, path, ValidationOutcome::Valid, has_more_work)?;
    Ok(())
}

/// Executes selected slice elements, checking stop and budgets before each
/// operation.
fn execute_elements(
    binding: &FieldRuleBinding,
    occurrence: usize,
    value: PropertyValue<'_>,
    path: &ValidationPath,
    budget: &mut ExecutionBudget<'_>,
    report: &mut ReportAccumulator<'_>,
    has_more_work: bool,
) -> Result<(), ExecutionError> {
    let PropertyValue::BorrowedSlice(values) = value else {
        return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed));
    };
    for index in 0..values.len() {
        if report.stopped() {
            break;
        }
        let element_path = path.clone().with_index(index);
        let depth = element_path.as_segments().len();
        budget
            .read(depth)
            .map_err(|error| error.with_path(element_path.clone()))?;
        let element = values.get(index).ok_or_else(|| {
            ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(element_path.clone())
        })?;
        let context = BoundValidationContext::new_with_paths(&[], &[])?;
        budget
            .invoke(depth, true)
            .map_err(|error| error.with_path(element_path.clone()))?;
        let FieldExecution::Registry { validator, .. } = binding.execution() else {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).with_path(element_path));
        };
        let outcome = validator
            .validate(reflected_value(&element), &context)
            .map_err(|error| prefix_error(error, &element_path))?;
        report
            .accept(
                occurrence,
                &element_path,
                outcome,
                index + 1 < values.len() || has_more_work,
            )
            .map_err(|error| error.with_path(element_path))?;
    }
    Ok(())
}

/// Prefixes a validator's relative error path without discarding its source.
fn prefix_error(error: ExecutionError, prefix: &ValidationPath) -> ExecutionError {
    let path =
        prefix
            .as_segments()
            .iter()
            .chain(error.path().as_segments())
            .fold(ValidationPath::root(), |path, segment| match segment {
                PathSegment::Field(field) => path.with_field(field),
                PathSegment::Index(index) => path.with_index(*index),
                PathSegment::MapEntry(index) => path.with_map_entry(*index),
                PathSegment::MapKey => path.with_map_key(),
                PathSegment::MapValue => path.with_map_value(),
                _ => path,
            });
    error.with_path(path)
}

/// Converts a reflected borrow into the validator input abstraction.
fn reflected_value<'a>(value: &'a ReflectedRef<'_>) -> ValidationValue<'a> {
    if let Some(text) = value.as_str() {
        return ValidationValue::Text(text);
    }
    if let Some(text) = value.downcast_ref::<String>() {
        return ValidationValue::Text(text.as_str());
    }
    ValidationValue::Typed(value.as_any().expect("non-text reflected value"))
}

/// Converts a property adapter result into a validator input abstraction.
fn property_value<'a>(value: &'a PropertyValue<'_>) -> ValidationValue<'a> {
    match value {
        PropertyValue::Borrowed(value) => reflected_value(value),
        PropertyValue::OptionalBorrowed(Some(value)) => reflected_value(value),
        PropertyValue::OptionalBorrowed(None) => ValidationValue::Missing,
        PropertyValue::Owned(value) => owned_value(value),
        PropertyValue::BorrowedSlice(_) => ValidationValue::Missing,
    }
}

/// Borrows the contained scalar from a reflected, field-backed `Option<T>`.
/// The outer `None` indicates an unexpected adapter shape; the inner `None`
/// means the optional field is absent. No value is cloned or formatted.
fn optional_scalar_value<'a>(value: &'a PropertyValue<'_>) -> Option<Option<ValidationValue<'a>>> {
    let PropertyValue::Borrowed(value) = value else {
        return None;
    };
    optional_typed_value::<BigDecimal>(value)
        .or_else(|| optional_typed_value::<DateTime<Utc>>(value))
        .or_else(|| optional_typed_value::<NaiveDateTime>(value))
        .or_else(|| optional_typed_value::<NaiveTime>(value))
}

/// Adapts one exact reflected optional type without exposing its contents.
fn optional_typed_value<'a, T: 'static>(value: &'a ReflectedRef<'_>) -> Option<Option<ValidationValue<'a>>> {
    value
        .downcast_ref::<Option<T>>()
        .map(|value| value.as_ref().map(|inner| ValidationValue::Typed(inner)))
}

/// Converts an owned reflected value into a validator input abstraction.
fn owned_value<'a>(value: &'a ReflectedOwned) -> ValidationValue<'a> {
    if let Some(text) = value.downcast_ref::<String>() {
        return ValidationValue::Text(text.as_str());
    }
    ValidationValue::Typed(value.as_any().expect("owned values are Any-compatible"))
}

/// Reports whether a validation path is selected by the caller.
fn selected(selection: &ValidationSelection, path: &ValidationPath) -> bool {
    match selection {
        ValidationSelection::All => true,
        ValidationSelection::Fields(fields) => fields.iter().any(|field| field_matches(field, path)),
    }
}

/// Matches the complete field-name sequence, ignoring collection indices.
fn field_matches(field: &FieldPath, path: &ValidationPath) -> bool {
    let fields: Vec<&str> = path
        .as_segments()
        .iter()
        .filter_map(|segment| match segment {
            PathSegment::Field(name) => Some(*name),
            _ => None,
        })
        .collect();
    field.segments().iter().map(String::as_str).eq(fields)
}
