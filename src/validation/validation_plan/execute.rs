// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Execution APIs sharing one report accumulator and one work budget.

use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;
use qubit_validator::BoundValidationContext;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::PathSegment;
use qubit_validator::RuleOutcome;
use qubit_validator::SkipReason;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

use super::ValidationPlan;
use crate::metadata::OnNone;
use crate::metadata::PropertyValue;
use crate::metadata::SelectorPosition;
use crate::resolve::ModelGraph;
use crate::validation::ModelValidationError;
use crate::validation::ValidationOptions;
use crate::validation::internal::execution_budget::ExecutionBudget;
use crate::validation::internal::execution_failure::ExecutionFailure;
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
                if !binding.input_type().accepts(input) {
                    return Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch));
                }
                let context = BoundValidationContext::new_with_paths(&[], &[])?;
                budget.invoke(0, false)?;
                let outcome = binding.validator().validate(input, &context)?;
                report.accept(occurrence, &path, outcome)
            })();
            if let Err(error) = result {
                return Err(
                    ModelValidationError::new(error.with_rule(binding.rule_id()), report.into_report())
                        .at_model(self.root(), Some(occurrence)),
                );
            }
        }
        for binding in self.bindings() {
            if report.stopped() {
                break;
            }
            let path = path_for(binding.value());
            if !selected(options.selection(), &path) {
                continue;
            }
            let occurrence = self.model_rules().len() + binding.occurrence();
            if let Err(failure) = execute_field(
                binding,
                occurrence,
                value.clone(),
                ancestors,
                self.graph(),
                &mut budget,
                &mut report,
            ) {
                let mut error = failure.error.with_rule(binding.rule_id());
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
) -> Result<(), ExecutionFailure> {
    let path = path_for(binding.value());
    let value = path_reader::read(binding.value(), root.clone(), 0, budget)?;
    if matches!(value, PropertyValue::OptionalBorrowed(None)) {
        if !binding.value().is_optional() {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
        }
        let outcome = if binding.on_none() == OnNone::Reject {
            RuleOutcome::Invalid(vec![Violation::new(
                binding.rule_id(),
                ViolationCode::new("value.required"),
            )])
        } else {
            RuleOutcome::Skipped {
                reason: SkipReason::MissingOptional,
                prerequisites: Vec::new(),
            }
        };
        report.accept(occurrence, &path, outcome)?;
        return Ok(());
    }
    if let Some(selector) = binding.selector() {
        if selector.position() != SelectorPosition::Element {
            return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation).into());
        }
        return execute_elements(binding, occurrence, value, &path, budget, report).map_err(Into::into);
    }
    let (dependencies, paths) = path_reader::dependencies(binding.dependencies(), root, ancestors, graph, budget)?;
    let values: Vec<_> = dependencies.iter().map(property_value).collect();
    let context = BoundValidationContext::new_with_paths(&values, &paths)?;
    let count = match &value {
        PropertyValue::BorrowedSlice(values) => Some(values.len()),
        _ => None,
    };
    let input = match (binding.standard_target(), &value) {
        (Some(StandardTarget::SequenceCount), PropertyValue::BorrowedSlice(_)) => {
            ValidationValue::Typed(count.as_ref().expect("slice count"))
        }
        (_, PropertyValue::BorrowedSlice(_)) => {
            return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).into());
        }
        _ => property_value(&value),
    };
    budget.invoke(path.as_segments().len(), false)?;
    let outcome = binding
        .validator()
        .validate(input, &context)
        .map_err(|error| prefix_error(error, &path))?;
    report.accept(occurrence, &path, outcome)?;
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
        let outcome = binding
            .validator()
            .validate(reflected_value(&element), &context)
            .map_err(|error| prefix_error(error, &element_path))?;
        report
            .accept(occurrence, &element_path, outcome)
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
                PathSegment::Field(field) => path.with_field(field.clone()),
                PathSegment::Index(index) => path.with_index(*index),
                PathSegment::MapEntry(index) => path.with_map_entry(*index),
                PathSegment::MapKey => path.with_map_key(),
                PathSegment::MapValue => path.with_map_value(),
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
            PathSegment::Field(name) => Some(name.as_ref()),
            _ => None,
        })
        .collect();
    field.segments().iter().map(String::as_str).eq(fields)
}
