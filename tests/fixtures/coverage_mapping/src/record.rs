// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Called and uncalled methods distinguish execution from mapping emission.

/// A value observed by two binaries with different sets of called methods.
pub struct Record {
    /// The input retained by both ordinary and inlined reads.
    pub value: u64,
}

impl Record {
    /// This inline read is called only by the later test executable.
    #[must_use]
    #[inline(always)]
    pub const fn value(&self) -> u64 {
        self.value
    }

    /// This ordinary read is called by both test executables.
    #[must_use]
    pub fn ordinary(&self) -> u64 {
        self.value
    }

    /// Negative control: no test calls this method, so its count must stay zero.
    #[must_use]
    #[inline(always)]
    pub const fn never_called(&self) -> u64 {
        self.value + 1
    }
}
