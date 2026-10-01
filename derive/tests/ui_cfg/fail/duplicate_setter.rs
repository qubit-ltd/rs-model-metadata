// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// Trybuild diagnostic fixture filenames describe the tested contract.

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
#[Model]
struct Conditional { value: u32 }
#[ModelImpl]
impl Conditional {
    #[cfg(all())]
    pub fn set_title(&mut self, value: u32) { self.value = value; }
    #[cfg_attr(any(), cfg(any()))]
    pub fn set_title(&mut self, value: u32) { self.value = value; }
}
fn main() {}
