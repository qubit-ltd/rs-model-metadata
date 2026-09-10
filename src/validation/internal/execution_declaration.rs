// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Original executable declaration retained during compilation.

use crate::metadata::ConstraintMetadata;
use crate::metadata::ValidatorMetadata;

/// A standard constraint or user validator at one source location.
#[derive(Clone, Copy, Debug)]
pub(crate) enum ExecutionDeclaration {
    /// A structural edge whose traversal cannot be executed.
    Traversal,
    /// A standard constraint, possibly mapping to several built-in rules.
    Constraint(&'static ConstraintMetadata),
    /// A named user validator with arguments and dependencies.
    Validator(&'static ValidatorMetadata),
}
