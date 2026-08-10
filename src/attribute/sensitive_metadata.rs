// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::SensitiveHandling;

/// Metadata describing how a sensitive value should be handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SensitiveMetadata {
    /// The policy applied to the sensitive value.
    handling: SensitiveHandling,
}

impl SensitiveMetadata {
    /// Creates sensitive-data metadata with the supplied handling policy.
    ///
    /// # Parameters
    ///
    /// * `handling` - The policy applied to the sensitive value.
    ///
    /// # Returns
    ///
    /// Sensitive-data metadata containing the supplied handling policy.
    #[must_use]
    #[inline(always)]
    pub const fn new(handling: SensitiveHandling) -> Self {
        Self { handling }
    }

    /// Returns the handling policy for this sensitive value.
    ///
    /// # Returns
    ///
    /// The policy applied to the sensitive value.
    #[must_use]
    #[inline(always)]
    pub const fn handling(self) -> SensitiveHandling {
        self.handling
    }
}
