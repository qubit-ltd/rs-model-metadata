// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runtime smoke coverage for all six shared macro entry points.

use std::any::TypeId;

use model_runtime::__private::TypeDescriptor;
use model_runtime::__private::qubit_id::Id;
use model_runtime::metadata::TypeMetadata;
use model_runtime::registry::ModelRegistry;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_derive::Projection;
use qubit_model_derive::Value;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::Reflect as ReflectType;

fn reflection_context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("explicit reflection context builds")
}

#[Entity(id = "example.EntityFixture")]
struct EntityFixture {
    #[identifier]
    id: Id,
}

#[Entity(id = "example.OpaqueIdentifierFixture")]
struct OpaqueIdentifierFixture {
    #[identifier]
    #[opaque]
    id: Id,
}

#[Projection(source_id = "example.EntityFixture")]
struct ProjectionFixture {
    #[identifier]
    id: Id,
}

#[Model]
struct ModelFixture {
    value: String,
}

#[Enum]
enum EnumFixture {
    Ready,
    Data(u64),
}

#[Value(transparent)]
struct ValueFixture(String);

#[Model(id = "example.GenericModel")]
struct GenericModel<T> {
    value: T,
}

#[ModelImpl(specialize(T = u64))]
impl<T: Clone + ReflectType> GenericModel<T> {
    pub fn current(&self) -> T {
        self.value.clone()
    }
}

#[Enum]
enum GenericEnum<T> {
    Value(T),
}

#[ModelImpl(specialize(T = u64))]
impl<T: Clone + ReflectType> GenericEnum<T> {
    pub fn current(&self) -> T {
        match self {
            Self::Value(value) => value.clone(),
        }
    }
}

#[Value]
struct GenericValue<T> {
    value: T,
}

#[ModelImpl(specialize(T = u64))]
impl<T: Clone + ReflectType> GenericValue<T> {
    pub fn current(&self) -> T {
        self.value.clone()
    }
}

#[ModelImpl]
impl ModelFixture {
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[test]
fn test_six_entry_points_share_reflection_expansion() {
    assert_eq!(TypeDescriptor::of::<EntityFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<OpaqueIdentifierFixture>().fields().len(), 0);
    assert_eq!(TypeMetadata::of::<OpaqueIdentifierFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<ProjectionFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<ModelFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<EnumFixture>().variants().len(), 2);
    assert_eq!(TypeDescriptor::of::<ValueFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<GenericModel<u64>>().fields().len(), 1);
    let generic_u64 = TypeMetadata::of::<GenericModel<u64>>();
    let generic_string = TypeMetadata::of::<GenericModel<String>>();
    assert!(!std::ptr::eq(generic_u64, generic_string));
    assert_eq!(generic_u64.model_id(), None);
    assert!(std::ptr::eq(
        generic_u64.generic_definition().expect("generic definition"),
        generic_string.generic_definition().expect("shared generic definition"),
    ));
    assert_eq!(ModelFixture { value: "ok".into() }.value(), "ok");
    let _ = EntityFixture { id: Id::new(1) }.id;
    let _ = OpaqueIdentifierFixture { id: Id::new(2) }.id;
    let _ = ProjectionFixture { id: Id::new(1) }.id;
    let _ = EnumFixture::Ready;
    let EnumFixture::Data(value) = EnumFixture::Data(2) else {
        unreachable!()
    };
    assert_eq!(value, 2);
    let _ = ValueFixture("value".into()).0;
    let _ = GenericModel { value: 3_u64 }.value;
}

#[test]
fn test_generic_metadata_initialization_is_unique_across_threads() {
    let addresses = (0..8)
        .map(|_| std::thread::spawn(|| TypeMetadata::of::<GenericModel<u32>>() as *const TypeMetadata as usize))
        .map(|thread| thread.join().expect("metadata thread must complete"))
        .collect::<Vec<_>>();

    assert!(addresses.iter().all(|address| *address == addresses[0]));
}

/// Effective model providers must remain specific to each concrete generic
/// type.
#[test]
fn test_registry_generic_model_providers_preserve_concrete_type() {
    let registry = ModelRegistry::try_global().expect("valid model registry");
    let definition = registry
        .generic("example.GenericModel")
        .expect("generic definition without instantiation");
    let context = registry.context().expect("registry reflection context");
    let definition_descriptor = definition
        .definition_in(context)
        .expect("generic definition binds in the registry context");
    assert!(std::ptr::eq(
        registry
            .generic_metadata_for(definition_descriptor.id())
            .expect("generic metadata indexed by definition identity"),
        definition
    ));
    assert!(
        registry
            .metadata_for(&TypeDescriptor::of::<GenericModel<u64>>())
            .expect("u64 has no concrete model metadata capability")
            .is_none()
    );
    assert!(
        registry
            .metadata_for(&TypeDescriptor::of::<GenericModel<String>>())
            .expect("String has no concrete model metadata capability")
            .is_none()
    );
}

/// Generic providers are visible only in contexts that include their typed
/// definition capability; descriptors returned by properties remain bound to
/// the context that resolved them.
#[test]
fn test_generic_model_impl_capabilities_are_context_bound() {
    let concrete_only = qubit_reflect::ReflectionContext::builder()
        .include_type::<GenericModel<u64>>()
        .build()
        .expect("concrete-only context builds");
    let concrete_registry = ModelRegistry::from_context(&concrete_only).expect("concrete-only model registry builds");
    let concrete_properties = concrete_registry
        .properties_for(TypeMetadata::of::<GenericModel<u64>>())
        .expect("metadata remains available without generic capability");
    assert!(concrete_properties.property("current").is_none());

    let linked = reflection_context();
    let linked_registry = ModelRegistry::from_context(&linked).expect("linked model registry");
    for metadata in [
        TypeMetadata::of::<GenericModel<u64>>(),
        TypeMetadata::of::<GenericEnum<u64>>(),
        TypeMetadata::of::<GenericValue<u64>>(),
    ] {
        let properties = linked_registry
            .properties_for(metadata)
            .expect("linked definition capability resolves");
        let current = properties
            .property("current")
            .expect("generic ModelImpl property is present");
        let descriptor = current
            .descriptor_in(&linked)
            .expect("property descriptor lookup")
            .expect("u64 output descriptor");
        assert_eq!(descriptor.type_id(), TypeId::of::<u64>());
        assert!(std::ptr::eq(descriptor.context(), &linked));
    }
}
