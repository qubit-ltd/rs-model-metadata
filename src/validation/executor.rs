//! Borrow-preserving execution of a bound validation plan.

use qubit_reflect::ReflectedRef;
use qubit_validator::{BoundValidationContext, ExecutionError, ExecutionErrorKind, RuleOutcome, SkipReason, SkippedValidation, ValidationPath, ValidationReport, ValidationValue, Violation, ViolationCode};

use super::compiled_property_path::CompiledPropertyPath;
use super::model_validation_error::ModelValidationError;
use super::validation_options::{FieldPath, ValidationMode, ValidationOptions, ValidationSelection};
use super::ValidationPlan;
use crate::{OnNone, PropertyValue};

impl<'a> ValidationPlan<'a> {
    /// Executes all selected bound rules against one borrowed model value.
    pub fn validate(&self, value: ReflectedRef<'_>, options: &ValidationOptions) -> Result<ValidationReport, ModelValidationError> {
        let mut report = ValidationReport::new();
        let mut nodes = 1usize;
        let _ = options.max_comparisons();
        if nodes > options.max_nodes() || options.max_depth() == 0 {
            return Err(ModelValidationError::new(ExecutionError::new(ExecutionErrorKind::TraversalLimit), report));
        }
        for binding in self.bindings() {
            let path = path_for(binding.value());
            if !selected(options.selection(), &path) { continue; }
            if nodes == options.max_nodes() {
                return Err(ModelValidationError::new(ExecutionError::new(ExecutionErrorKind::TraversalLimit).with_rule(binding.rule_id()), report));
            }
            nodes += 1;
            let dependencies = match read_direct_values(binding.dependencies(), value.clone()) {
                Ok(values) => values,
                Err(error) => return Err(ModelValidationError::new(error.with_rule(binding.rule_id()).with_path(path.clone()), report)),
            };
            if let Err(error) = execute_path(binding.value(), value.clone(), binding.validator(), &dependencies, binding.on_none(), &mut report, &path, binding.occurrence(), binding.rule_id(), options) {
                return Err(ModelValidationError::new(error.with_rule(binding.rule_id()).with_path(path), report));
            }
            if options.mode() == ValidationMode::FailFast && !report.violations().is_empty() {
                report.mark_truncated();
                break;
            }
            if report.violations().len() >= options.max_violations() {
                report.mark_truncated();
                break;
            }
        }
        Ok(report)
    }
}

fn read_direct_values<'a>(paths: &[CompiledPropertyPath], root: ReflectedRef<'a>) -> Result<Vec<PropertyValue<'a>>, ExecutionError> {
    paths.iter().map(|path| {
        if path.steps().len() != 1 {
            return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed));
        }
        path.steps()[0].property().get(root.clone()).map_err(|_| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed))
    }).collect()
}

