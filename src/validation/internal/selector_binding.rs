// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compiled collection selector.

use crate::metadata::SelectorPosition;

/// A validator bound to a sequence selector.
#[derive(Clone, Debug)]
pub(crate) struct SelectorBinding {
    /// Collection position validated by this binding.
    pub(crate) position: SelectorPosition,
}

impl SelectorBinding {
    /// Returns the selected collection position.
    pub(crate) const fn position(&self) -> SelectorPosition {
        self.position
    }
}
