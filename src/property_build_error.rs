// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One structured local property assembly failure.

use qubit_reflect::capability::CapabilityOrigin;

use crate::metadata::PropertyBuildErrorKind;
use crate::property_accessor_conflict::PropertyAccessorConflict;

/// Describes one incompatible property fragment combination.
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
    pub(crate) const fn new(kind: PropertyBuildErrorKind, property_name: &'static str) -> Self {
        Self {
            kind,
            property_name,
            conflict: None,
        }
    }

    /// Creates an accessor conflict with both selected methods and origins.
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
    #[must_use]
    #[inline]
    pub const fn kind(&self) -> PropertyBuildErrorKind {
        self.kind
    }

    /// Returns the canonical property name associated with the failure.
    #[must_use]
    #[inline]
    pub const fn property_name(&self) -> &'static str {
        self.property_name
    }

    /// Returns both accessor methods and origins for conflicts, or `None` for
    /// static errors.
    #[must_use]
    #[inline]
    pub const fn conflict(&self) -> Option<&PropertyAccessorConflict> {
        self.conflict.as_ref()
    }
}

impl core::fmt::Display for PropertyBuildError {
    /// Formats a stable English diagnostic for this property failure.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "property {:?} for `{}`", self.kind, self.property_name)?;
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
