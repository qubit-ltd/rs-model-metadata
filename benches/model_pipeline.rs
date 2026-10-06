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
use std::sync::Arc;
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
use qubit_model_metadata::validation::FieldPath;
use qubit_model_metadata::validation::ModelRuleBinding;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_model_metadata::validation::ValidationSelection;
use qubit_reflect::ReflectedRef;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::ReflectRegistry;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::InputType;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistry;

struct AcceptModel;

impl PreparedValidator for AcceptModel {
    fn input_type(&self) -> InputType {
        InputType::of::<Pipeline1>()
    }

    fn dependency_specs(&self) -> &'static [DependencySpec] {
        &[]
    }

    fn validate(
        &self,
        _: ValidationValue<'_>,
        _: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        Ok(PreparedOutcome::Valid)
    }
}

/// Measures plan construction plus model-rule attachment at fixed sample
/// counts.
fn model_rule_batch_build(criterion: &mut Criterion, reflection: &ReflectRegistry) {
    const SAMPLES: usize = 30;
    println!(
        "model_rule_batch_build: samples={SAMPLES}, profile={}",
        if cfg!(debug_assertions) { "debug" } else { "release" }
    );
    let models = ModelRegistry::from_reflect_registry(reflection).expect("valid benchmark registrations");
    let root = TypeMetadata::of::<Pipeline1>();
    let roots = [root];
    let graph = Arc::new(
        StructureResolver::for_roots(ResolveInputs {
            models: &models,
            roots: &roots,
        })
        .resolve()
        .expect("benchmark graph"),
    );
    let validators = ValidatorRegistry::empty();
    let prepared: Arc<dyn PreparedValidator> = Arc::new(AcceptModel);
    let mut group = criterion.benchmark_group("model_rule_batch_build");
    group.sample_size(SAMPLES);
    for count in [1, 8, 32, 128] {
        let bindings: Vec<_> = (0..count)
            .map(|_| {
                ModelRuleBinding::from_prepared::<Pipeline1>(
                    ValidatorId::new("benchmark.model.accept"),
                    Arc::clone(&prepared),
                )
                .expect("prepared model shape")
            })
            .collect();
        group.bench_with_input(
            BenchmarkId::new("repeated_append_baseline", count),
            &bindings,
            |bencher, bindings| {
                bencher.iter(|| {
                    let plan = ValidationPlan::build(
                        root,
                        ValidationBuildInputs {
                            graph: Arc::clone(&graph),
                            validators: &validators,
                        },
                    )
                    .expect("benchmark plan");
                    let plan = black_box(bindings.iter().cloned())
                        .fold(plan, |plan, binding| plan.with_model_rules([binding]));
                    let _ = black_box(plan);
                });
            },
        );
        group.bench_with_input(BenchmarkId::new("batch_api", count), &bindings, |bencher, bindings| {
            bencher.iter(|| {
                let plan = ValidationPlan::build(
                    root,
                    ValidationBuildInputs {
                        graph: Arc::clone(&graph),
                        validators: &validators,
                    },
                )
                .expect("benchmark plan")
                .with_model_rules(black_box(bindings.iter().cloned()));
                let _ = black_box(plan);
            });
        });
    }
    group.finish();
}

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

    let mut properties = criterion.benchmark_group("model_properties_merge_warm");
    for &(root, size) in roots {
        properties.bench_with_input(BenchmarkId::new("fields_and_impls", size), &root, |bencher, root| {
            bencher.iter(|| black_box(root.try_properties_in(black_box(reflection))));
        });
    }
    properties.finish();

    // Prime every registry entry before Criterion starts measuring cached lookup.
    for &(root, _) in roots {
        models.properties_for(root).expect("warm model properties");
    }
    let mut cached = criterion.benchmark_group("model_properties_registry_cached");
    for &(root, size) in roots {
        cached.bench_with_input(BenchmarkId::new("fields_and_impls", size), &root, |bencher, root| {
            bencher.iter(|| black_box(models.properties_for(black_box(*root))));
        });
    }
    cached.finish();

    let mut graphs = criterion.benchmark_group("model_graph_build");
    for &(root, size) in roots {
        let root_set = [root];
        let graph = StructureResolver::for_roots(ResolveInputs {
            models: &models,
            roots: &root_set,
        })
        .resolve()
        .expect("valid benchmark graph");
        assert_eq!(graph.models().len(), 1);
        assert_eq!(graph.models()[0].type_id(), root.type_id());
        graphs.bench_with_input(BenchmarkId::new("fields", size), &root_set, |bencher, roots| {
            bencher.iter(|| {
                black_box(
                    StructureResolver::for_roots(ResolveInputs {
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
        let graph = Arc::new(
            StructureResolver::for_roots(ResolveInputs {
                models: &models,
                roots: &root_set,
            })
            .resolve()
            .expect("graph outside plan timing"),
        );
        let inputs = ValidationBuildInputs {
            graph: Arc::clone(&graph),
            validators: &validators,
        };
        let plan = ValidationPlan::build(root, inputs).expect("all text declarations bind");
        assert_eq!(plan.binding_count(), size);
        plans.bench_with_input(BenchmarkId::new("fields", size), &root, |bencher, root| {
            bencher.iter(|| {
                black_box(ValidationPlan::build(
                    black_box(root),
                    ValidationBuildInputs {
                        graph: Arc::clone(&graph),
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
        let graph = Arc::new(
            StructureResolver::for_roots(ResolveInputs {
                models: &models,
                roots: &root_set,
            })
            .resolve()
            .expect("graph outside execution timing"),
        );
        let plan = ValidationPlan::build(
            root,
            ValidationBuildInputs {
                graph: Arc::clone(&graph),
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
        let last_selected = ValidationOptions::builder()
            .selection(ValidationSelection::Fields(vec![FieldPath::new(format!(
                "field_{}",
                size - 1
            ))]))
            .build();
        execution.bench_function(BenchmarkId::new("fields_last_selected", size), |bencher| {
            bencher.iter(|| black_box(plan.validate(black_box(value.clone()), black_box(&last_selected))));
        });
    }
    execution.finish();
}

/// Measures registration, full and reachable graph construction, and plan
/// binding while keeping metadata, provenance, and validators outside loops.
fn model_count_pipeline(criterion: &mut Criterion) {
    let metadata = fixtures::all_metadata();
    let sources: Vec<_> = (0..metadata.len())
        .map(|index| {
            FragmentIdentity::new(
                "model-pipeline-bench",
                "static-metadata",
                1,
                1,
                "model",
                index as u64 + 1,
            )
        })
        .collect();
    let registrations: Vec<_> = metadata.iter().copied().zip(&sources).collect();
    let validators = ValidatorRegistry::empty();
    let counts = [1, 8, 32];

    for model_count in counts {
        let entries = &registrations[..model_count];
        let models = ModelRegistry::from_static_metadata(entries).expect("valid static registrations");
        criterion.bench_function(
            &format!("model_registry_construction_models/{model_count}/static_entries"),
            |bencher| {
                bencher.iter(|| {
                    black_box(
                        ModelRegistry::from_static_metadata(black_box(entries)).expect("valid static registrations"),
                    )
                })
            },
        );

        let roots: Vec<_> = entries.iter().map(|(model, _)| *model).collect();
        let full = StructureResolver::new(ResolveInputs {
            models: &models,
            roots: &[],
        })
        .resolve()
        .expect("complete fixture graph");
        assert_eq!(models.entries().len(), model_count);
        assert_eq!(
            full.models().len(),
            model_count,
            "complete graph for {model_count} entries included IDs: {:?}",
            full.models().iter().map(|model| model.model_id()).collect::<Vec<_>>(),
        );
        let root = roots[0];
        let root_set = [root];
        let reachable = Arc::new(
            StructureResolver::for_roots(ResolveInputs {
                models: &models,
                roots: &root_set,
            })
            .resolve()
            .expect("single-root fixture graph"),
        );
        assert_eq!(reachable.models().len(), 1);

        criterion.bench_function(
            &format!("model_graph_complete_models/{model_count}/all_registered"),
            |bencher| {
                bencher.iter(|| {
                    black_box(
                        StructureResolver::new(ResolveInputs {
                            models: black_box(&models),
                            roots: &[],
                        })
                        .resolve(),
                    )
                });
            },
        );

        criterion.bench_function(
            &format!("model_graph_reachable_models/{model_count}/one_root"),
            |bencher| {
                bencher.iter(|| {
                    black_box(
                        StructureResolver::for_roots(ResolveInputs {
                            models: black_box(&models),
                            roots: black_box(&root_set),
                        })
                        .resolve(),
                    )
                });
            },
        );

        let plan = ValidationPlan::build(
            root,
            ValidationBuildInputs {
                graph: Arc::clone(&reachable),
                validators: &validators,
            },
        )
        .expect("single-root fixture plan");
        assert_eq!(plan.binding_count(), 1);
        criterion.bench_function(
            &format!("validation_plan_build_root_models/{model_count}/one_root"),
            |bencher| {
                bencher.iter(|| {
                    black_box(ValidationPlan::build(
                        black_box(root),
                        ValidationBuildInputs {
                            graph: black_box(Arc::clone(&reachable)),
                            validators: black_box(&validators),
                        },
                    ))
                });
            },
        );
    }
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
    model_count_pipeline(&mut criterion);
    model_rule_batch_build(&mut criterion, reflection);
    criterion.final_summary();
}
