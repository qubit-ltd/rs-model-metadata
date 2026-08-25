// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Parsed text repertoire.
#[derive(Clone, Copy)]
pub(crate) enum TextRepertoire {
    /// All Unicode scalar values.
    Unicode,
    /// ASCII characters only.
    Ascii,
}
