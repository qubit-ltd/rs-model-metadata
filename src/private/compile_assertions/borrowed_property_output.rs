// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Marker for a getter output borrowed from its property value.

use core::marker::PhantomData;

/// Lifetime-independent marker for a getter returning `&T`.
///
/// # Type Parameters
///
/// - `T`: Value type borrowed by the getter.
#[doc(hidden)]
pub struct BorrowedPropertyOutput<T: ?Sized>(
    /// Carries the borrowed output type without storing a value.
    PhantomData<fn() -> T>,
);
