// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use core::mem::size_of;

use model_a::Source;
use model_a::TargetView;
use model_b::CODEC;
use model_b::RULE;
use model_b::Target;
use qubit_codec::ValueCodecRegistry;
use qubit_model_metadata::__private::ReflectedRef;
use qubit_model_metadata::codec::CodecBindInputs;
use qubit_model_metadata::codec::bind_codecs;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_validator::ValidatorRegistry;

fn main() {
    let _ = size_of::<Source>();
    let _ = size_of::<Target>();
    let registry = ModelRegistry::try_global().expect("cross-crate registrations should be valid");
    assert!(registry.metadata("test.linked.Source").is_some());
    assert!(registry.metadata("test.linked.Target").is_some());
    let graph = StructureResolver::new(ResolveInputs {
        roots: &[],
        models: registry,
    })
    .resolve()
    .expect("cross-crate reference should resolve");
    let field = TypeMetadata::of::<Source>().field("target_id").expect("source field");
    assert_eq!(
        graph
            .reference(field.location().expect("concrete declaration"))
            .expect("resolved reference")
            .target()
            .model_id()
            .unwrap()
            .as_str(),
        "test.linked.Target",
    );

    let projection = TypeMetadata::of::<TargetView>().type_id();
    assert_eq!(
        graph.projection_source(projection).unwrap().target().type_id(),
        TypeMetadata::of::<Target>().type_id(),
    );
    let validators = ValidatorRegistry::from_registrations([RULE]).unwrap();
    let plan = ValidationPlan::build(
        TypeMetadata::of::<Source>(),
        ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        },
    )
    .unwrap();
    assert_eq!(plan.binding_count(), 1);
    let value = Source {
        target_id: Default::default(),
        text: "cross-crate".to_owned(),
    };
    let report = plan
        .validate(ReflectedRef::new(&value), &ValidationOptions::default())
        .unwrap();
    assert!(report.is_valid());
    let codecs = ValueCodecRegistry::from_registrations([&CODEC]).unwrap();
    let bound = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .unwrap();
    assert_eq!(bound.bindings().len(), 1);
}
