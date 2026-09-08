// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Explicit validation selector execution capabilities.

use crate::metadata::SelectorPosition;

/// Capabilities supported by the erased validation executor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ValidationCapabilities;

impl ValidationCapabilities {
    /// Returns whether the executor supports `position`.
    #[must_use]
    pub const fn supports(self, position: SelectorPosition) -> bool {
        matches!(position, SelectorPosition::Element)
    }
}
