// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[qubit_model_derive::Model(id = "test.derive.Organization", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Organization {
    id: i64,
}

#[qubit_model_derive::Model(
    id = "test.derive.Invalid",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize,
    primary_key(fields(id, id, missing), generated(name, missing)),
    unique(fields(name, name, id, missing), ignore_case(id, other)),
    unique(name = "empty_unique"),
    index(fields(id, id, missing)),
    index(name = "empty_index"),
    key(fields(name, name, missing)),
    key(name = "empty_key")
)]
struct Invalid {
    id: i64,
    name: String,
    other: String,
    #[field(reference(
        entity = "test.derive.Organization",
        property = id,
        path = "missing"
    ))]
    organization_id: i64,
}

fn main() {}
