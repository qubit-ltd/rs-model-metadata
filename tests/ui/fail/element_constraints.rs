// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[derive(qubit_model_derive::ModelMetadata)]
struct Invalid {
    #[model(element(text(repertoire = ascii)))]
    scalar: String,
    #[model(element(decimal(scale = 2)))]
    strings: Vec<String>,
    #[model(element(text(repertoire = ascii)))]
    set: std::collections::HashSet<String>,
}

fn main() {}
