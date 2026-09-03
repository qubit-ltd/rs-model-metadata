// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
// The filename is part of a Cargo or trybuild fixture protocol.

use model_runtime::__private::qubit_id::Id;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Projection;
use qubit_model_derive::Value;

#[Entity(id = "trybuild.Target")]
struct Target {
    #[identifier]
    id: Id,
}

#[Projection(id = "trybuild.Projection")]
struct InvalidProjection {
    #[identifier(assigned_by = database)]
    id: Id,
    #[key_part(order = 0)]
    value: String,
}

#[Entity(id = "trybuild.Entity")]
struct InvalidEntity {
    #[identifier]
    id: Id,
    #[key_part(order = 0)]
    value: String,
}

#[Enum]
enum InvalidEnum {
    Named {
        #[key_part(order = 0)]
        value: String,
    },
}

#[Value]
struct PositionalValue(#[key_part(order = 0)] String);

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
