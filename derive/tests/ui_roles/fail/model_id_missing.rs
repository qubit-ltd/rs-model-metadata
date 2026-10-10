// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use model_runtime::metadata::HasModelId;
use qubit_model_derive::Model;

#[Model]
struct Anonymous {
    value: u8,
}

#[Model(id = "fixture.Generic")]
struct Generic<T> {
    value: T,
}

fn requires_model_id<T: HasModelId>() {}

fn main() {
    requires_model_id::<Anonymous>();
    requires_model_id::<Generic<u8>>();
}
