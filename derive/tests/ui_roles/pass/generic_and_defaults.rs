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

#[Entity(id = "trybuild.Source")]
struct Source {
    #[identifier]
    id: Id,
}

#[Projection(source = Source)]
struct View {
    #[identifier]
    id: Id,
}

#[Model(no_serialize, no_deserialize)]
struct Buffer<const N: usize> {
    bytes: [u8; N],
}

#[Value(no_redact, transparent)]
struct Revision(u64);

#[Enum(no_copy)]
enum Status {
    Ready,
    Failed,
}

#[Model(no_redact)]
#[derive(serde::Serialize)]
struct ExistingSafeSerialize {
    value: String,
}

fn main() {
    let _ = Buffer::<4> { bytes: [0; 4] };
    let _ = Revision(1).to_string();
    let _ = Status::Ready;
    let _ = View { id: Id::new(1) };
    let _ = ExistingSafeSerialize {
        value: String::new(),
    };
}
