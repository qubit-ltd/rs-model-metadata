// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

use qubit_model_derive::ModelMetadata;

#[derive(ModelMetadata)]
struct Organization {
    #[model(identifier)]
    id: i64,
}

#[derive(ModelMetadata)]
struct ValidAttributes {
    #[model(identifier(generated))]
    id: Option<i64>,
    #[model(unique(ignore_case), index, text(max_chars = 32))]
    username: String,
    #[model(time(precision = millisecond, normalization = utc))]
    created_at: chrono::DateTime<chrono::Utc>,
    #[model(decimal(precision = 8, scale = 3))]
    ratio: bigdecimal::BigDecimal,
    #[model(money(precision = 12, scale = 2))]
    balance: bigdecimal::BigDecimal,
    #[model(reference(target = Organization, target_field = id))]
    organization_id: i64,
}

fn main() {}
