//! Borrow-preserving execution of a bound validation plan.

use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;
use qubit_validator::BoundValidationContext;
use qubit_validator::BoundValidator;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::PathSegment;
use qubit_validator::RuleOutcome;
use qubit_validator::SkipReason;
use qubit_validator::SkippedValidation;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorId;
use qubit_validator::Violation;
use qubit_validator::ViolationCode;

use super::ValidationPlan;
use super::compiled_property_path::CompiledPropertyPath;
use super::model_validation_error::ModelValidationError;
use super::standard_constraints::StandardTarget;
use super::validation_options::FieldPath;
use super::validation_options::ValidationMode;
use super::validation_options::ValidationOptions;
use super::validation_options::ValidationSelection;
use crate::OnNone;
use crate::PropertyValue;
use crate::SelectorPosition;

impl<'a> ValidationPlan<'a> {
    /// Executes all selected bound rules against one borrowed model value.
    pub fn validate(
        &self,
        value: ReflectedRef<'_>,
        options: &ValidationOptions,
    ) -> Result<ValidationReport, ModelValidationError> {
        let mut report = ValidationReport::new();
        let mut nodes = 1usize;
        let mut comparisons = 0usize;
        if nodes > options.max_nodes() || options.max_depth() == 0 {
            return Err(ModelValidationError::new(
                ExecutionError::new(ExecutionErrorKind::TraversalLimit),
                report,
            ));
        }
        for binding in self.bindings() {
            let path = path_for(binding.value());
            if !selected(options.selection(), &path) {
                continue;
            }
            if path.as_segments().len() > options.max_depth() {
                return Err(ModelValidationError::new(
                    ExecutionError::new(ExecutionErrorKind::TraversalLimit)
                        .with_rule(binding.rule_id()),
                    report,
                ));
            }
            if let Err(error) = consume_node(&mut nodes, options, binding.rule_id(), &path) {
                return Err(ModelValidationError::new(error, report));
            }
            let (dependencies, dependency_paths) =
                match read_dependency_values(binding.dependencies(), value.clone()) {
                    Ok(values) => values,
                    Err(error) => {
                        return Err(ModelValidationError::new(
                            error.with_rule(binding.rule_id()).with_path(path.clone()),
                            report,
                        ));
                    }
                };
            let result = match binding.selector() {
                Some(selector) => execute_selector(
                    binding.value(),
                    value.clone(),
                    selector,
                    binding.validator(),
                    &mut report,
                    &path,
                    binding.occurrence(),
                    options,
                    &mut nodes,
                    &mut comparisons,
                ),
                None => execute_path(
                    binding.value(),
                    value.clone(),
                    binding.validator(),
                    &dependencies,
                    &dependency_paths,
                    binding.on_none(),
                    &mut report,
                    &path,
                    binding.occurrence(),
                    binding.rule_id(),
                    options,
                    binding.standard_target(),
                ),
            };
            if let Err(error) = result {
                return Err(ModelValidationError::new(
                    error.with_rule(binding.rule_id()).with_path(path),
                    report,
                ));
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

fn consume_node(
    nodes: &mut usize,
    options: &ValidationOptions,
    rule_id: ValidatorId,
    path: &ValidationPath,
) -> Result<(), ExecutionError> {
    if *nodes == options.max_nodes() {
        return Err(ExecutionError::new(ExecutionErrorKind::TraversalLimit)
            .with_rule(rule_id)
            .with_path(path.clone()));
    }
    *nodes += 1;
    Ok(())
}

fn read_dependency_values<'a>(
    paths: &[CompiledPropertyPath],
    root: ReflectedRef<'a>,
) -> Result<(Vec<PropertyValue<'a>>, Vec<ValidationPath>), ExecutionError> {
    let mut values = Vec::with_capacity(paths.len());
    let mut dependency_paths = Vec::with_capacity(paths.len());
    for path in paths {
        values.push(read_path(path, root.clone())?);
        dependency_paths.push(path_for(path));
    }
    Ok((values, dependency_paths))
}

/// Reads every step of a compiled path while preserving the root borrow.
fn read_path<'a>(
    path: &CompiledPropertyPath,
    root: ReflectedRef<'a>,
) -> Result<PropertyValue<'a>, ExecutionError> {
    let mut receiver = root;
    for (index, step) in path.steps().iter().enumerate() {
        let output = step
            .property()
            .get(receiver)
            .map_err(|_| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed))?;
        if index + 1 == path.steps().len() {
            return Ok(output);
        }
        receiver = match output {
            PropertyValue::Borrowed(value) => value,
            PropertyValue::OptionalBorrowed(Some(value)) => value,
            PropertyValue::OptionalBorrowed(None) => {
                return Ok(PropertyValue::OptionalBorrowed(None));
            }
            PropertyValue::Owned(_) | PropertyValue::BorrowedSlice(_) => {
                return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed));
            }
        };
    }
    Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed))
}

