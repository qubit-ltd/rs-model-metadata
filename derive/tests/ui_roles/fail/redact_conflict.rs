// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
//! Trybuild compile-fail fixture: `no_redact` conflicts with a field-level redaction declaration; this filename is part of the fixture protocol.

use qubit_model_derive::Model;

#[Model(no_redact)]
struct Secret {
    #[redact(level = "confidential")]
    value: String,
}

fn main() {}
