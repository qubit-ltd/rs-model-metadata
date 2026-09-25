// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned views of effective properties resolved from a reflection snapshot.

use std::sync::Arc;

use crate::metadata::LocalPropertySet;
use crate::metadata::PropertyMetadata;

/// Properties whose storage remains alive for as long as this view.
#[derive(Clone, Debug)]
pub enum ResolvedProperties {
    /// Properties emitted as part of static model metadata.
    Static(&'static LocalPropertySet),
    /// Properties assembled from an explicit reflection snapshot.
    Merged(Arc<[PropertyMetadata]>),
}

impl ResolvedProperties {
    /// Returns the properties retained by this view.
    #[must_use]
    pub fn properties(&self) -> &[PropertyMetadata] {
        match self {
            Self::Static(properties) => properties.properties(),
            Self::Merged(properties) => properties,
        }
    }

    /// Finds a property by name within this view.
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&PropertyMetadata> {
        self.properties().iter().find(|property| property.name() == name)
    }
}
