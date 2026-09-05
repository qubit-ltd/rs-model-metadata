// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects repeated singleton field markers at their second occurrence.

use qubit_model_derive::Model;

#[Model]
struct DuplicateMarkers {
    #[identifier]
    #[identifier]
    identifier: String,
    #[identifier(assigned_by = application, assigned_by = database)]
    assignment: String,
    #[indexed]
    #[indexed]
    indexed: String,
    #[opaque]
    #[opaque]
    opaque: String,
}

fn main() {}
