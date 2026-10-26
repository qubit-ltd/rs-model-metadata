// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Initial node-set selection for a structure resolution attempt.

/// Selects the initial set of models processed by a resolution attempt.
#[derive(Clone, Copy)]
pub(in crate::resolve) enum ResolveScope {
    /// Includes every concrete model registered in the configured registry.
    AllRegistered,
    /// Includes only explicit roots and models reachable from those roots.
    ReachableFromRoots,
}