#[allow(clippy::too_many_arguments)]
fn execute_path(
    path: &CompiledPropertyPath,
    root: ReflectedRef<'_>,
    validator: &BoundValidator,
    dependencies: &[PropertyValue<'_>],
    dependency_paths: &[ValidationPath],
    on_none: OnNone,
    report: &mut ValidationReport,
    rule_path: &ValidationPath,
    occurrence: usize,
    rule_id: ValidatorId,
    options: &ValidationOptions,
    standard_target: Option<StandardTarget>,
) -> Result<(), ExecutionError> {
    let value = read_path(path, root)?;
    let sequence_count = match &value {
        PropertyValue::BorrowedSlice(values)
            if matches!(standard_target, Some(StandardTarget::SequenceCount)) =>
        {
            Some(values.len())
        }
        _ => None,
    };
    let input = match &value {
        PropertyValue::OptionalBorrowed(None) if path.is_optional() => ValidationValue::Missing,
        PropertyValue::OptionalBorrowed(Some(value)) => reflected_value(value),
        PropertyValue::Borrowed(value) => reflected_value(value),
        PropertyValue::Owned(value) => owned_value(value),
        PropertyValue::BorrowedSlice(_) => {
            if !matches!(standard_target, Some(StandardTarget::SequenceCount)) {
                return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed));
            }
            ValidationValue::Typed(sequence_count.as_ref().expect("slice count"))
        }
        PropertyValue::OptionalBorrowed(None) => ValidationValue::Missing,
    };
    let mut values = Vec::with_capacity(dependencies.len());
    for dependency in dependencies {
        values.push(property_value(dependency));
    }
    let context = BoundValidationContext::new_with_paths(&values, dependency_paths)?;
    match input {
        ValidationValue::Missing if path.is_optional() => {
            if on_none == OnNone::Reject {
                report.push(
                    Violation::new(rule_id, ViolationCode::new("value.required"))
                        .with_path(rule_path.clone()),
                );
            } else {
                report.record_skip(SkippedValidation::new(
                    occurrence,
                    rule_path.clone(),
                    SkipReason::MissingOptional,
                ));
            }
            Ok(())
        }
        input => match validator.validate(input, &context)? {
            RuleOutcome::Valid => Ok(()),
            RuleOutcome::Invalid(violations) => {
                if violations.is_empty() {
                    return Err(ExecutionError::new(
                        ExecutionErrorKind::AdapterContractViolation,
                    ));
                }
                for violation in violations {
                    if report.violations().len() >= options.max_violations() {
                        report.mark_truncated();
                        break;
                    }
                    report.push(prefix_violation(violation, rule_path));
                }
                Ok(())
            }
            RuleOutcome::Skipped {
                reason,
                prerequisites,
            } => {
                if matches!(reason, SkipReason::MissingOptional) && !prerequisites.is_empty()
                    || matches!(reason, SkipReason::FailedPrerequisite) && prerequisites.is_empty()
                {
                    return Err(ExecutionError::new(
                        ExecutionErrorKind::AdapterContractViolation,
                    ));
                }
                for violation in prerequisites {
                    report.push(prefix_violation(violation, rule_path));
                }
                report.record_skip(SkippedValidation::new(
                    occurrence,
                    rule_path.clone(),
                    reason,
                ));
                Ok(())
            }
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_selector(
    path: &CompiledPropertyPath,
    root: ReflectedRef<'_>,
    selector: &super::validation_plan::SelectorBinding,
    validator: &BoundValidator,
    report: &mut ValidationReport,
    rule_path: &ValidationPath,
    occurrence: usize,
    options: &ValidationOptions,
    nodes: &mut usize,
    comparisons: &mut usize,
) -> Result<(), ExecutionError> {
    if !matches!(selector.position(), SelectorPosition::Element) {
        return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed));
    }
    let value = read_path(path, root)?;
    let PropertyValue::BorrowedSlice(values) = value else {
        return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed));
    };
    for index in 0..values.len() {
        let element = values
            .get(index)
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed))?;
        let element_path = rule_path.clone().with_index(index);
        if element_path.as_segments().len() > options.max_depth() {
            return Err(ExecutionError::new(ExecutionErrorKind::TraversalLimit));
        }
        if *nodes == options.max_nodes() {
            return Err(ExecutionError::new(ExecutionErrorKind::TraversalLimit));
        }
        *nodes += 1;
        if *comparisons == options.max_comparisons() {
            return Err(ExecutionError::new(ExecutionErrorKind::TraversalLimit));
        }
        *comparisons += 1;
        let context = BoundValidationContext::new_with_paths(&[], &[])?;
        let input = reflected_value(&element);
        match validator.validate(input, &context)? {
            RuleOutcome::Valid => {}
            RuleOutcome::Invalid(violations) => {
                if violations.is_empty() {
                    return Err(ExecutionError::new(
                        ExecutionErrorKind::AdapterContractViolation,
                    ));
                }
                for violation in violations {
                    if report.violations().len() >= options.max_violations() {
                        report.mark_truncated();
                        break;
                    }
                    report.push(prefix_violation(violation, &element_path));
                }
            }
            RuleOutcome::Skipped {
                reason,
                prerequisites,
            } => {
                if matches!(reason, SkipReason::MissingOptional) && !prerequisites.is_empty()
                    || matches!(reason, SkipReason::FailedPrerequisite) && prerequisites.is_empty()
                {
                    return Err(ExecutionError::new(
                        ExecutionErrorKind::AdapterContractViolation,
                    ));
                }
                for violation in prerequisites {
                    report.push(prefix_violation(violation, &element_path));
                }
                report.record_skip(SkippedValidation::new(occurrence, element_path, reason));
            }
        }
        if options.mode() == ValidationMode::FailFast && !report.violations().is_empty() {
            report.mark_truncated();
            break;
        }
    }
    Ok(())
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

