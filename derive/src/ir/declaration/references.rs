// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Relationship and uniqueness IR.

// qubit-style: allow multiple-public-types

use syn::LitStr;
use syn::Type;

#[derive(Clone)]
pub(crate) struct UniqueIr {
    /// Property paths scoping uniqueness.
    pub(crate) respect_to: Vec<Vec<String>>,
    /// Whether textual comparison ignores case.
    pub(crate) ignore_case: Option<bool>,
}

#[derive(Clone)]
pub(crate) enum ReferenceTargetIr {
    /// Target supplied by a Rust type.
    RustType(Box<Type>),
    /// Target supplied by a stable model ID.
    ModelId(LitStr),
}

#[derive(Clone)]
pub(crate) struct ReferenceIr {
    /// Referenced entity target.
    pub(crate) target: ReferenceTargetIr,
    /// Selected target property path.
    pub(crate) property: Option<Vec<String>>,
    /// Whether the target must already exist.
    pub(crate) existing: bool,
    /// Equivalent local property path.
    pub(crate) same_as: Option<Vec<String>>,
}
