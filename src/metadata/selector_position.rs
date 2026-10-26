// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural positions selected by nested field semantics.

/// The structural position selected by nested field semantics.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::{SelectorMetadata, SelectorPosition};
///
/// let selector = SelectorMetadata::new(SelectorPosition::MapKey, &[], &[], None, None);
/// assert_eq!(selector.position(), SelectorPosition::MapKey);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorPosition {
    /// The element type of a sequence.
    Element,
    /// The key type of a map.
    MapKey,
    /// The value type of a map.
    MapValue,
}
