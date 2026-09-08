// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "generic")]

//! Generic metadata specialization cache contracts.

use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;

#[Model(id = "generic.CacheFixture")]
struct CacheFixture<T> {
    value: T,
}

#[test]
fn specialization_cache_is_stable_per_concrete_type() {
    let first = TypeMetadata::of::<CacheFixture<u32>>();
    let second = TypeMetadata::of::<CacheFixture<u32>>();
    let different = TypeMetadata::of::<CacheFixture<u64>>();

    assert!(core::ptr::eq(first, second));
    assert!(!core::ptr::eq(first, different));
    assert!(first.generic_definition().is_some());
}
