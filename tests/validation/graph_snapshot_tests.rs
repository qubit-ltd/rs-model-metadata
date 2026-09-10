// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Graph snapshots, rather than caller overlays, own validation declarations.

use qubit_model_derive::Model;
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectedRef;
use qubit_reflect::TypeDescriptor;
use qubit_validator::ValidatorRegistry;

#[Model]
struct Record {
    #[text(non_blank)]
    name: String,
}

#[Model]
struct UniqueValues {
    #[sequence(unique_items)]
    values: Vec<String>,
}

/// Builds a second checked overlay for the same concrete descriptor, without
/// domain constraints. This models explicitly selected snapshot declarations.
fn unconstrained<T: Reflect>() -> &'static TypeMetadata {
    let descriptor = TypeDescriptor::of::<T>();
    let fields = descriptor
        .fields()
        .iter()
        .map(|field| FieldMetadata::from_reflect(descriptor.type_id(), field))
        .collect();
    v7::leak(
        v7::GeneratedTypeMetadataBuilder::new(descriptor, None, v7::leak_slice(fields), v7::leak(v7::model_role()))
            .finish::<T>(),
    )
}

/// Passing a less constrained caller overlay cannot erase graph declarations.
#[test]
fn test_plan_uses_the_graphs_canonical_root_declarations() {
    let declared = TypeMetadata::of::<Record>();
    let stripped = unconstrained::<Record>();
    let roots = [declared];
    let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("declared graph");
    assert!(std::ptr::eq(
        graph.model(declared.type_id()).expect("graph root"),
        declared
    ));
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        stripped,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("canonical declarations bind");
    assert_eq!(plan.binding_count(), 1);
    assert!(std::ptr::eq(plan.root(), declared));
    let report = plan
        .validate(
            ReflectedRef::new(&Record { name: String::new() }),
            &ValidationOptions::default(),
        )
        .expect("execution");
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.violations()[0].path().render(), "name");
}

/// Caller and global overlays cannot add work absent from an explicit graph.
#[test]
fn test_plan_does_not_import_constraints_from_another_overlay() {
    let declared = TypeMetadata::of::<Record>();
    let stripped = unconstrained::<Record>();
    let roots = [stripped];
    let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("explicit unconstrained graph");
    assert!(std::ptr::eq(
        graph.model(declared.type_id()).expect("graph root"),
        stripped
    ));
    let validators = ValidatorRegistry::empty();
    let plan = ValidationPlan::build(
        declared,
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .expect("snapshot declarations bind");
    assert_eq!(plan.binding_count(), 0);
    assert!(std::ptr::eq(plan.root(), stripped));
    assert!(
        plan.validate(
            ReflectedRef::new(&Record { name: String::new() }),
            &ValidationOptions::default()
        )
        .expect("execution")
        .is_valid()
    );
}

/// Capability checks cannot bypass an unsupported graph declaration using a
/// caller overlay with the same type but an empty rule set.
#[test]
fn test_capabilities_check_graph_declarations_for_the_requested_type() {
    let declared = TypeMetadata::of::<UniqueValues>();
    let stripped = unconstrained::<UniqueValues>();
    let roots = [declared];
    let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("uniqueness is structurally valid");
    let errors = ValidationCapabilities::check(stripped, &graph).expect_err("erased uniqueness is unsupported");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
    assert_eq!(errors[0].field_location(), declared.fields()[0].location());
    assert!(errors[0].constraint().is_some());
}
