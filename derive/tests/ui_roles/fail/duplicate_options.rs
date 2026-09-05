// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects repeated declaration and field option values at their second span.

use model_runtime::__private::qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Model;

#[Entity(id = "trybuild.DuplicateTarget")]
struct DuplicateTarget {
    #[identifier]
    id: Id,
}

#[Model(no_hash, no_hash)]
struct DuplicateOptions {
    #[unique(ignore_case = true, ignore_case = false)]
    #[text(min_chars = 1, min_chars = 2)]
    name: String,
    #[decimal(precision = 8, precision = 9)]
    amount: i64,
    #[reference(entity = "trybuild.DuplicateTarget", existing = true, existing = false)]
    owner: Id,
    #[serde(rename = "first", rename = "second")]
    alias: String,
}

fn main() {}
