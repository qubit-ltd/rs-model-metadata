// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[derive(Clone)]
struct NotDebug;

#[qubit_model_derive::Enum(id = "test.derive.DisplayData", no_debug, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum DisplayData {
    Value(#[opaque] NotDebug),
}

fn main() {}
