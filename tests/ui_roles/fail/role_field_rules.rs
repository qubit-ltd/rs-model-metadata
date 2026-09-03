// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;

#[Model]
struct InvalidModelIdentifier {
    #[identifier]
    id: u64,
}

#[Value]
struct InvalidValueReference {
    #[reference(entity_id = "example.Entity")]
    entity: u64,
}

#[Enum]
enum InvalidEnumReference {
    Data {
        #[reference(entity_id = "example.Entity")]
        entity: u64,
    },
}

fn main() {}
