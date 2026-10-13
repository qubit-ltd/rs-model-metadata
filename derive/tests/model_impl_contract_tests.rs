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
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_reflect::Reflect;
use qubit_reflect::registry::ReflectRegistry;

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
    let meta = TypeMetadata::of::<Person>();
    assert!(meta.try_property("unavailable").expect("properties").is_none());
    assert!(meta.try_property("diagnostic").expect("properties").is_none());
    assert!(
        meta.try_property("full_name")
            .expect("properties")
            .expect("computed")
            .is_computed()
    );
    assert!(
        meta.try_property("name")
            .expect("properties")
            .expect("field")
            .is_writable()
    );
    let person = Person { name: "Ada".into() };
    assert_eq!(person.diagnostic(), 3);
    assert_eq!(person.name(), "Ada");
    person.set_schedule(1, 2);
    let reflection = ReflectRegistry::initialize().expect("reflection");
    assert!(
        reflection
            .implementations(TypeId::of::<Person>())
            .iter()
            .flat_map(|implementation| implementation.methods())
            .any(|method| method.rust_name() == "diagnostic")
    );
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
    assert!(
        TypeMetadata::of::<Person>()
            .try_property("greeting")
            .expect("properties")
            .is_none()
    );
    assert_eq!(Person { name: "Ada".into() }.greeting(), "Ada");
    let reflection = ReflectRegistry::initialize().expect("reflection");
    assert!(
        reflection
            .implementations(TypeId::of::<Person>())
            .iter()
            .flat_map(|implementation| implementation.methods())
            .any(|method| method.rust_name() == "greeting")
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
    let metadata = TypeMetadata::of::<SplitAccessors>();
    let properties = metadata.try_properties().expect("split accessors assemble");
    let title = properties.property("title").expect("computed property");
    assert!(title.getter().is_some());
    assert!(title.setter().is_some());
    let repeated = metadata.try_properties().expect("repeat resolution");
    let repeated_title = repeated.property("title").expect("repeated computed property");
    assert!(repeated_title.getter().is_some());
    assert!(repeated_title.setter().is_some());
}
