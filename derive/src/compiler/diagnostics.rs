// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Aggregates independent macro diagnostics before emitting compiler errors.

use syn::Error;
use syn::Result;

/// Accumulates recoverable declaration diagnostics.
#[derive(Default)]
pub(crate) struct Diagnostics(
    /// Holds the combined diagnostics, or `None` until the first error arrives.
    Option<Error>,
);

impl Diagnostics {
    /// Adds one diagnostic without discarding previously collected errors.
    pub(crate) fn push(&mut self, error: Error) {
        if let Some(existing) = &mut self.0 {
            existing.combine(error);
        } else {
            self.0 = Some(error);
        }
    }

    /// Returns all accumulated diagnostics, if any.
    #[inline]
    pub(crate) fn finish(self) -> Result<()> {
        self.0.map_or(Ok(()), Err)
    }
}
