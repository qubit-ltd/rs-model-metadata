// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Snapshot-selected implementation overlays preserve conflicts and ownership.

use std::ptr::eq;
use std::sync::OnceLock;

use qubit_model_derive::Model;
use qubit_model_metadata::__private::ModelImplProvider;
use qubit_model_metadata::__private::model_impl_fragment_key;
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::ModelImplMetadata;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyBuildErrorKind;
use qubit_model_metadata::metadata::PropertyFragmentSource;
use qubit_model_metadata::metadata::PropertyResolutionError;
use qubit_model_metadata::metadata::PropertySetFailure;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SetterMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::ReflectRegistry;
use qubit_reflect::ReflectedMut;
use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;
use qubit_reflect::capability::CapabilityDescriptor;
use qubit_reflect::identity::FragmentIdentity;
use qubit_reflect::registry::RegistrySnapshotBuilder;

#[Model]
struct Record {
    name: String,
    count: u32,
}

fn read_name(target: ReflectedRef<'_>) -> Result<PropertyValue<'_>, PropertyAccessError> {
    let target = target
        .downcast::<Record>()
        .map_err(|_| PropertyAccessError::user("wrong owner"))?;
    Ok(PropertyValue::Borrowed(ReflectedRef::new(&target.name)))
}

fn write_name(target: ReflectedMut<'_>, value: ReflectedOwned) -> Result<(), PropertySetFailure> {
    let target = target
        .downcast::<Record>()
        .unwrap_or_else(|_| panic!("validated owner"));
    target.name = value.downcast::<String>().unwrap_or_else(|_| panic!("validated input"));
    Ok(())
}

// ModelImpl code generation repeats the declaration's backing fields in each
// overlay. These fixtures reproduce that ABI shape with independently owned
// getter/setter descriptors, without exposing an internal merge test hook.
fn overlay(getter_name: Option<&'static str>, setter_name: Option<&'static str>) -> ModelImplMetadata {
    let owner = TypeMetadata::of::<Record>();
    let name = &owner.fields()[0];
    let count = &owner.fields()[1];
    let getter = getter_name.map(|method| {
        v7::leak(GetterMetadata::new::<Record>(
            method,
            name.type_ref(),
            GetterOutputKind::Borrowed,
            read_name,
        ))
    });
    let setter = setter_name.map(|method| {
        v7::leak(SetterMetadata::new::<Record, String>(
            method,
            name.type_ref(),
            write_name,
        ))
    });
    let mut fragments = vec![
        v7::property_fragment("name", name.type_ref(), PropertyFragmentSource::Field(name)),
        v7::property_fragment("count", count.type_ref(), PropertyFragmentSource::Field(count)),
    ];
    if let Some(getter) = getter {
        fragments.push(v7::property_fragment(
            "name",
            name.type_ref(),
            PropertyFragmentSource::Getter(getter),
        ));
    }
    if let Some(setter) = setter {
        fragments.push(v7::property_fragment(
            "name",
            name.type_ref(),
            PropertyFragmentSource::Setter(setter),
        ));
    }
    let properties = v7::leak_slice(vec![
        v7::property_metadata("name", name.type_ref(), Some(name), getter, setter),
        v7::property_metadata("count", count.type_ref(), Some(count), None, None),
    ]);
    owner
        .validate_properties(properties)
        .expect("each overlay is individually valid");
    v7::model_impl_metadata(
        v7::leak_slice(fragments),
        Ok(v7::leak(v7::local_property_set(properties))),
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

fn snapshot(first: ModelImplProvider, second: ModelImplProvider) -> ReflectRegistry {
    let mut builder = RegistrySnapshotBuilder::new();
    for (key, provider) in [
        ("qubit.model.impl.v1.fmerge_a", first),
        ("qubit.model.impl.v1.fmerge_b", second),
    ] {
        builder.add_type_capabilities(
            TypeMetadata::of::<Record>().descriptor(),
            vec![CapabilityDescriptor::with_adapter(
                model_impl_fragment_key(key),
                provider,
            )],
            FragmentIdentity::new("model-impl-test", key, 1, 1, "capability", 1),
        );
    }
    builder.build().expect("distinct capability slots")
}

#[test]
fn test_complementary_overlays_merge_accessors_and_coalesce_field_fragments() {
    let owner = TypeMetadata::of::<Record>();
    let registry = snapshot(getter_a, setter_a);
    let properties = owner.try_properties_in(&registry).expect("complementary accessors");
    assert!(eq(
        properties,
        owner.try_properties_in(&registry).expect("cached merge")
    ));
    assert_eq!(properties.properties().len(), 2);
    let name = properties.property("name").expect("merged name");
    assert_eq!(name.getter().expect("getter").rust_method_name(), "get_name");
    assert_eq!(name.setter().expect("setter").rust_method_name(), "set_name");
    let mut value = Record {
        name: "before".to_owned(),
        count: 7,
    };
    name.set(ReflectedMut::new(&mut value), ReflectedOwned::new("after".to_owned()))
        .expect("merged setter");
    let PropertyValue::Borrowed(read) = name.get(ReflectedRef::new(&value)).expect("merged getter") else {
        panic!("borrowed getter");
    };
    assert!(eq(read.downcast_ref::<String>().expect("String output"), &value.name));
    assert_eq!(value.name, "after");
    assert_eq!(value.count, 7);
    let fragments = owner.property_fragments_in(&registry).expect("raw fragments");
    assert_eq!(fragments.len(), 4);
    assert_eq!(
        fragments
            .iter()
            .filter(|fragment| matches!(fragment.source(), PropertyFragmentSource::Field(_)))
            .count(),
        2
    );
    for fragment in fragments {
        assert!(eq(
            fragment.type_ref(),
            properties
                .property(fragment.name())
                .expect("fragment property")
                .type_ref()
        ));
    }
}

#[test]
fn test_distinct_accessors_conflict_without_poisoning_other_snapshots() {
    let owner = TypeMetadata::of::<Record>();
    for (first, second) in [
        (getter_a as ModelImplProvider, getter_b as ModelImplProvider),
        (setter_a, setter_b),
    ] {
        let registry = snapshot(first, second);
        let PropertyResolutionError::Assembly(errors) = owner
            .try_properties_in(&registry)
            .expect_err("distinct accessors conflict")
        else {
            panic!("property assembly error");
        };
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(errors.errors()[0].kind(), PropertyBuildErrorKind::InvalidName);
        assert_eq!(errors.errors()[0].property_name(), "name");
        let PropertyResolutionError::Assembly(repeated) =
            owner.try_properties_in(&registry).expect_err("cached conflict")
        else {
            panic!("same assembly category");
        };
        assert!(eq(errors, repeated));
        assert_eq!(
            owner
                .property_fragments_in(&registry)
                .expect("diagnostic source facts")
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
fn test_repeated_identity_is_not_a_distinct_accessor_conflict() {
    let owner = TypeMetadata::of::<Record>();
    for provider in [getter_a as ModelImplProvider, setter_a as ModelImplProvider] {
        let registry = snapshot(provider, provider);
        let merged = owner.try_properties_in(&registry).expect("same accessor identity");
        assert_eq!(merged.properties().len(), 2);
        let repeated = owner.try_properties_in(&registry).expect("cached result");
        assert!(eq(merged, repeated));
    }
}
