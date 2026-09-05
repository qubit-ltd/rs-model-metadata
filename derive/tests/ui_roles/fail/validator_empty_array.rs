// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects validator arrays whose element type cannot be inferred.

use qubit_model_derive::Model;

#[Model]
struct EmptyValidatorArray {
    #[validator(id = "trybuild.values", params(values = []))]
    value: String,
}

fn main() {}
