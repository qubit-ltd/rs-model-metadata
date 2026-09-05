// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Return-shape representation for validated property getters.

use syn::Type;

/// Classifies the Rust return shape accepted for a property getter.
#[derive(Clone)]
pub(in crate::expand::model_impl) enum GetterReturn {
    /// Getter returns an owned value.
    Owned(Type),
    /// Getter returns a borrowed value.
    Borrowed(Type),
    /// Getter returns a borrowed string.
    BorrowedStr,
    /// Getter returns a borrowed slice.
    BorrowedSlice(Type),
    /// Getter returns an optional borrowed value.
    OptionalBorrowed(Type),
    /// Getter returns an optional borrowed string.
    OptionalBorrowedStr,
}
