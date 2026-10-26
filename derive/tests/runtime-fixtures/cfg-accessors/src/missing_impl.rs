// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Fixture for an impl-level condition that removes an unavailable target.

use qubit_model_derive::ModelImpl;

#[ModelImpl]
#[cfg(any())]
impl MissingImplTarget {
    /// No signature or provider reference survives the impl condition.
    pub fn missing_impl(&self) -> MissingImplReturn { unreachable!() }
}
