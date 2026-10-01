// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// The filename is part of a Cargo or trybuild fixture protocol.
//! Compile-fail fixture ensuring identifier fields use supported types.

use qubit_model_derive::Entity;

#[Entity(id = "example.InvalidIdentifier")]
struct InvalidIdentifier {
    #[identifier]
    id: u64,
}

fn main() {}
