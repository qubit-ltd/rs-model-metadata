// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One standard rule mapping shared by binding and diagnostics.

use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidatorId;

use super::StandardTarget;

/// A constraint mapping visited synchronously while its arguments are alive.
pub(super) enum StandardRule<'arguments> {
    /// A supported rule with its adapter input and configuration.
    Executable {
        /// Stable rule identity, independent of registry lookup.
        id: ValidatorId,
        /// Projection used to read the validator input.
        target: StandardTarget,
        /// Borrowed argument list; all argument values reference static data.
        args: &'arguments [NamedValidationArgument<'static>],
    },
    /// A declaration whose erased execution adapter is not implemented.
    Unsupported {
        /// Known rule identity, or None when no backend mapping exists.
        id: Option<ValidatorId>,
    },
}
