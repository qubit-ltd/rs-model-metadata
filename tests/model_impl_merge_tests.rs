// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Snapshot-selected implementation overlays preserve conflicts and ownership.

use std::ptr::eq;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_model_derive::Model;
use qubit_model_metadata::__private::ModelImplFragment;
use qubit_model_metadata::__private::ModelImplProvider;
use qubit_model_metadata::__private::ModelImplSource;
use qubit_model_metadata::__private::v8;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::ModelImplMetadata;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyBuildErrorKind;
use qubit_model_metadata::metadata::PropertyFragmentSource;
use qubit_model_metadata::metadata::PropertyResolutionError;
use qubit_model_metadata::metadata::PropertySetFailure;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::ResolvedProperties;
use qubit_model_metadata::metadata::SetterMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::ReflectionContext;
use qubit_reflect::value::DynamicMut;
use qubit_reflect::value::DynamicOwned;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

#[Model]
struct Record {
    name: String,
    count: u32,
}

fn read_name(target: DynamicRef<'_, Local>) -> Result<PropertyValue<'_>, PropertyAccessError> {
    let target = target
        .downcast::<Record>()
        .map_err(|_| PropertyAccessError::user("wrong owner"))?;
    Ok(PropertyValue::Borrowed(DynamicRef::<Local>::new(&target.name)))
}

fn write_name(target: DynamicMut<'_, Local>, value: DynamicOwned<Local>) -> Result<(), PropertySetFailure> {
    let target = target
        .downcast::<Record>()
        .unwrap_or_else(|_| panic!("validated owner"));
    target.name = value.downcast::<String>().unwrap_or_else(|_| panic!("validated input"));
    Ok(())
}

const SOURCE_A: ModelImplSource = ModelImplSource {
    module_path: "model_impl_merge_tests",
    file: file!(),
    line: 1,
    fingerprint: 0xA001,
};
const SOURCE_B: ModelImplSource = ModelImplSource {
    module_path: "model_impl_merge_tests",
    file: file!(),
    line: 2,
    fingerprint: 0xB002,
};
// ModelImpl code generation repeats the declaration's backing fields in each
// overlay. These fixtures reproduce that ABI shape with independently owned
// getter/setter descriptors, without exposing an internal merge test hook.
fn overlay(getter_name: Option<&'static str>, setter_name: Option<&'static str>) -> ModelImplMetadata {
    let owner = TypeMetadata::of::<Record>();
    let name = &owner.fields()[0];
    let count = &owner.fields()[1];
    let name_expression = name.type_expression_data().expect("name expression");
    let count_expression = count.type_expression_data().expect("count expression");
    let getter = getter_name.map(|method| {
        v8::leak(GetterMetadata::new::<Record>(
            method,
            name_expression,
            std::any::TypeId::of::<String>,
            GetterOutputKind::Borrowed,
            read_name,
        ))
    });
    let setter = setter_name.map(|method| {
        v8::leak(SetterMetadata::new::<Record, String>(
            method,
            name_expression,
            write_name,
        ))
    });
    let mut fragments = vec![
        v8::property_fragment("name", Some(name_expression), PropertyFragmentSource::Field(name)),
        v8::property_fragment("count", Some(count_expression), PropertyFragmentSource::Field(count)),
    ];
    if let Some(getter) = getter {
        fragments.push(v8::property_fragment(
            "name",
            Some(name_expression),
            PropertyFragmentSource::Getter(getter),
        ));
    }
    if let Some(setter) = setter {
        fragments.push(v8::property_fragment(
            "name",
            Some(name_expression),
            PropertyFragmentSource::Setter(setter),
        ));
    }
    let properties = v8::leak_slice(vec![
        v8::property_metadata("name", Some(name_expression), Some(name), getter, setter),
        v8::property_metadata("count", Some(count_expression), Some(count), None, None),
    ]);
    owner
        .validate_properties_in(ReflectionContext::global(), properties)
        .expect("each overlay is individually valid");
    v8::model_impl_metadata(
        v8::leak_slice(fragments),
        Ok(v8::leak(v8::local_property_set(properties))),
    )
}

fn getter_a() -> &'static ModelImplMetadata {
    static VALUE: OnceLock<ModelImplMetadata> = OnceLock::new();
    VALUE.get_or_init(|| overlay(Some("get_name"), None))
}

fn getter_b() -> &'static ModelImplMetadata {
    static VALUE: OnceLock<ModelImplMetadata> = OnceLock::new();
    VALUE.get_or_init(|| overlay(Some("read_name"), None))
}

