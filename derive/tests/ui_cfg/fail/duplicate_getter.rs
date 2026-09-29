// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow test-file-name
// Trybuild diagnostic fixture filenames describe the tested contract.

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
#[Model]
struct Conditional { value: u32 }
#[ModelImpl]
impl Conditional {
    #[cfg(all())]
    pub fn title(&self) -> u32 { self.value }
    #[cfg_attr(any(), cfg(any()))]
    pub fn title(&self) -> u32 { self.value }
}
fn main() {}
