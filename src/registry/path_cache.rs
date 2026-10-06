// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Bounded successful property-path cache for one model registry.

use std::any::TypeId;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

/// Maximum number of successful compiled paths retained by one registry.
const CACHE_CAPACITY: usize = 256;

/// Distinguishes root metadata allocations, path segments, and access modes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(super) struct PathCacheKey {
    type_id: TypeId,
    metadata_address: usize,
    segments: Vec<String>,
    write: bool,
}

impl PathCacheKey {
    /// Copies the path inputs into a key for this registry's cache.
    pub(super) fn new(type_id: TypeId, metadata_address: usize, segments: &[&str], write: bool) -> Self {
        Self {
            type_id,
            metadata_address,
            segments: segments.iter().map(|segment| (*segment).to_owned()).collect(),
            write,
        }
    }
}

/// Stores successful paths in insertion order and evicts the oldest entry.
pub(super) struct PathCache<T> {
    entries: HashMap<PathCacheKey, Arc<T>>,
    insertion_order: VecDeque<PathCacheKey>,
}

impl<T> std::fmt::Debug for PathCache<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PathCache")
            .field("len", &self.entries.len())
            .finish()
    }
}

impl<T> Default for PathCache<T> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            insertion_order: VecDeque::new(),
        }
    }
}

impl<T> PathCache<T> {
    /// Clones a cached value, returning `None` for a miss without changing order.
    pub(super) fn get(&self, key: &PathCacheKey) -> Option<Arc<T>> {
        self.entries.get(key).map(Arc::clone)
    }

    /// Keeps an existing value on a racing insert, or adds the new success.
    ///
    /// The oldest inserted key is evicted once this registry exceeds 256 paths.
    pub(super) fn insert_or_existing(&mut self, key: PathCacheKey, compiled: Arc<T>) -> Arc<T> {
        if let Some(existing) = self.get(&key) {
            return existing;
        }
        self.insertion_order.push_back(key.clone());
        self.entries.insert(key, Arc::clone(&compiled));
        if self.entries.len() > CACHE_CAPACITY {
            let oldest = self.insertion_order.pop_front().expect("cache insertion has an oldest key");
            self.entries.remove(&oldest);
        }
        compiled
    }

    /// Returns the count of retained successes for the capacity regression.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::sync::Arc;

    use super::CACHE_CAPACITY;
    use super::PathCache;
    use super::PathCacheKey;

    /// Inserting past the bound evicts the earliest key and allows recompilation.
    #[test]
    fn test_path_cache_evicts_earliest_success_after_256_entries() {
        let mut cache = PathCache::default();
        let first_key = PathCacheKey::new(TypeId::of::<usize>(), 1, &["path-0"], false);
        let first = cache.insert_or_existing(first_key.clone(), Arc::new(0));

        for index in 1..=CACHE_CAPACITY {
            let segment = format!("path-{index}");
            let key = PathCacheKey::new(TypeId::of::<usize>(), 1, &[&segment], false);
            cache.insert_or_existing(key, Arc::new(index));
            assert!(cache.len() <= CACHE_CAPACITY);
        }

        assert!(cache.get(&first_key).is_none(), "the oldest key should be evicted");
        let replacement = cache.insert_or_existing(first_key, Arc::new(999));
        assert!(!Arc::ptr_eq(&first, &replacement));
        assert_eq!(*replacement, 999);
        assert_eq!(cache.len(), CACHE_CAPACITY);
    }
}
