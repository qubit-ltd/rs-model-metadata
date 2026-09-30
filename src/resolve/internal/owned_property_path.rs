// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned property paths used while assembling resolved query metadata.

use crate::metadata::PropertyPath;

/// An owned runtime path whose segment names originate in static declarations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OwnedPropertyPath {
    /// Owned static property-name segments in declaration order.
    segments: Box<[&'static str]>,
}

impl OwnedPropertyPath {
    /// Copies one runtime-generated segment sequence.
    ///
    /// # Parameters
    ///
    /// - `segments`: Static property-name fragments in declaration order.
    ///
    /// # Returns
    ///
    /// An owned path retaining the supplied property-name sequence.
    pub(super) fn from_segments(segments: &[&'static str]) -> Self {
        Self {
            segments: segments.into(),
        }
    }

    /// Borrows this owned path as the public lightweight view.
    ///
    /// # Returns
    ///
    /// A [`PropertyPath`] borrowing the stored segments for this path's
    /// lifetime.
    #[inline]
    pub(super) fn as_path(&self) -> PropertyPath<'_> {
        PropertyPath::new(&self.segments)
    }
}
