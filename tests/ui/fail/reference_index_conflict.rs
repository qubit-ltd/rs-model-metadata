// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[qubit_model_derive::Model(
    id = "test.derive.InvalidReferenceIndex",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
struct InvalidReferenceIndex {
    #[field(reference(entity = "test.derive.Target", property = id), index)]
    target_id: i64,
}

#[qubit_model_derive::Model(
    id = "test.derive.Target",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
struct Target {
    id: i64,
}

fn main() {}
