// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrow-preserving property and dependency reads under one execution budget.

use qubit_reflect::ReflectionContext;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::ValidationPath;

use super::execution_budget::ExecutionBudget;
use super::execution_failure::ExecutionFailure;
use crate::metadata::PropertyAccessError;
use crate::metadata::PropertyBorrowed;
use crate::metadata::PropertyPath;
use crate::metadata::PropertyValue;
use crate::metadata::TargetMode;
use crate::resolve::ModelGraph;
use crate::validation::compiled_property_path::CompiledPropertyPath;

/// Reads every actual property step after reserving its node and depth budget.
///
/// # Parameters
///
/// * `path` - The compiled property steps to read.
/// * `root` - The reflected value at which path traversal starts.
/// * `parent_depth` - The depth already consumed by the enclosing traversal.
/// * `budget` - The shared execution budget charged for each property read.
///
/// # Returns
///
/// Returns the final property value while preserving its borrow from `root`.
/// An absent optional intermediate value is returned as
/// `OptionalBorrowed(None)`.
///
/// # Errors
///
/// Returns an execution error when the budget is exhausted or a property step
/// cannot be read.
pub(crate) fn read<'value>(
    context: &ReflectionContext,
    path: &CompiledPropertyPath,
    root: PropertyBorrowed<'value>,
    parent_depth: usize,
    budget: &mut ExecutionBudget<'_>,
) -> Result<Option<PropertyValue<'value>>, ExecutionError> {
    let mut receiver = root;
    for (index, step) in path.steps().iter().enumerate() {
        if let Some(guard) = path.variant_guard().filter(|guard| guard.after_steps == index) {
            let depth = parent_depth
                .checked_add(index)
                .and_then(|depth| depth.checked_add(1))
                .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::TraversalLimit))?;
            budget.read(depth).map_err(|error| error.with_path(path_for(path)))?;
            let variant = context
                .find_type_by_id(guard.declaring_type)
                .ok()
                .flatten()
                .and_then(|descriptor| descriptor.variants().get_at(guard.variant_index))
                .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)))?;
            if !variant.is_active_dynamic(receiver.clone()).map_err(|error| {
                ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
                    .with_trusted_source(error)
                    .with_path(path_for(path))
            })? {
                return Ok(None);
            }
        }
        let depth = parent_depth
            .checked_add(index)
            .and_then(|depth| depth.checked_add(1))
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::TraversalLimit))?;
        budget.read(depth).map_err(|error| error.with_path(path_for(path)))?;
        // The path was compiled from registered Model metadata and its owner,
        // field descriptor, and target type were validated during resolution.
        // This is the trusted model-validation boundary for private fields.
        let output = unsafe {
            match step.owner() {
                Some(owner) => step.property().get_dynamic_unsafe_for(owner, context, receiver),
                None => step.property().get_dynamic_unsafe_in(context, receiver),
            }
        }
        .map_err(|error| property_read_error(error, path_for(path)))?;
        if index + 1 == path.steps().len() {
            return Ok(Some(output));
        }
        receiver = match output {
            PropertyValue::Borrowed(value) if step.optional_element().is_some() => {
                let ops = step.property().optional_ops().ok_or_else(|| {
                    ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path))
                })?;
                match ops.project_ref(value).map_err(|error| {
                    property_read_error(PropertyAccessError::ValueTypeMismatch(error), path_for(path))
                })? {
                    Some(value) if Some(ops.element_type_id()) == step.optional_element() => value,
                    Some(_) => {
                        return Err(
                            ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path))
                        );
                    }
                    None => return Ok(Some(PropertyValue::OptionalBorrowed(None))),
                }
            }
            PropertyValue::Borrowed(value) | PropertyValue::OptionalBorrowed(Some(value)) => value,
            PropertyValue::OptionalBorrowed(None) => {
                return Ok(Some(PropertyValue::OptionalBorrowed(None)));
            }
            PropertyValue::Owned(_) | PropertyValue::BorrowedSlice(_) => {
                return Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)));
            }
        };
    }
    Err(ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)))
}

