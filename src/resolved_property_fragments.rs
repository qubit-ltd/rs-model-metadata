// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned views of property fragments resolved from a reflection snapshot.

use std::sync::Arc;

use crate::metadata::PropertyFragment;

/// Property fragments whose storage remains alive for as long as this view.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ResolvedPropertyFragments;
///
/// let fragments = ResolvedPropertyFragments::Static(&[]);
/// assert!(fragments.fragments().is_empty());
/// ```
#[derive(Clone, Debug)]
pub enum ResolvedPropertyFragments {
    /// Fragments emitted as part of static model metadata.
    Static(&'static [PropertyFragment]),
    /// Fragments collected from an explicit reflection snapshot.
    Merged(Arc<[PropertyFragment]>),
}

impl ResolvedPropertyFragments {
    /// Returns the fragments retained by this view.
    ///
    /// # Returns
    ///
    /// A slice borrowed from this view. Its storage remains valid for the
    /// lifetime of the view, whether the fragments are static or
    /// snapshot-owned.
    #[must_use]
    #[inline]
    pub fn fragments(&self) -> &[PropertyFragment] {
        match self {
            Self::Static(fragments) => fragments,
            Self::Merged(fragments) => fragments,
        }
    }
}
