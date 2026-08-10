// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::TextFormat;
use super::TextRepertoire;

/// Text constraints that apply to a string field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextConstraint {
    /// The minimum number of Unicode scalar values, if constrained.
    min_chars: Option<u32>,
    /// The maximum number of Unicode scalar values, if constrained.
    max_chars: Option<u32>,
    /// The minimum UTF-8 byte length, if constrained.
    min_bytes: Option<u32>,
    /// The maximum UTF-8 byte length, if constrained.
    max_bytes: Option<u32>,
    /// The permitted character repertoire.
    repertoire: TextRepertoire,
    /// Whether whitespace-only values are forbidden.
    non_blank: bool,
    /// The required semantic text format, if any.
    format: Option<TextFormat>,
}

impl TextConstraint {
    /// Creates text constraints from character, byte, repertoire, and format
    /// limits.
    ///
    /// # Parameters
    ///
    /// * `min_chars` - The optional minimum number of Unicode scalar values.
    /// * `max_chars` - The optional maximum number of Unicode scalar values.
    /// * `min_bytes` - The optional minimum UTF-8 byte length.
    /// * `max_bytes` - The optional maximum UTF-8 byte length.
    /// * `repertoire` - The permitted character repertoire.
    /// * `non_blank` - Whether whitespace-only values are forbidden.
    /// * `format` - The optional required semantic text format.
    ///
    /// # Panics
    ///
    /// Panics when either supplied minimum exceeds its corresponding maximum.
    ///
    /// # Returns
    ///
    /// Text constraints containing the supplied limits and policies.
    #[must_use]
    pub const fn new(
        min_chars: Option<u32>,
        max_chars: Option<u32>,
        min_bytes: Option<u32>,
        max_bytes: Option<u32>,
        repertoire: TextRepertoire,
        non_blank: bool,
        format: Option<TextFormat>,
    ) -> Self {
        if let (Some(min_chars), Some(max_chars)) = (min_chars, max_chars) {
            assert!(
                min_chars <= max_chars,
                "minimum character count cannot exceed maximum character count"
            );
        }
        if let (Some(min_bytes), Some(max_bytes)) = (min_bytes, max_bytes) {
            assert!(
                min_bytes <= max_bytes,
                "minimum byte count cannot exceed maximum byte count"
            );
        }
        Self {
            min_chars,
            max_chars,
            min_bytes,
            max_bytes,
            repertoire,
            non_blank,
            format,
        }
    }

    /// Returns the minimum number of Unicode scalar values, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the minimum scalar-value count when constrained; otherwise,
    /// `None`.
    #[must_use]
    #[inline(always)]
    pub const fn min_chars(self) -> Option<u32> {
        self.min_chars
    }

    /// Returns the maximum number of Unicode scalar values, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the maximum scalar-value count when constrained; otherwise,
    /// `None`.
    #[must_use]
    #[inline(always)]
    pub const fn max_chars(self) -> Option<u32> {
        self.max_chars
    }

    /// Returns the minimum UTF-8 byte length, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the minimum byte length when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn min_bytes(self) -> Option<u32> {
        self.min_bytes
    }

    /// Returns the maximum UTF-8 byte length, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the maximum byte length when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn max_bytes(self) -> Option<u32> {
        self.max_bytes
    }

    /// Returns the permitted character repertoire.
    ///
    /// # Returns
    ///
    /// The permitted character repertoire.
    #[must_use]
    #[inline(always)]
    pub const fn repertoire(self) -> TextRepertoire {
        self.repertoire
    }

    /// Returns whether whitespace-only values are forbidden.
    ///
    /// # Returns
    ///
    /// `true` when whitespace-only values are forbidden; otherwise, `false`.
    #[must_use]
    #[inline(always)]
    pub const fn is_non_blank(self) -> bool {
        self.non_blank
    }

    /// Returns the required semantic text format, if any.
    ///
    /// # Returns
    ///
    /// `Some` with the required format when one is configured; otherwise,
    /// `None`.
    #[must_use]
    #[inline(always)]
    pub const fn format(self) -> Option<TextFormat> {
        self.format
    }
}
