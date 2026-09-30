// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Both accessor methods and their provider origins in a property conflict.

use qubit_reflect::capability::CapabilityOrigin;

/// Identifies both methods and provider origins in an accessor conflict.
///
/// Values are returned by property assembly when two providers declare
/// different methods for the same accessor. Inspect both method names and
/// origins to identify the conflicting declarations.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyAccessorConflict;
///
/// fn describe(conflict: &PropertyAccessorConflict) -> (&str, &str) {
///     (conflict.first_method(), conflict.second_method())
/// }
/// ```
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyAccessorConflict {
    /// Name of the method selected from the first provider.
    first_method: &'static str,
    /// Name of the conflicting method from the later provider.
    second_method: &'static str,
    /// Provider origin associated with the first selected method.
    first_origin: CapabilityOrigin,
    /// Provider origin associated with the later conflicting method.
    second_origin: CapabilityOrigin,
}

impl PropertyAccessorConflict {
    /// Creates the retained details for two conflicting provider methods.
    ///
    /// # Parameters
    ///
    /// * `first_method` - Method selected from the first provider.
    /// * `second_method` - Conflicting method from the later provider.
    /// * `first_origin` - Origin of the first selected method.
    /// * `second_origin` - Origin of the later conflicting method.
    ///
    /// # Returns
    ///
    /// The retained method names and provider origins.
    #[inline]
    pub(crate) const fn new(
        first_method: &'static str,
        second_method: &'static str,
        first_origin: CapabilityOrigin,
        second_origin: CapabilityOrigin,
    ) -> Self {
        Self {
            first_method,
            second_method,
            first_origin,
            second_origin,
        }
    }

    /// Returns the method selected from the first provider.
    ///
    /// # Returns
    ///
    /// The first provider's method name.
    #[must_use]
    #[inline]
    pub const fn first_method(&self) -> &'static str {
        self.first_method
    }

    /// Returns the conflicting method from the later provider.
    ///
    /// # Returns
    ///
    /// The later provider's conflicting method name.
    #[must_use]
    #[inline]
    pub const fn second_method(&self) -> &'static str {
        self.second_method
    }

    /// Returns the origin of the first selected method.
    ///
    /// # Returns
    ///
    /// A shared reference to the first provider's origin.
    #[must_use]
    #[inline]
    pub const fn first_origin(&self) -> &CapabilityOrigin {
        &self.first_origin
    }

    /// Returns the origin of the later conflicting method.
    ///
    /// # Returns
    ///
    /// A shared reference to the later provider's origin.
    #[must_use]
    #[inline]
    pub const fn second_origin(&self) -> &CapabilityOrigin {
        &self.second_origin
    }
}
