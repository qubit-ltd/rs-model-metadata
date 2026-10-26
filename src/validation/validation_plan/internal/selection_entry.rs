// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Per-occurrence selection and path state.

use qubit_validator::ValidationPath;

/// Selection state associated with one model or field rule occurrence.
pub(in crate::validation::validation_plan) struct SelectionEntry {
    /// Whether the caller selected this occurrence.
    pub(super) selected: bool,
    /// Whether a later occurrence is selected, for report stop-policy
    /// decisions.
    pub(super) later_selected: bool,
    /// Cached field path, consumed when the occurrence is executed.
    pub(super) field_path: Option<ValidationPath>,
}
