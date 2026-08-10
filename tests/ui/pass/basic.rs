// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_derive::ModelMetadata;

type TextAlias = String;

#[derive(ModelMetadata)]
#[model(id = "test.derive.Valid")]
struct Valid {
    #[model(unique(ignore_case), text(min_chars = 1, max_chars = 8))]
    value: TextAlias,
}

fn main() {}
