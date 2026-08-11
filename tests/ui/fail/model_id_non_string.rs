// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================


#[qubit_model_derive::Model(id = 42)]
struct NonString {
    value: String,
}

fn main() {}
