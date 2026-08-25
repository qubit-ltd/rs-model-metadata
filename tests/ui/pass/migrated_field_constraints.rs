// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[qubit_model_derive::Model(id = "test.derive.Target", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct Target {
    id: i64,
}

#[qubit_model_derive::Model(id = "test.derive.MigratedConstraints", no_clone, no_debug, no_display, no_partial_eq, no_hash, no_serialize, no_deserialize)]
struct MigratedConstraints {
    target: Target,
    #[field(reference(entity = "test.derive.Target", property = id, path = "target.id"))]
    target_id: i64,
    #[field(element(text(repertoire = ascii)))]
    codes: [String; 2],
    #[field(element(decimal(scale = 2)))]
    values: Vec<bigdecimal::BigDecimal>,
    #[field(text(format = mobile))]
    mobile: String,
    verification_code: String,
}

fn main() {}

#[qubit_model_derive::Model(
    id = "test.derive.SensitiveMigration",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize,
    redact,
)]
struct SensitiveMigration {
    #[redact(level = "secret")]
    verification_code: String,
}
