// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

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
