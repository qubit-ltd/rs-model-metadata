// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One structured local property assembly failure.

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;

use qubit_reflect::capability::CapabilityOrigin;

use crate::metadata::PropertyBuildErrorKind;
use crate::property_accessor_conflict::PropertyAccessorConflict;

/// Describes one incompatible property fragment combination.
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
///         for error in errors.errors() {
///             let _diagnostic = (error.kind(), error.property_name(), error.conflict());
///         }
///     }
///     Err(error) => panic!("unexpected property resolution error: {error}"),
/// }
/// # }
/// ```
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyBuildError {
    /// Stable failure category.
    kind: PropertyBuildErrorKind,
    /// Canonical property name, possibly empty for an invalid declaration.
    property_name: &'static str,
    /// Accessor details, present only for cross-provider conflicts.
    conflict: Option<PropertyAccessorConflict>,
}

impl PropertyBuildError {
    /// Creates one property assembly failure.
    ///
    /// # Parameters
    ///
    /// - `kind`: Stable category describing the validation failure.
    /// - `property_name`: Canonical name associated with the failure.
    ///
    /// # Returns
    ///
    /// A failure without accessor-conflict details.
    pub(crate) const fn new(kind: PropertyBuildErrorKind, property_name: &'static str) -> Self {
        Self {
            kind,
            property_name,
            conflict: None,
        }
    }

    /// Creates an accessor conflict with both selected methods and origins.
    ///
    /// # Parameters
    ///
    /// - `kind`: Conflict category, such as a duplicate getter or setter.
    /// - `property_name`: Canonical name of the conflicting property.
    /// - `first_method`: Name of the first selected accessor.
    /// - `second_method`: Name of the second selected accessor.
    /// - `first_origin`: Declaration source for the first accessor.
    /// - `second_origin`: Declaration source for the second accessor.
    ///
    /// # Returns
    ///
    /// A failure retaining both accessor names and declaration origins.
    pub(crate) fn with_conflict(
        kind: PropertyBuildErrorKind,
        property_name: &'static str,
        first_method: &'static str,
        second_method: &'static str,
        first_origin: CapabilityOrigin,
        second_origin: CapabilityOrigin,
    ) -> Self {
        Self {
            kind,
            property_name,
            conflict: Some(PropertyAccessorConflict::new(
                first_method,
                second_method,
                first_origin,
                second_origin,
            )),
        }
    }

    /// Returns the stable failure category.
    ///
    /// # Returns
    ///
    /// The category recorded when the error was assembled.
    #[must_use = "the failure category should be handled"]
    #[inline]
    pub const fn kind(&self) -> PropertyBuildErrorKind {
        self.kind
    }

    /// Returns the canonical property name associated with the failure.
    ///
    /// # Returns
    ///
    /// The property name retained by this error.
    #[must_use]
    #[inline]
    pub const fn property_name(&self) -> &'static str {
        self.property_name
    }

    /// Returns both accessor methods and origins for conflicts, or `None` for
    /// static errors.
    ///
    /// # Returns
    ///
    /// Both accessor methods and origins for a conflict, or `None` when the
    /// error describes a static validation failure.
    #[must_use = "the optional accessor conflict should be checked"]
    #[inline]
    pub const fn conflict(&self) -> Option<&PropertyAccessorConflict> {
        self.conflict.as_ref()
    }
}

impl Display for PropertyBuildError {
    /// Formats a stable English diagnostic for this property failure.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination formatter supplied by the formatting caller.
    ///
    /// # Returns
    ///
    /// `Ok(())` after all diagnostic text is written.
    ///
    /// # Errors
    ///
    /// Returns the formatter error if writing the diagnostic fails.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(
            formatter,
            "property {:?} for `{}`",
            self.kind, self.property_name
        )?;
        if let Some(conflict) = &self.conflict {
            write!(
                formatter,
                ": `{}` from {:?} conflicts with `{}` from {:?}",
                conflict.first_method(),
                conflict.first_origin(),
                conflict.second_method(),
                conflict.second_origin()
            )?;
        }
        Ok(())
    }
}
