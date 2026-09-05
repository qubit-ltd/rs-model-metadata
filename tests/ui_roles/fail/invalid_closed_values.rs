// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects unknown values from closed constraint and redaction vocabularies.

use qubit_model_derive::Model;

#[Model]
struct InvalidClosedValues {
    #[text(allowed_chars = unsupported)]
    allowed_chars: String,
    #[text(format = unsupported)]
    format: String,
    #[decimal(rounding = unsupported)]
    rounding: i64,
    #[time(precision = unsupported)]
    precision: String,
    #[redact(level = "unsupported")]
    redact: String,
}

fn main() {}
