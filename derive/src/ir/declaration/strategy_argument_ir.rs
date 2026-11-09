// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Literal argument shapes accepted by validator strategies.

use syn::LitStr;

/// Literal value shape accepted as a validator strategy argument.
#[derive(Clone)]
pub(crate) enum StrategyArgumentIr {
    /// Boolean argument.
    Bool(
        /// The value of the parsed boolean literal.
        bool,
    ),
    /// Signed integer argument.
    Integer(
        /// The signed value parsed from an integer literal.
        i128,
    ),
    /// Unsigned integer argument.
    Unsigned(
        /// The non-negative value parsed from an integer literal.
        u128,
    ),
    /// String argument.
    String(
        /// The parsed string literal, including its source span.
        LitStr,
    ),
    /// Boolean list argument.
    BoolList(
        /// Values from a non-empty homogeneous boolean literal array.
        Vec<bool>,
    ),
    /// Signed integer list argument.
    IntegerList(
        /// Signed values from a non-empty homogeneous integer literal array.
        Vec<i128>,
    ),
    /// Unsigned integer list argument.
    UnsignedList(
        /// Non-negative values from a non-empty homogeneous integer literal
        /// array.
        Vec<u128>,
    ),
    /// String list argument.
    StringList(
        /// String literals from a non-empty homogeneous array, with their
        /// spans.
        Vec<LitStr>,
    ),
}
