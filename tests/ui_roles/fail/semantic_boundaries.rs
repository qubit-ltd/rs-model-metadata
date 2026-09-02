// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

use model_runtime::__private::qubit_id::Id;
use qubit_model_derive::{Entity, Model, Projection, Value};

#[Entity(id = "trybuild.Target")]
struct Target {
    #[identifier]
    id: Id,
}

#[Projection(id = "trybuild.Projection")]
struct InvalidProjection {
    #[identifier(assigned_by = database)]
    id: Id,
}

#[Entity(id = "trybuild.Entity")]
struct InvalidEntity {
    #[identifier]
    id: Id,
    #[key_part(order = 0)]
    value: String,
}

#[Model]
struct InvalidUnique {
    #[unique(ignore_case = true)]
    value: u64,
}

#[Value]
struct InvalidText {
    #[text(min_chars = 3, max_chars = 2, min_bytes = 4, max_bytes = 1)]
    value: String,
}

fn main() {}
