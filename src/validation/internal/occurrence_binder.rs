// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared access checks and rule binding for every declaration location.

use std::collections::HashMap;

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Utc;
use qubit_reflect::ReflectionContext;
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
use crate::metadata::DependencyBindingMetadata;
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
///
/// # Parameters
///
/// * `occurrence` - The declaration occurrence and its resolved field metadata.
/// * `graph` - The model graph used to resolve getters and property paths.
///
/// # Returns
///
/// Returns the compiled value path when its adapter shape is supported.
///
/// # Errors
///
/// Returns a boxed build error when the declaration cannot be represented by
/// the available adapters or its path is invalid.
pub(crate) fn check_access(
    occurrence: &ValidationOccurrence,
    graph: &ModelGraph<'_>,
) -> Result<CompiledPropertyPath, Box<ValidationBuildError>> {
    let context = graph
        .registry()
        .context()
        .ok_or_else(|| Box::new(ValidationBuildError::unsupported(occurrence)))?;
    let requires_slice = match occurrence.declaration {
        ExecutionDeclaration::Traversal => {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
        ExecutionDeclaration::Constraint(
            ConstraintMetadata::Text(_)
            | ConstraintMetadata::Map(_)
            | ConstraintMetadata::Decimal(_)
            | ConstraintMetadata::Time(_),
        ) => false,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Sequence(_)) if occurrence.selector.is_none() => true,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Sequence(_)) => false,
        ExecutionDeclaration::Validator(value) => {
            if let Some(selector) = occurrence.selector {
                if !matches!(
                    selector,
                    SelectorPosition::Element | SelectorPosition::MapKey | SelectorPosition::MapValue
                ) || !value.depends_on().is_empty()
                    || !value.dependency_bindings().is_empty()
                {
                    return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
                }
                false
            } else {
                false
            }
        }
    };
    if occurrence.selector.is_some()
        && occurrence
            .field
            .name()
            .and_then(|name| graph.properties(occurrence.owner)?.property(name))
            .and_then(|property| property.getter())
            .is_some_and(|getter| getter.output_kind() == GetterOutputKind::Owned)
    {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    }
    let is_map = matches!(
        occurrence.declaration,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Map(_))
    );
    let scalar_constraint = matches!(
        occurrence.declaration,
        ExecutionDeclaration::Constraint(
            ConstraintMetadata::Text(_) | ConstraintMetadata::Decimal(_) | ConstraintMetadata::Time(_)
        )
    );
    let compile = |target| -> Result<_, Box<ValidationBuildError>> {
        let compiled = if occurrence.enum_variant.is_some() {
            CompiledPropertyPath::compile_enum_payload(context, occurrence.root, occurrence, graph, target)
        } else {
            CompiledPropertyPath::compile(
                context,
                occurrence.root,
                &PropertyPath::new(&occurrence.segments),
                graph,
                target,
            )
        };
        compiled.map_err(|error| {
            Box::new(if error.kind() == BindErrorKind::UnsupportedConstraint {
                ValidationBuildError::unsupported(occurrence)
            } else {
                ValidationBuildError::at_occurrence(occurrence, error)
            })
        })
    };
    let initial_target = if requires_slice || occurrence.selector.is_some() {
        TargetMode::Container
    } else {
        match occurrence.declaration {
            ExecutionDeclaration::Validator(value) => value.target(),
            _ => TargetMode::Value,
        }
    };
    // A direct Option<T> field needs container compilation before its
    // terminal descriptor can be projected. The compiled step also includes
    // transparent fields, which are not named graph properties.
    let (path, compiled_target) = match compile(initial_target) {
        Ok(path) => (path, initial_target),
        Err(_) if is_map || scalar_constraint => (compile(TargetMode::Container)?, TargetMode::Container),
        Err(error) => return Err(error),
    };
    let Some(property) = path.steps().last().map(|step| step.property()) else {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    };
    let optional_map_getter = is_map
        && property
            .getter()
            .is_some_and(|getter| getter.output_kind() == GetterOutputKind::OptionalBorrowed);
    let optional_scalar_candidate = scalar_constraint
        && property.getter().is_none()
        && property
            .descriptor_in_owner(occurrence.owner, context)
            .ok()
            .flatten()
            .is_some_and(|descriptor| matches!(descriptor.kind(), TypeKind::Optional(_)));
    let optional_projection_available = property.optional_ops().is_some();
    if optional_scalar_candidate && !optional_projection_available {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    }
    let direct_optional_scalar = optional_scalar_candidate;
    let target = if requires_slice
        || occurrence.selector.is_some()
        || is_map && !optional_map_getter
        || direct_optional_scalar
    {
        TargetMode::Container
    } else {
        match occurrence.declaration {
            ExecutionDeclaration::Validator(value) => value.target(),
            _ => TargetMode::Value,
        }
    };
    let path = if target == compiled_target {
        path
    } else {
        compile(target)?
    };
    let path = if direct_optional_scalar {
        let element = property
            .descriptor_in_owner(occurrence.owner, context)
            .ok()
            .flatten()
            .and_then(|descriptor| generic_type_argument(descriptor, 0))
            .ok_or_else(|| Box::new(ValidationBuildError::unsupported(occurrence)))?;
        match path.unwrap_terminal_optional(element) {
            Ok(path) => path,
            Err(error) => {
                return Err(Box::new(ValidationBuildError::at_occurrence(occurrence, error)));
            }
        }
    } else {
        path
    };
    let slice = path
        .steps()
        .last()
        .and_then(|step| step.property().getter())
        .is_some_and(|getter| getter.output_kind() == GetterOutputKind::BorrowedSlice);
    if requires_slice && !slice {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    }
    if let Some(selector) = occurrence.selector {
        let items = occurrence.field.collection_ops().and_then(|ops| ops.collection_items());
        let supported_type = selector_input_type(context, &path, selector).is_some();
        let borrowed_output = path
            .steps()
            .last()
            .and_then(|step| step.property().getter())
            .is_none_or(|getter| getter.output_kind() != GetterOutputKind::Owned);
        if items.is_none() || !supported_type || !borrowed_output {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
    }
    if matches!(occurrence.declaration, ExecutionDeclaration::Constraint(ConstraintMetadata::Sequence(sequence)) if sequence.unique_items())
    {
        let item_eq = occurrence.field.collection_ops().and_then(|ops| ops.item_eq());
        let elements_match = occurrence
            .owner
            .field_descriptor_in(occurrence.field, context)
            .and_then(|field| field.find_concrete_type())
            .and_then(|declared| generic_type_argument(declared, 0))
            .zip(
                path.steps()
                    .last()
                    .and_then(|step| step.property().getter())
                    .and_then(|getter| getter.output_element_type_id()),
            )
            .is_some_and(|(declared, output)| declared.type_id() == output);
        if item_eq.is_none() || !elements_match {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        }
    }
    if is_map {
        let Some(property) = path.steps().last().map(|step| step.property()) else {
            return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
        };
        let map_type = property
            .descriptor_in_owner(occurrence.owner, context)
            .ok()
            .flatten()
            .is_some_and(|descriptor| {
                if matches!(descriptor.kind(), TypeKind::Map(_)) {
                    return true;
                }
                property.optional_ops().is_some_and(|ops| {
                    generic_type_argument(descriptor, 0).is_some_and(|inner| {
                        inner.type_id() == ops.element_type_id() && matches!(inner.kind(), TypeKind::Map(_))
                    })
                })
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
    let selected_type = occurrence
        .selector
        .and_then(|selector| selector_input_type(context, &path, selector));
    let selected_input = selected_type.map(|descriptor| {
        if matches!(descriptor.kind(), TypeKind::Text(_)) {
            InputType::Text
        } else {
            InputType::Typed(descriptor.type_id())
        }
    });
    if matches!(
        occurrence.declaration,
        ExecutionDeclaration::Constraint(ConstraintMetadata::Text(_))
    ) && selected_input.unwrap_or_else(|| path.input_type()) != InputType::Text
    {
        return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
    }
    if let ExecutionDeclaration::Constraint(constraint) = occurrence.declaration {
        let input = selected_input.unwrap_or_else(|| path.input_type());
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
            let Some(descriptor) = selected_type else {
                return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
            };
            // Selected members are direct borrows, so optional values and
            // smart pointers need a separate projection adapter.
            if declaration.target() == TargetMode::Value
                && matches!(descriptor.kind(), TypeKind::Optional(_) | TypeKind::SmartPointer(_))
            {
                return Err(Box::new(ValidationBuildError::unsupported(occurrence)));
            }
        }
        let prefix = &occurrence.segments[..occurrence.segments.len() - 1];
        for binding in declaration.dependency_bindings() {
            CompiledPropertyPath::compile_dependency(
                context,
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
                CompiledPropertyPath::compile(
                    context,
                    occurrence.root,
                    &PropertyPath::new(&segments),
                    graph,
                    TargetMode::Value,
                )
                .map_err(|error| access_error(occurrence, error))?;
            }
        }
    }
    Ok(path)
}

/// Confirms that a borrowed slice exposes the declared collection element
/// type, independent of the slice adapter's function pointer.
///
/// # Parameters
///
/// * `declared` - The field's declared sequence, array, or slice descriptor.
/// * `output` - The resolved descriptor exposed by the borrowed slice.
///
/// # Returns
///
/// Returns `true` only when both descriptors resolve to the same element type.
#[must_use]
#[cfg(test)]
fn sequence_element_matches(declared: TypeDescriptor<'_>, output: TypeDescriptor<'_>) -> bool {
    let declared_element = generic_type_argument(declared, 0);
    let output_element = generic_type_argument(output, 0);
    declared_element
        .zip(output_element)
        .is_some_and(|(declared, output)| declared.type_id() == output.type_id())
}

fn generic_type_argument<'ctx>(descriptor: TypeDescriptor<'ctx>, index: usize) -> Option<TypeDescriptor<'ctx>> {
    descriptor
        .generic_arguments()
        .iter()
        .nth(index)?
        .type_argument()
        .ok()?
        .find_concrete_type()
}

