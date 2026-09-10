// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared access checks and rule binding for every declaration location.

use qubit_reflect::descriptor::TypeKind;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidator;
use qubit_validator::InputType;
use qubit_validator::ValidatorRegistry;

use super::execution_declaration::ExecutionDeclaration;
use super::field_rule_binding::FieldRuleBinding;
use super::selector_binding::SelectorBinding;
use super::validation_occurrence::ValidationOccurrence;
use crate::metadata::ConstraintMetadata;
use crate::metadata::GetterOutputKind;
use crate::metadata::OnNone;
use crate::metadata::PropertyPath;
use crate::metadata::SelectorPosition;
use crate::metadata::TargetMode;
use crate::metadata::TypeMetadata;
use crate::metadata::ValidatorMetadata;
use crate::resolve::ModelGraph;
use crate::validation::ValidationBuildError;
use crate::validation::compiled_property_path::CompiledPropertyPath;
use crate::validation::standard_constraints;
use crate::validation::validator_arguments;

/// Checks actual adapter shape without calling getters or binding registries.
pub(crate) fn check_access(
    occurrence: &ValidationOccurrence,
    graph: &ModelGraph<'_>,
) -> Result<CompiledPropertyPath, Box<ValidationBuildError>> {
    let requires_slice = match occurrence.declaration {
        ExecutionDeclaration::Traversal => return Err(Box::new(ValidationBuildError::unsupported(occurrence))),
        ExecutionDeclaration::Constraint(_) if occurrence.selector.is_some() => {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
        ExecutionDeclaration::Constraint(ConstraintMetadata::Text(_)) => false,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Sequence(value)) if !value.unique_items() => true,
        ExecutionDeclaration::Constraint(_) => return Err(Box::new(ValidationBuildError::unsupported(occurrence))),
        ExecutionDeclaration::Validator(value) => {
            if let Some(selector) = occurrence.selector {
                if selector != SelectorPosition::Element
                    || !value.depends_on().is_empty()
                    || !value.dependency_bindings().is_empty()
                {
                    return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
                }
                true
            } else {
                false
            }
        }
    };
    let target = if requires_slice {
        TargetMode::Container
    } else {
        match occurrence.declaration {
            ExecutionDeclaration::Validator(value) => value.target(),
            _ => TargetMode::Value,
        }
    };
    let path = CompiledPropertyPath::compile(occurrence.root, &PropertyPath::new(&occurrence.segments), graph, target)
        .map_err(|error| {
            if error.kind() == BindErrorKind::UnsupportedConstraint {
                ValidationBuildError::unsupported(occurrence)
            } else {
                ValidationBuildError::at_occurrence(occurrence, error)
            }
        })?;
    let slice = path
        .steps()
        .last()
        .and_then(|step| step.property().getter())
        .is_some_and(|getter| getter.output_kind() == GetterOutputKind::BorrowedSlice);
    if slice != requires_slice {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    }
    if matches!(
        occurrence.declaration,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Text(_))
    ) && path.input_type() != InputType::Text
    {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    }
    if let ExecutionDeclaration::Validator(declaration) = occurrence.declaration {
        if occurrence.selector.is_some() {
            let descriptor = path
                .steps()
                .last()
                .and_then(|step| step.property().getter())
                .and_then(|getter| getter.output_type().as_resolved())
                .and_then(|descriptor| descriptor.as_slice())
                .and_then(|slice| slice.element_type().as_resolved())
                .ok_or_else(|| Box::new(ValidationBuildError::unsupported(occurrence)))?;
            // Slice access borrows the element itself. It has no adapter to
            // expose an optional value or a pointee with the promised lifetime.
            if declaration.target() == TargetMode::Value
                && (descriptor.as_optional().is_some() || descriptor.as_smart_pointer().is_some())
            {
                return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
            }
        }
        let prefix = &occurrence.segments[..occurrence.segments.len() - 1];
        for binding in declaration.dependency_bindings() {
            CompiledPropertyPath::compile_dependency(
                occurrence.root,
                prefix,
                binding,
                graph,
                &[],
                InputType::of::<()>(),
            )
            .map_err(|error| access_error(occurrence, error))?;
        }
        if declaration.dependency_bindings().is_empty() {
            for dependency in declaration.depends_on() {
                let segments: Vec<_> = prefix.iter().chain(dependency.segments()).copied().collect();
                CompiledPropertyPath::compile(occurrence.root, &PropertyPath::new(&segments), graph, TargetMode::Value)
                    .map_err(|error| access_error(occurrence, error))?;
            }
        }
    }
    Ok(path)
}

/// Distinguishes an unsupported access adapter from a malformed declaration.
fn access_error(occurrence: &ValidationOccurrence, error: BindError) -> ValidationBuildError {
    if error.kind() == BindErrorKind::UnsupportedConstraint {
        ValidationBuildError::unsupported(occurrence)
    } else {
        ValidationBuildError::at_occurrence(occurrence, error)
    }
}

