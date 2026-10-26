// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Both accessor methods and their provider origins in a property conflict.

use crate::reflect_facade::ModelImplSource;

/// Identifies both methods and provider origins in an accessor conflict.
///
/// Values are returned by property assembly when two providers declare
/// different methods for the same accessor. Inspect both method names and
/// origins to identify the conflicting declarations.
///
/// The value is available through
/// [`crate::metadata::PropertyBuildError::conflict`]
/// when [`crate::metadata::TypeMetadata::try_properties`] reports a property
/// assembly failure.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyAccessorConflict;
/// use qubit_model_metadata::metadata::PropertyBuildError;
/// use qubit_model_metadata::metadata::PropertyResolutionError;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// fn describe(conflict: &PropertyAccessorConflict) -> (&str, &str, &str, &str) {
///     let first_origin = conflict.first_origin();
///     let second_origin = conflict.second_origin();
///     (
///         conflict.first_method(),
///         conflict.second_method(),
///         first_origin.module_path,
///         second_origin.module_path,
///     )
/// }
///
/// fn inspect_failure(error: &PropertyBuildError) {
///     if let Some(conflict) = error.conflict() {
///         let _details = describe(conflict);
///     }
/// }
///
/// fn inspect_model(metadata: &'static TypeMetadata) {
///     if let Err(PropertyResolutionError::Assembly(errors)) = metadata.try_properties() {
///         for failure in errors.errors() {
///             inspect_failure(failure);
///         }
///     }
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
    first_origin: ModelImplSource,
    /// Provider origin associated with the later conflicting method.
    second_origin: ModelImplSource,
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
        first_origin: ModelImplSource,
        second_origin: ModelImplSource,
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
    pub const fn first_origin(&self) -> &ModelImplSource {
        &self.first_origin
    }

    /// Returns the origin of the later conflicting method.
    ///
    /// # Returns
    ///
    /// A shared reference to the later provider's origin.
    #[must_use]
    #[inline]
    pub const fn second_origin(&self) -> &ModelImplSource {
        &self.second_origin
    }
}
