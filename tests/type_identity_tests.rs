// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use qubit_model_metadata::TypeIdentity;

#[test]
fn test_type_identity_compares_the_same_rust_type() {
    assert_eq!(TypeIdentity::of::<u64>(), TypeIdentity::of::<u64>());
}

#[test]
fn test_type_identity_exposes_name_debug_and_hash() {
    let identity = TypeIdentity::of::<u64>();
    let mut hasher = DefaultHasher::new();
    identity.hash(&mut hasher);
    let mut same_hasher = DefaultHasher::new();
    TypeIdentity::of::<u64>().hash(&mut same_hasher);

    assert_eq!(identity.type_name(), core::any::type_name::<u64>());
    assert_eq!(format!("{identity:?}"), "TypeIdentity(\"u64\")");
    assert_eq!(hasher.finish(), same_hasher.finish());
}
