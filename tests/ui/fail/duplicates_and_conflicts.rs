// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[derive(qubit_model_derive::ModelMetadata)]
#[model(
    id = "test.derive.Invalid",
    primary_key(fields(id)),
    primary_key(fields(id)),
    ownership(owner = Invalid),
    ownership(owner = Invalid)
)]
struct Invalid {
    id: i64,
    #[model(
        text(min_chars = 1, min_chars = 2, non_blank, non_blank),
        text(max_chars = 8)
    )]
    name: String,
    #[model(decimal(scale = 2), money(scale = 2))]
    amount: bigdecimal::BigDecimal,
    #[model(opaque, opaque)]
    external: External,
}

struct External;

fn main() {}
