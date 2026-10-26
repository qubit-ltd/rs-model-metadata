// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Input projection selected by a standard constraint binding.

/// Whether a standard binding validates the field value or its item count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StandardTarget {
    /// Pass the field value to the rule.
    Value,
    /// Pass the borrowed sequence length as a `usize`.
    SequenceCount,
    /// Pass the borrowed map length as a `usize`.
    MapCount,
}
