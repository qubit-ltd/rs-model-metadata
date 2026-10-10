// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Trybuild compile-fail fixture: derived models reject borrowed field lifetimes;
//! this filename is part of the fixture protocol.

use qubit_model_derive::Model;

#[Model]
struct Borrowed<'a> {
    value: &'a str,
}

fn main() {}
