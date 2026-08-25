// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[qubit_model_derive::Model(
    id = "test.derive.Phone",
    textual,
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize,
)]
struct Phone {
    country_area: Option<String>,
    city_area: Option<String>,
    number: String,
}

#[qubit_model_derive::Model(
    id = "test.derive.LoginParams",
    no_clone,
    no_debug,
    no_display,
    no_partial_eq,
    no_hash,
    no_serialize,
    no_deserialize,
)]
struct LoginParams {
    #[text(format = mobile)]
    mobile: Option<Phone>,
}

fn main() {}
