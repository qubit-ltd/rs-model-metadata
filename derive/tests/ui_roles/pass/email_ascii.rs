// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Trybuild pass fixture: accepts the `email_ascii` text format and verifies its metadata representation.

use model_runtime::metadata::TextFormat;
use model_runtime::metadata::TypeMetadata;
use qubit_model_derive::Model;

#[Model]
struct EmailAsciiModel {
    #[text(format = email_ascii)]
    address: String,
}

fn main() {
    let field = TypeMetadata::of::<EmailAsciiModel>()
        .field("address")
        .expect("address field");
    assert_eq!(
        field.text_constraint().expect("text constraint").format(),
        Some(TextFormat::EmailAscii)
    );
}
