// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Identity for a successful compiled property path.

use std::any::TypeId;
use std::hash::Hash;

use qubit_reflect::context::ReflectionContextId;

/// Distinguishes root metadata allocations, path segments, and access modes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct PathCacheKey {
    /// Identifies the reflection context that owns the metadata.
    pub(super) context_id: ReflectionContextId,
    /// Identifies the exact Rust type used to compile the path.
    pub(super) type_id: TypeId,
    /// Distinguishes metadata allocations for the same Rust type.
    pub(super) metadata_address: usize,
    /// Owns the ordered path segments used for lookup.
    pub(super) segments: Vec<String>,
    /// Distinguishes writable paths from read-only paths.
    pub(super) write: bool,
}

impl PathCacheKey {
    /// Copies the path inputs into a key for this registry's cache.
    #[must_use]
    pub(super) fn new(
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
