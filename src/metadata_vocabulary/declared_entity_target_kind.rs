// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Distinguishes a Rust-type target from a stable model-ID target.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{DeclaredEntityTarget, DeclaredEntityTargetKind, ModelId};
///
/// let target = DeclaredEntityTarget::ModelId(ModelId::new("example.Customer"));
/// assert_eq!(target.kind(), DeclaredEntityTargetKind::ModelId);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeclaredEntityTargetKind {
    /// The target is supplied by a static metadata provider.
    RustType,
    /// The target is supplied as a stable model ID.
    ModelId,
}
