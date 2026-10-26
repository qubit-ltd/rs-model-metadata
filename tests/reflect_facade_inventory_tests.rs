// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for model implementation inventory aggregation.

use qubit_model_metadata::__private::ModelImplFragment;
use qubit_model_metadata::__private::ModelImplProvider;
use qubit_model_metadata::__private::ModelImplSource;
use qubit_model_metadata::__private::v8::model_impl_fragments;
use qubit_model_metadata::metadata::ModelImplMetadata;
use qubit_reflect::Reflect;

#[derive(Reflect)]
struct AggregatedFixture;

fn first_provider() -> &'static ModelImplMetadata {
    panic!("aggregation test does not invoke providers")
}

fn second_provider() -> &'static ModelImplMetadata {
    panic!("aggregation test does not invoke providers")
}

qubit_reflect::__private::__inventory::submit! {
    ModelImplFragment::for_type::<AggregatedFixture>(
        first_provider as ModelImplProvider,
        ModelImplSource {
            module_path: "reflect_facade_inventory_tests",
            file: file!(),
            line: line!(),
            fingerprint: 1,
        },
    )
}

qubit_reflect::__private::__inventory::submit! {
    ModelImplFragment::for_type::<AggregatedFixture>(
        second_provider as ModelImplProvider,
        ModelImplSource {
            module_path: "reflect_facade_inventory_tests",
            file: file!(),
            line: line!(),
            fingerprint: 2,
        },
    )
}

#[test]
fn test_collects_multiple_impl_providers_for_one_concrete_type() {
    let fragments = model_impl_fragments::<AggregatedFixture>();
    assert_eq!(fragments.len(), 2, "both submitted providers should be retained");
    let mut fingerprints: Vec<_> = fragments.iter().map(|fragment| fragment.source().fingerprint).collect();
    fingerprints.sort_unstable();
    assert_eq!(fingerprints, [1, 2], "both declaration fingerprints should be retained");
}
