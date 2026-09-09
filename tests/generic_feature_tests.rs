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

/// Concurrent queries share one metadata allocation for each concrete type.
#[test]
fn concurrent_specializations_share_metadata() {
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|_| scope.spawn(TypeMetadata::of::<CacheFixture<String>>))
            .collect();
        let expected = TypeMetadata::of::<CacheFixture<String>>();
        for handle in handles {
            assert!(core::ptr::eq(handle.join().unwrap(), expected));
        }
    });
}
