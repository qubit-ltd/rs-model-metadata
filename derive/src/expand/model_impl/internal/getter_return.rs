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
    Owned(
        /// The Rust type of the owned getter result.
        Type,
    ),
    /// Getter returns a borrowed value.
    Borrowed(
        /// The pointee type of the getter's shared reference result.
        Type,
    ),
    /// Getter returns a borrowed string.
    BorrowedStr,
    /// Getter returns a borrowed slice.
    BorrowedSlice(
        /// The element type of the getter's borrowed slice result.
        Type,
    ),
    /// Getter returns an optional borrowed value.
    OptionalBorrowed(
        /// The pointee type carried by the optional shared reference result.
        Type,
    ),
    /// Getter returns an optional borrowed string.
    OptionalBorrowedStr,
}
