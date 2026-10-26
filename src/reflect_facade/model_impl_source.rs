// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable source coordinates for model implementation declarations.

/// Stable source coordinates for one independently declared model impl.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModelImplSource {
    /// Rust module containing the declaration.
    pub module_path: &'static str,
    /// Source file containing the declaration.
    pub file: &'static str,
    /// One-based source line of the declaration.
    pub line: u32,
    /// Stable fingerprint distinguishing declarations on the same line.
    pub fingerprint: u64,
}
