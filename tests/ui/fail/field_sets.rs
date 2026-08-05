// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[derive(qubit_model_derive::ModelMetadata)]
struct Organization {
    id: i64,
}

#[derive(qubit_model_derive::ModelMetadata)]
#[model(
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
    #[model(reference(
        target = Organization,
        target_field = id,
        same_as = missing
    ))]
    organization_id: i64,
}

fn main() {}
