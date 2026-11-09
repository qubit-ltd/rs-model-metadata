// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "validation")]

//! Tests for explicit erased-selector execution capabilities.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::BorrowedCollection;
use qubit_model_metadata::metadata::BorrowedPropertySlice;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SelectorPosition;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_model_metadata::validation::ValidationBuildErrorKind;
use qubit_model_metadata::validation::ValidationBuildInputs;
use qubit_model_metadata::validation::ValidationCapabilities;
use qubit_model_metadata::validation::ValidationOptions;
use qubit_model_metadata::validation::ValidationPlan;
use qubit_reflect::ReflectionContext;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;
use qubit_validator::ValidatorRegistry;

#[Model(id = "validation.MapKeySelector")]
struct MapKeySelector {
    #[map_key(text(non_blank))]
    values: HashMap<String, String>,
}

#[Model(id = "validation.OwnedSelectorGetter")]
struct OwnedSelectorGetter {
    #[element(text(non_blank))]
    values: Vec<String>,
}

#[Model(id = "validation.CollectionItemAdapters")]
struct CollectionItemAdapters {
    #[sequence(min_items = 0)]
    #[element(validator(id = "runtime.vector_item"))]
    values: Vec<i32>,
    #[sequence(unique_items)]
    #[element(validator(id = "runtime.array_item"))]
    array: [i32; 2],
    #[map(min_entries = 0)]
    #[map_key(validator(id = "runtime.hash_key"))]
    #[map_value(validator(id = "runtime.hash_value"))]
    hashed: HashMap<String, i32>,
    #[map(min_entries = 0)]
    #[map_key(validator(id = "runtime.tree_key"))]
    #[map_value(validator(id = "runtime.tree_value"))]
    ordered: BTreeMap<String, i32>,
    #[sequence(min_items = 0)]
    #[element(validator(id = "runtime.optional_item"))]
    optional: Option<Vec<i32>>,
}

#[ModelImpl]
impl CollectionItemAdapters {
    pub fn array(&self) -> &[i32] {
        &self.array
    }
}

#[Model(id = "validation.UnsupportedBoxedSliceSelector")]
struct UnsupportedBoxedSliceSelector {
    #[element(text(non_blank))]
    values: Box<[String]>,
}

#[ModelImpl]
impl OwnedSelectorGetter {
    #[must_use]
    pub fn values(&self) -> Vec<String> {
        self.values.clone()
    }
}

#[test]
fn test_capability_matrix_accepts_supported_map_key_selector() {
    let models = ModelRegistry::try_global().unwrap();
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .unwrap();
    assert!(ValidationCapabilities::check(TypeMetadata::of::<MapKeySelector>(), &graph).is_ok());
}

#[test]
fn test_map_key_selector_violation_reports_map_key_path() {
    let source = ModelRegistryOrigin {
        crate_name: "validation-tests",
        crate_version: None,
        module_path: module_path!(),
        file: Some(file!()),
        line: Some(line!()),
        column: None,
    };
    let metadata = TypeMetadata::of::<MapKeySelector>();
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[(metadata, &source)])
        .expect("model registry");
    let graph = Arc::new(
        StructureResolver::new(ResolveInputs {
            roots: &[],
            models: &models,
        })
        .resolve()
        .expect("model graph"),
    );
    let validators = ValidatorRegistry::empty();

    let result = ValidationPlan::build(
        metadata,
        ValidationBuildInputs {
            graph: Arc::clone(&graph),
            validators: &validators,
        },
    );
    let plan = result.expect("map-key selector plan builds");
    let report = plan
        .validate_dynamic(
            DynamicRef::<Local>::new(&MapKeySelector {
                values: HashMap::from([(String::new(), "value".to_owned())]),
            }),
            &ValidationOptions::default(),
        )
        .expect("map-key validation executes");
    assert_eq!(report.violations().len(), 1);
    assert!(report.violations()[0].path().render().ends_with(".<map-key>"));
}

#[test]
fn test_owned_collection_getter_is_rejected_at_build() {
    let models = ModelRegistry::try_global().expect("model registry");
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("model graph");
    let errors = ValidationCapabilities::check(TypeMetadata::of::<OwnedSelectorGetter>(), &graph)
        .expect_err("owned collection output cannot provide member borrows");
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
}

