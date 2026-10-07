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
use qubit_model_metadata::__private::ModelImplProvider;
use qubit_model_metadata::__private::model_impl_fragment_key;
use qubit_model_metadata::__private::v7;
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
use qubit_reflect::ReflectedMut;
use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::capability::CapabilityKey;
use qubit_reflect::identity::CapabilityId;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::ReflectRegistry;
use qubit_reflect::registry::RegistrySnapshotBuilder;

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

fn read_name(target: ReflectedRef<'_>) -> Result<PropertyValue<'_>, PropertyAccessError> {
    let target = target
        .downcast::<ResolutionFixture>()
        .map_err(|_| PropertyAccessError::user("wrong owner"))?;
    Ok(PropertyValue::Borrowed(ReflectedRef::new(&target.name)))
}

fn write_name(target: ReflectedMut<'_>, value: ReflectedOwned) -> Result<(), PropertySetFailure> {
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
    let getter = getter.then(|| {
        v7::leak(GetterMetadata::new::<ResolutionFixture>(
            "read_name",
            field.type_ref(),
            GetterOutputKind::Borrowed,
            read_name,
        ))
    });
    let setter = getter.is_none().then(|| {
        v7::leak(SetterMetadata::new::<ResolutionFixture, String>(
            "write_name",
            field.type_ref(),
            write_name,
        ))
    });
    let mut fragments = vec![v7::property_fragment(
        "name",
        field.type_ref(),
        PropertyFragmentSource::Field(field),
    )];
    if let Some(getter) = getter {
        fragments.push(v7::property_fragment(
            "name",
            field.type_ref(),
            PropertyFragmentSource::Getter(getter),
        ));
    }
    if let Some(setter) = setter {
        fragments.push(v7::property_fragment(
            "name",
            field.type_ref(),
            PropertyFragmentSource::Setter(setter),
        ));
    }
    let properties = v7::leak_slice(vec![v7::property_metadata(
        "name",
        field.type_ref(),
        Some(field),
        getter,
        setter,
    )]);
    owner.validate_properties(properties).expect("valid overlay");
    v7::model_impl_metadata(
        v7::leak_slice(fragments),
        Ok(v7::leak(v7::local_property_set(properties))),
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

/// Creates one explicit snapshot with unique model and unrelated capability
/// sources.
fn snapshot(two_providers: bool, unrelated: bool) -> ReflectRegistry {
    let owner = TypeMetadata::of::<ResolutionFixture>();
    let mut builder = RegistrySnapshotBuilder::new();
    builder.add_type(
        owner.descriptor(),
        FragmentIdentity::new("property-resolution-bench", "type", 1, 1, "type", 1),
    );
    let providers = [
        (
            "qubit.model.impl.v1.fbench_getter",
            getter_provider as ModelImplProvider,
        ),
        (
            "qubit.model.impl.v1.fbench_setter",
            setter_provider as ModelImplProvider,
        ),
    ];
    for (index, (id, provider)) in providers
        .into_iter()
        .enumerate()
        .take(if two_providers { 2 } else { 1 })
    {
        builder.add_type_capabilities(
            owner.descriptor(),
            vec![CapabilityDescriptor::with_adapter(
                model_impl_fragment_key(id),
                provider,
            )],
            FragmentIdentity::new("property-resolution-bench", id, 1, 1, "capability", index as u64 + 2),
        );
    }
    if unrelated {
        for index in 0..128_u64 {
            let id = Box::leak(format!("qubit.bench.unrelated.v1.item_{index:03}").into_boxed_str());
            builder.add_type_capabilities(
                owner.descriptor(),
                vec![CapabilityDescriptor::without_adapter(CapabilityKey::<()>::new(
                    CapabilityId::new(id).expect("valid unrelated ID"),
                ))],
                FragmentIdentity::new("property-resolution-bench", id, 1, 1, "capability", index + 4),
            );
        }
    }
    builder.build().expect("valid explicit benchmark snapshot")
}

/// Builds one explicit registry for full three-segment compilation cases.
fn path_snapshot() -> ReflectRegistry {
    let mut builder = RegistrySnapshotBuilder::new();
    for (index, descriptor) in [
        TypeMetadata::of::<ResolutionPathLeaf>().descriptor(),
        TypeMetadata::of::<ResolutionPathMiddle>().descriptor(),
        TypeMetadata::of::<ResolutionPathNine>().descriptor(),
        TypeMetadata::of::<ResolutionPathSeventeen>().descriptor(),
        TypeMetadata::of::<ResolutionPathFiftyOne>().descriptor(),
    ]
    .into_iter()
    .enumerate()
    {
        builder.add_type(
            descriptor,
            FragmentIdentity::new(
                "property-resolution-bench",
                Box::leak(format!("path-type-{index}").into_boxed_str()),
                1,
                1,
                "type",
                index as u64 + 1,
            ),
        );
    }
    builder.build().expect("valid path benchmark snapshot")
}

/// Checks the result before measurements and registers four resolution paths.
fn property_resolution(criterion: &mut Criterion) {
    let owner = TypeMetadata::of::<ResolutionFixture>();
    let one = snapshot(false, false);
    let two = snapshot(true, false);
    let many = snapshot(true, true);
    let models = ModelRegistry::from_reflect_registry(&two).expect("model registry");

    let merged = owner.try_properties_in(&two).expect("two providers merge");
    let property = merged.property("name").expect("merged name property");
    assert!(property.is_getter() && property.is_setter());
    let mut value = ResolutionFixture {
        name: "before".to_owned(),
    };
    property
        .set(ReflectedMut::new(&mut value), ReflectedOwned::new("after".to_owned()))
        .expect("merged setter works");
    let PropertyValue::Borrowed(read) = property.get(ReflectedRef::new(&value)).expect("merged getter works") else {
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

    let path_snapshot = path_snapshot();
    let path_models =
        ModelRegistry::from_reflect_registry(&path_snapshot).expect("path model registry");
    let path_cases = [
        (
            9,
            TypeMetadata::of::<ResolutionPathNine>(),
            ["f08", "leaf", "value"],
        ),
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
    let mut paths = criterion.benchmark_group("full_property_path_compile");
    for (width, root, segments) in path_cases {
        PropertyAccessPath::compile(&path_models, root, &segments).expect("uncached path compiles");
        path_models
            .compile_read_path_cached(root, &segments)
            .expect("cached path compiles before measurement");
        paths.bench_function(
            format!("uncached_last_field_{width}_fields_3_segments"),
            |bencher| {
                bencher.iter(|| {
                    black_box(
                        PropertyAccessPath::compile(
                            black_box(&path_models),
                            black_box(root),
                            black_box(&segments),
                        )
                        .expect("uncached full path"),
                    )
                });
            },
        );
        paths.bench_function(
            format!("cached_hit_last_field_{width}_fields_3_segments"),
            |bencher| {
                bencher.iter(|| {
                    black_box(
                        path_models
                            .compile_read_path_cached(black_box(root), black_box(&segments))
                            .expect("cached full path"),
                    )
                });
            },
        );
    }
    paths.finish();
}

criterion_group!(benches, property_resolution);
criterion_main!(benches);
