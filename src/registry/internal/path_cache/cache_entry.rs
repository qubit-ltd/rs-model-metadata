// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One cached compiled path and its least-recently-used access stamp.

use std::sync::Arc;

use super::PathCacheKey;

/// One full key and its compiled path inside a hash bucket.
///
/// # Type Parameters
///
/// - `T`: Compiled path value retained by this entry.
pub(super) struct CacheEntry<T> {
    /// Full key retained to verify hash collisions.
    pub(super) key: PathCacheKey,
    /// Compiled value shared with registry callers.
    pub(super) compiled: Arc<T>,
    /// Monotonic stamp used to select the least recently used entry.
    pub(super) last_access: u64,
}
