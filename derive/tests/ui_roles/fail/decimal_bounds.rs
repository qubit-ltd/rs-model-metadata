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
struct InvalidDecimalBounds {
    #[decimal(min = "10.0", max = "2.0")]
    reversed: String,
    #[decimal(min = "1", max = "1", min_inclusive = false, max_inclusive = false)]
    empty: String,
    #[decimal(min = "not-a-number")]
    malformed: String,
}

fn main() {}
