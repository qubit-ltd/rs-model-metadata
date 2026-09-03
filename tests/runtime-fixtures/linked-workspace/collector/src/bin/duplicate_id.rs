// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use core::mem::size_of;

use model_a::Duplicate as ModelADuplicate;
use model_b::Duplicate as ModelBDuplicate;
use qubit_model_metadata::ModelRegistry;

fn main() {
    let _ = size_of::<ModelADuplicate>();
    let _ = size_of::<ModelBDuplicate>();
    let error = ModelRegistry::try_global()
        .expect_err("duplicate IDs must make the global registry invalid");
    assert!(error.to_string().contains("test.linked.Duplicate"));
}
