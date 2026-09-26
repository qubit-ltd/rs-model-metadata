// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared access checks and rule binding for every declaration location.

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Utc;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeKind;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidator;
use qubit_validator::InputType;
use qubit_validator::ValidatorRegistry;

use super::execution_declaration::ExecutionDeclaration;
use super::field_rule_binding::FieldExecution;
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
use crate::property::MapLenAdapter;
use crate::resolve::ModelGraph;
use crate::validation::ValidationBuildError;
use crate::validation::compiled_property_path::CompiledPropertyPath;
use crate::validation::standard_constraints;
use crate::validation::validator_arguments::validator_arguments;

/// Checks actual adapter shape without calling getters or binding registries.
pub(crate) fn check_access(
    occurrence: &ValidationOccurrence,
    graph: &ModelGraph<'_>,
) -> Result<CompiledPropertyPath, Box<ValidationBuildError>> {
    let requires_slice = match occurrence.declaration {
        ExecutionDeclaration::Traversal => {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
        ExecutionDeclaration::Constraint(_) if occurrence.selector.is_some() => {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
        ExecutionDeclaration::Constraint(
            ConstraintMetadata::Text(_)
            | ConstraintMetadata::Map(_)
            | ConstraintMetadata::Decimal(_)
            | ConstraintMetadata::Time(_),
        ) => false,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Sequence(_)) => true,
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
    let is_map = matches!(
        occurrence.declaration,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Map(_))
    );
    let optional_map_getter = is_map
        && occurrence
            .segments
            .last()
            .and_then(|name| {
                graph
                    .properties(occurrence.owner)
                    .and_then(|properties| properties.property(name))
            })
            .and_then(|property| property.getter())
            .is_some_and(|getter| getter.output_kind() == GetterOutputKind::OptionalBorrowed);
    let direct_optional_scalar = matches!(
        occurrence.declaration,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Decimal(_) | ConstraintMetadata::Time(_))
    ) && occurrence
        .segments
        .last()
        .and_then(|name| {
            graph
                .properties(occurrence.owner)
                .and_then(|properties| properties.property(name))
        })
        .is_some_and(|property| {
            property.getter().is_none()
                && property
                    .descriptor()
                    .is_some_and(|descriptor| descriptor.as_optional().is_some())
        });
    let target = if requires_slice || is_map && !optional_map_getter || direct_optional_scalar {
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
    let path = if direct_optional_scalar {
        match path.unwrap_terminal_optional() {
            Ok(path) => path,
            Err(error) => return Err(Box::new(ValidationBuildError::at_occurrence(occurrence, error))),
        }
    } else {
        path
    };
    let slice = path
        .steps()
        .last()
        .and_then(|step| step.property().getter())
        .is_some_and(|getter| getter.output_kind() == GetterOutputKind::BorrowedSlice);
    if slice != requires_slice {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    }
    if matches!(occurrence.declaration, ExecutionDeclaration::Constraint(ConstraintMetadata::Sequence(sequence)) if sequence.unique_items())
    {
        let item_eq = occurrence.field.collection_ops().and_then(|ops| ops.item_eq());
        let elements_match = occurrence
            .field
            .descriptor()
            .zip(
                path.steps()
                    .last()
                    .and_then(|step| step.property().getter())
                    .and_then(|getter| getter.output_type().as_resolved()),
            )
            .is_some_and(|(declared, output)| sequence_element_matches(declared, output));
        if item_eq.is_none() || !elements_match {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
    }
    if is_map {
        let Some(property) = path.steps().last().map(|step| step.property()) else {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        };
        let map_type = property.descriptor().is_some_and(|descriptor| {
            matches!(descriptor.kind(), TypeKind::Map)
                || descriptor
                    .as_optional()
                    .and_then(|optional| optional.element_type().as_resolved())
                    .is_some_and(|inner| matches!(inner.kind(), TypeKind::Map))
        });
        let borrowed_output = property.getter().is_none_or(|getter| {
            matches!(
                getter.output_kind(),
                GetterOutputKind::Borrowed | GetterOutputKind::OptionalBorrowed
            )
        });
        let has_adapter = occurrence
            .field
            .collection_ops()
            .and_then(|ops| ops.map_len())
            .is_some();
        if !map_type || !borrowed_output || !has_adapter {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
    }
    if matches!(
        occurrence.declaration,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Text(_))
    ) && path.input_type() != InputType::Text
    {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    }
    if let ExecutionDeclaration::Constraint(constraint) = occurrence.declaration {
        let input = path.input_type();
        let accepted = match constraint {
            ConstraintMetadata::Decimal(_) => input == InputType::of::<BigDecimal>(),
            ConstraintMetadata::Time(_) => {
                input == InputType::of::<DateTime<Utc>>()
                    || input == InputType::of::<NaiveDateTime>()
                    || input == InputType::of::<NaiveTime>()
            }
            _ => true,
        };
        if !accepted {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
    }
    if let ExecutionDeclaration::Validator(declaration) = occurrence.declaration {
        if occurrence.selector.is_some() {
            let Some(descriptor) = path
                .steps()
                .last()
                .and_then(|step| step.property().getter())
                .and_then(|getter| getter.output_type().as_resolved())
                .and_then(|descriptor| descriptor.as_slice())
                .and_then(|slice| slice.element_type().as_resolved())
            else {
                return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
            };
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

/// Confirms that a borrowed slice exposes the declared collection element
/// type, independent of the slice adapter's function pointer.
fn sequence_element_matches(declared: &TypeDescriptor, output: &TypeDescriptor) -> bool {
    let declared_element = declared
        .as_sequence()
        .map(|sequence| sequence.element_type())
        .or_else(|| declared.as_array().map(|array| array.element_type()))
        .or_else(|| declared.as_slice().map(|slice| slice.element_type()))
        .and_then(|element| element.as_resolved());
    let output_element = output.as_slice().and_then(|slice| slice.element_type().as_resolved());
    declared_element
        .zip(output_element)
        .is_some_and(|(declared, output)| declared.type_id() == output.type_id())
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
    let map_len: Option<MapLenAdapter> = if matches!(
        occurrence.declaration,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Map(_))
    ) {
        occurrence.field.collection_ops().and_then(|ops| ops.map_len())
    } else {
        None
    };
    match occurrence.declaration {
        ExecutionDeclaration::Constraint(constraint) => {
            let standards =
                standard_constraints::bind(constraint, validators, value.input_type()).map_err(|errors| {
                    errors
                        .into_iter()
                        .map(|error| ValidationBuildError::at_occurrence(occurrence, error))
                        .collect::<Vec<_>>()
                })?;
            let mut bindings: Vec<_> = standards
                .into_iter()
                .map(|standard| FieldRuleBinding {
                    context: occurrence.clone(),
                    occurrence: occurrence.ordinal,
                    rule_id: standard.validator.rule_id(),
                    value: value.clone(),
                    dependencies: Box::new([]),
                    execution: FieldExecution::Registry {
                        validator: standard.validator,
                        target: Some(standard.target),
                        map_len,
                    },
                    on_none: OnNone::Skip,
                    selector: None,
                })
                .collect();
            if matches!(constraint, ConstraintMetadata::Sequence(sequence) if sequence.unique_items()) {
                let Some(item_eq) = occurrence.field.collection_ops().and_then(|ops| ops.item_eq()) else {
                    return Err(vec![ValidationBuildError::unsupported(occurrence)]);
                };
                bindings.push(FieldRuleBinding {
                    context: occurrence.clone(),
                    occurrence: occurrence.ordinal,
                    rule_id: standard_constraints::SEQUENCE_UNIQUE_ID,
                    value,
                    dependencies: Box::new([]),
                    execution: FieldExecution::SequenceUnique { item_eq },
                    on_none: OnNone::Skip,
                    selector: None,
                });
            }
            Ok(bindings)
        }
        ExecutionDeclaration::Validator(declaration) => {
            let input = if occurrence.selector.is_some() {
                let Some(descriptor) = value
                    .steps()
                    .last()
                    .and_then(|step| step.property().getter())
                    .and_then(|getter| getter.output_type().as_resolved())
                    .and_then(|descriptor| descriptor.as_slice())
                    .and_then(|slice| slice.element_type().as_resolved())
                else {
                    return Err(vec![ValidationBuildError::unsupported(occurrence)]);
                };
                if matches!(descriptor.kind(), TypeKind::Text(_)) {
                    InputType::Text
                } else {
                    InputType::Typed(descriptor.type_id())
                }
            } else {
                value.input_type()
            };
            let arguments = validator_arguments(declaration.params());
            let validator = validators
                .bind(declaration.declared_id(), input, &arguments)
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
                rule_id: validator.rule_id(),
                value,
                dependencies,
                execution: FieldExecution::Registry {
                    validator,
                    target: None,
                    map_len: None,
                },
                on_none: declaration.on_none(),
                selector: occurrence.selector.map(|position| SelectorBinding { position }),
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
    let rule_id = validator.rule_id();
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
            Ok(path)
                if path.input_type() == spec.input()
                    && path.deferred().is_empty()
                    && path.is_optional()
                    && !spec.optional() =>
            {
                errors.push(
                    BindError::new(BindErrorKind::DependencyOptionalityMismatch)
                        .with_rule(rule_id)
                        .with_dependency(spec.name()),
                )
            }
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

#[cfg(test)]
mod tests {
    use qubit_reflect::TypeDescriptor;

    use super::sequence_element_matches;

    #[test]
    fn test_unique_element_shape_requires_matching_type_ids() {
        assert!(sequence_element_matches(
            TypeDescriptor::of::<Vec<i32>>(),
            TypeDescriptor::of::<[i32]>(),
        ));
        assert!(sequence_element_matches(
            TypeDescriptor::of::<[i32; 2]>(),
            TypeDescriptor::of::<[i32]>(),
        ));
        assert!(!sequence_element_matches(
            TypeDescriptor::of::<Vec<i32>>(),
            TypeDescriptor::of::<[u8]>(),
        ));
    }
}
