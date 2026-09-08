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
    segments: Box<[&'static str]>,
}

impl OwnedPropertyPath {
    /// Copies one runtime-generated segment sequence.
    pub(super) fn from_segments(segments: &[&'static str]) -> Self {
        Self {
            segments: segments.into(),
        }
    }

    /// Copies one statically declared path.
    pub(super) fn from_static(path: PropertyPath<'static>) -> Self {
        Self::from_segments(path.segments())
    }

    /// Borrows this owned path as the public lightweight view.
    pub(super) fn as_path(&self) -> PropertyPath<'_> {
        PropertyPath::new(&self.segments)
    }
}
