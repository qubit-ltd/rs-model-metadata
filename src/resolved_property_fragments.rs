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
#[derive(Clone, Debug)]
pub enum ResolvedPropertyFragments {
    /// Fragments emitted as part of static model metadata.
    Static(&'static [PropertyFragment]),
    /// Fragments collected from an explicit reflection snapshot.
    Merged(Arc<[PropertyFragment]>),
}

impl ResolvedPropertyFragments {
    /// Returns the fragments retained by this view.
    #[must_use]
    pub fn fragments(&self) -> &[PropertyFragment] {
        match self {
            Self::Static(fragments) => fragments,
            Self::Merged(fragments) => fragments,
        }
    }
}
