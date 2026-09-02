// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use qubit_model_derive::Enum;

#[Enum(id = "trybuild.VariantNames")]
enum VariantNames {
    #[variant(name = "DUPLICATE")]
    First,
    #[variant(name = "DUPLICATE")]
    Second,
}

fn main() {}
