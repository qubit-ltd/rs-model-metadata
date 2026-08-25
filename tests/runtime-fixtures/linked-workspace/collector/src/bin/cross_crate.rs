// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::ModelRegistry;

fn main() {
    let _ = core::mem::size_of::<model_a::Source>();
    let _ = core::mem::size_of::<model_b::Target>();
    let registry = ModelRegistry::try_global()
        .expect("cross-crate registrations should be valid");
    assert!(registry.get("test.linked.Source").is_some());
    assert!(registry.get("test.linked.Target").is_some());
}
