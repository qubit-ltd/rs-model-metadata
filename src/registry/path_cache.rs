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
}

/// Stores successful paths in insertion order and evicts the oldest entry.
pub(super) struct PathCache<T> {
    hash_builder: RandomState,
    entries: HashMap<u64, Vec<CacheEntry<T>>>,
    insertion_order: VecDeque<PathCacheKey>,
    #[cfg(test)]
    forced_hash: Option<u64>,
}

impl<T> std::fmt::Debug for PathCache<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PathCache")
            .field("len", &self.insertion_order.len())
            .finish()
    }
}

impl<T> Default for PathCache<T> {
    fn default() -> Self {
        Self {
            hash_builder: RandomState::new(),
            entries: HashMap::new(),
            insertion_order: VecDeque::new(),
            #[cfg(test)]
            forced_hash: None,
        }
    }
}

impl<T> PathCache<T> {
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
    /// A miss returns `None` and does not change insertion order.
    pub(super) fn get_borrowed(
        &self,
        type_id: TypeId,
        metadata_address: usize,
        segments: &[&str],
        write: bool,
    ) -> Option<Arc<T>> {
        let hash = self.hash_borrowed(type_id, metadata_address, segments, write);
        self.entries
            .get(&hash)?
            .iter()
            .find(|entry| entry.key.matches_borrowed(type_id, metadata_address, segments, write))
            .map(|entry| Arc::clone(&entry.compiled))
    }

    /// Clones a cached value for an owned key, or returns `None` on a miss.
    pub(super) fn get(&self, key: &PathCacheKey) -> Option<Arc<T>> {
        let hash = self.hash_owned(key);
        self.entries
            .get(&hash)?
            .iter()
            .find(|entry| entry.key == *key)
            .map(|entry| Arc::clone(&entry.compiled))
    }

    /// Keeps an existing value on a racing insert, or adds the new success.
    ///
    /// The oldest inserted key is evicted once this registry exceeds 256 paths.
    pub(super) fn insert_or_existing(&mut self, key: PathCacheKey, compiled: Arc<T>) -> Arc<T> {
        if let Some(existing) = self.get(&key) {
            return existing;
        }
        let hash = self.hash_owned(&key);
        self.insertion_order.push_back(key.clone());
        self.entries.entry(hash).or_default().push(CacheEntry {
            key,
            compiled: Arc::clone(&compiled),
        });
        if self.insertion_order.len() > CACHE_CAPACITY {
            let oldest = self
                .insertion_order
                .pop_front()
                .expect("cache insertion has an oldest key");
            let oldest_hash = self.hash_owned(&oldest);
            if let Some(bucket) = self.entries.get_mut(&oldest_hash) {
                bucket.retain(|entry| entry.key != oldest);
                if bucket.is_empty() {
                    self.entries.remove(&oldest_hash);
                }
            }
        }
        compiled
    }

    /// Returns the count of retained successes for the capacity regression.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.insertion_order.len()
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
}