/// Distinguishes an unsupported access adapter from a malformed declaration.
///
/// # Parameters
///
/// * `occurrence` - The declaration whose path access failed.
/// * `error` - The low-level property binding error to contextualize.
///
/// # Returns
///
/// Returns an unsupported build error for unsupported adapters, or an
/// occurrence-specific build error for other binding failures.
fn access_error(occurrence: &ValidationOccurrence, error: BindError) -> ValidationBuildError {
    if error.kind() == BindErrorKind::UnsupportedConstraint {
        ValidationBuildError::unsupported(occurrence)
    } else {
        ValidationBuildError::at_occurrence(occurrence, error)
    }
}

/// Resolves the selected member type from the exact collection descriptor.
fn selector_input_type<'ctx>(
    context: &'ctx ReflectionContext,
    path: &CompiledPropertyPath,
    position: SelectorPosition,
) -> Option<TypeDescriptor<'ctx>> {
    let property = path.steps().last()?.property();
    let mut descriptor = property
        .getter()
        .filter(|getter| getter.output_kind() != GetterOutputKind::OptionalBorrowed)
        .and_then(|getter| context.find_type_by_id(getter.output_type_id()).ok().flatten())
        .or_else(|| property.descriptor_in(context).ok().flatten())?;
    if let Some(optional) = property.optional_ops() {
        descriptor = generic_type_argument(descriptor, 0)?;
        if descriptor.type_id() != optional.element_type_id() {
            return None;
        }
    }
    let selected = match position {
        SelectorPosition::Element => generic_type_argument(descriptor, 0),
        SelectorPosition::MapKey => generic_type_argument(descriptor, 0),
        SelectorPosition::MapValue => generic_type_argument(descriptor, 1),
    }?;
    Some(selected)
}

