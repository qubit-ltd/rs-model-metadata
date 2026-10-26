// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Prepared executable standard constraint binding.

use qubit_validator::BoundValidator;

use super::standard_target::StandardTarget;

/// Pairs a prepared standard validator with the projection used to produce its
/// input.
pub(crate) struct StandardBinding {
    /// Prepared validator invoked for this constraint occurrence.
    pub(crate) validator: BoundValidator,
    /// Projection used to derive the value passed to the validator.
    pub(crate) target: StandardTarget,
}
