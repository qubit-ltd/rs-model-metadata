// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::CollectionItemsAdapter;
use super::ItemEqAdapter;
use super::MapLenAdapter;

/// Safe, read-only operations available for a declared collection field.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::CollectionOps;
///
/// let operations = CollectionOps::new(None, None);
/// assert!(operations.map_len().is_none());
/// ```
#[derive(Clone, Copy, Debug)]
pub struct CollectionOps {
    /// Optional adapter that reads a map length.
    map_len: Option<MapLenAdapter>,
    /// Optional adapter that compares exact collection element types.
    item_eq: Option<ItemEqAdapter>,
    /// Optional adapter that iterates borrowed collection members.
    collection_items: Option<CollectionItemsAdapter>,
}

impl CollectionOps {
    /// Creates an operation set. Generated adapters are tied to exact Rust
    /// types.
    ///
    /// # Parameters
    ///
    /// * `map_len` - Optional adapter for reading a map's length.
    /// * `item_eq` - Optional adapter for comparing collection elements.
    ///
    /// # Returns
    ///
    /// The collection operation set containing the supplied adapters.
    #[must_use]
    #[inline]
    pub const fn new(map_len: Option<MapLenAdapter>, item_eq: Option<ItemEqAdapter>) -> Self {
        Self {
            map_len,
            item_eq,
            collection_items: None,
        }
    }

    /// Adds the exact-type borrowing adapter used by executable selectors.
    ///
    /// # Parameters
    ///
    /// * `collection_items` - Adapter that borrows members from the exact
    ///   collection value.
    ///
    /// # Returns
    ///
    /// This operation set with collection member iteration enabled.
    #[doc(hidden)]
    #[must_use]
    pub const fn with_items(mut self, collection_items: CollectionItemsAdapter) -> Self {
        self.collection_items = Some(collection_items);
        self
    }

    /// Returns the optional map length adapter.
    ///
    /// # Returns
    ///
    /// `Some` when a map-length adapter is configured, or `None` otherwise.
    #[inline]
    pub const fn map_len(&self) -> Option<MapLenAdapter> {
        self.map_len
    }

    /// Returns the optional element equality adapter.
    ///
    /// # Returns
    ///
    /// `Some` when an element-equality adapter is configured, or `None`
    /// otherwise.
    #[inline]
    pub const fn item_eq(&self) -> Option<ItemEqAdapter> {
        self.item_eq
    }

    /// Returns the exact-type borrowing adapter for sequence or map members.
    ///
    /// # Returns
    ///
    /// `Some` when a collection member adapter is configured, or `None`
    /// otherwise.
    #[inline]
    pub const fn collection_items(&self) -> Option<CollectionItemsAdapter> {
        self.collection_items
    }
}
