// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private execution state for validation plans.

// Stores cached selection state for each planned occurrence.
pub(super) mod selection_entry;

// Tracks selection state across the ordered execution plan.
pub(super) mod selection_mask;
