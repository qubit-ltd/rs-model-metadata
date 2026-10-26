// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::fmt;

use super::validation_argument::ValidationArgument;

/// One named validator parameter borrowing its name and value.
///
/// # Type Parameters
///
/// - `'a`: Lifetime shared by the borrowed parameter name and any references
///   carried by its typed value.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{NamedValidationArgument, ValidationArgument};
///
/// let argument = NamedValidationArgument::new("limit", ValidationArgument::Integer(5));
/// assert_eq!(argument.name(), "limit");
/// assert!(matches!(argument.value(), ValidationArgument::Integer(5)));
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NamedValidationArgument<'a> {
    /// Name used by the validator's parameter schema.
    name: &'a str,
    /// Typed value supplied for the named parameter.
    value: ValidationArgument<'a>,
}

impl<'a> NamedValidationArgument<'a> {
    /// Creates a named parameter; panics if `name` is empty.
    ///
    /// # Parameters
    ///
    /// * `name` - Non-empty key expected by the validator schema.
    /// * `value` - Typed value borrowed by the validator.
    ///
    /// # Returns
    ///
    /// The named parameter borrowing `name` and `value`.
    ///
    /// # Panics
    ///
    /// Panics if `name` is empty.
    #[must_use]
    #[inline]
    pub const fn new(name: &'a str, value: ValidationArgument<'a>) -> Self {
        assert!(!name.is_empty(), "validator parameter name cannot be empty");
        Self { name, value }
    }

    /// Returns the borrowed parameter name.
    ///
    /// # Returns
    ///
    /// The schema key borrowed for the lifetime of this argument.
    #[must_use]
    #[inline]
    pub const fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the typed parameter value.
    ///
    /// # Returns
    ///
    /// The copyable value supplied for this named parameter.
    #[inline]
    pub const fn value(&self) -> ValidationArgument<'a> {
        self.value
    }
}

impl fmt::Debug for NamedValidationArgument<'_> {
    /// Formats this argument using the value's redacted debug representation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NamedValidationArgument")
            .field("value", &self.value)
            .finish()
    }
}