/// Binds one occurrence using the same path and dependency rules at every
/// depth.
pub(crate) fn bind(
    occurrence: &ValidationOccurrence,
    graph: &ModelGraph<'_>,
    validators: &ValidatorRegistry,
    ancestors: &[&'static TypeMetadata],
) -> Result<Vec<FieldRuleBinding>, Vec<ValidationBuildError>> {
    let value = check_access(occurrence, graph).map_err(|error| vec![*error])?;
    match occurrence.declaration {
        ExecutionDeclaration::Constraint(constraint) => {
            let standards = standard_constraints::bind(constraint, validators).map_err(|errors| {
                errors
                    .into_iter()
                    .map(|error| ValidationBuildError::at_occurrence(occurrence, error))
                    .collect::<Vec<_>>()
            })?;
            Ok(standards
                .into_iter()
                .map(|standard| FieldRuleBinding {
                    context: occurrence.clone(),
                    occurrence: occurrence.ordinal,
                    rule_id: standard.validator.rule_id().expect("bound rule ID"),
                    value: value.clone(),
                    dependencies: Box::new([]),
                    validator: standard.validator,
                    on_none: OnNone::Skip,
                    selector: None,
                    standard_target: Some(standard.target),
                })
                .collect())
        }
        ExecutionDeclaration::Validator(declaration) => {
            let input = if occurrence.selector.is_some() {
                let descriptor = value
                    .steps()
                    .last()
                    .and_then(|step| step.property().getter())
                    .and_then(|getter| getter.output_type().as_resolved())
                    .and_then(|descriptor| descriptor.as_slice())
                    .and_then(|slice| slice.element_type().as_resolved())
                    .ok_or_else(|| vec![ValidationBuildError::unsupported(occurrence)])?;
                if matches!(descriptor.kind(), TypeKind::Text(_)) {
                    InputType::Text
                } else {
                    InputType::Typed(descriptor.type_id())
                }
            } else {
                value.input_type()
            };
            let params = validator_arguments(declaration.params());
            let validator = validators
                .bind(declaration.declared_id(), input, &params)
                .map_err(|error| vec![ValidationBuildError::at_occurrence(occurrence, error)])?;
            if occurrence.selector.is_some() && !validator.dependency_specs().is_empty() {
                return Err(vec![ValidationBuildError::unsupported(occurrence)]);
            }
            let dependencies =
                bind_dependencies(occurrence, declaration, &validator, graph, ancestors).map_err(|errors| {
                    errors
                        .into_iter()
                        .map(|error| access_error(occurrence, error))
                        .collect::<Vec<_>>()
                })?;
            Ok(vec![FieldRuleBinding {
                context: occurrence.clone(),
                occurrence: occurrence.ordinal,
                rule_id: validator.rule_id().expect("bound rule ID"),
                value,
                dependencies,
                validator,
                on_none: declaration.on_none(),
                selector: occurrence.selector.map(|position| SelectorBinding { position }),
                standard_target: None,
            }])
        }
        ExecutionDeclaration::Traversal => Err(vec![ValidationBuildError::unsupported(occurrence)]),
    }
}

/// Compiles dependencies relative to the declaring object's usage path.
fn bind_dependencies(
    occurrence: &ValidationOccurrence,
    declaration: &ValidatorMetadata,
    validator: &BoundValidator,
    graph: &ModelGraph<'_>,
    ancestors: &[&'static TypeMetadata],
) -> Result<Box<[CompiledPropertyPath]>, Vec<BindError>> {
    let declared = declaration.dependency_bindings();
    let legacy = declaration.depends_on();
    let specs = validator.dependency_specs();
    let count = if declared.is_empty() {
        legacy.len()
    } else {
        declared.len()
    };
    let rule_id = validator.rule_id().expect("bound rule ID");
    if specs.len() != count {
        return Err(vec![
            BindError::new(if specs.len() > count {
                BindErrorKind::MissingDependencyDeclaration
            } else {
                BindErrorKind::UnknownDependencyDeclaration
            })
            .with_rule(rule_id),
        ]);
    }
    let prefix = &occurrence.segments[..occurrence.segments.len() - 1];
    let mut dependencies = Vec::new();
    let mut errors = Vec::new();
    for (slot, spec) in specs.iter().enumerate() {
        let path = if declared.is_empty() {
            let dependency = legacy[slot];
            if dependency.to_string() != spec.name() {
                errors.push(
                    BindError::new(BindErrorKind::UnknownDependencyDeclaration)
                        .with_rule(rule_id)
                        .with_dependency(spec.name()),
                );
                continue;
            }
            let segments: Vec<_> = prefix.iter().chain(dependency.segments()).copied().collect();
            CompiledPropertyPath::compile(occurrence.root, &PropertyPath::new(&segments), graph, TargetMode::Value)
        } else {
            let Some(binding) = declared.iter().find(|binding| binding.name() == spec.name()) else {
                errors.push(
                    BindError::new(BindErrorKind::UnknownDependencyDeclaration)
                        .with_rule(rule_id)
                        .with_dependency(spec.name()),
                );
                continue;
            };
            CompiledPropertyPath::compile_dependency(occurrence.root, prefix, binding, graph, ancestors, spec.input())
        };
        match path {
            Ok(path) if path.input_type() == spec.input() => dependencies.push(path),
            Ok(_) => errors.push(
                BindError::new(BindErrorKind::DependencyTypeMismatch)
                    .with_rule(rule_id)
                    .with_dependency(spec.name()),
            ),
            Err(error) => errors.push(error.with_rule(rule_id).with_dependency(spec.name())),
        }
    }
    if errors.is_empty() {
        Ok(dependencies.into_boxed_slice())
    } else {
        Err(errors)
    }
}
