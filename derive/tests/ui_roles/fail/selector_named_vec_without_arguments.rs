// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Trybuild compile-fail fixture: a named `Vec` selector must supply the required type arguments.

use qubit_model_derive::Model;
use qubit_reflect::Reflect;
use serde::Deserialize;
use serde::Serialize;

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Hash,
    PartialEq,
    Reflect,
    Deserialize,
    Serialize,
)]
#[reflect(crate = qubit_reflect)]
struct Vec;

#[Model]
struct InvalidSelectorContainer {
    #[element(text(max_chars = 8))]
    value: Vec,
}

fn main() {}
