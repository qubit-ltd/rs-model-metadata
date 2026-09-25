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
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyAccessorConflict {
    first_method: &'static str,
    second_method: &'static str,
    first_origin: CapabilityOrigin,
    second_origin: CapabilityOrigin,
}

impl PropertyAccessorConflict {
    /// Creates the retained details for two conflicting provider methods.
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
    #[must_use]
    pub const fn first_method(&self) -> &'static str {
        self.first_method
    }

    /// Returns the conflicting method from the later provider.
    #[must_use]
    pub const fn second_method(&self) -> &'static str {
        self.second_method
    }

    /// Returns the origin of the first selected method.
    #[must_use]
    pub const fn first_origin(&self) -> &CapabilityOrigin {
        &self.first_origin
    }

    /// Returns the origin of the later conflicting method.
    #[must_use]
    pub const fn second_origin(&self) -> &CapabilityOrigin {
        &self.second_origin
    }
}
