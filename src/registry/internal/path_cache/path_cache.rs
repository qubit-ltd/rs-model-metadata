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
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::hash::RandomState;
use std::sync::Arc;

use qubit_reflect::context::ReflectionContextId;

use super::PathCacheKey;
use super::cache_entry::CacheEntry;

/// Maximum number of successful compiled paths retained by one registry.
const CACHE_CAPACITY: usize = 256;

/// Stores successful paths and evicts the least recently used entry.
///
/// # Type Parameters
///
/// - `T`: Compiled path value retained by the cache.
pub(in crate::registry) struct PathCache<T> {
    /// Hasher used to place keys into collision-checked buckets.
    hash_builder: RandomState,
    /// Cached entries grouped by their computed hash.
    entries: HashMap<u64, Vec<CacheEntry<T>>>,
    /// Number of successful paths currently retained.
    len: usize,
    /// Stamp assigned to the next cache access.
    next_access: u64,
    /// Test-only override used to force bucket collisions in cache tests.
    #[cfg(test)]
    forced_hash: Option<u64>,
}

impl<T> Debug for PathCache<T> {
    /// Formats the number of retained entries for diagnostic output.
    ///
    /// # Parameters
    ///
    /// - `formatter`: Destination for the formatted cache summary.
    ///
    /// # Returns
    ///
    /// `Ok(())` after writing the summary, or the formatting error reported by
    /// the formatter.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.debug_struct("PathCache").field("len", &self.len).finish()
    }
}

impl<T> Default for PathCache<T> {
    /// Creates an empty cache with a fresh randomized hash builder.
    fn default() -> Self {
        Self {
            hash_builder: RandomState::new(),
            entries: HashMap::new(),
            len: 0,
            next_access: 0,
            #[cfg(test)]
            forced_hash: None,
        }
    }
}

impl<T> PathCache<T> {
    /// Clones a cached value on an exact borrowed-key hit without allocating.
    /// A miss returns `None` without changing any retained entry's recency.
    ///
    /// # Parameters
    ///
    /// - `context_id`: Reflection context that owns the metadata path.
    /// - `type_id`: Rust type identity of the root metadata.
    /// - `metadata_address`: Address distinguishing metadata allocations.
    /// - `segments`: Borrowed property names forming the path.
    /// - `write`: Whether the path must support writes.
    ///
    /// # Returns
    ///
    /// The retained value on an exact key match, or `None` when the cache has
    /// no entry for the borrowed key.
    pub(in crate::registry) fn get_borrowed(
        &mut self,
        context_id: ReflectionContextId,
        type_id: TypeId,
        metadata_address: usize,
        segments: &[&str],
        write: bool,
    ) -> Option<Arc<T>> {
        let hash = self.hash_borrowed(context_id, type_id, metadata_address, segments, write);
        self.ensure_access_stamp();
        let entry = self.entries.get_mut(&hash)?.iter_mut().find(|entry| {
            entry
                .key
                .matches_borrowed(context_id, type_id, metadata_address, segments, write)
        })?;
        entry.last_access = self.next_access;
        self.next_access += 1;
        Some(Arc::clone(&entry.compiled))
    }

    /// Clones a cached value for an owned key, or returns `None` on a miss.
    ///
    /// # Parameters
    ///
    /// - `key`: Owned identity of the path to look up.
    ///
    /// # Returns
    ///
    /// The retained value on an exact key match, or `None` when the cache has
    /// no entry for the key.
    pub(super) fn get(&mut self, key: &PathCacheKey) -> Option<Arc<T>> {
        let hash = self.hash_owned(key);
        self.ensure_access_stamp();
        let entry = self
            .entries
            .get_mut(&hash)?
            .iter_mut()
            .find(|entry| entry.key == *key)?;
        entry.last_access = self.next_access;
        self.next_access += 1;
        Some(Arc::clone(&entry.compiled))
    }

    /// Keeps an existing value on a racing insert, or adds the new success.
    ///
    /// The least recently used key is evicted when 256 paths are retained.
    ///
    /// # Parameters
    ///
    /// - `key`: Owned identity of the compiled path.
    /// - `compiled`: Shared compiled value to retain on a cache miss.
    ///
    /// # Returns
    ///
    /// The existing cached value for a duplicate key, or the newly inserted
    /// value.
    ///
    /// # Panics
    ///
    /// Panics if the cache length and retained buckets violate the capacity
    /// invariant.
    pub(in crate::registry) fn insert_or_existing(&mut self, key: PathCacheKey, compiled: Arc<T>) -> Arc<T> {
        if let Some(existing) = self.get(&key) {
            return existing;
        }
        if self.len == CACHE_CAPACITY {
            let (oldest_hash, oldest_index, _) = self
                .entries
                .iter()
                .flat_map(|(&hash, bucket)| {
                    bucket
                        .iter()
                        .enumerate()
                        .map(move |(index, entry)| (hash, index, entry.last_access))
                })
                .min_by_key(|(_, _, stamp)| *stamp)
                .expect("full cache has a least recently used entry");
            let bucket = self
                .entries
                .get_mut(&oldest_hash)
                .expect("oldest bucket remains present");
            bucket.remove(oldest_index);
            if bucket.is_empty() {
                self.entries.remove(&oldest_hash);
            }
            self.len -= 1;
        }
        self.ensure_access_stamp();
        let hash = self.hash_owned(&key);
        self.entries.entry(hash).or_default().push(CacheEntry {
            key,
            compiled: Arc::clone(&compiled),
            last_access: self.next_access,
        });
        self.next_access += 1;
        self.len += 1;
        compiled
    }

