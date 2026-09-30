// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Resolution state for one or more model implementation providers.

use std::sync::Arc;

use crate::metadata::ModelImplMetadata;
use crate::model_impl_metadata::MergedModelImpl;

/// A static implementation block or an owned multi-provider merge.
///
/// Static entries borrow generated metadata, while merged entries own the
/// combined result for one reflection snapshot.
pub(crate) enum ModelImplResolution {
    /// Uses the single implementation block's generated static metadata.
    Static(&'static ModelImplMetadata),
    /// Owns the merged metadata assembled from multiple providers.
    Merged(Arc<MergedModelImpl>),
}
