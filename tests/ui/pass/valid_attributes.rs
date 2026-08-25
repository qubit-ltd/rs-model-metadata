// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[qubit_model_derive::Model(
    id = "test.derive.Organization",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
struct Organization {
    #[identifier]
    id: i64,
}

#[qubit_model_derive::Model(
    id = "test.derive.ValidAttributes",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize
)]
struct ValidAttributes {
    #[identifier(generated)]
    id: Option<i64>,
    #[unique(respectTo = [organization_id], ignoreCase = true)]
    #[text(max_chars = 32)]
    username: String,
    #[time(precision = millisecond)]
    created_at: chrono::DateTime<chrono::Utc>,
    #[decimal(precision = 8, scale = 3)]
    ratio: bigdecimal::BigDecimal,
    #[money(precision = 12, scale = 2)]
    balance: bigdecimal::BigDecimal,
    #[reference(entity = "test.derive.Organization", property = id)]
    organization_id: i64,
    #[lookup_relation(target = Organization, target_field = id)]
    organization_lookup: i64,
}

fn main() {}
