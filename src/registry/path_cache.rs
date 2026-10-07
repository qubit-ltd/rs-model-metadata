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
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::hash::RandomState;
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

    /// Compares an owned key against caller-borrowed path segments.
    fn matches_borrowed(&self, type_id: TypeId, metadata_address: usize, segments: &[&str], write: bool) -> bool {
        self.type_id == type_id
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

/// One full key and its compiled path inside a hash bucket.
struct CacheEntry<T> {
    key: PathCacheKey,
    compiled: Arc<T>,
    last_access: u64,
}

/// Stores successful paths and evicts the least recently used entry.
pub(super) struct PathCache<T> {
    hash_builder: RandomState,
    entries: HashMap<u64, Vec<CacheEntry<T>>>,
    len: usize,
    next_access: u64,
    #[cfg(test)]
    forced_hash: Option<u64>,
}

impl<T> std::fmt::Debug for PathCache<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("PathCache").field("len", &self.len).finish()
    }
}

impl<T> Default for PathCache<T> {
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
    fn hash_borrowed(&self, type_id: TypeId, metadata_address: usize, segments: &[&str], write: bool) -> u64 {
        #[cfg(test)]
        if let Some(hash) = self.forced_hash {
            return hash;
        }
        let mut hasher = self.hash_builder.build_hasher();
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
    fn hash_owned(&self, key: &PathCacheKey) -> u64 {
        #[cfg(test)]
        if let Some(hash) = self.forced_hash {
            return hash;
        }
        let mut hasher = self.hash_builder.build_hasher();
        key.type_id.hash(&mut hasher);
        key.metadata_address.hash(&mut hasher);
        key.write.hash(&mut hasher);
        key.segments.len().hash(&mut hasher);
        for segment in &key.segments {
            segment.hash(&mut hasher);
        }
        hasher.finish()
    }

    /// Clones a cached value on an exact borrowed-key hit without allocating.
    /// A miss returns `None` without changing any retained entry's recency.
    pub(super) fn get_borrowed(
        &mut self,
        type_id: TypeId,
        metadata_address: usize,
        segments: &[&str],
        write: bool,
    ) -> Option<Arc<T>> {
        let hash = self.hash_borrowed(type_id, metadata_address, segments, write);
        self.ensure_access_stamp();
        let entry = self
            .entries
            .get_mut(&hash)?
            .iter_mut()
            .find(|entry| entry.key.matches_borrowed(type_id, metadata_address, segments, write))?;
        entry.last_access = self.next_access;
        self.next_access += 1;
        Some(Arc::clone(&entry.compiled))
    }

    /// Clones a cached value for an owned key, or returns `None` on a miss.
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
    pub(super) fn insert_or_existing(&mut self, key: PathCacheKey, compiled: Arc<T>) -> Arc<T> {
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
}

#[cfg(test)]
mod tests {
    use std::any::TypeId;
    use std::sync::Arc;

    use super::CACHE_CAPACITY;
    use super::PathCache;
    use super::PathCacheKey;

    /// A borrowed lookup returns the retained allocation for the exact path.
    #[test]
    fn test_path_cache_borrowed_lookup_reuses_value() {
        let mut cache = PathCache::default();
        let type_id = TypeId::of::<usize>();
        let value = cache.insert_or_existing(PathCacheKey::new(type_id, 7, &["middle", "leaf"], false), Arc::new(42));

        let hit = cache
            .get_borrowed(type_id, 7, &["middle", "leaf"], false)
            .expect("borrowed path should hit");
        assert!(Arc::ptr_eq(&value, &hit));
        assert!(cache.get_borrowed(type_id, 7, &["middle", "other"], false).is_none());
        assert!(cache.get_borrowed(type_id, 8, &["middle", "leaf"], false).is_none());
        assert!(cache.get_borrowed(type_id, 7, &["middle", "leaf"], true).is_none());
    }

    /// A second insertion of the same key keeps the first compiled Arc.
    #[test]
    fn test_path_cache_duplicate_insert_keeps_existing_value() {
        let mut cache = PathCache::default();
        let key = PathCacheKey::new(TypeId::of::<usize>(), 7, &["middle", "leaf"], false);
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
        let first = cache.insert_or_existing(PathCacheKey::new(type_id, 7, &["a", "b"], false), Arc::new(1));
        let second = cache.insert_or_existing(PathCacheKey::new(type_id, 7, &["a", "c"], false), Arc::new(2));
        let read = cache.insert_or_existing(PathCacheKey::new(type_id, 7, &["a", "b"], true), Arc::new(3));
        let alternate_metadata =
            cache.insert_or_existing(PathCacheKey::new(type_id, 8, &["a", "b"], false), Arc::new(4));
        let alternate_type = cache.insert_or_existing(
            PathCacheKey::new(TypeId::of::<u8>(), 7, &["a", "b"], false),
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
                .get_borrowed(lookup_type, address, segments, write)
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
        let first_key = PathCacheKey::new(TypeId::of::<usize>(), 1, &["path-0"], false);
        let first = cache.insert_or_existing(first_key.clone(), Arc::new(0));
        let second_key = PathCacheKey::new(TypeId::of::<usize>(), 1, &["path-1"], false);
        let second = cache.insert_or_existing(second_key.clone(), Arc::new(1));

        let mut newest = None;
        for index in 2..=CACHE_CAPACITY {
            let segment = format!("path-{index}");
            let key = PathCacheKey::new(TypeId::of::<usize>(), 1, &[&segment], false);
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
        let newest_key = PathCacheKey::new(TypeId::of::<usize>(), 1, &["path-256"], false);
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
        let hot_key = PathCacheKey::new(type_id, 1, &["hot"], false);
        let hot = cache.insert_or_existing(hot_key.clone(), Arc::new(0));
        for address in 2..=CACHE_CAPACITY {
            cache.insert_or_existing(PathCacheKey::new(type_id, address, &["cold"], false), Arc::new(address));
        }

        let hit = cache
            .get_borrowed(type_id, 1, &["hot"], false)
            .expect("hot path remains cached before eviction");
        assert!(Arc::ptr_eq(&hot, &hit));
        cache.insert_or_existing(
            PathCacheKey::new(type_id, CACHE_CAPACITY + 1, &["cold"], false),
            Arc::new(CACHE_CAPACITY + 1),
        );

        assert!(Arc::ptr_eq(
            &hot,
            &cache.get(&hot_key).expect("recent hot path remains cached")
        ));
        assert!(cache.get_borrowed(type_id, 2, &["cold"], false).is_none());
        assert_eq!(cache.len(), CACHE_CAPACITY);
    }

    /// Owned-key reads also keep the returned entry newer than cold entries.
    #[test]
    fn test_path_cache_owned_lookup_refreshes_recency() {
        let mut cache = PathCache::default();
        let type_id = TypeId::of::<usize>();
        let hot_key = PathCacheKey::new(type_id, 1, &["hot"], false);
        let hot = cache.insert_or_existing(hot_key.clone(), Arc::new(0));
        for address in 2..=CACHE_CAPACITY {
            cache.insert_or_existing(PathCacheKey::new(type_id, address, &["cold"], false), Arc::new(address));
        }
        assert!(Arc::ptr_eq(&hot, &cache.get(&hot_key).expect("owned-key hit")));
        cache.insert_or_existing(
            PathCacheKey::new(type_id, CACHE_CAPACITY + 1, &["cold"], false),
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
        let hot_key = PathCacheKey::new(type_id, 1, &["hot"], false);
        let hot = cache.insert_or_existing(hot_key.clone(), Arc::new(0));
        for address in 2..=CACHE_CAPACITY {
            cache.insert_or_existing(PathCacheKey::new(type_id, address, &["cold"], false), Arc::new(address));
        }
        assert!(Arc::ptr_eq(
            &hot,
            &cache.insert_or_existing(hot_key.clone(), Arc::new(999))
        ));
        cache.insert_or_existing(
            PathCacheKey::new(type_id, CACHE_CAPACITY + 1, &["cold"], false),
            Arc::new(0),
        );
        assert!(Arc::ptr_eq(&hot, &cache.get(&hot_key).expect("duplicate hit remains")));
    }

    /// Counter overflow preserves the relative age of entries during eviction.
    #[test]
    fn test_path_cache_access_counter_overflow_preserves_recency() {
        let mut cache = PathCache::default();
        let type_id = TypeId::of::<usize>();
        let hot_key = PathCacheKey::new(type_id, 1, &["hot"], false);
        let hot = cache.insert_or_existing(hot_key.clone(), Arc::new(0));
        for address in 2..=CACHE_CAPACITY {
            cache.insert_or_existing(PathCacheKey::new(type_id, address, &["cold"], false), Arc::new(address));
        }
        cache.next_access = u64::MAX;
        assert!(Arc::ptr_eq(
            &hot,
            &cache.get_borrowed(type_id, 1, &["hot"], false).expect("hot hit")
        ));
        cache.insert_or_existing(
            PathCacheKey::new(type_id, CACHE_CAPACITY + 1, &["cold"], false),
            Arc::new(0),
        );

        assert!(Arc::ptr_eq(
            &hot,
            &cache.get(&hot_key).expect("hot path survives overflow")
        ));
        assert!(cache.get_borrowed(type_id, 2, &["cold"], false).is_none());
        assert_eq!(cache.len(), CACHE_CAPACITY);
    }
}
