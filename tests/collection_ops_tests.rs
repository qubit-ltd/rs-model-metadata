// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generated collection adapters retain exact types and fail safely.

#![cfg(feature = "validation")]

use std::collections::BTreeMap;
use std::collections::HashMap;

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_reflect::ReflectedRef;

#[Model(no_hash)]
struct Collections {
    #[map(min_entries = 1)]
    hashed: HashMap<String, i32>,
    #[map(min_entries = 1)]
    ordered: BTreeMap<String, i32>,
    #[sequence(unique_items)]
    items: Vec<String>,
}

#[ModelImpl]
impl Collections {
    pub fn hashed(&self) -> &HashMap<String, i32> {
        &self.hashed
    }

    pub fn ordered(&self) -> &BTreeMap<String, i32> {
        &self.ordered
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }
}

#[Model]
struct ArrayWithSliceGetter {
    #[sequence(unique_items)]
    items: [String; 2],
}

#[ModelImpl]
impl ArrayWithSliceGetter {
    pub fn items(&self) -> &[String] {
        &self.items
    }
}

#[Model]
struct ArrayWithoutGetter {
    #[sequence(unique_items)]
    items: [String; 2],
}

/// Both supported map families report their true length and reject a wrong
/// value type.
#[test]
fn test_generated_map_adapters_check_exact_type() {
    let model = TypeMetadata::of::<Collections>();
    let hashed = HashMap::from([(String::from("one"), 1)]);
    let ordered = BTreeMap::from([(String::from("one"), 1), (String::from("two"), 2)]);
    for (name, value, expected) in [
        ("hashed", PropertyValue::Borrowed(ReflectedRef::new(&hashed)), 1),
        ("ordered", PropertyValue::Borrowed(ReflectedRef::new(&ordered)), 2),
    ] {
        let ops = model
            .field(name)
            .expect("declared field")
            .collection_ops()
            .expect("collection adapter");
        let map_len = ops.map_len().expect("map length adapter");
        assert_eq!(map_len(&value).expect("matching concrete map"), Some(expected));
        assert!(matches!(
            map_len(&PropertyValue::Borrowed(ReflectedRef::new(&42_i32))),
            Err(PropertyAccessError::ValueTypeMismatch(_))
        ));
    }
}

/// Generated item equality compares values and rejects unrelated element types.
#[test]
fn test_generated_sequence_adapter_checks_exact_element_type() {
    let model = TypeMetadata::of::<Collections>();
    let ops = model
        .field("items")
        .expect("declared field")
        .collection_ops()
        .expect("collection adapter");
    let item_eq = ops.item_eq().expect("item equality adapter");
    let one = String::from("same");
    let two = String::from("same");
    let other = String::from("other");
    assert!(item_eq(ReflectedRef::new(&one), ReflectedRef::new(&two)).expect("matching elements"));
    assert!(!item_eq(ReflectedRef::new(&one), ReflectedRef::new(&other)).expect("different elements"));
    assert!(matches!(
        item_eq(ReflectedRef::new(&one), ReflectedRef::new(&42_i32)),
        Err(PropertyAccessError::ValueTypeMismatch(_))
    ));
}

/// An array with a borrowed slice getter has an exact element comparator.
#[test]
fn test_array_slice_getter_generates_item_equality() {
    let model = TypeMetadata::of::<ArrayWithSliceGetter>();
    let roots = [model];
    let graph = StructureResolver::new(ResolveInputs {
        models: ModelRegistry::global(),
        roots: &roots,
    })
    .resolve()
    .expect("array getter structure");
    let property = graph
        .properties(model)
        .expect("resolved properties")
        .property("items")
        .expect("array property");
    let array = ArrayWithSliceGetter {
        items: [String::from("same"), String::from("same")],
    };
    let value = property.get(ReflectedRef::new(&array)).expect("slice getter");
    assert!(matches!(value, PropertyValue::BorrowedSlice(ref values) if values.len() == 2));
    let ops = model
        .field("items")
        .expect("array field")
        .collection_ops()
        .expect("array adapter");
    let item_eq = ops.item_eq().expect("array element equality");
    let left = String::from("same");
    let right = String::from("same");
    assert!(item_eq(ReflectedRef::new(&left), ReflectedRef::new(&right)).expect("matching elements"));
}

/// Without a slice getter, array uniqueness remains unsupported at binding.
#[test]
fn test_array_without_getter_is_rejected_by_capability_check() {
    let root = TypeMetadata::of::<ArrayWithoutGetter>();
    let roots = [root];
    let models = ModelRegistry::from_static_metadata(&[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("array structure");
    let errors = ValidationCapabilities::check(root, &graph).expect_err("missing slice getter");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
}
