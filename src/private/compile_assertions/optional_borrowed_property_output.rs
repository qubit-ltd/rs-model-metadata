// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Marker for an optional getter output borrowed from its property value.

use core::marker::PhantomData;

/// Lifetime-independent marker for a getter returning `Option<&T>`.
///
/// # Type Parameters
///
/// - `T`: Value type borrowed when the getter returns `Some`.
#[doc(hidden)]
pub struct OptionalBorrowedPropertyOutput<T: ?Sized>(
    /// Carries the optional borrowed output type without storing a value.
    PhantomData<fn() -> T>,
);
