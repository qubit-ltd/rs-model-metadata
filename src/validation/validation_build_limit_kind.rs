// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Categories of fixed validation-plan build limits.

/// The category of a fixed validation-plan build limit.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::validation::ValidationBuildLimitKind;
///
/// let category = ValidationBuildLimitKind::Work;
/// assert_eq!(category, ValidationBuildLimitKind::Work);
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationBuildLimitKind {
    /// Maximum nested model or expression depth.
    Depth,
    /// Maximum discovery and binding work units.
    Work,
    /// Maximum declaration occurrences.
    Occurrences,
    /// Maximum emitted validation bindings.
    Bindings,
    /// Maximum ordinary diagnostics retained before a terminal error.
    Diagnostics,
}
