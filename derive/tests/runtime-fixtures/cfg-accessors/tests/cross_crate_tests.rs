// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Verifies reflected metadata from a separately compiled model crate.

use std::any::TypeId;

use model_derive_cfg_accessors_fixture::Configured;
use model_runtime::metadata::TypeMetadata;
use qubit_reflect::registry::ReflectRegistry;

/// The renamed runtime resolves cross-crate conditional contributions.
#[test]
fn test_cross_crate_conditional_contributions() {
    let metadata = TypeMetadata::of::<Configured>();
    let reflection = ReflectRegistry::initialize().expect("cross-crate reflection");
    assert!(metadata.try_property("paired").expect("properties").is_some());
    assert_eq!(
        metadata.try_property("feature_value").expect("properties").is_some(),
        cfg!(feature = "accessors"),
        "feature_value metadata should follow the accessors feature"
    );
    let present = reflection.implementations(TypeId::of::<Configured>()).iter()
        .flat_map(|implementation| implementation.methods())
        .any(|method| method.rust_name() == "feature_value");
    assert_eq!(
        present,
        cfg!(feature = "accessors"),
        "feature_value reflection should follow the accessors feature"
    );
}
