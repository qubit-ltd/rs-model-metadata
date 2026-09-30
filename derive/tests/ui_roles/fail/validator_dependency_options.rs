// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
// qubit-style: allow test-file-name
//! Trybuild compile-fail fixture: rejects repeated dependency paths/properties, unknown options, and duplicate dependency targets.

use qubit_model_derive::Model;

#[Model]
struct RepeatedNavigation {
    #[validator(id = "example.dependency", depends_on(owner(path = "..", path = "..", property = name)))]
    value: String,
}

#[Model]
struct RepeatedProperty {
    #[validator(id = "example.dependency", depends_on(owner(property = name, property = title)))]
    value: String,
}

#[Model]
struct UnknownDependencyOption {
    #[validator(id = "example.dependency", depends_on(owner(optional = true, property = name)))]
    value: String,
}

#[Model]
struct RepeatedDependencyTarget {
    #[validator(id = "example.dependency", depends_on(first(property = name), second(property = name)))]
    value: String,
}

fn main() {}
