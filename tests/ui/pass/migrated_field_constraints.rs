// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[derive(qubit_model_derive::ModelMetadata)]
#[model(id = "test.derive.Target")]
struct Target {
    id: i64,
}

#[derive(qubit_model_derive::ModelMetadata)]
#[model(id = "test.derive.MigratedConstraints")]
struct MigratedConstraints {
    target: Target,
    #[model(reference(target = "test.derive.Target", target_field = id, same_as = "target.id"))]
    target_id: i64,
    #[model(element(text(repertoire = ascii)))]
    codes: [String; 2],
    #[model(element(decimal(scale = 2)))]
    values: Vec<bigdecimal::BigDecimal>,
    #[model(text(format = mobile))]
    mobile: String,
    #[model(sensitive(token))]
    verification_code: String,
}

fn main() {}

#[derive(
    qubit_model_derive::ModelMetadata,
    qubit_redact_derive::Redact,
)]
#[model(id = "test.derive.SensitiveMigration")]
struct SensitiveMigration {
    #[model(sensitive(token))]
    #[redact(level = "secret")]
    verification_code: String,
}
