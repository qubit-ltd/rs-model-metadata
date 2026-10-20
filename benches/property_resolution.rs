// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Measures direct property resolution and warm model-registry lookup.

use std::hint::black_box;
use std::sync::OnceLock;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_model_derive::Model;
use qubit_model_metadata::__private::ModelImplFragment;
use qubit_model_metadata::__private::ModelImplProvider;
use qubit_model_metadata::__private::ModelImplSource;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::PropertyAccessPath;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::ModelImplMetadata;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyFragmentSource;
use qubit_model_metadata::metadata::PropertySetFailure;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SetterMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::ReflectionContext;
use qubit_reflect::value::DynamicMut;
use qubit_reflect::value::DynamicOwned;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

#[Model]
struct ResolutionFixture {
    name: String,
}

#[Model]
struct ResolutionPathLeaf {
    value: String,
}

#[Model]
struct ResolutionPathMiddle {
    leaf: ResolutionPathLeaf,
}

// Root widths match the platform's observed p50, nearest-rank p90, and maximum.
#[Model]
struct ResolutionPathNine {
    f00: u8,
    f01: u8,
    f02: u8,
    f03: u8,
    f04: u8,
    f05: u8,
    f06: u8,
    f07: u8,
    f08: ResolutionPathMiddle,
}

#[Model]
struct ResolutionPathSeventeen {
    f00: u8,
    f01: u8,
    f02: u8,
    f03: u8,
    f04: u8,
    f05: u8,
    f06: u8,
    f07: u8,
    f08: u8,
    f09: u8,
    f10: u8,
    f11: u8,
    f12: u8,
    f13: u8,
    f14: u8,
    f15: u8,
    f16: ResolutionPathMiddle,
}

#[Model]
struct ResolutionPathFiftyOne {
    f00: u8,
    f01: u8,
    f02: u8,
    f03: u8,
    f04: u8,
    f05: u8,
    f06: u8,
    f07: u8,
    f08: u8,
    f09: u8,
    f10: u8,
    f11: u8,
    f12: u8,
    f13: u8,
    f14: u8,
    f15: u8,
    f16: u8,
    f17: u8,
    f18: u8,
    f19: u8,
    f20: u8,
    f21: u8,
    f22: u8,
    f23: u8,
    f24: u8,
    f25: u8,
    f26: u8,
    f27: u8,
    f28: u8,
    f29: u8,
    f30: u8,
    f31: u8,
    f32: u8,
    f33: u8,
    f34: u8,
    f35: u8,
    f36: u8,
    f37: u8,
    f38: u8,
    f39: u8,
    f40: u8,
    f41: u8,
    f42: u8,
    f43: u8,
    f44: u8,
    f45: u8,
    f46: u8,
    f47: u8,
    f48: u8,
    f49: u8,
    f50: ResolutionPathMiddle,
}