fn owned_value<'a>(value: &'a ReflectedOwned) -> ValidationValue<'a> {
    if let Some(text) = value.downcast_ref::<String>() {
        return ValidationValue::Text(text.as_str());
    }
    ValidationValue::Typed(value.as_any().expect("owned values are Any-compatible"))
}

fn path_for(path: &CompiledPropertyPath) -> ValidationPath {
    path.steps()
        .iter()
        .fold(ValidationPath::root(), |path, step| {
            path.with_field(step.property().name())
        })
}

fn selected(selection: &ValidationSelection, path: &ValidationPath) -> bool {
    match selection {
        ValidationSelection::All => true,
        ValidationSelection::Fields(fields) => {
            fields.iter().any(|field| field_matches(field, path))
        }
    }
}

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

fn prefix_violation(violation: Violation, prefix: &ValidationPath) -> Violation {
    let path = prefix
        .as_segments()
        .iter()
        .chain(violation.path().as_segments())
        .fold(ValidationPath::root(), |path, segment| match segment {
            PathSegment::Field(field) => path.with_field(field.clone()),
            PathSegment::Index(index) => path.with_index(*index),
            PathSegment::MapEntry(index) => path.with_map_entry(*index),
            PathSegment::MapKey => path.with_map_key(),
            PathSegment::MapValue => path.with_map_value(),
        });
    violation.with_path(path)
}