/// Binds one occurrence using the same path and dependency rules at every
/// depth.
///
/// # Parameters
///
/// * `occurrence` - The declaration occurrence to bind.
/// * `graph` - The resolved model graph containing its property paths.
/// * `validators` - The registry used to resolve declared validators.
/// * `ancestors` - The owning model chain used for dependency navigation.
///
/// # Returns
///
/// Returns all runtime bindings produced by the declaration.
///
/// # Errors
///
/// Returns contextual build errors when access, validator lookup, or dependency
/// binding fails.
pub(crate) fn bind(
    occurrence: &ValidationOccurrence,
    graph: &ModelGraph<'_>,
    validators: &ValidatorRegistry,
    ancestors: &[&'static TypeMetadata],
) -> Result<Vec<FieldRuleBinding>, Vec<ValidationBuildError>> {
    let Some(context) = graph.registry().context() else {
        return Err(vec![ValidationBuildError::unsupported(occurrence)]);
    };
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
            let input = occurrence
                .selector
                .and_then(|selector| selector_input_type(context, &value, selector))
                .map(|descriptor| {
                    if matches!(descriptor.kind(), TypeKind::Text(_)) {
                        InputType::Text
                    } else {
                        InputType::Typed(descriptor.type_id())
                    }
                })
                .unwrap_or_else(|| value.input_type());
            let standards = standard_constraints::bind(constraint, validators, input).map_err(|errors| {
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
                    selector: occurrence.selector.map(|position| SelectorBinding {
                        position,
                        items: occurrence
                            .field
                            .collection_ops()
                            .and_then(|ops| ops.collection_items())
                            .expect("selector adapters are checked before binding"),
                    }),
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
                let Some(descriptor) = occurrence
                    .selector
                    .and_then(|selector| selector_input_type(context, &value, selector))
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
            let dependencies = bind_dependencies(occurrence, declaration, &validator, &value, graph, ancestors)
                .map_err(|errors| {
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
                selector: occurrence.selector.map(|position| SelectorBinding {
                    position,
                    items: occurrence
                        .field
                        .collection_ops()
                        .and_then(|ops| ops.collection_items())
                        .expect("selector adapters are checked before binding"),
                }),
            }])
        }
        ExecutionDeclaration::Traversal => Err(vec![ValidationBuildError::unsupported(occurrence)]),
    }
}

