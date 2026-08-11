// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================


#[qubit_model_derive::Model(id = "test.derive.OtherModel", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct TypeNameMismatch {
    value: String,
}

fn main() {}
