// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Rejects equal decimal bounds when either endpoint is exclusive.
// The filename is part of a trybuild fixture protocol.

use qubit_model_derive::Model;

#[Model]
struct EqualDecimalBounds {
    #[decimal(min = "1.0", max = "1", min_inclusive = false)]
    lower_exclusive: String,
    #[decimal(min = "1", max = "1.0", max_inclusive = false)]
    upper_exclusive: String,
}

fn main() {}
