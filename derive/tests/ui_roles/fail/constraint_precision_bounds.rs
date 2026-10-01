// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects model declarations with missing temporal precision or a decimal scale
//! greater than its declared precision.

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
