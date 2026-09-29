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
    #[cfg(any())]
    pub fn title(&self) -> MissingGetter { unreachable!() }
    #[cfg(all())]
    pub fn title(&self) -> u32 { self.value }
    #[cfg(any())]
    pub fn set_title(&mut self, _: MissingSetter) { unreachable!() }
    #[cfg(all())]
    pub fn set_title(&mut self, value: u32) { self.value = value; }
    #[cfg_attr(all(), cfg_attr(all(), cfg(any())), inline)]
    pub fn nested(&self) -> MissingNested { unreachable!() }
}
fn main() {}
