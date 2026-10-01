// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Trybuild compile-fail fixture: rejects invalid qualified keys in validator `depends_on` and `params` declarations.

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