fn read_name(target: DynamicRef<'_, Local>) -> Result<PropertyValue<'_>, PropertyAccessError> {
    let target = target
        .downcast::<ResolutionFixture>()
        .map_err(|_| PropertyAccessError::user("wrong owner"))?;
    Ok(PropertyValue::Borrowed(DynamicRef::<Local>::new(&target.name)))
}

fn write_name(target: DynamicMut<'_, Local>, value: DynamicOwned<Local>) -> Result<(), PropertySetFailure> {
    let target = target
        .downcast::<ResolutionFixture>()
        .unwrap_or_else(|_| panic!("validated owner"));
    target.name = value.downcast::<String>().unwrap_or_else(|_| panic!("validated input"));
    Ok(())
}

/// Builds the same field plus one independently supplied accessor as generated
/// impl metadata.
fn overlay(getter: bool) -> ModelImplMetadata {
    let owner = TypeMetadata::of::<ResolutionFixture>();
    let field = &owner.fields()[0];
    let expression = field.type_expression_data().expect("field type expression");
    let getter = getter.then(|| {
        v8::leak(GetterMetadata::new::<ResolutionFixture>(
            "read_name",
            expression,
            std::any::TypeId::of::<String>,
            GetterOutputKind::Borrowed,
            read_name,
        ))
    });
    let setter = getter.is_none().then(|| {
        v8::leak(SetterMetadata::new::<ResolutionFixture, String>(
            "write_name",
            expression,
            write_name,
        ))
    });
    let mut fragments = vec![v8::property_fragment(
        "name",
        Some(expression),
        PropertyFragmentSource::Field(field),
    )];
    if let Some(getter) = getter {
        fragments.push(v8::property_fragment(
            "name",
            Some(expression),
            PropertyFragmentSource::Getter(getter),
        ));
    }
    if let Some(setter) = setter {
        fragments.push(v8::property_fragment(
            "name",
            Some(expression),
            PropertyFragmentSource::Setter(setter),
        ));
    }
    let properties = v8::leak_slice(vec![v8::property_metadata(
        "name",
        Some(expression),
        Some(field),
        getter,
        setter,
    )]);
    let reflection = ReflectionContext::builder()
        .include_type::<ResolutionFixture>()
        .include_type::<String>()
        .build()
        .expect("overlay reflection context");
    owner
        .validate_properties_in(&reflection, properties)
        .expect("valid overlay");
    v8::model_impl_metadata(
        v8::leak_slice(fragments),
        Ok(v8::leak(v8::local_property_set(properties))),
    )
}

fn getter_provider() -> &'static ModelImplMetadata {
    static VALUE: OnceLock<ModelImplMetadata> = OnceLock::new();
    VALUE.get_or_init(|| overlay(true))
}

fn setter_provider() -> &'static ModelImplMetadata {
    static VALUE: OnceLock<ModelImplMetadata> = OnceLock::new();
    VALUE.get_or_init(|| overlay(false))
}

/// Builds a context with owner fragments and unrelated fragments to filter.
fn snapshot(two_providers: bool, unrelated_count: usize) -> ReflectionContext {
    let providers = [
        getter_provider as ModelImplProvider,
        setter_provider as ModelImplProvider,
    ];
    let mut fragments: Vec<_> = providers
        .into_iter()
        .take(if two_providers { 2 } else { 1 })
        .enumerate()
        .map(|(index, provider)| {
            ModelImplFragment::for_type::<ResolutionFixture>(provider, model_impl_source(index as u64 + 1))
        })
        .collect();
    fragments.extend(
        (0..unrelated_count)
            .map(|index| ModelImplFragment::for_type::<String>(getter_provider, model_impl_source(index as u64 + 3))),
    );
    let fragments = v8::leak_slice(fragments);
    ReflectionContext::builder()
        .include_linked()
        .register(v8::model_impl_capability_registration::<ResolutionFixture>(fragments))
        .build()
        .expect("valid explicit benchmark context")
}

fn model_impl_source(fingerprint: u64) -> ModelImplSource {
    ModelImplSource {
        module_path: module_path!(),
        file: file!(),
        line: line!(),
        fingerprint,
    }
}

/// Builds a context for full three-segment compilation cases.
fn path_snapshot() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("valid path benchmark context")
}

