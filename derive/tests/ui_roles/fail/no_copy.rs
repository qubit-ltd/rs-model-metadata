// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Trybuild compile-fail fixture: `no_copy` enum output must not satisfy a `Copy` bound.
//! The filename is part of the fixture protocol.

use qubit_model_derive::Enum;

#[Enum(no_copy)]
enum Status {
    Ready,
    Failed,
}

fn requires_copy<T: Copy>() {}

fn main() {
    requires_copy::<Status>();
}
