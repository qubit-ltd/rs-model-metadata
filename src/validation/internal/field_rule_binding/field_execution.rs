// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Selected execution strategy for a compiled field rule.

use qubit_validator::BoundValidator;

use crate::property::ItemEqAdapter;
use crate::property::MapLenAdapter;
use crate::validation::standard_constraints::StandardTarget;

/// Registry-backed validation or metadata-owned sequence equality.
#[derive(Clone, Debug)]
pub(crate) enum FieldExecution {
    /// A prepared registered rule and its checked input projection.
    Registry {
        /// Prepared rule.
        validator: BoundValidator,
        /// Standard target, if this is a built-in constraint.
        target: Option<StandardTarget>,
        /// Checked map count adapter, when needed.
        map_len: Option<MapLenAdapter>,
    },
    /// Exact element comparison adapter for one sequence declaration.
    SequenceUnique {
        /// Typed equality operation generated for the field.
        item_eq: ItemEqAdapter,
    },
}
