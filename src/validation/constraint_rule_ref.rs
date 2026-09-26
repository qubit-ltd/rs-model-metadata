// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Rule identities reported for standard validation constraints.

use qubit_validator::ValidatorId;

/// Identifies how a standard constraint's rule is executed.
///
/// A registry rule can be bound only when a matching registration, input type,
/// and argument list are available. A model-intrinsic rule is executed by the
/// metadata validation plan and has no registry binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConstraintRuleRef {
    /// A rule selected through a validator registry.
    Registry(ValidatorId),
    /// A rule implemented directly by the metadata validation plan.
    ModelIntrinsic(ValidatorId),
}

impl ConstraintRuleRef {
    /// Returns the stable rule ID for diagnostics and violations.
    #[must_use]
    pub const fn id(self) -> ValidatorId {
        match self {
            Self::Registry(id) | Self::ModelIntrinsic(id) => id,
        }
    }

    /// Returns an ID eligible for registry binding, or `None` for an intrinsic
    /// rule. A returned ID alone does not guarantee that binding succeeds.
    #[must_use]
    pub const fn registry_id(self) -> Option<ValidatorId> {
        match self {
            Self::Registry(id) => Some(id),
            Self::ModelIntrinsic(_) => None,
        }
    }
}