#[test]
fn test_boxed_slice_selector_is_rejected_without_a_borrow_adapter() {
    let models = ModelRegistry::try_global().expect("model registry");
    let graph = StructureResolver::new(ResolveInputs { models, roots: &[] })
        .resolve()
        .expect("model graph");
    let errors = ValidationCapabilities::check(TypeMetadata::of::<UnsupportedBoxedSliceSelector>(), &graph)
        .expect_err("unsupported collection shapes remain build errors");
    assert_eq!(errors[0].kind(), ValidationBuildErrorKind::UnsupportedExecution);
}

#[test]
fn test_generated_collection_items_borrow_values_for_all_supported_shapes() {
    let metadata = TypeMetadata::of::<CollectionItemAdapters>();
    let values = vec![3, 5];
    let array = [7, 11];
    let hashed = HashMap::from([(String::from("hash-key"), 13)]);
    let ordered = BTreeMap::from([(String::from("tree-key"), 17)]);
    let optional_values = vec![19, 23];

    fn items<'a>(
        metadata: &TypeMetadata,
        name: &str,
        value: &PropertyValue<'a>,
        position: SelectorPosition,
    ) -> Option<BorrowedCollection<'a>> {
        metadata
            .field(name)
            .expect("declared field")
            .collection_ops()
            .expect("generated collection operations")
            .collection_items()
            .expect("generated item adapter")(value, position)
        .expect("matching collection value")
    }
    fn numbers(collection: BorrowedCollection<'_>) -> Vec<i32> {
        collection
            .into_iter()
            .map(|item| *item.downcast_ref::<i32>().expect("integer member"))
            .collect::<Vec<_>>()
    }

    let borrowed_values = PropertyValue::Borrowed(DynamicRef::<Local>::new(&values));
    assert_eq!(
        numbers(items(metadata, "values", &borrowed_values, SelectorPosition::Element).unwrap()),
        [3, 5]
    );
    let borrowed_array = PropertyValue::Borrowed(DynamicRef::<Local>::new(&array));
    assert_eq!(
        numbers(items(metadata, "array", &borrowed_array, SelectorPosition::Element).unwrap()),
        [7, 11]
    );

    let borrowed_hash = PropertyValue::Borrowed(DynamicRef::<Local>::new(&hashed));
    let hash_keys = items(metadata, "hashed", &borrowed_hash, SelectorPosition::MapKey).unwrap();
    assert_eq!(
        hash_keys
            .map(|item| item.downcast_ref::<String>().unwrap().as_str().to_owned())
            .collect::<Vec<_>>(),
        ["hash-key"]
    );
    let hash_values = items(metadata, "hashed", &borrowed_hash, SelectorPosition::MapValue).unwrap();
    assert_eq!(
        hash_values
            .map(|item| *item.downcast_ref::<i32>().unwrap())
            .collect::<Vec<_>>(),
        [13]
    );

    let borrowed_tree = PropertyValue::Borrowed(DynamicRef::<Local>::new(&ordered));
    let tree_keys = items(metadata, "ordered", &borrowed_tree, SelectorPosition::MapKey).unwrap();
    assert_eq!(
        tree_keys
            .map(|item| item.downcast_ref::<String>().unwrap().as_str().to_owned())
            .collect::<Vec<_>>(),
        ["tree-key"]
    );
    let tree_values = items(metadata, "ordered", &borrowed_tree, SelectorPosition::MapValue).unwrap();
    assert_eq!(
        tree_values
            .map(|item| *item.downcast_ref::<i32>().unwrap())
            .collect::<Vec<_>>(),
        [17]
    );

    let optional_some = PropertyValue::OptionalBorrowed(Some(DynamicRef::<Local>::new(&optional_values)));
    assert_eq!(
        numbers(items(metadata, "optional", &optional_some, SelectorPosition::Element).unwrap()),
        [19, 23]
    );
    let optional_none = PropertyValue::OptionalBorrowed(None);
    assert!(items(metadata, "optional", &optional_none, SelectorPosition::Element).is_none());

    let slice_values = [29, 31];
    let borrowed_slice = PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(&slice_values));
    assert_eq!(
        numbers(items(metadata, "values", &borrowed_slice, SelectorPosition::Element).unwrap()),
        [29, 31]
    );
}
