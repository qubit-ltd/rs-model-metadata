// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects repeated constraint and validator options at their second span.

use qubit_model_derive::Model;

#[Model]
struct AdvancedDuplicateOptions {
    #[sequence(min_items = 1, min_items = 2)]
    values: Vec<String>,
    #[map(min_entries = 1, min_entries = 2)]
    labels: std::collections::HashMap<String, String>,
    #[time(precision = second, precision = millisecond)]
    timestamp: String,
    #[validator(id = "trybuild.value", params(limit = 1, limit = 2))]
    value: String,
    #[validator(id = "trybuild.dependency", depends_on(value, value))]
    dependency: String,
}

fn main() {}
