// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Model implementation methods preserve reflection independently of
//! properties.

use std::any::TypeId;

use model_runtime::metadata::TypeMetadata;
use model_runtime::registry::ModelRegistry;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::Reflect as ReflectType;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

fn reflection_context() -> ReflectionContext {
    ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("explicit reflection context builds")
}

#[Model]
struct Person {
    name: String,
}

#[ModelImpl]
impl Person {
    /// Inactive signatures cannot leak into adapters or property metadata.
    #[cfg(any())]
    pub fn unavailable(&self) -> MissingGetterType {
        unreachable!()
    }

    /// Inactive setter signatures cannot leak into adapters or pair assertions.
    #[cfg(any())]
    pub fn set_unavailable(&mut self, _: MissingSetterType) {
        unreachable!()
    }

    /// Excludes this getter while preserving its same-named stored field.
    #[model_property(skip)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retains reflection while excluding the computed property.
    #[model_property(skip)]
    pub fn diagnostic(&self) -> usize {
        self.name.len()
    }

    /// Provides a computed property without a separate marker.
    pub fn full_name(&self) -> String {
        self.name.clone()
    }

    /// A business operation whose name happens to start with set_.
    pub fn set_schedule(&self, _day: u32, _hour: u32) {}
}

/// Exclusion removes only the method contribution.
#[test]
fn test_exclusion_preserves_computed_and_stored_properties() {
    let context = reflection_context();
    let registry = ModelRegistry::from_context(&context).expect("model registry");
    let meta = TypeMetadata::of::<Person>();
    let properties = registry.properties_for(meta).expect("properties");
    assert!(properties.property("unavailable").is_none());
    assert!(properties.property("diagnostic").is_none());
    assert!(properties.property("full_name").expect("computed").is_computed());
    assert!(properties.property("name").expect("field").is_writable());
    let descriptor = qubit_reflect::TypeDescriptor::of_in::<Person>(&context);
    assert!(descriptor.method("diagnostic").is_ok());
    let person = Person { name: "Ada".into() };
    assert_eq!(person.diagnostic(), 3);
    assert_eq!(person.name(), "Ada");
    person.set_schedule(1, 2);
}

#[Reflect]
trait Greeting {
    /// Provides trait behavior, independently of model properties.
    fn greeting(&self) -> String;
}

#[ModelImpl]
impl Greeting for Person {
    /// Contributes reflected trait behavior without a computed property.
    fn greeting(&self) -> String {
        self.name.clone()
    }
}

/// Trait methods remain reflected but do not become inherent properties.
#[test]
fn test_trait_methods_do_not_contribute_properties() {
    let context = reflection_context();
    let registry = ModelRegistry::from_context(&context).expect("model registry");
    let metadata = TypeMetadata::of::<Person>();
    assert!(
        registry
            .properties_for(metadata)
            .expect("properties")
            .property("greeting")
            .is_none()
    );
    assert!(
        qubit_reflect::TypeDescriptor::of_in::<Person>(&context)
            .method("greeting")
            .is_ok()
    );
    assert_eq!(Person { name: "Ada".into() }.greeting(), "Ada");
}

#[Model]
struct GenericName<T> {
    value: T,
}

#[ModelImpl(specialize(T = String), specialize(T = u32))]
impl<T: Clone + ReflectType> GenericName<T> {
    /// A computed property specialized using the reflection support boundary.
    pub fn current(&self) -> T {
        self.value.clone()
    }
}

/// Generic impl specializations keep concrete property identities separate.
#[test]
fn test_generic_impl_specializations_have_exact_property_types() {
    let registry = ModelRegistry::try_global().expect("generic model registry");
    let context = registry.context().expect("registry reflection context");
    let text = TypeMetadata::of::<GenericName<String>>();
    let number = TypeMetadata::of::<GenericName<u32>>();
    let text_properties = registry.properties_for(text).expect("text properties");
    let number_properties = registry.properties_for(number).expect("number properties");
    let text_property = text_properties.property("current").expect("text property");
    let number_property = number_properties.property("current").expect("number property");
    assert_eq!(
        text_property
            .descriptor_in(context)
            .expect("descriptor lookup")
            .expect("text type")
            .type_id(),
        TypeId::of::<String>()
    );
    assert_eq!(
        number_property
            .descriptor_in(context)
            .expect("descriptor lookup")
            .expect("number type")
            .type_id(),
        TypeId::of::<u32>()
    );
    let value = GenericName { value: 7u32 };
    assert!(
        number_property
            .get_dynamic_in(context, DynamicRef::<Local>::new(&value))
            .is_ok()
    );
}

#[Model]
struct Fixed<const N: usize> {
    bytes: [u8; N],
}

#[ModelImpl(specialize(N = 2))]
impl<const N: usize> Fixed<N> {
    /// Reports the concrete const-specialized storage length.
    pub fn length(&self) -> usize {
        self.bytes.len()
    }
}

/// Const substitutions include generic argument positions, not only arrays.
#[test]
fn test_const_impl_specialization() {
    let registry = ModelRegistry::try_global().expect("const specialization model registry");
    let context = registry.context().expect("registry reflection context");
    let metadata = TypeMetadata::of::<Fixed<2>>();
    let properties = registry.properties_for(metadata).expect("properties");
    let property = properties.property("length").expect("length property");
    assert!(
        property
            .get_dynamic_in(context, DynamicRef::<Local>::new(&Fixed { bytes: [1, 2] }))
            .is_ok()
    );
}

#[Model]
struct SplitAccessors {
    value: String,
}

#[ModelImpl]
impl SplitAccessors {
    pub fn title(&self) -> String {
        self.value.clone()
    }
}

#[ModelImpl]
impl SplitAccessors {
    pub fn set_title(&mut self, value: String) {
        self.value = value;
    }
}

/// Separate inherent blocks contribute to one checked property set.
#[test]
fn test_multiple_impl_blocks_merge_accessors() {
    let context = reflection_context();
    let registry = ModelRegistry::from_context(&context).expect("model registry");
    let metadata = TypeMetadata::of::<SplitAccessors>();
    let properties = registry.properties_for(metadata).expect("split accessors assemble");
    let title = properties.property("title").expect("computed property");
    assert!(title.getter().is_some());
    assert!(title.setter().is_some());
    let repeated = registry.properties_for(metadata).expect("repeat resolution");
    let repeated_title = repeated.property("title").expect("repeated computed property");
    assert!(repeated_title.getter().is_some());
    assert!(repeated_title.setter().is_some());
}
