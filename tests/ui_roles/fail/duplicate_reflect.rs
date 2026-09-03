// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use model_runtime::Reflect;
use qubit_model_derive::Model;

#[Model]
#[derive(Reflect)]
#[reflect(crate = model_runtime)]
struct DuplicateReflect;

fn main() {}