/// Retains an owned property adapter failure behind the trusted diagnostic
/// accessor while keeping ordinary execution output safe.
///
/// # Parameters
///
/// * `error` - The original property adapter failure retained for trusted
///   diagnostics.
/// * `path` - The structured property path associated with the failure.
///
/// # Returns
///
/// Returns a safe `PropertyReadFailed` execution error that retains `error`
/// only through its trusted-source accessor.
fn property_read_error(error: PropertyAccessError, path: ValidationPath) -> ExecutionError {
    ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
        .with_trusted_source(error)
        .with_path(path)
}

/// Reads dependency slots under the same budget as the value and validator
/// calls.
///
/// # Parameters
///
/// * `paths` - Compiled dependency paths in validator slot order.
/// * `root` - The reflected value owning the current occurrence.
/// * `ancestors` - Reflected enclosing values for external dependency paths.
/// * `graph` - The resolved graph used for deferred property lookup.
/// * `budget` - The shared budget charged for all dependency reads.
///
/// # Returns
///
/// Returns dependency values and their report paths in the same slot order.
/// Values borrow from `root` and the supplied ancestors.
///
/// # Errors
///
/// Returns the first dependency execution failure with its slot context.
pub(crate) fn dependencies<'value>(
    paths: &[CompiledPropertyPath],
    root: PropertyBorrowed<'value>,
    ancestors: &[PropertyBorrowed<'value>],
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
                    error: Box::new(error),
                    dependency: path.dependency(),
                });
            }
        }
        report_paths.push(path_for(path));
    }
    Ok((values, report_paths))
}

/// Resolves optional external type context and reads one dependency slot.
///
/// # Parameters
///
/// * `path` - The compiled dependency path and its required context depth.
/// * `root` - The reflected value at the current occurrence.
/// * `ancestors` - Reflected enclosing values available for external context.
/// * `graph` - The resolved graph used to compile deferred path segments.
/// * `budget` - The shared execution budget for depth and property reads.
///
/// # Returns
///
/// Returns the dependency value borrowing from the selected reflected value.
///
/// # Errors
///
/// Returns an execution error for missing context, failed deferred lookup,
/// mismatched dependency type, or an exhausted traversal budget.
fn read_dependency<'value>(
    path: &CompiledPropertyPath,
    root: PropertyBorrowed<'value>,
    ancestors: &[PropertyBorrowed<'value>],
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
        let context = graph
            .registry()
            .context()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed))?;
        return read(context, path, owner, path.context_depth(), budget)?
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)));
    }
    let metadata = graph
        .model(owner.clone().type_id())
        .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)))?;
    let context = graph
        .registry()
        .context()
        .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed))?;
    let compiled = CompiledPropertyPath::compile(
        context,
        metadata,
        &PropertyPath::new(path.deferred()),
        graph,
        TargetMode::Value,
    )
    .map_err(|error| {
        ExecutionError::new(ExecutionErrorKind::PropertyReadFailed)
            .with_trusted_source(error)
            .with_path(path_for(path))
    })?;
    if compiled.input_type() != path.input_type() {
        return Err(ExecutionError::new(ExecutionErrorKind::DependencyTypeMismatch).with_path(path_for(path)));
    }
    read(context, &compiled, owner, path.context_depth(), budget)?
        .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::PropertyReadFailed).with_path(path_for(path)))
}

