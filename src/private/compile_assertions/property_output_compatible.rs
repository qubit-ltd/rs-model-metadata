// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compatibility relation between generated getter and setter value types.

use super::BorrowedPropertyOutput;
use super::OptionalBorrowedPropertyOutput;

/// Proves that a getter output can represent the value accepted by a
/// setter for the same logical property.
///
/// The implementation set preserves borrowing while recognizing the
/// canonical owned forms supported by model properties.
///
/// # Type Parameters
///
/// - `Setter`: Value type accepted by the corresponding setter.
#[doc(hidden)]
pub trait PropertyOutputCompatible<Setter: ?Sized> {}

impl<T: ?Sized> PropertyOutputCompatible<T> for T {}
impl<T: ?Sized> PropertyOutputCompatible<T> for BorrowedPropertyOutput<T> {}
impl PropertyOutputCompatible<String> for BorrowedPropertyOutput<str> {}
impl<T> PropertyOutputCompatible<Vec<T>> for BorrowedPropertyOutput<[T]> {}
impl<T> PropertyOutputCompatible<Option<T>> for OptionalBorrowedPropertyOutput<T> {}
impl PropertyOutputCompatible<Option<String>> for OptionalBorrowedPropertyOutput<str> {}
