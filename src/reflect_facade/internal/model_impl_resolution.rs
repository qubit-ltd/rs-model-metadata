// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolution state for one or more model implementation providers.

use std::sync::Arc;

use crate::model_impl_metadata::MergedModelImpl;

/// An owned merge of implementation providers for one reflection snapshot.
pub(crate) enum ModelImplResolution {
    /// Owns the merged metadata assembled from multiple providers.
    Merged(Arc<MergedModelImpl>),
}
