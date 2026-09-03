// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
//
// =============================================================================

//! One structured local property assembly failure.

use crate::PropertyBuildErrorKind;

/// Describes one incompatible property fragment combination.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyBuildError {
    /// Stable failure category.
    kind: PropertyBuildErrorKind,
    /// Canonical property name, possibly empty for an invalid declaration.
    property_name: &'static str,
}

impl PropertyBuildError {
    /// Creates one property assembly failure.
    pub(crate) const fn new(kind: PropertyBuildErrorKind, property_name: &'static str) -> Self {
        Self { kind, property_name }
    }

    /// Returns the stable failure category.
    #[must_use]
    #[inline(always)]
    pub const fn kind(&self) -> PropertyBuildErrorKind {
        self.kind
    }

    /// Returns the canonical property name associated with the failure.
    #[must_use]
    #[inline(always)]
    pub const fn property_name(&self) -> &'static str {
        self.property_name
    }
}

impl core::fmt::Display for PropertyBuildError {
    /// Formats a stable English diagnostic for this property failure.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "property {:?} for `{}`", self.kind, self.property_name)
    }
}
