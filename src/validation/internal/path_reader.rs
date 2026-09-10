// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrow-preserving property and dependency reads under one execution budget.

use qubit_reflect::ReflectedRef;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::ValidationPath;

use super::execution_budget::ExecutionBudget;
use super::execution_failure::ExecutionFailure;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyValue;
use crate::metadata::TargetMode;
use crate::resolve::ModelGraph;
use crate::validation::compiled_property_path::CompiledPropertyPath;

/// Reads every actual property step after reserving its node and depth budget.
pub(crate) fn read<'value>(
    path: &CompiledPropertyPath,
    root: ReflectedRef<'value>,
    parent_depth: usize,
    budget: &mut ExecutionBudget<'_>,
) -> Result<PropertyValue<'value>, ExecutionError> {
    let mut receiver = root;
    for (index, step) in path.steps().iter().enumerate() {
        let depth = parent_depth
            .checked_add(index)
            .and_then(|depth| depth.checked_add(1))
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::TraversalLimit))?;
        budget.read(depth).map_err(|error| error.with_path(path_for(path)))?;
        let output = step.property().get(receiver).map_err(|source| {
            ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                .with_source(source)
                .with_path(path_for(path))
        })?;
        if index + 1 == path.steps().len() {
            return Ok(output);
        }
        receiver = match output {
            PropertyValue::Borrowed(value) | PropertyValue::OptionalBorrowed(Some(value)) => value,
            PropertyValue::OptionalBorrowed(None) => return Ok(PropertyValue::OptionalBorrowed(None)),
            PropertyValue::Owned(_) | PropertyValue::BorrowedSlice(_) => {
                return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)));
            }
        };
    }
    Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)))
}

/// Reads dependency slots under the same budget as the value and validator
/// calls.
pub(crate) fn dependencies<'value>(
    paths: &[CompiledPropertyPath],
    root: ReflectedRef<'value>,
    ancestors: &[ReflectedRef<'value>],
    graph: &ModelGraph<'_>,
    budget: &mut ExecutionBudget<'_>,
) -> Result<(Vec<PropertyValue<'value>>, Vec<ValidationPath>), ExecutionFailure> {
    let mut values = Vec::with_capacity(paths.len());
    let mut report_paths = Vec::with_capacity(paths.len());
    for path in paths {
        let result = read_dependency(path, root.clone(), ancestors, graph, budget);
        match result {
            Ok(value) => values.push(value),
            Err(error) => {
                return Err(ExecutionFailure {
                    error,
                    dependency: path.dependency(),
                });
            }
        }
        report_paths.push(path_for(path));
    }
    Ok((values, report_paths))
}

/// Resolves optional external type context and reads one dependency slot.
fn read_dependency<'value>(
    path: &CompiledPropertyPath,
    root: ReflectedRef<'value>,
    ancestors: &[ReflectedRef<'value>],
    graph: &ModelGraph<'_>,
    budget: &mut ExecutionBudget<'_>,
) -> Result<PropertyValue<'value>, ExecutionError> {
    budget
        .check_depth(path.context_depth())
        .map_err(|error| error.with_path(path_for(path)))?;
    let owner = if path.context_depth() == 0 {
        root
    } else {
        ancestors.get(path.context_depth() - 1).cloned().ok_or_else(|| {
            ExecutionError::new(ExecutionErrorKind::MissingRequiredDependencyValue).with_path(path_for(path))
        })?
    };
    if path.deferred().is_empty() {
        return read(path, owner, path.context_depth(), budget);
    }
    let metadata = graph
        .model(owner.value_type_id())
        .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)))?;
    let compiled =
        CompiledPropertyPath::compile(metadata, &PropertyPath::new(path.deferred()), graph, TargetMode::Value)
            .map_err(|source| {
                ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                    .with_source(source)
                    .with_path(path_for(path))
            })?;
    if compiled.input_type() != path.input_type() {
        return Err(ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch).with_path(path_for(path)));
    }
    read(&compiled, owner, path.context_depth(), budget)
}

/// Converts a compiled property suffix into its structured report path.
pub(crate) fn path_for(path: &CompiledPropertyPath) -> ValidationPath {
    if !path.deferred().is_empty() {
        return path
            .deferred()
            .iter()
            .fold(ValidationPath::root(), |path, name| path.with_field(*name));
    }
    path.steps().iter().fold(ValidationPath::root(), |path, step| {
        path.with_field(step.property().name())
    })
}

#[cfg(test)]
mod tests {
    use qubit_reflect::ReflectedRef;
    use qubit_validator::ExecutionErrorKind;
    use qubit_validator::InputType;

    use super::CompiledPropertyPath;
    use super::ExecutionBudget;
    use super::read_dependency;
    use crate::registry::ModelRegistry;
    use crate::resolve::ResolveInputs;
    use crate::resolve::StructureResolver;
    use crate::validation::ValidationOptions;

    #[test]
    fn missing_external_context_is_reported_before_graph_lookup() {
        let models = ModelRegistry::from_metadata(&[]).unwrap();
        let graph = StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &[],
        })
        .resolve()
        .unwrap();
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let path = CompiledPropertyPath {
            context_depth: 1,
            dependency: None,
            deferred: Box::new([]),
            steps: Box::new([]),
            input: InputType::of::<()>(),
            optional: false,
        };
        let error = match read_dependency(&path, ReflectedRef::new(&()), &[], &graph, &mut budget) {
            Ok(_) => panic!("missing context unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::MissingRequiredDependencyValue);
    }
}
