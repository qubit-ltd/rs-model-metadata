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
struct NegativeBoolean {
    #[validator(id = "example.parameter", params(value = -true))]
    value: String,
}

#[Model]
struct FloatingPoint {
    #[validator(id = "example.parameter", params(value = 1.5))]
    value: String,
}

#[Model]
struct MixedUnsignedList {
    #[validator(id = "example.parameter", params(values = [1, "two"]))]
    value: String,
}

#[Model]
struct MixedStringList {
    #[validator(id = "example.parameter", params(values = ["one", 2]))]
    value: String,
}

#[Model]
struct MixedSignedList {
    #[validator(id = "example.parameter", params(values = [-1, "two"]))]
    value: String,
}

fn main() {}
