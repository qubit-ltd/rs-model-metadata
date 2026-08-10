// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// The repertoire accepted by a text constraint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextRepertoire {
    /// All Unicode scalar values are accepted.
    #[default]
    Unicode,
    /// Only ASCII characters are accepted.
    Ascii,
}
