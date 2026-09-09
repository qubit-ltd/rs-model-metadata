// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Invalid object navigation metadata.

/// A property step contains an empty or ambiguous name.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid object navigation property at step {index}: {name}")]
pub struct ObjectPathError {
    /// The zero-based invalid navigation step.
    pub index: usize,
    /// The rejected property name.
    pub name: &'static str,
}
