// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Storage for one collision-resolved path-cache entry.

use std::sync::Arc;

use super::super::path_cache_key::PathCacheKey;

/// One full key and its compiled path inside a hash bucket.
///
/// # Type Parameters
///
/// - `T`: the compiled property-path value shared with cache callers.
pub(super) struct CacheEntry<T> {
    /// Complete path identity used to resolve hash collisions.
    pub(super) key: PathCacheKey,
    /// Shared compiled path retained for successful lookups.
    pub(super) compiled: Arc<T>,
    /// Monotonic cache access stamp used to select eviction candidates.
    pub(super) last_access: u64,
}
