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
struct MissingTimePrecision {
    #[time()]
    value: String,
}

#[Model]
struct ExcessDecimalScale {
    #[decimal(precision = 2, scale = 3)]
    value: String,
}

fn main() {}
