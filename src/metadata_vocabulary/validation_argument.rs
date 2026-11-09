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
    Bool(
        /// The Boolean value passed to a validator.
        bool,
    ),
    /// A signed integer value.
    Integer(
        /// The signed integer passed to a validator.
        i128,
    ),
    /// An unsigned integer value.
    Unsigned(
        /// The unsigned integer passed to a validator.
        u128,
    ),
    /// A borrowed string value.
    String(
        /// The string borrowed for the duration of the validation call.
        &'a str,
    ),
    /// A borrowed Boolean list.
    BoolList(
        /// Boolean values borrowed for the duration of the validation call.
        &'a [bool],
    ),
    /// A borrowed signed integer list.
    IntegerList(
        /// Signed integer values borrowed for the duration of the validation
        /// call.
        &'a [i128],
    ),
    /// A borrowed unsigned integer list.
    UnsignedList(
        /// Unsigned integer values borrowed for the duration of the validation
        /// call.
        &'a [u128],
    ),
    /// A borrowed string list.
    StringList(
        /// Strings and their slice are borrowed for the validation call.
        &'a [&'a str],
    ),
}

impl fmt::Debug for ValidationArgument<'_> {
    /// Formats the argument without exposing its validation value.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination for the redacted representation.
    ///
    /// # Returns
    ///
    /// `Ok(())` when formatting succeeds, or the formatter's error.
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
