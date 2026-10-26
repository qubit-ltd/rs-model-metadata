// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized uniqueness metadata.

/// Normalized uniqueness scope and comparison options for one field.
#[derive(Clone)]
pub(crate) struct UniqueIr {
    /// Property paths scoping uniqueness.
    pub(crate) respect_to: Vec<Vec<String>>,
    /// Whether textual comparison ignores case.
    pub(crate) ignore_case: Option<bool>,
}
