// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use std::collections::HashSet;

type SetAlias = HashSet<String>;

#[derive(qubit_model_derive::ModelMetadata)]
struct Invalid {
    #[model(sequence(unique_items))]
    values: SetAlias,
}

fn main() {}
