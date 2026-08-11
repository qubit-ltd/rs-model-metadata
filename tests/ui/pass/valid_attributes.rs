// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]


#[qubit_model_derive::Model(id = "test.derive.Organization", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Organization {
    #[field(identifier)]
    id: i64,
}

#[qubit_model_derive::Model(id = "test.derive.ValidAttributes", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct ValidAttributes {
    #[field(identifier(generated))]
    id: Option<i64>,
    #[field(unique(ignore_case), index, text(max_chars = 32))]
    username: String,
    #[field(time(precision = millisecond, normalization = utc))]
    created_at: chrono::DateTime<chrono::Utc>,
    #[field(decimal(precision = 8, scale = 3))]
    ratio: bigdecimal::BigDecimal,
    #[field(money(precision = 12, scale = 2))]
    balance: bigdecimal::BigDecimal,
    #[field(reference(target = "test.derive.Organization", target_field = id))]
    organization_id: i64,
    #[field(lookup_relation(target = Organization, target_field = id))]
    organization_lookup: i64,
}

fn main() {}
