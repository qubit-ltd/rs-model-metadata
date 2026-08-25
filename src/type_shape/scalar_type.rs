// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Scalar types supported by the metadata system.

/// A scalar type supported by the metadata system.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::HasTypeShape;
/// use qubit_model_metadata::ScalarType;
/// use qubit_model_metadata::TypeShape;
///
/// assert!(matches!(i64::TYPE_SHAPE, TypeShape::Scalar(ScalarType::I64)));
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ScalarType {
    /// The Rust `bool` type.
    Bool,
    /// The Rust `char` type.
    Char,
    /// The Rust `i8` type.
    I8,
    /// The Rust `i16` type.
    I16,
    /// The Rust `i32` type.
    I32,
    /// The Rust `i64` type.
    I64,
    /// The Rust `i128` type.
    I128,
    /// The Rust `isize` type.
    Isize,
    /// The Rust `u8` type.
    U8,
    /// The Rust `u16` type.
    U16,
    /// The Rust `u32` type.
    U32,
    /// The Rust `u64` type.
    U64,
    /// The Rust `u128` type.
    U128,
    /// The Rust `usize` type.
    Usize,
    /// The Rust `f32` type.
    F32,
    /// The Rust `f64` type.
    F64,
    /// The Rust `String` type.
    String,
    /// The `chrono::NaiveDate` type.
    Date,
    /// The `chrono::NaiveTime` type.
    Time,
    /// The `chrono::NaiveDateTime` type.
    DateTime,
    /// The `chrono::DateTime<chrono::Utc>` type.
    Instant,
    /// The `bigdecimal::BigDecimal` type.
    BigDecimal,
}
