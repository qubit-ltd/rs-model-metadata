// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Identity key for one registry-owned compiled property path.

use std::any::TypeId;

use qubit_reflect::context::ReflectionContextId;

/// Distinguishes root metadata allocations, path segments, and access modes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::registry::model_registry) struct PathCacheKey {
    /// Reflection context that owns the path.
    pub(super) context_id: ReflectionContextId,
    /// Rust type identity of the root metadata.
    pub(super) type_id: TypeId,
    /// Address distinguishing metadata allocations for the same type.
    pub(super) metadata_address: usize,
    /// Owned property names making up the access path.
    pub(super) segments: Vec<String>,
    /// Whether the compiled path supports writes.
    pub(super) write: bool,
}

impl PathCacheKey {
    /// Copies the path inputs into a key for this registry's cache.
    #[must_use]
    pub(in crate::registry::model_registry) fn new(
        context_id: ReflectionContextId,
        type_id: TypeId,
        metadata_address: usize,
        segments: &[&str],
        write: bool,
    ) -> Self {
        Self {
            context_id,
            type_id,
            metadata_address,
            segments: segments.iter().map(|segment| (*segment).to_owned()).collect(),
            write,
        }
    }

    /// Compares an owned key against caller-borrowed path segments.
    #[must_use]
    #[inline]
    pub(super) fn matches_borrowed(
        &self,
        context_id: ReflectionContextId,
        type_id: TypeId,
        metadata_address: usize,
        segments: &[&str],
        write: bool,
    ) -> bool {
        self.context_id == context_id
            && self.type_id == type_id
            && self.metadata_address == metadata_address
            && self.write == write
            && self.segments.len() == segments.len()
            && self
                .segments
                .iter()
                .zip(segments)
                .all(|(owned, borrowed)| owned == borrowed)
    }
}