fn setter_a() -> &'static ModelImplMetadata {
    static VALUE: OnceLock<ModelImplMetadata> = OnceLock::new();
    VALUE.get_or_init(|| overlay(None, Some("set_name")))
}

fn setter_b() -> &'static ModelImplMetadata {
    static VALUE: OnceLock<ModelImplMetadata> = OnceLock::new();
    VALUE.get_or_init(|| overlay(None, Some("replace_name")))
}

fn combined_a() -> &'static ModelImplMetadata {
    static VALUE: OnceLock<ModelImplMetadata> = OnceLock::new();
    VALUE.get_or_init(|| overlay(Some("get_name"), Some("set_name")))
}

fn combined_b() -> &'static ModelImplMetadata {
    static VALUE: OnceLock<ModelImplMetadata> = OnceLock::new();
    VALUE.get_or_init(|| overlay(Some("read_name"), Some("replace_name")))
}

static PROVIDER_CALLS: AtomicUsize = AtomicUsize::new(0);

fn counting_getter() -> &'static ModelImplMetadata {
    PROVIDER_CALLS.fetch_add(1, Ordering::SeqCst);
    getter_a()
}

static PANIC_ONCE: AtomicBool = AtomicBool::new(true);

fn panic_once_getter() -> &'static ModelImplMetadata {
    assert!(!PANIC_ONCE.swap(false, Ordering::SeqCst), "provider panic fixture");
    getter_a()
}

fn source_at(source: ModelImplSource, provider: ModelImplProvider) -> ModelImplFragment {
    ModelImplFragment::for_type::<Record>(provider, source)
}

fn snapshot_with(providers: &[(ModelImplProvider, ModelImplSource)]) -> ReflectionContext {
    let fragments = v8::leak_slice(
        providers
            .iter()
            .map(|(provider, source)| source_at(*source, *provider))
            .collect(),
    );
    ReflectionContext::builder()
        .include_type::<Record>()
        .include_type::<String>()
        .include_type::<u32>()
        .register(v8::model_impl_capability_registration::<Record>(fragments))
        .build()
        .expect("typed model implementation registration")
}

fn snapshot(first: ModelImplProvider, second: ModelImplProvider) -> ReflectionContext {
    snapshot_with(&[(first, SOURCE_A), (second, SOURCE_B)])
}

#[test]
fn test_model_impl_aggregate_registers_each_source_fragment() {
    let owner = TypeMetadata::of::<Record>();
    let registry = snapshot_with(&[(getter_a as ModelImplProvider, SOURCE_A), (setter_a, SOURCE_B)]);
    let properties = owner
        .try_properties_in(&registry)
        .expect("aggregate capability selects each registered source fragment");
    let name = properties.property("name").expect("name property");
    assert_eq!(name.getter().expect("base getter").rust_method_name(), "get_name");
    assert_eq!(name.setter().expect("fragment setter").rust_method_name(), "set_name");
}

#[test]
fn test_complementary_overlays_merge_accessors_and_coalesce_field_fragments() {
    let owner = TypeMetadata::of::<Record>();
    let static_properties = owner.try_properties().expect("generated static properties");
    assert_eq!(
        static_properties.property("name").map(|property| property.name()),
        Some("name")
    );
    assert!(static_properties.property("missing").is_none());
    let registry = snapshot(getter_a, setter_a);
    let properties = owner.try_properties_in(&registry).expect("complementary accessors");
    let repeated = owner.try_properties_in(&registry).expect("owned merge");
    assert_eq!(properties.properties().len(), repeated.properties().len());
    assert_eq!(properties.properties().len(), 2);
    let name = properties.property("name").expect("merged name");
    assert_eq!(name.getter().expect("getter").rust_method_name(), "get_name");
    assert_eq!(name.setter().expect("setter").rust_method_name(), "set_name");
    let mut value = Record {
        name: "before".to_owned(),
        count: 7,
    };
    name.set_dynamic(
        DynamicMut::<Local>::new(&mut value),
        DynamicOwned::<Local>::new("after".to_owned()),
    )
    .expect("merged setter");
    let PropertyValue::Borrowed(read) = name
        .get_dynamic(DynamicRef::<Local>::new(&value))
        .expect("merged getter")
    else {
        panic!("borrowed getter");
    };
    assert!(eq(read.downcast_ref::<String>().expect("String output"), &value.name));
    assert_eq!(value.name, "after");
    assert_eq!(value.count, 7);
    let fragments = owner.property_fragments_in(&registry).expect("raw fragments");
    let reflection = &registry;
    let fragments = fragments.fragments();
    assert_eq!(fragments.len(), 4);
    assert_eq!(
        fragments
            .iter()
            .filter(|fragment| matches!(fragment.source(), PropertyFragmentSource::Field(_)))
            .count(),
        2
    );
    for fragment in fragments {
        assert!(fragment.type_expression_in(reflection).is_some());
        assert!(
            properties
                .property(fragment.name())
                .expect("fragment property")
                .type_expression_in(reflection)
                .is_some()
        );
    }
}

