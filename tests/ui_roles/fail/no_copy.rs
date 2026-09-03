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

#[Enum(no_copy)]
enum Status {
    Ready,
    Failed,
}

fn requires_copy<T: Copy>() {}

fn main() {
    requires_copy::<Status>();
}
