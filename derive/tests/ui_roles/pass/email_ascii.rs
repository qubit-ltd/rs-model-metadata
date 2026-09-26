// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

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
