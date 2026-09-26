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
use crate::validation::ConstraintRuleRef;

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
    /// Sequence equality is executed by the metadata adapter, outside the
    /// registry.
    SequenceUnique {
        /// Stable rule identity.
        id: ValidatorId,
    },
}

impl StandardRule<'_> {
    /// Returns the execution category and stable ID for diagnostics.
    pub(super) const fn diagnostic_ref(&self) -> ConstraintRuleRef {
        match self {
            Self::Executable { id, .. } => ConstraintRuleRef::Registry(*id),
            Self::SequenceUnique { id } => ConstraintRuleRef::ModelIntrinsic(*id),
        }
    }
}