/// Converts a compiled property suffix into its structured report path.
///
/// # Parameters
///
/// * `path` - The compiled path whose resolved or deferred fields are reported.
///
/// # Returns
///
/// Returns deferred field segments when present; otherwise returns the names
/// of the resolved property steps.
#[must_use]
pub(crate) fn path_for(path: &CompiledPropertyPath) -> ValidationPath {
    if !path.deferred().is_empty() {
        return path
            .deferred()
            .iter()
            .fold(ValidationPath::root(), |path, name| path.with_field(name));
    }
    path.steps()
        .iter()
        .enumerate()
        .fold(ValidationPath::root(), |report_path, (index, step)| {
            let report_path = if path.variant_guard().is_some_and(|guard| guard.after_steps == index) {
                report_path.with_field(path.variant_name().expect("variant guard has a name"))
            } else {
                report_path
            };
            let property = step.property();
            match property.field().filter(|field| field.name().is_none()) {
                Some(field) => report_path.with_index(field.index()),
                None => report_path.with_field(property.name()),
            }
        })
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::num::NonZeroUsize;
    use std::sync::OnceLock;

    use qubit_reflect::Reflect;
    use qubit_reflect::ReflectionContext;
    use qubit_reflect::TypeDescriptor;
    use qubit_reflect::value::DynamicRef;
    use qubit_reflect::value::Local;
    use qubit_reflect::value::TypeMismatch;
    use qubit_validator::ExecutionErrorKind;
    use qubit_validator::InputType;
    use qubit_validator::ValidationPath;

    use super::CompiledPropertyPath;
    use super::ExecutionBudget;
    use super::dependencies;
    use super::property_read_error;
    use super::read_dependency;
    use crate::__private::v8;
    use crate::field_metadata::FieldMetadata;
    use crate::metadata::ModelId;
    use crate::metadata::PropertyAccessError;
    use crate::metadata::PropertyBorrowed;
    use crate::metadata::PropertyMetadata;
    use crate::metadata::PropertyPath;
    use crate::metadata::TargetMode;
    use crate::metadata::TypeMetadata;
    use crate::property::OptionalOps;
    use crate::registry::ModelRegistry;
    use crate::registry::ModelRegistryOrigin;
    use crate::resolve::ResolveInputs;
    use crate::resolve::StructureResolver;
    use crate::validation::ValidationOptions;
    use crate::validation::compiled_property_path::VariantGuard;

    fn reflection_context() -> &'static ReflectionContext {
        ReflectionContext::global()
    }

    #[Reflect]
    struct DependencyRoot {
        pub value: String,
    }

    #[Reflect]
    struct OptionalDependencyRoot {
        pub child: Option<DependencyRoot>,
    }

    #[Reflect]
    enum GuardMismatchFixture {
        Named { value: String },
    }

    fn dependency_metadata() -> &'static TypeMetadata {
        static ROOT: OnceLock<&'static TypeMetadata> = OnceLock::new();
        static FIELDS: OnceLock<[FieldMetadata; 1]> = OnceLock::new();
        static PROPERTIES: OnceLock<[PropertyMetadata; 1]> = OnceLock::new();
        ROOT.get_or_init(|| {
            let descriptor = TypeDescriptor::of_in::<DependencyRoot>(reflection_context());
            let fields = FIELDS.get_or_init(|| {
                [FieldMetadata::from_reflect(
                    descriptor.type_id(),
                    descriptor.fields().get_at(0).expect("dependency value field"),
                )]
            });
            let properties = PROPERTIES.get_or_init(|| {
                [PropertyMetadata::new(
                    "value",
                    fields[0].type_expression_data(),
                    Some(fields.first().expect("dependency value field metadata")),
                    None,
                    None,
                )]
            });
            v8::leak(
                v8::GeneratedTypeMetadataBuilder::new::<DependencyRoot>(
                    Some(ModelId::new("validation.path_reader.DependencyRoot")),
                    fields,
                    v8::leak(v8::model_role()),
                )
                .properties(properties)
                .finish::<DependencyRoot>(),
            )
        })
    }

    fn optional_dependency_metadata() -> &'static TypeMetadata {
        static ROOT: OnceLock<&'static TypeMetadata> = OnceLock::new();
        static FIELDS: OnceLock<[FieldMetadata; 1]> = OnceLock::new();
        static PROPERTIES: OnceLock<[PropertyMetadata; 1]> = OnceLock::new();
        ROOT.get_or_init(|| {
            let descriptor = TypeDescriptor::of_in::<OptionalDependencyRoot>(reflection_context());
            let fields = FIELDS.get_or_init(|| {
                [FieldMetadata::from_reflect(
                    descriptor.type_id(),
                    descriptor.fields().get_at(0).expect("optional dependency child field"),
                )
                .with_optional_ops(OptionalOps::for_element::<DependencyRoot>())]
            });
            let properties = PROPERTIES.get_or_init(|| {
                [PropertyMetadata::new(
                    "child",
                    fields[0].type_expression_data(),
                    Some(&fields[0]),
                    None,
                    None,
                )]
            });
            v8::leak(
                v8::GeneratedTypeMetadataBuilder::new::<OptionalDependencyRoot>(
                    Some(ModelId::new("validation.path_reader.OptionalDependencyRoot")),
                    fields,
                    v8::leak(v8::model_role()),
                )
                .properties(properties)
                .finish::<OptionalDependencyRoot>(),
            )
        })
    }

    fn dependency_models() -> &'static ModelRegistry<'static> {
        static MODELS: OnceLock<ModelRegistry<'static>> = OnceLock::new();
        static SOURCE: OnceLock<ModelRegistryOrigin> = OnceLock::new();
        MODELS.get_or_init(|| {
            let source = SOURCE.get_or_init(|| ModelRegistryOrigin {
                crate_name: "fixture",
                crate_version: None,
                module_path: "path_reader",
                file: None,
                line: None,
                column: None,
            });
            ModelRegistry::from_static_metadata(reflection_context(), &[(dependency_metadata(), source)])
                .expect("isolated dependency registry")
        })
    }

    fn optional_dependency_graph() -> crate::resolve::ModelGraph<'static> {
        static MODELS: OnceLock<ModelRegistry<'static>> = OnceLock::new();
        static SOURCES: OnceLock<[ModelRegistryOrigin; 2]> = OnceLock::new();
        static ROOTS: OnceLock<[&'static TypeMetadata; 1]> = OnceLock::new();
        let sources = SOURCES.get_or_init(|| {
            [
                ModelRegistryOrigin {
                    crate_name: "fixture",
                    crate_version: None,
                    module_path: "dependency",
                    file: None,
                    line: None,
                    column: None,
                },
                ModelRegistryOrigin {
                    crate_name: "fixture",
                    crate_version: None,
                    module_path: "optional-root",
                    file: None,
                    line: None,
                    column: None,
                },
            ]
        });
        let models = MODELS.get_or_init(|| {
            ModelRegistry::from_static_metadata(
                reflection_context(),
                &[
                    (dependency_metadata(), &sources[0]),
                    (optional_dependency_metadata(), &sources[1]),
                ],
            )
            .expect("optional dependency registry")
        });
        let roots = ROOTS.get_or_init(|| [optional_dependency_metadata()]);
        StructureResolver::new(ResolveInputs { models, roots })
            .resolve()
            .expect("optional dependency graph")
    }

    fn dependency_graph() -> crate::resolve::ModelGraph<'static> {
        static ROOTS: OnceLock<[&'static TypeMetadata; 1]> = OnceLock::new();
        let models = dependency_models();
        let roots = ROOTS.get_or_init(|| [dependency_metadata()]);
        StructureResolver::new(ResolveInputs { models, roots })
            .resolve()
            .expect("dependency root graph")
    }

    fn deferred_value_path(input: InputType) -> CompiledPropertyPath {
        CompiledPropertyPath {
            context_depth: 1,
            dependency: None,
            deferred: vec!["value"].into_boxed_slice(),
            steps: Box::new([]),
            optional_steps: Box::new([]),
            input,
            optional: false,
            variant_guard: None,
            variant_name: None,
        }
    }

    #[test]
    fn test_property_getter_cause_is_available_only_through_trusted_diagnostics() {
        let error = property_read_error(
            PropertyAccessError::user("sensitive"),
            ValidationPath::root().with_field("name"),
        );
        let cause = error.trusted_source().expect("owned getter error retained");
        assert!(cause.is::<PropertyAccessError>());
        assert!(cause.to_string().contains("sensitive"));
        assert!(!error.to_string().contains("sensitive"));
        assert!(!format!("{error:?}").contains("sensitive"));
        assert!(Error::source(&error).is_none());
    }

    #[test]
    fn test_missing_external_context_is_reported_before_graph_lookup() {
        let models = ModelRegistry::from_static_metadata(reflection_context(), &[]).unwrap();
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
            optional_steps: Box::new([]),
            input: InputType::of::<()>(),
            optional: false,
            variant_guard: None,
            variant_name: None,
        };
        let error = match read_dependency(&path, PropertyBorrowed::new(&()), &[], &graph, &mut budget) {
            Ok(_) => panic!("missing context unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::MissingRequiredDependencyValue);
        assert_eq!(error.path(), &ValidationPath::root());
    }

    #[test]
    fn test_deferred_external_dependency_reads_ancestor_and_checks_input_type() {
        let graph = dependency_graph();
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let root = DependencyRoot {
            value: "parent value".to_owned(),
        };
        let ancestors = [DynamicRef::<Local>::new(&root)];
        let path = deferred_value_path(InputType::Text);

        let value = read_dependency(&path, DynamicRef::<Local>::new(&()), &ancestors, &graph, &mut budget)
            .expect("deferred path resolves against its ancestor");
        let crate::metadata::PropertyValue::Borrowed(value) = value else {
            panic!("deferred field returns a borrow");
        };
        assert_eq!(value.downcast_ref::<String>().map(String::as_str), Some("parent value"));

        let mismatch = deferred_value_path(InputType::of::<u64>());
        let error = match read_dependency(
            &mismatch,
            DynamicRef::<Local>::new(&()),
            &ancestors,
            &graph,
            &mut budget,
        ) {
            Ok(_) => panic!("deferred path unexpectedly accepted a different input type"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::DependencyTypeMismatch);
        assert_eq!(error.path().render(), "value");
    }

    #[test]
    fn test_deferred_external_dependency_reports_resolution_and_depth_failures() {
        let graph = dependency_graph();
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let root = DependencyRoot {
            value: "parent value".to_owned(),
        };
        let ancestors = [DynamicRef::<Local>::new(&root)];
        let missing = CompiledPropertyPath {
            deferred: vec!["missing"].into_boxed_slice(),
            ..deferred_value_path(InputType::Text)
        };
        let error = match read_dependency(&missing, DynamicRef::<Local>::new(&()), &ancestors, &graph, &mut budget) {
            Ok(_) => panic!("missing deferred property unexpectedly resolved"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::PropertyReadFailed);
        assert!(error.trusted_source().is_some());
        assert_eq!(error.path().render(), "missing");

        let limited = ValidationOptions::builder()
            .max_depth(NonZeroUsize::new(1).expect("positive depth"))
            .build();
        let mut limited_budget = ExecutionBudget::new(&limited);
        let too_deep = CompiledPropertyPath {
            context_depth: 2,
            ..deferred_value_path(InputType::Text)
        };
        let error = match read_dependency(
            &too_deep,
            DynamicRef::<Local>::new(&()),
            &[DynamicRef::<Local>::new(&root), DynamicRef::<Local>::new(&root)],
            &graph,
            &mut limited_budget,
        ) {
            Ok(_) => panic!("over-depth dependency unexpectedly resolved"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::TraversalLimit);

        let absent_graph = StructureResolver::new(ResolveInputs {
            models: dependency_models(),
            roots: &[],
        })
        .resolve()
        .expect("empty graph");
        let unknown_ancestor = [DynamicRef::<Local>::new(&())];
        let error = match read_dependency(
            &deferred_value_path(InputType::Text),
            DynamicRef::<Local>::new(&()),
            &unknown_ancestor,
            &absent_graph,
            &mut budget,
        ) {
            Ok(_) => panic!("unregistered ancestor model unexpectedly resolved"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::PropertyReadFailed);
    }

    #[test]
    fn test_property_read_budget_failure_keeps_traversal_context() {
        let graph = dependency_graph();
        let path = CompiledPropertyPath::compile(
            reflection_context(),
            dependency_metadata(),
            &PropertyPath::new(&["value"]),
            &graph,
            TargetMode::Value,
        )
        .expect("root field path compiles");
        let options = ValidationOptions::builder()
            .max_nodes(NonZeroUsize::new(1).expect("positive node budget"))
            .build();
        let mut budget = ExecutionBudget::new(&options);
        let root = DependencyRoot {
            value: "value".to_owned(),
        };

        let error = match super::read(
            reflection_context(),
            &path,
            DynamicRef::<Local>::new(&root),
            0,
            &mut budget,
        ) {
            Ok(_) => panic!("unfunded property read unexpectedly succeeded"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::TraversalLimit);
        assert_eq!(error.path().render(), "value");
    }

    #[test]
    fn test_dependency_batch_preserves_values_and_report_paths() {
        let graph = dependency_graph();
        let path = CompiledPropertyPath::compile(
            reflection_context(),
            dependency_metadata(),
            &PropertyPath::new(&["value"]),
            &graph,
            TargetMode::Value,
        )
        .expect("root field path compiles");
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let root = DependencyRoot {
            value: "dependency value".to_owned(),
        };

        let (values, paths) = match dependencies(&[path], DynamicRef::<Local>::new(&root), &[], &graph, &mut budget) {
            Ok(result) => result,
            Err(_) => panic!("dependency batch resolves"),
        };

        assert_eq!(paths, [ValidationPath::root().with_field("value")]);
        let crate::metadata::PropertyValue::Borrowed(value) = &values[0] else {
            panic!("dependency field returns a borrow");
        };
        assert_eq!(
            value.downcast_ref::<String>().map(String::as_str),
            Some("dependency value")
        );
    }

    #[test]
    fn test_empty_dependency_batch_returns_no_values_or_report_paths() {
        let graph = dependency_graph();
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let root = DependencyRoot {
            value: "unused".to_owned(),
        };

        let (values, paths) = match dependencies(&[], DynamicRef::<Local>::new(&root), &[], &graph, &mut budget) {
            Ok(batch) => batch,
            Err(_) => panic!("a rule with no dependency slots has an empty successful batch"),
        };

        assert!(values.is_empty());
        assert!(paths.is_empty());
    }

    #[test]
    fn test_optional_intermediate_preserves_borrow_and_absence() {
        let graph = optional_dependency_graph();
        let path = CompiledPropertyPath::compile(
            reflection_context(),
            optional_dependency_metadata(),
            &PropertyPath::new(&["child", "value"]),
            &graph,
            TargetMode::Value,
        )
        .expect("optional child path compiles");
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);
        let child = DependencyRoot {
            value: "nested value".to_owned(),
        };
        let root = OptionalDependencyRoot { child: Some(child) };
        let result = super::read(
            reflection_context(),
            &path,
            DynamicRef::<Local>::new(&root),
            0,
            &mut budget,
        )
        .unwrap_or_else(|_| panic!("present optional child can be read"));
        let crate::metadata::PropertyValue::Borrowed(value) = result.unwrap() else {
            panic!("nested field returns a borrow");
        };
        assert_eq!(value.downcast_ref::<String>().map(String::as_str), Some("nested value"));

        let absent = OptionalDependencyRoot { child: None };
        let result = super::read(
            reflection_context(),
            &path,
            DynamicRef::<Local>::new(&absent),
            0,
            &mut budget,
        )
        .unwrap_or_else(|_| panic!("absent optional child is not a read failure"));
        assert!(matches!(
            result,
            Some(crate::metadata::PropertyValue::OptionalBorrowed(None))
        ));
    }

    #[test]
    fn test_variant_guard_type_mismatch_retains_trusted_cause_and_variant_path() {
        let graph = dependency_graph();
        let mut path = CompiledPropertyPath::compile(
            reflection_context(),
            dependency_metadata(),
            &PropertyPath::new(&["value"]),
            &graph,
            TargetMode::Value,
        )
        .expect("dependency field path compiles");
        path.variant_guard = Some(VariantGuard {
            after_steps: 0,
            declaring_type: TypeDescriptor::of_in::<GuardMismatchFixture>(reflection_context()).type_id(),
            variant_index: TypeDescriptor::of_in::<GuardMismatchFixture>(reflection_context())
                .variants()
                .get_at(0)
                .expect("guard variant")
                .index(),
        });
        path.variant_name = Some("Named");
        let root = DependencyRoot {
            value: "value".to_owned(),
        };
        let options = ValidationOptions::default();
        let mut budget = ExecutionBudget::new(&options);

        let error = match super::read(
            reflection_context(),
            &path,
            DynamicRef::<Local>::new(&root),
            0,
            &mut budget,
        ) {
            Ok(_) => panic!("guard for a different enum type must be rejected"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), ExecutionErrorKind::PropertyReadFailed);
        assert!(error.trusted_source().is_some_and(|source| source.is::<TypeMismatch>()));
        assert_eq!(
            error.path(),
            &ValidationPath::root().with_field("Named").with_field("value")
        );
    }
}
