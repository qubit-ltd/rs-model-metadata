// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized relationship metadata.

use super::ReferenceTargetIr;

/// Normalized relationship target and property-mapping requirements.
#[derive(Clone)]
pub(crate) struct ReferenceIr {
    /// Referenced entity target.
    pub(crate) target: ReferenceTargetIr,
    /// Selected target property path.
    pub(crate) property: Option<Vec<String>>,
    /// Whether the target must already exist.
    pub(crate) existing: bool,
    /// Equivalent local property path.
    pub(crate) same_as: Option<Vec<String>>,
}
