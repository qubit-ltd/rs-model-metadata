// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::fmt;

/// One statically typed validator parameter value.
///
/// # Type Parameters
///
/// - `'a`: Lifetime of any borrowed string or slice carried by the value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ValidationArgument;
///
/// let argument = ValidationArgument::String("example");
/// assert!(matches!(argument, ValidationArgument::String("example")));
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
#[must_use = "validation arguments should be passed to a validator"]
pub enum ValidationArgument<'a> {
    /// A Boolean value.
    Bool(bool),
    /// A signed integer value.
    Integer(i128),
    /// An unsigned integer value.
    Unsigned(u128),
    /// A borrowed string value.
    String(&'a str),
    /// A borrowed Boolean list.
    BoolList(&'a [bool]),
    /// A borrowed signed integer list.
    IntegerList(&'a [i128]),
    /// A borrowed unsigned integer list.
    UnsignedList(&'a [u128]),
    /// A borrowed string list.
    StringList(&'a [&'a str]),
}

impl fmt::Debug for ValidationArgument<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(_) => formatter.write_str("Bool(<redacted>)"),
            Self::Integer(_) => formatter.write_str("Integer(<redacted>)"),
            Self::Unsigned(_) => formatter.write_str("Unsigned(<redacted>)"),
            Self::String(_) => formatter.write_str("String(<redacted>)"),
            Self::BoolList(values) => formatter.debug_struct("BoolList").field("len", &values.len()).finish(),
            Self::IntegerList(values) => formatter
                .debug_struct("IntegerList")
                .field("len", &values.len())
                .finish(),
            Self::UnsignedList(values) => formatter
                .debug_struct("UnsignedList")
                .field("len", &values.len())
                .finish(),
            Self::StringList(values) => formatter
                .debug_struct("StringList")
                .field("len", &values.len())
                .finish(),
        }
    }
}
