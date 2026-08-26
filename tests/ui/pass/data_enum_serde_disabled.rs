// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[qubit_model_derive::Enum(id = "test.derive.SerdeDisabled", no_serialize, no_deserialize)]
enum SerdeDisabled {
    Value(#[serde(default)] Option<String>),
}

fn main() {}
