// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Aggregated local property assembly failures.

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;
use std::error::Error;

use crate::metadata::PropertyBuildError;

/// Deterministically ordered failures produced while merging properties.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::metadata::PropertyResolutionError;
/// use qubit_model_metadata::metadata::TypeMetadata;
///
/// #[Model]
/// struct Record { name: String }
/// # fn main() {
/// let metadata = TypeMetadata::of::<Record>();
/// match metadata.try_properties() {
///     Ok(_) => {}
///     Err(PropertyResolutionError::Assembly(errors)) => {
///         assert!(errors.errors().iter().all(|error| !error.property_name().is_empty()));
///     }
///     Err(error) => panic!("unexpected property resolution error: {error}"),
/// }
/// # }
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct PropertyBuildErrors {
    /// Failures ordered by property name and stable category.
    errors: Box<[PropertyBuildError]>,
}

impl PropertyBuildErrors {
    /// Creates and deterministically orders an aggregate failure.
    ///
    /// # Parameters
    ///
    /// - `errors`: Individual failures to order by property name and category.
    ///
    /// # Returns
    ///
    /// The aggregate with its failures stored in deterministic order.
    pub(crate) fn new(mut errors: Vec<PropertyBuildError>) -> Self {
        errors.sort_by(|left, right| {
            left.property_name()
                .cmp(right.property_name())
                .then_with(|| left.kind().cmp(&right.kind()))
        });
        Self {
            errors: errors.into_boxed_slice(),
        }
    }

    /// Returns every local property failure in deterministic order.
    ///
    /// # Returns
    ///
    /// All failures ordered by property name and stable category.
    #[must_use = "inspect the property assembly failures"]
    #[inline]
    pub const fn errors(&self) -> &[PropertyBuildError] {
        &self.errors
    }
}

impl Display for PropertyBuildErrors {
    /// Formats all failures on separate lines.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination formatter supplied by the formatting caller.
    ///
    /// # Returns
    ///
    /// `Ok(())` after every failure has been written.
    ///
    /// # Errors
    ///
    /// Returns the formatter error if writing any failure fails.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        for (index, error) in self.errors.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            error.fmt(formatter)?;
        }
        Ok(())
    }
}

impl Error for PropertyBuildErrors {}