/// Compiles dependencies relative to the declaring object's usage path.
///
/// # Parameters
///
/// * `occurrence` - The field occurrence that owns the validator.
/// * `declaration` - The validator's declared dependency names and paths.
/// * `validator` - The bound validator's ordered dependency specifications.
/// * `target` - The compiled value path used to check optionality coverage.
/// * `graph` - The model graph used to resolve dependency properties.
/// * `ancestors` - The owning model chain available during object navigation.
///
/// # Returns
///
/// Returns compiled dependency paths in validator specification order.
///
/// # Errors
///
/// Returns one or more binding errors when a declaration is missing, has an
/// incompatible input type, or violates optionality requirements.
fn bind_dependencies(
    occurrence: &ValidationOccurrence,
    declaration: &ValidatorMetadata,
    validator: &BoundValidator,
    target: &CompiledPropertyPath,
    graph: &ModelGraph<'_>,
    ancestors: &[&'static TypeMetadata],
) -> Result<Box<[CompiledPropertyPath]>, Vec<BindError>> {
    let Some(context) = graph.registry().context() else {
        return Err(vec![BindError::new(BindErrorKind::UnsupportedInput)]);
    };
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
    let declared_by_name = (!declared.is_empty()).then(|| index_declared_bindings(declared));
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
            CompiledPropertyPath::compile(
                context,
                occurrence.root,
                &PropertyPath::new(&segments),
                graph,
                TargetMode::Value,
            )
        } else {
            let Some(binding) = declared_by_name
                .as_ref()
                .and_then(|bindings| bindings.get(spec.name()))
                .copied()
            else {
                errors.push(
                    BindError::new(BindErrorKind::UnknownDependencyDeclaration)
                        .with_rule(rule_id)
                        .with_dependency(spec.name()),
                );
                continue;
            };
            CompiledPropertyPath::compile_dependency(
                context,
                occurrence.root,
                prefix,
                binding,
                graph,
                ancestors,
                spec.input(),
            )
        };
        match path {
            Ok(path)
                if path.input_type() == spec.input()
                    && path.deferred().is_empty()
                    && path.is_optional()
                    && !spec.optional()
                    && !path.optionality_covered_by(target, context) =>
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

/// Indexes dependency declarations by validator slot while retaining the first
/// declaration for each name.
///
/// # Parameters
///
/// * `declared` - Dependency declarations in their source order.
///
/// # Returns
///
/// Returns a name-to-declaration index; duplicate names retain their first
/// declaration to match a source-order search.
#[must_use]
fn index_declared_bindings(declared: &[DependencyBindingMetadata]) -> HashMap<&str, &DependencyBindingMetadata> {
    let mut indexed = HashMap::with_capacity(declared.len());
    for binding in declared {
        indexed.entry(binding.name()).or_insert(binding);
    }
    indexed
}

#[cfg(test)]
mod tests {
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::TypeDescriptor;

    use super::index_declared_bindings;
    use super::sequence_element_matches;
    use crate::metadata::DependencyBindingMetadata;
    use crate::metadata::PropertyPath;

    #[test]
    fn test_dependency_binding_index_preserves_first_duplicate() {
        static BINDINGS: [DependencyBindingMetadata; 2] = [
            DependencyBindingMetadata::new("owner", PropertyPath::new(&["first"])),
            DependencyBindingMetadata::new("owner", PropertyPath::new(&["second"])),
        ];

        let indexed = index_declared_bindings(&BINDINGS);

        assert!(std::ptr::eq(*indexed.get("owner").expect("indexed slot"), &BINDINGS[0]));
    }

    #[test]
    fn test_unique_element_shape_requires_matching_type_ids() {
        let context = ReflectionContext::global();
        assert!(sequence_element_matches(
            TypeDescriptor::of_in::<Vec<i32>>(context),
            TypeDescriptor::of_in::<[i32]>(context),
        ));
        assert!(sequence_element_matches(
            TypeDescriptor::of_in::<[i32; 2]>(context),
            TypeDescriptor::of_in::<[i32]>(context),
        ));
        assert!(!sequence_element_matches(
            TypeDescriptor::of_in::<Vec<i32>>(context),
            TypeDescriptor::of_in::<[u8]>(context),
        ));
    }
}
