// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public build-budget regression cases.

use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_validator::ValidatorRegistry;

#[Model(id = "budget.SharedLeaf")]
struct SharedLeaf {
    #[text(non_blank)]
    value: String,
}

macro_rules! binary_models {
    ($(($name:ident, $id:literal, $previous:ty)),+ $(,)?) => {
        $(
            #[Model(id = $id)]
            struct $name {
                left: $previous,
                right: $previous,
            }
        )+
    };
}

binary_models! {
    (Shared1, "budget.Shared1", SharedLeaf),
    (Shared2, "budget.Shared2", Shared1),
    (Shared3, "budget.Shared3", Shared2),
    (Shared4, "budget.Shared4", Shared3),
    (Shared5, "budget.Shared5", Shared4),
    (Shared6, "budget.Shared6", Shared5),
    (Shared7, "budget.Shared7", Shared6),
    (Shared8, "budget.Shared8", Shared7),
    (Shared9, "budget.Shared9", Shared8),
    (Shared10, "budget.Shared10", Shared9),
    (Shared11, "budget.Shared11", Shared10),
    (Shared12, "budget.Shared12", Shared11),
    (Shared13, "budget.Shared13", Shared12),
    (Shared14, "budget.Shared14", Shared13),
}

#[test]
fn test_plan_rejects_exponential_occurrence_graph_without_materializing_values() {
    let root = TypeMetadata::of::<Shared14>();
    let models = ModelRegistry::try_global().expect("linked model registry");
    let graph = Arc::new(
        StructureResolver::for_static_roots(models, [root])
            .resolve()
            .expect("small shared graph resolves"),
    );
    assert_eq!(graph.models().len(), 15);

    let capability_errors = ValidationCapabilities::check(root, graph.as_ref())
        .expect_err("capability discovery shares the occurrence limit");
    assert!(
        capability_errors
            .iter()
            .any(|error| error.kind() == ValidationBuildErrorKind::BuildLimitExceeded)
    );
    assert_eq!(
        capability_errors.iter().last().map(|error| error.kind()),
        Some(ValidationBuildErrorKind::BuildLimitExceeded),
        "the terminal resource error follows retained ordinary diagnostics"
    );
    let terminal = capability_errors
        .iter()
        .find(|error| error.kind() == ValidationBuildErrorKind::BuildLimitExceeded)
        .expect("terminal build-limit diagnostic");
    assert_eq!(terminal.root_type_id(), root.type_id());
    assert!(terminal.field_location().is_none());
    assert!(terminal.declaration().is_none());
    assert!(terminal.path().is_none());
    assert!(terminal.source_error().is_none());

    let validators = ValidatorRegistry::empty();
    let errors = match ValidationPlan::build(
        root,
        ValidationBuildInputs {
            graph,
            validators: &validators,
        },
    ) {
        Err(errors) => errors,
        Ok(_) => panic!("occurrence paths exceed the fixed build limit"),
    };
    let terminal = errors
        .iter()
        .find(|error| error.kind() == ValidationBuildErrorKind::BuildLimitExceeded)
        .expect("terminal build-limit diagnostic");
    assert_eq!(terminal.root_type_id(), root.type_id());
    assert!(terminal.field_location().is_none());
    assert!(terminal.declaration().is_none());
    assert!(terminal.path().is_none());
    assert!(terminal.source_error().is_none());
    assert_eq!(
        errors.iter().last().map(|error| error.kind()),
        Some(ValidationBuildErrorKind::BuildLimitExceeded),
        "failed builds return the terminal resource error after ordinary diagnostics"
    );
    assert!(
        errors
            .iter()
            .any(|error| error.kind() == ValidationBuildErrorKind::BuildLimitExceeded)
    );
}
