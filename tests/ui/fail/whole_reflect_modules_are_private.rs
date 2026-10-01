// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::capability::clone_key;
use qubit_model_metadata::registry::ReflectRegistry;

fn main() {
    let _ = clone_key();
    let _ = ReflectRegistry::initialize();
}
