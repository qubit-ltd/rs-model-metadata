// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[qubit_model_derive::Model(id = "test.derive.Organization", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Organization {
    id: i64,
}

#[qubit_model_derive::Model(id = "test.derive.Invalid", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Invalid {
    nested: String,
    #[field(reference(target = "test.derive.Organization", target_field = id, same_as = "missing.id"))]
    organization_id: i64,
}

fn main() {}
