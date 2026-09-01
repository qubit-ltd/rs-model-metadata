// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_model_derive::Model;

#[Model]
struct InvalidValidatorArgument {
    #[validator(
        id = "example.validator",
        params(values = [340282366920938463463374607431768211456])
    )]
    value: u64,
}

fn main() {}
