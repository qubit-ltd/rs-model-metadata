// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Machine-readable validation plan construction failures.

use qubit_validator::BindErrorKind;

/// Machine-readable validation plan construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationBuildErrorKind {
    /// An executable validator could not be bound.
    ValidatorBinding(
        /// The validator registry's original binding failure category.
        BindErrorKind,
    ),
    /// A declaration or access shape cannot be executed by this backend.
    UnsupportedExecution,
    /// The explicitly requested root was not included in the supplied graph.
    RootNotInGraph,
}
