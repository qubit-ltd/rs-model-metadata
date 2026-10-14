// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compiled collection selector.

use crate::metadata::SelectorPosition;
use crate::property::CollectionItemsAdapter;

/// A validator bound to a sequence selector.
#[derive(Clone, Debug)]
pub(crate) struct SelectorBinding {
    /// Collection position validated by this binding.
    pub(crate) position: SelectorPosition,
    /// Exact-type iterator over the selected members.
    pub(crate) items: CollectionItemsAdapter,
}

impl SelectorBinding {
    /// Returns the selected collection position.
    ///
    /// # Returns
    /// The sequence or map position validated by this binding.
    #[inline]
    #[must_use]
    pub(crate) const fn position(&self) -> SelectorPosition {
        self.position
    }

    /// Returns the generated member iterator adapter.
    #[inline]
    #[must_use]
    pub(crate) const fn items(&self) -> CollectionItemsAdapter {
        self.items
    }
}
