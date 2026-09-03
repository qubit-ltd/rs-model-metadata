// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use qubit_model_derive::Model;
use qubit_model_derive::Value;

#[Model]
struct RedundantMarker {
    #[keep_serializing]
    value: u64,
}

#[Value]
struct PositionalMarker(#[keep_serializing] Option<u64>);

fn main() {}