    /// Returns the count of retained successes for the capacity regression.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.len
    }

    /// Compresses access stamps in their existing order without heap
    /// allocation.
    fn renumber_accesses(&mut self) {
        let mut stamps = [0; CACHE_CAPACITY];
        let mut count = 0;
        for bucket in self.entries.values() {
            for entry in bucket {
                stamps[count] = entry.last_access;
                count += 1;
            }
        }
        stamps[..count].sort_unstable();
        for bucket in self.entries.values_mut() {
            for entry in bucket {
                entry.last_access = stamps[..count]
                    .binary_search(&entry.last_access)
                    .expect("retained access stamp is present") as u64;
            }
        }
        self.next_access = count as u64;
    }

    /// Makes room for the next access stamp while preserving recency order.
    fn ensure_access_stamp(&mut self) {
        if self.next_access == u64::MAX {
            self.renumber_accesses();
        }
    }

    /// Hashes borrowed fields in the same order as their owned counterpart.
    ///
    /// # Parameters
    ///
    /// - `context_id`: Reflection context that owns the metadata path.
    /// - `type_id`: Rust type identity of the root metadata.
    /// - `metadata_address`: Address distinguishing metadata allocations.
    /// - `segments`: Borrowed property names forming the path.
    /// - `write`: Whether the path must support writes.
    ///
    /// # Returns
    ///
    /// The hash used to select the bucket for keys with these fields.
    fn hash_borrowed(
        &self,
        context_id: ReflectionContextId,
        type_id: TypeId,
        metadata_address: usize,
        segments: &[&str],
        write: bool,
    ) -> u64 {
        #[cfg(test)]
        if let Some(hash) = self.forced_hash {
            return hash;
        }
        let mut hasher = self.hash_builder.build_hasher();
        context_id.hash(&mut hasher);
        type_id.hash(&mut hasher);
        metadata_address.hash(&mut hasher);
        write.hash(&mut hasher);
        segments.len().hash(&mut hasher);
        for segment in segments {
            segment.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Hashes an owned key without allocating borrowed segment storage.
    ///
    /// # Parameters
    ///
    /// - `key`: Owned identity of the path to hash.
    ///
    /// # Returns
    ///
    /// The hash used to select the bucket for this key.
    fn hash_owned(&self, key: &PathCacheKey) -> u64 {
        #[cfg(test)]
        if let Some(hash) = self.forced_hash {
            return hash;
        }
        let mut hasher = self.hash_builder.build_hasher();
        key.context_id.hash(&mut hasher);
        key.type_id.hash(&mut hasher);
        key.metadata_address.hash(&mut hasher);
        key.write.hash(&mut hasher);
        key.segments.len().hash(&mut hasher);
        for segment in &key.segments {
            segment.hash(&mut hasher);
        }
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::sync::Arc;
    use std::sync::OnceLock;

    use qubit_reflect::ReflectionContext;
    use qubit_reflect::context::ReflectionContextId;

    use super::CACHE_CAPACITY;
    use super::PathCache;
    use super::PathCacheKey;

    fn test_context_id() -> ReflectionContextId {
        static CONTEXT: OnceLock<ReflectionContext> = OnceLock::new();
        CONTEXT
            .get_or_init(|| ReflectionContext::builder().build().expect("empty context"))
            .id()
    }

    #[test]
    fn test_context_is_part_of_path_cache_identity() {
        let first_context = test_context_id();
        let second_context = ReflectionContext::builder().build().expect("second empty context").id();
        let mut cache = PathCache::default();
        let key = PathCacheKey::new(first_context, TypeId::of::<usize>(), 1, &["value"], false);
        cache.insert_or_existing(key, Arc::new(42));

        assert!(
            cache
                .get_borrowed(second_context, TypeId::of::<usize>(), 1, &["value"], false)
                .is_none()
        );
    }

    /// A borrowed lookup returns the retained allocation for the exact path.
    #[test]
    fn test_path_cache_borrowed_lookup_reuses_value() {
        let mut cache = PathCache::default();
        let type_id = TypeId::of::<usize>();
        let value = cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, 7, &["middle", "leaf"], false),
            Arc::new(42),
        );

        let hit = cache
            .get_borrowed(test_context_id(), type_id, 7, &["middle", "leaf"], false)
            .expect("borrowed path should hit");
        assert!(Arc::ptr_eq(&value, &hit));
        assert!(
            cache
                .get_borrowed(test_context_id(), type_id, 7, &["middle", "other"], false)
                .is_none()
        );
        assert!(
            cache
                .get_borrowed(test_context_id(), type_id, 8, &["middle", "leaf"], false)
                .is_none()
        );
        assert!(
            cache
                .get_borrowed(test_context_id(), type_id, 7, &["middle", "leaf"], true)
                .is_none()
        );
    }

    /// A second insertion of the same key keeps the first compiled Arc.
    #[test]
    fn test_path_cache_duplicate_insert_keeps_existing_value() {
        let mut cache = PathCache::default();
        let key = PathCacheKey::new(test_context_id(), TypeId::of::<usize>(), 7, &["middle", "leaf"], false);
        let first = cache.insert_or_existing(key.clone(), Arc::new(42));
        let competing = Arc::new(99);

        let second = cache.insert_or_existing(key.clone(), Arc::clone(&competing));

        assert!(Arc::ptr_eq(&first, &second));
        assert!(!Arc::ptr_eq(&competing, &second));
        assert!(Arc::ptr_eq(
            &first,
            &cache.get(&key).expect("first value remains cached")
        ));
        assert_eq!(cache.len(), 1);
    }

    /// A forced hash collision must still compare every key field and segment.
    #[test]
    fn test_path_cache_collision_keeps_distinct_paths() {
        let mut cache = PathCache {
            forced_hash: Some(1),
            ..PathCache::default()
        };
        let type_id = TypeId::of::<usize>();
        let first = cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, 7, &["a", "b"], false),
            Arc::new(1),
        );
        let second = cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, 7, &["a", "c"], false),
            Arc::new(2),
        );
        let read = cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, 7, &["a", "b"], true),
            Arc::new(3),
        );
        let alternate_metadata = cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, 8, &["a", "b"], false),
            Arc::new(4),
        );
        let alternate_type = cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), TypeId::of::<u8>(), 7, &["a", "b"], false),
            Arc::new(5),
        );

        for (lookup_type, address, segments, write, expected) in [
            (type_id, 7, &["a", "b"][..].as_ref(), false, &first),
            (type_id, 7, &["a", "c"][..].as_ref(), false, &second),
            (type_id, 7, &["a", "b"][..].as_ref(), true, &read),
            (type_id, 8, &["a", "b"][..].as_ref(), false, &alternate_metadata),
            (TypeId::of::<u8>(), 7, &["a", "b"][..].as_ref(), false, &alternate_type),
        ] {
            let actual = cache
                .get_borrowed(test_context_id(), lookup_type, address, segments, write)
                .expect("exact colliding key");
            assert!(Arc::ptr_eq(&actual, expected));
        }
        assert_eq!(cache.len(), 5);
    }

    /// Inserting past the bound evicts the earliest key and allows
    /// recompilation.
    #[test]
    fn test_path_cache_evicts_earliest_success_after_256_entries() {
        let mut cache = PathCache {
            forced_hash: Some(1),
            ..PathCache::default()
        };
        let first_key = PathCacheKey::new(test_context_id(), TypeId::of::<usize>(), 1, &["path-0"], false);
        let first = cache.insert_or_existing(first_key.clone(), Arc::new(0));
        let second_key = PathCacheKey::new(test_context_id(), TypeId::of::<usize>(), 1, &["path-1"], false);
        let second = cache.insert_or_existing(second_key.clone(), Arc::new(1));

        let mut newest = None;
        for index in 2..=CACHE_CAPACITY {
            let segment = format!("path-{index}");
            let key = PathCacheKey::new(test_context_id(), TypeId::of::<usize>(), 1, &[&segment], false);
            let retained = cache.insert_or_existing(key, Arc::new(index));
            if index == CACHE_CAPACITY {
                newest = Some(retained);
            }
            assert!(cache.len() <= CACHE_CAPACITY);
        }

        assert!(cache.get(&first_key).is_none(), "the oldest key should be evicted");
        assert!(Arc::ptr_eq(
            &second,
            &cache.get(&second_key).expect("second colliding key remains")
        ));
        let newest_key = PathCacheKey::new(test_context_id(), TypeId::of::<usize>(), 1, &["path-256"], false);
        assert!(Arc::ptr_eq(
            &newest.expect("257th insertion is retained"),
            &cache.get(&newest_key).expect("newest colliding key remains")
        ));
        let replacement = cache.insert_or_existing(first_key, Arc::new(999));
        assert!(!Arc::ptr_eq(&first, &replacement));
        assert_eq!(*replacement, 999);
        assert_eq!(cache.len(), CACHE_CAPACITY);
    }

    /// A recently read hot path survives one more cold insertion at capacity.
    #[test]
    fn test_path_cache_retains_recently_read_hot_path() {
        let mut cache = PathCache::default();
        let type_id = TypeId::of::<usize>();
        let hot_key = PathCacheKey::new(test_context_id(), type_id, 1, &["hot"], false);
        let hot = cache.insert_or_existing(hot_key.clone(), Arc::new(0));
        for address in 2..=CACHE_CAPACITY {
            cache.insert_or_existing(
                PathCacheKey::new(test_context_id(), type_id, address, &["cold"], false),
                Arc::new(address),
            );
        }

        let hit = cache
            .get_borrowed(test_context_id(), type_id, 1, &["hot"], false)
            .expect("hot path remains cached before eviction");
        assert!(Arc::ptr_eq(&hot, &hit));
        cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, CACHE_CAPACITY + 1, &["cold"], false),
            Arc::new(CACHE_CAPACITY + 1),
        );

        assert!(Arc::ptr_eq(
            &hot,
            &cache.get(&hot_key).expect("recent hot path remains cached")
        ));
        assert!(
            cache
                .get_borrowed(test_context_id(), type_id, 2, &["cold"], false)
                .is_none()
        );
        assert_eq!(cache.len(), CACHE_CAPACITY);
    }

    /// Owned-key reads also keep the returned entry newer than cold entries.
    #[test]
    fn test_path_cache_owned_lookup_refreshes_recency() {
        let mut cache = PathCache::default();
        let type_id = TypeId::of::<usize>();
        let hot_key = PathCacheKey::new(test_context_id(), type_id, 1, &["hot"], false);
        let hot = cache.insert_or_existing(hot_key.clone(), Arc::new(0));
        for address in 2..=CACHE_CAPACITY {
            cache.insert_or_existing(
                PathCacheKey::new(test_context_id(), type_id, address, &["cold"], false),
                Arc::new(address),
            );
        }
        assert!(Arc::ptr_eq(&hot, &cache.get(&hot_key).expect("owned-key hit")));
        cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, CACHE_CAPACITY + 1, &["cold"], false),
            Arc::new(0),
        );
        assert!(Arc::ptr_eq(
            &hot,
            &cache.get(&hot_key).expect("recent owned-key hit remains")
        ));
    }

    /// A racing duplicate insertion counts as use of the retained value.
    #[test]
    fn test_path_cache_duplicate_insert_refreshes_recency() {
        let mut cache = PathCache::default();
        let type_id = TypeId::of::<usize>();
        let hot_key = PathCacheKey::new(test_context_id(), type_id, 1, &["hot"], false);
        let hot = cache.insert_or_existing(hot_key.clone(), Arc::new(0));
        for address in 2..=CACHE_CAPACITY {
            cache.insert_or_existing(
                PathCacheKey::new(test_context_id(), type_id, address, &["cold"], false),
                Arc::new(address),
            );
        }
        assert!(Arc::ptr_eq(
            &hot,
            &cache.insert_or_existing(hot_key.clone(), Arc::new(999))
        ));
        cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, CACHE_CAPACITY + 1, &["cold"], false),
            Arc::new(0),
        );
        assert!(Arc::ptr_eq(&hot, &cache.get(&hot_key).expect("duplicate hit remains")));
    }

    /// Counter overflow preserves the relative age of entries during eviction.
    #[test]
    fn test_path_cache_access_counter_overflow_preserves_recency() {
        let mut cache = PathCache::default();
        let type_id = TypeId::of::<usize>();
        let hot_key = PathCacheKey::new(test_context_id(), type_id, 1, &["hot"], false);
        let hot = cache.insert_or_existing(hot_key.clone(), Arc::new(0));
        for address in 2..=CACHE_CAPACITY {
            cache.insert_or_existing(
                PathCacheKey::new(test_context_id(), type_id, address, &["cold"], false),
                Arc::new(address),
            );
        }
        cache.next_access = u64::MAX;
        assert!(Arc::ptr_eq(
            &hot,
            &cache
                .get_borrowed(test_context_id(), type_id, 1, &["hot"], false)
                .expect("hot hit")
        ));
        cache.insert_or_existing(
            PathCacheKey::new(test_context_id(), type_id, CACHE_CAPACITY + 1, &["cold"], false),
            Arc::new(0),
        );

        assert!(Arc::ptr_eq(
            &hot,
            &cache.get(&hot_key).expect("hot path survives overflow")
        ));
        assert!(
            cache
                .get_borrowed(test_context_id(), type_id, 2, &["cold"], false)
                .is_none()
        );
        assert_eq!(cache.len(), CACHE_CAPACITY);
    }
}