#[test]
fn test_distinct_accessors_conflict_without_poisoning_other_snapshots() {
    let owner = TypeMetadata::of::<Record>();
    for (first, second, kind, first_method, second_method) in [
        (
            getter_a as ModelImplProvider,
            getter_b as ModelImplProvider,
            PropertyBuildErrorKind::ConflictingGetter,
            "get_name",
            "read_name",
        ),
        (
            setter_a,
            setter_b,
            PropertyBuildErrorKind::ConflictingSetter,
            "set_name",
            "replace_name",
        ),
    ] {
        let registry = snapshot(first, second);
        let PropertyResolutionError::Assembly(errors) = owner
            .try_properties_in(&registry)
            .expect_err("distinct accessors conflict")
        else {
            panic!("property assembly error");
        };
        assert_eq!(errors.errors().len(), 1);
        let error = &errors.errors()[0];
        assert_eq!(error.kind(), kind);
        assert_eq!(error.property_name(), "name");
        let conflict = error.conflict().expect("accessor conflict details");
        assert_eq!(conflict.first_method(), first_method);
        assert_eq!(conflict.second_method(), second_method);
        assert_eq!(conflict.first_origin(), &SOURCE_A);
        assert_eq!(conflict.second_origin(), &SOURCE_B);
        let display = error.to_string();
        assert!(display.contains(first_method));
        assert!(display.contains(second_method));
        assert!(display.contains("model_impl_merge_tests"));
        let PropertyResolutionError::Assembly(repeated) =
            owner.try_properties_in(&registry).expect_err("cached conflict")
        else {
            panic!("same assembly category");
        };
        assert_eq!(errors.errors().len(), repeated.errors().len());
        assert_eq!(
            owner
                .property_fragments_in(&registry)
                .expect("diagnostic source facts")
                .fragments()
                .len(),
            4
        );
    }
    let valid = snapshot(getter_a, setter_a);
    assert!(
        owner.try_properties_in(&valid).is_ok(),
        "a failed snapshot must not contaminate another overlay set"
    );
}

#[test]
fn test_distinct_getter_and_setter_report_both_conflicts() {
    let owner = TypeMetadata::of::<Record>();
    let registry = snapshot(combined_a, combined_b);
    let PropertyResolutionError::Assembly(errors) = owner
        .try_properties_in(&registry)
        .expect_err("both accessor kinds conflict")
    else {
        panic!("property assembly error");
    };
    assert_eq!(errors.errors().len(), 2);
    assert_eq!(errors.errors()[0].kind(), PropertyBuildErrorKind::ConflictingGetter);
    assert_eq!(errors.errors()[1].kind(), PropertyBuildErrorKind::ConflictingSetter);
    assert!(errors.errors().iter().all(|error| error.conflict().is_some()));
}

#[test]
fn test_repeated_identity_is_not_a_distinct_accessor_conflict() {
    let owner = TypeMetadata::of::<Record>();
    for provider in [getter_a as ModelImplProvider, setter_a as ModelImplProvider] {
        let registry = snapshot(provider, provider);
        let merged = owner.try_properties_in(&registry).expect("same accessor identity");
        assert_eq!(merged.properties().len(), 2);
        let repeated = owner.try_properties_in(&registry).expect("owned result");
        assert_eq!(merged.properties().len(), repeated.properties().len());
    }
}

#[test]
fn test_registry_cache_owns_dynamic_results_and_releases_them_with_the_registry() {
    let owner = TypeMetadata::of::<Record>();
    let reflection = snapshot(getter_a, setter_a);
    let registry = ModelRegistry::from_context(&reflection).expect("snapshot model registry");
    let first = registry.properties_for(owner).expect("first merge");
    let second = registry.properties_for(owner).expect("cached merge");
    let weak = match (&first, &second) {
        (ResolvedProperties::Merged(first), ResolvedProperties::Merged(second)) => {
            assert!(Arc::ptr_eq(first, second));
            Arc::downgrade(first)
        }
        _ => panic!("expected dynamic merge"),
    };
    drop(first);
    drop(second);
    drop(registry);
    assert!(weak.upgrade().is_none(), "registry cache retained merged storage");
}

