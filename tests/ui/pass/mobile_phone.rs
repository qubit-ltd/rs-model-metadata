// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(dead_code)]

#[derive(qubit_model_derive::Model)]
struct Phone(String);

#[derive(qubit_model_derive::Model)]
struct LoginParams {
    #[model(text(format = mobile))]
    mobile: Option<Phone>,
}

fn main() {}
