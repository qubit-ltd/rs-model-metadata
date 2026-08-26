// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[qubit_model_derive::Enum(id = "test.derive.CopyData", no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
enum CopyData {
    Value(i64),
}

fn assert_copy<T: Copy>() {}

fn main() {
    assert_copy::<CopyData>();
}
