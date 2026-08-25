// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Disabled default capabilities after dependency normalization.
#[derive(Default)]
pub(crate) struct DisabledCapabilities {
    /// Disables `Clone`.
    pub(crate) clone: bool,
    /// Disables `Copy`.
    pub(crate) copy: bool,
    /// Disables `Debug`.
    pub(crate) debug: bool,
    /// Disables `Display`.
    pub(crate) display: bool,
    /// Disables `Eq`.
    pub(crate) eq: bool,
    /// Disables `PartialEq`.
    pub(crate) partial_eq: bool,
    /// Disables `PartialOrd`.
    pub(crate) partial_ord: bool,
    /// Disables `Ord`.
    pub(crate) ord: bool,
    /// Disables `Hash`.
    pub(crate) hash: bool,
    /// Disables `Serialize`.
    pub(crate) serialize: bool,
    /// Disables `Deserialize`.
    pub(crate) deserialize: bool,
}

impl DisabledCapabilities {
    /// Applies the trait dependency rules defined by the public macro API.
    pub(super) fn normalize(&mut self) {
        if self.partial_eq {
            self.eq = true;
            self.partial_ord = true;
            self.ord = true;
        }
        if self.eq || self.partial_ord {
            self.ord = true;
        }
    }
}
