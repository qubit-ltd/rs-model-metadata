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

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Hash,
    PartialEq,
    model_runtime::Reflect,
    serde::Deserialize,
    serde::Serialize,
)]
#[reflect(crate = model_runtime)]
struct Vec;

impl model_runtime::__private::serde_helpers::IsEmpty for Vec {
    fn is_empty(&self) -> bool {
        true
    }
}

#[Model]
struct InvalidSelectorContainer {
    #[element(text(max_chars = 8))]
    value: Vec,
}

fn main() {}