#[test]
fn test_direct_snapshot_queries_own_independent_merges() {
    let owner = TypeMetadata::of::<Record>();
    let reflection = snapshot(getter_a, setter_a);
    let first = owner.try_properties_in(&reflection).expect("first merge");
    let second = owner.try_properties_in(&reflection).expect("second merge");
    let (ResolvedProperties::Merged(first), ResolvedProperties::Merged(second)) = (&first, &second) else {
        panic!("expected dynamic merge");
    };
    assert!(!Arc::ptr_eq(first, second), "direct queries do not keep a cache");
}

#[test]
fn test_distinct_model_registries_do_not_share_their_property_cache() {
    let owner = TypeMetadata::of::<Record>();
    let first_reflection = snapshot(getter_a, setter_a);
    let second_reflection = snapshot(getter_a, setter_a);
    let first_registry = ModelRegistry::from_context(&first_reflection).expect("first model registry");
    let second_registry = ModelRegistry::from_context(&second_reflection).expect("second model registry");
    let first = first_registry.properties_for(owner).expect("first merge");
    let second = second_registry.properties_for(owner).expect("second merge");
    let (ResolvedProperties::Merged(first), ResolvedProperties::Merged(second)) = (&first, &second) else {
        panic!("expected dynamic merge");
    };
    assert!(!Arc::ptr_eq(first, second));
}

#[test]
fn test_registry_cache_initializes_once_for_concurrent_queries() {
    PROVIDER_CALLS.store(0, Ordering::SeqCst);
    let owner = TypeMetadata::of::<Record>();
    let reflection = snapshot(counting_getter, setter_a);
    let registry = ModelRegistry::from_context(&reflection).expect("model registry");
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                registry.properties_for(owner).expect("concurrent merge");
            });
        }
    });
    assert_eq!(PROVIDER_CALLS.load(Ordering::SeqCst), 1);
    registry.properties_for(owner).expect("cached merge");
    assert_eq!(
        PROVIDER_CALLS.load(Ordering::SeqCst),
        1,
        "cached queries do not call providers again"
    );
}

#[test]
fn test_fixed_capability_aggregates_sources_and_preserves_conflicts() {
    let owner = TypeMetadata::of::<Record>();
    let registry = snapshot(getter_a, setter_a);
    let properties = owner
        .try_properties_in(&registry)
        .expect("all registered source fragments are merged");
    let name = properties.property("name").expect("name property");
    assert_eq!(name.getter().expect("getter").rust_method_name(), "get_name");
    assert_eq!(name.setter().expect("setter").rust_method_name(), "set_name");

    let conflicting = snapshot(getter_a, getter_b);
    let PropertyResolutionError::Assembly(errors) = owner
        .try_properties_in(&conflicting)
        .expect_err("conflicting provider accessors remain an assembly error")
    else {
        panic!("property assembly error");
    };
    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), PropertyBuildErrorKind::ConflictingGetter);
    assert_eq!(errors.errors()[0].property_name(), "name");
}

#[test]
fn test_provider_panic_does_not_poison_registry_cache() {
    PANIC_ONCE.store(true, Ordering::SeqCst);
    let owner = TypeMetadata::of::<Record>();
    let reflection = snapshot(panic_once_getter, setter_a);
    let registry = ModelRegistry::from_context(&reflection).expect("model registry");
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| registry.properties_for(owner)));
    assert!(panic.is_err());
    assert!(registry.properties_for(owner).is_ok());
}

#[test]
fn test_registry_cache_releases_failed_assembly_diagnostics_on_drop() {
    let owner = TypeMetadata::of::<Record>();
    let reflection = snapshot(getter_a, getter_b);
    let registry = ModelRegistry::from_context(&reflection).expect("model registry");
    let first = registry.properties_for(owner).expect_err("conflicting getters");
    let second = registry.properties_for(owner).expect_err("cached diagnostic");
    let (PropertyResolutionError::Assembly(first), PropertyResolutionError::Assembly(second)) = (first, second) else {
        panic!("expected assembly failures");
    };
    assert!(Arc::ptr_eq(&first, &second));
    let weak = Arc::downgrade(&first);
    drop(first);
    drop(second);
    drop(registry);
    assert!(weak.upgrade().is_none(), "registry cache retained failed diagnostics");
}
