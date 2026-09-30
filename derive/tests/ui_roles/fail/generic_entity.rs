// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.
//! Compile-fail case for the unsupported generic parameter on an entity.

use qubit_model_derive::Entity;

#[Entity(id = "example.GenericEntity")]
struct GenericEntity<T> {
    id: T,
}

fn main() {}
