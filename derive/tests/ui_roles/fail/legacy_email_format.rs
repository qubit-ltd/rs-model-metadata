// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo or trybuild fixture protocol.
//! Rejects the legacy unquoted email text format in model metadata.

use qubit_model_derive::Model;

#[Model]
struct LegacyEmailFormat {
    #[text(format = email)]
    address: String,
}

fn main() {}