/// Checks the result before measurements and registers four resolution paths.
fn property_resolution(criterion: &mut Criterion) {
    let owner = TypeMetadata::of::<ResolutionFixture>();
    let one = snapshot(false, 0);
    let two = snapshot(true, 0);
    let many = snapshot(true, 128);
    let models = ModelRegistry::from_context(&two).expect("model registry");

    let merged = owner.try_properties_in(&two).expect("two providers merge");
    let property = merged.property("name").expect("merged name property");
    assert!(property.is_getter() && property.is_setter());
    let mut value = ResolutionFixture {
        name: "before".to_owned(),
    };
    property
        .set_dynamic(
            DynamicMut::<Local>::new(&mut value),
            DynamicOwned::<Local>::new("after".to_owned()),
        )
        .expect("merged setter works");
    let PropertyValue::Borrowed(read) = property
        .get_dynamic(DynamicRef::<Local>::new(&value))
        .expect("merged getter works")
    else {
        panic!("expected borrowed name");
    };
    assert_eq!(read.downcast_ref::<String>().expect("String getter"), "after");
    let unrelated = owner.try_properties_in(&many).expect("unrelated capabilities ignored");
    assert_eq!(unrelated.properties().len(), merged.properties().len());
    assert!(unrelated.property("name").expect("same name property").is_getter());
    assert!(unrelated.property("name").expect("same name property").is_setter());
    models.properties_for(owner).expect("prime model registry cache");

    let mut group = criterion.benchmark_group("model_property_resolution");
    group.bench_function("direct_one", |bencher| {
        bencher.iter(|| black_box(owner.try_properties_in(black_box(&one)).expect("one provider")));
    });
    group.bench_function("direct_two", |bencher| {
        bencher.iter(|| black_box(owner.try_properties_in(black_box(&two)).expect("two providers")));
    });
    group.bench_function("registry_cached_two", |bencher| {
        bencher.iter(|| black_box(models.properties_for(black_box(owner)).expect("cached two providers")));
    });
    group.bench_function("direct_many_unrelated", |bencher| {
        bencher.iter(|| {
            black_box(
                owner
                    .try_properties_in(black_box(&many))
                    .expect("unrelated capabilities"),
            )
        });
    });
    group.finish();

    let roots: Vec<&'static TypeMetadata> = (0..256)
        .map(|_| -> &'static TypeMetadata { Box::leak(Box::new(*owner)) })
        .collect();
    let (final_cold_root, first_cold_roots) = roots.split_last().expect("256 cold roots");
    let path = &["name"];
    models
        .compile_read_path_cached(owner, path)
        .expect("hot path compiles before measurement");
    let mut paths = criterion.benchmark_group("property_path_cache");
    paths.bench_function("single_path_hit", |bencher| {
        bencher.iter(|| {
            black_box(
                models
                    .compile_read_path_cached(black_box(owner), black_box(path))
                    .expect("cached hot path"),
            )
        });
    });
    paths.bench_function("hot_path_refreshed_amid_256_cold_roots", |bencher| {
        bencher.iter(|| {
            black_box(models.compile_read_path_cached(owner, path).expect("initial hot path"));
            for root in first_cold_roots {
                black_box(models.compile_read_path_cached(root, path).expect("cold root path"));
            }
            black_box(
                models
                    .compile_read_path_cached(owner, path)
                    .expect("refreshed hot path"),
            );
            black_box(
                models
                    .compile_read_path_cached(final_cold_root, path)
                    .expect("final cold root path"),
            );
            black_box(
                models
                    .compile_read_path_cached(owner, path)
                    .expect("hot path after cold roots"),
            );
        });
    });
    paths.finish();

    let path_snapshot = path_snapshot();
    let path_models = ModelRegistry::from_context(&path_snapshot).expect("path model registry");
    let path_cases = [
        (9, TypeMetadata::of::<ResolutionPathNine>(), ["f08", "leaf", "value"]),
        (
            17,
            TypeMetadata::of::<ResolutionPathSeventeen>(),
            ["f16", "leaf", "value"],
        ),
        (
            51,
            TypeMetadata::of::<ResolutionPathFiftyOne>(),
            ["f50", "leaf", "value"],
        ),
    ];
    let mut full_paths = criterion.benchmark_group("full_property_path_compile");
    for (width, root, segments) in path_cases {
        PropertyAccessPath::compile(&path_models, root, &segments).expect("uncached path compiles");
        path_models
            .compile_read_path_cached(root, &segments)
            .expect("cached path compiles before measurement");
        full_paths.bench_function(format!("uncached_last_field_{width}_fields_3_segments"), |bencher| {
            bencher.iter(|| {
                black_box(
                    PropertyAccessPath::compile(black_box(&path_models), black_box(root), black_box(&segments))
                        .expect("uncached full path"),
                )
            });
        });
        full_paths.bench_function(format!("cached_hit_last_field_{width}_fields_3_segments"), |bencher| {
            bencher.iter(|| {
                black_box(
                    path_models
                        .compile_read_path_cached(black_box(root), black_box(&segments))
                        .expect("cached full path"),
                )
            });
        });
    }
    full_paths.finish();
}

criterion_group!(benches, property_resolution);
criterion_main!(benches);
