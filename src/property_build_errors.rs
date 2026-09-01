// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Aggregated local property assembly failures.

use crate::PropertyBuildError;

/// Deterministically ordered failures produced while merging properties.
#[derive(Clone, Debug)]
pub struct PropertyBuildErrors {
    /// Failures ordered by property name and stable category.
    errors: Box<[PropertyBuildError]>,
}

impl PropertyBuildErrors {
    /// Creates and deterministically orders an aggregate failure.
    #[must_use]
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
    #[must_use]
    pub const fn errors(&self) -> &[PropertyBuildError] {
        &self.errors
    }
}

impl core::fmt::Display for PropertyBuildErrors {
    /// Formats all failures on separate lines.
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (index, error) in self.errors.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            error.fmt(formatter)?;
        }
        Ok(())
    }
}

impl std::error::Error for PropertyBuildErrors {}
