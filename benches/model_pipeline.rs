// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Separates property initialization, warm lookup, graph construction, plan
//! binding and repeated execution at 1, 8 and 32 declared fields.
//!
//! Cold initialization is a single timed observation per fixture and process,
//! printed before Criterion runs. Use `--cold-properties` to run only these
//! observations in a fresh process. Repeated lookups are never labelled cold.

#[path = "model_pipeline/fixtures.rs"]
mod fixtures;

use std::hint::black_box;
use std::time::Instant;

use criterion::BenchmarkId;
use criterion::Criterion;
use fixtures::Pipeline1;
use fixtures::Pipeline8;
use fixtures::Pipeline32;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectedRef;
use qubit_reflect::registry::ReflectRegistry;
use qubit_validator::ValidatorRegistry;

/// Records the first property-provider initialization and assembly separately
/// from both reflection startup and the later statistical warm benchmarks.
fn cold_properties(roots: &[(&'static TypeMetadata, usize)], reflection: &ReflectRegistry) {
    for &(root, size) in roots {
        let started = Instant::now();
        let properties =
            black_box(root.try_properties_in(black_box(reflection))).expect("compatible independent getter fragments");
        let elapsed = started.elapsed();
        assert_eq!(properties.properties().len(), size);
        assert!(properties.properties().iter().all(|property| property.is_getter()));
        println!(
            "cold_properties/fields_{size}/impls_{size}: {} ns (one first-use observation)",
            elapsed.as_nanos()
        );
    }
}

/// Measures each stable pipeline stage without recreating business inputs.
fn pipeline(criterion: &mut Criterion, reflection: &ReflectRegistry, roots: &[(&'static TypeMetadata, usize)]) {
    let models = ModelRegistry::from_reflect_registry(reflection).expect("valid benchmark registrations");
    let validators = ValidatorRegistry::empty();
    let options = ValidationOptions::default();
    let one = Pipeline1::sample();
    let eight = Pipeline8::sample();
    let thirty_two = Pipeline32::sample();
    let values = [
        ReflectedRef::new(&one),
        ReflectedRef::new(&eight),
        ReflectedRef::new(&thirty_two),
    ];

    let mut properties = criterion.benchmark_group("multi_impl_properties_warm");
    for &(root, size) in roots {
        properties.bench_with_input(BenchmarkId::new("fields_and_impls", size), &root, |bencher, root| {
            bencher.iter(|| black_box(root.try_properties_in(black_box(reflection))));
        });
    }
    properties.finish();

    let mut graphs = criterion.benchmark_group("model_graph_build");
    for &(root, size) in roots {
        let root_set = [root];
        let graph = StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &root_set,
        })
        .resolve()
        .expect("valid benchmark graph");
        assert_eq!(graph.models().len(), 1);
        graphs.bench_with_input(BenchmarkId::new("fields", size), &root_set, |bencher, roots| {
            bencher.iter(|| {
                black_box(
                    StructureResolver::new(ResolveInputs {
                        models: &models,
                        roots: black_box(roots),
                    })
                    .resolve(),
                )
            });
        });
    }
    graphs.finish();

    let mut plans = criterion.benchmark_group("validation_plan_build");
    for &(root, size) in roots {
        let root_set = [root];
        let graph = StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &root_set,
        })
        .resolve()
        .expect("graph outside plan timing");
        let inputs = ValidationBuildInputs {
            graph: &graph,
            validators: &validators,
        };
        let plan = ValidationPlan::build(root, inputs).expect("all text declarations bind");
        assert_eq!(plan.binding_count(), size);
        plans.bench_with_input(BenchmarkId::new("fields", size), &root, |bencher, root| {
            bencher.iter(|| {
                black_box(ValidationPlan::build(
                    black_box(root),
                    ValidationBuildInputs {
                        graph: &graph,
                        validators: &validators,
                    },
                ))
            });
        });
    }
    plans.finish();

    let mut execution = criterion.benchmark_group("validation_plan_execute");
    for (&(root, size), value) in roots.iter().zip(values) {
        let root_set = [root];
        let graph = StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &root_set,
        })
        .resolve()
        .expect("graph outside execution timing");
        let plan = ValidationPlan::build(
            root,
            ValidationBuildInputs {
                graph: &graph,
                validators: &validators,
            },
        )
        .expect("plan outside execution timing");
        let report = plan.validate(value.clone(), &options).expect("valid fixture execution");
        assert!(report.violations().is_empty());
        assert!(report.skipped().is_empty());
        assert!(!report.is_truncated());
        execution.bench_function(BenchmarkId::new("fields", size), |bencher| {
            bencher.iter(|| black_box(plan.validate(black_box(value.clone()), black_box(&options))));
        });
    }
    execution.finish();
}

/// Keeps cold probes ahead of any code that could initialize property caches.
fn main() {
    let reflection = ReflectRegistry::initialize().expect("reflection startup outside property timing");
    let roots = [
        (TypeMetadata::of::<Pipeline1>(), 1),
        (TypeMetadata::of::<Pipeline8>(), 8),
        (TypeMetadata::of::<Pipeline32>(), 32),
    ];
    cold_properties(&roots, reflection);
    if std::env::args().any(|argument| argument == "--cold-properties") {
        return;
    }
    let mut criterion = Criterion::default().configure_from_args();
    pipeline(&mut criterion, reflection, &roots);
    criterion.final_summary();
}
