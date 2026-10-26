// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Declares the ordering of one key component.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::KeyPartMetadata;
///
/// let part = KeyPartMetadata::new(0);
/// assert_eq!(part.order(), 0);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KeyPartMetadata {
    /// The zero-based position within the composite key.
    order: usize,
}

impl KeyPartMetadata {
    /// Creates key-part metadata.
    ///
    /// # Parameters
    ///
    /// * `order` - Zero-based position within the composite key.
    ///
    /// # Returns
    ///
    /// Metadata for that component position.
    #[must_use]
    #[inline]
    pub const fn new(order: usize) -> Self {
        Self { order }
    }

    /// Returns the zero-based component order.
    ///
    /// # Returns
    ///
    /// The component's position in the composite key.
    #[must_use]
    #[inline]
    pub const fn order(&self) -> usize {
        self.order
    }
}
