// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_model_derive::Model;

#[Model(no_eq, no_hash)]
struct InvalidConstraintTargets {
    #[text(min_chars = 1)]
    number: u64,
    #[decimal(scale = 2)]
    decimal: f64,
    #[sequence(unique_items)]
    scalar: String,
    #[sequence(unique_items)]
    set: std::collections::HashSet<u64>,
    #[map(min_entries = 1)]
    not_a_map: Vec<u64>,
}

fn main() {}
