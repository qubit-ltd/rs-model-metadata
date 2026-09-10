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
struct ScalarUnderflow {
    #[validator(id = "example.bounds", params(value = -170141183460469231731687303715884105729))]
    value: String,
}

#[Model]
struct ListUnderflow {
    #[validator(id = "example.bounds", params(values = [-170141183460469231731687303715884105729, 0]))]
    value: String,
}

#[Model]
struct SignedListOverflow {
    #[validator(id = "example.bounds", params(values = [-1, 170141183460469231731687303715884105728]))]
    value: String,
}

fn main() {}
