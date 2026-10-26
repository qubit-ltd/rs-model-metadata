// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use bitflags::bitflags;

bitflags! {
    /// Explains every declaration fact that makes a field indexed.
    ///
    /// # Examples
    ///
    /// ```
    /// use qubit_model_metadata::metadata::IndexingReasons;
    ///
    /// let reasons = IndexingReasons::EXPLICIT | IndexingReasons::IDENTIFIER;
    /// assert!(reasons.contains(IndexingReasons::EXPLICIT));
    /// ```
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct IndexingReasons: u8 {
        /// The source explicitly requested an index.
        const EXPLICIT = 0b0001;
        /// Identifier lookup requires an index.
        const IDENTIFIER = 0b0010;
        /// Uniqueness enforcement requires an index.
        const UNIQUE = 0b0100;
        /// Reference lookup requires an index.
        const REFERENCE = 0b1000;
    }
}
