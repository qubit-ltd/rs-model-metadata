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
    Bool(bool),
    /// Signed integer argument.
    Integer(i128),
    /// Unsigned integer argument.
    Unsigned(u128),
    /// String argument.
    String(LitStr),
    /// Boolean list argument.
    BoolList(Vec<bool>),
    /// Signed integer list argument.
    IntegerList(Vec<i128>),
    /// Unsigned integer list argument.
    UnsignedList(Vec<u128>),
    /// String list argument.
    StringList(Vec<LitStr>),
}