fn execute_path(
    path: &CompiledPropertyPath,
    root: ReflectedRef<'_>,
    validator: &qubit_validator::BoundValidator,
    dependencies: &[PropertyValue<'_>],
    on_none: OnNone,
    report: &mut ValidationReport,
    rule_path: &ValidationPath,
    occurrence: usize,
    rule_id: qubit_validator::ValidatorId,
    options: &ValidationOptions,
) -> Result<(), ExecutionError> {
    if path.steps().len() != 1 {
        return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed));
    }
    let property = path.steps()[0].property();
    let value = property.get(root).map_err(|_| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed))?;
    let input = match &value {
        PropertyValue::OptionalBorrowed(None) if path.is_optional() => ValidationValue::Missing,
        PropertyValue::OptionalBorrowed(Some(value)) => reflected_value(value),
        PropertyValue::Borrowed(value) => reflected_value(value),
        PropertyValue::Owned(value) => owned_value(value),
        PropertyValue::BorrowedSlice(_) => return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)),
        PropertyValue::OptionalBorrowed(None) => ValidationValue::Missing,
    };
    let mut values = Vec::with_capacity(dependencies.len());
    for dependency in dependencies {
        values.push(property_value(dependency));
    }
    let context = BoundValidationContext::new(&values);
    match input {
        ValidationValue::Missing if path.is_optional() => {
            if on_none == OnNone::Reject {
                report.push(Violation::new(rule_id, ViolationCode::new("value.required")).with_path(rule_path.clone()));
            } else {
                report.record_skip(SkippedValidation::new(occurrence, rule_path.clone(), SkipReason::MissingOptional));
            }
            Ok(())
        }
        input => match validator.validate(input, &context)? {
            RuleOutcome::Valid => Ok(()),
            RuleOutcome::Invalid(violations) => {
                if violations.is_empty() { return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation)); }
                for violation in violations {
                    if report.violations().len() >= options.max_violations() { report.mark_truncated(); break; }
                    report.push(prefix_violation(violation, rule_path));
                }
                Ok(())
            }
            RuleOutcome::Skipped { reason, prerequisites } => {
                if matches!(reason, SkipReason::MissingOptional) && !prerequisites.is_empty() || matches!(reason, SkipReason::FailedPrerequisite) && prerequisites.is_empty() {
                    return Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation));
                }
                for violation in prerequisites { report.push(prefix_violation(violation, rule_path)); }
                report.record_skip(SkippedValidation::new(occurrence, rule_path.clone(), reason));
                Ok(())
            }
        },
    }
}

fn reflected_value<'a>(value: &'a ReflectedRef<'_>) -> ValidationValue<'a> {
    if let Some(text) = value.as_str() {
        return ValidationValue::Text(text);
    }
    if let Some(text) = value.downcast_ref::<String>() {
        return ValidationValue::Text(text.as_str());
    }
    ValidationValue::Typed(value.as_any().expect("non-text reflected value"))
}

fn property_value<'a>(value: &'a PropertyValue<'_>) -> ValidationValue<'a> {
    match value {
        PropertyValue::Borrowed(value) => reflected_value(value),
        PropertyValue::OptionalBorrowed(Some(value)) => reflected_value(value),
        PropertyValue::OptionalBorrowed(None) => ValidationValue::Missing,
        PropertyValue::Owned(value) => owned_value(value),
        PropertyValue::BorrowedSlice(_) => ValidationValue::Missing,
    }
}

fn owned_value<'a>(value: &'a qubit_reflect::ReflectedOwned) -> ValidationValue<'a> {
    if let Some(text) = value.downcast_ref::<String>() {
        return ValidationValue::Text(text.as_str());
    }
    ValidationValue::Typed(value.as_any().expect("owned values are Any-compatible"))
}

fn path_for(path: &CompiledPropertyPath) -> ValidationPath {
    path.steps().iter().fold(ValidationPath::root(), |path, step| path.with_field(step.property().name()))
}

fn selected(selection: &ValidationSelection, path: &ValidationPath) -> bool {
    match selection {
        ValidationSelection::All => true,
        ValidationSelection::Fields(fields) => fields.iter().any(|field| field_matches(field, path)),
    }
}

fn field_matches(field: &FieldPath, path: &ValidationPath) -> bool {
    let fields: Vec<&str> = path.as_segments().iter().filter_map(|segment| match segment { qubit_validator::PathSegment::Field(name) => Some(name.as_ref()), _ => None }).collect();
    field.segments().iter().map(String::as_str).eq(fields)
}

fn prefix_violation(violation: Violation, prefix: &ValidationPath) -> Violation {
    let path = prefix.as_segments().iter().chain(violation.path().as_segments()).fold(ValidationPath::root(), |path, segment| match segment {
        qubit_validator::PathSegment::Field(field) => path.with_field(field.clone()),
        qubit_validator::PathSegment::Index(index) => path.with_index(*index),
        qubit_validator::PathSegment::MapEntry(index) => path.with_map_entry(*index),
        qubit_validator::PathSegment::MapKey => path.with_map_key(),
        qubit_validator::PathSegment::MapValue => path.with_map_value(),
    });
    violation.with_path(path)
}
