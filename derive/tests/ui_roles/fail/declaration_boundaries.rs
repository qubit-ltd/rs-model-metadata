// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

//! Rejects unions, missing entity IDs, and missing role identifiers.

use model_runtime::__private::qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;

#[Model]
union UnsupportedUnion {
    value: u64,
}

#[Entity]
struct MissingEntityId {
    #[identifier]
    id: Id,
}

#[Projection]
struct MissingProjectionIdentifier {
    value: String,
}

fn main() {}

const _: Option<Id> = None;
