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

#[Model]
struct QualifiedDependency {
    #[validator(id = "example.binding", depends_on(owner::name))]
    value: String,
}

#[Model]
struct MissingDependencyProperty {
    #[validator(id = "example.binding", depends_on(owner(path = "..")))]
    value: String,
}

#[Model]
struct QualifiedParameter {
    #[validator(id = "example.binding", params(options::limit = 1))]
    value: String,
}

fn main() {}
