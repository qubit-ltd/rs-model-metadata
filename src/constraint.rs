// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow multiple-public-types
//! Strongly typed value objects for field constraints.

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

/// The repertoire accepted by a text constraint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextRepertoire {
    /// All Unicode scalar values are accepted.
    #[default]
    Unicode,
    /// Only ASCII characters are accepted.
    Ascii,
}

/// A semantic format accepted by a text constraint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFormat {
    /// An email address.
    Email,
    /// A URI.
    Uri,
    /// A UUID string.
    Uuid,
}

/// Constraints that apply to an ordered sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SequenceConstraint {
    /// The minimum number of items, if constrained.
    min_items: Option<u32>,
    /// The maximum number of items, if constrained.
    max_items: Option<u32>,
    /// Whether sequence elements must be unique.
    unique_items: bool,
}

impl SequenceConstraint {
    /// Creates sequence constraints from item limits and uniqueness semantics.
    ///
    /// # Parameters
    ///
    /// * `min_items` - The optional minimum item count.
    /// * `max_items` - The optional maximum item count.
    /// * `unique_items` - Whether sequence elements must be unique.
    ///
    /// # Panics
    ///
    /// Panics when the supplied minimum item count exceeds the maximum.
    ///
    /// # Returns
    ///
    /// Sequence constraints containing the supplied limits and uniqueness
    /// policy.
    #[must_use]
    pub const fn new(
        min_items: Option<u32>,
        max_items: Option<u32>,
        unique_items: bool,
    ) -> Self {
        if let (Some(min_items), Some(max_items)) = (min_items, max_items) {
            assert!(
                min_items <= max_items,
                "minimum item count cannot exceed maximum item count"
            );
        }
        Self {
            min_items,
            max_items,
            unique_items,
        }
    }

    /// Returns the minimum number of items, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the minimum item count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn min_items(self) -> Option<u32> {
        self.min_items
    }

    /// Returns the maximum number of items, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the maximum item count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn max_items(self) -> Option<u32> {
        self.max_items
    }

    /// Returns whether sequence elements must be unique.
    ///
    /// # Returns
    ///
    /// `true` when sequence elements must be unique; otherwise, `false`.
    #[must_use]
    #[inline(always)]
    pub const fn has_unique_items(self) -> bool {
        self.unique_items
    }
}

/// Constraints that apply to a map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapConstraint {
    /// The minimum number of entries, if constrained.
    min_entries: Option<u32>,
    /// The maximum number of entries, if constrained.
    max_entries: Option<u32>,
}

impl MapConstraint {
    /// Creates map constraints from entry limits.
    ///
    /// # Parameters
    ///
    /// * `min_entries` - The optional minimum entry count.
    /// * `max_entries` - The optional maximum entry count.
    ///
    /// # Panics
    ///
    /// Panics when the supplied minimum entry count exceeds the maximum.
    ///
    /// # Returns
    ///
    /// Map constraints containing the supplied entry limits.
    #[must_use]
    pub const fn new(
        min_entries: Option<u32>,
        max_entries: Option<u32>,
    ) -> Self {
        if let (Some(min_entries), Some(max_entries)) =
            (min_entries, max_entries)
        {
            assert!(
                min_entries <= max_entries,
                "minimum entry count cannot exceed maximum entry count"
            );
        }
        Self {
            min_entries,
            max_entries,
        }
    }

    /// Returns the minimum number of entries, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the minimum entry count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn min_entries(self) -> Option<u32> {
        self.min_entries
    }

    /// Returns the maximum number of entries, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the maximum entry count when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn max_entries(self) -> Option<u32> {
        self.max_entries
    }
}

/// Constraints for temporal values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalConstraint {
    /// The required temporal precision.
    precision: TemporalPrecision,
    /// The temporal normalization policy.
    normalization: TemporalNormalization,
}

impl TemporalConstraint {
    /// Creates temporal constraints from precision and normalization semantics.
    ///
    /// # Parameters
    ///
    /// * `precision` - The required temporal precision.
    /// * `normalization` - The temporal normalization policy.
    ///
    /// # Returns
    ///
    /// Temporal constraints containing the supplied precision and policy.
    #[must_use]
    #[inline(always)]
    pub const fn new(
        precision: TemporalPrecision,
        normalization: TemporalNormalization,
    ) -> Self {
        Self {
            precision,
            normalization,
        }
    }

    /// Returns the required temporal precision.
    ///
    /// # Returns
    ///
    /// The required temporal precision.
    #[must_use]
    #[inline(always)]
    pub const fn precision(self) -> TemporalPrecision {
        self.precision
    }

    /// Returns the temporal normalization policy.
    ///
    /// # Returns
    ///
    /// The temporal normalization policy.
    #[must_use]
    #[inline(always)]
    pub const fn normalization(self) -> TemporalNormalization {
        self.normalization
    }
}

/// The resolution retained for temporal values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalPrecision {
    /// Whole seconds.
    Second,
    /// Milliseconds.
    Millisecond,
    /// Microseconds.
    Microsecond,
    /// Nanoseconds.
    Nanosecond,
}

/// The normalization policy for temporal values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalNormalization {
    /// Preserve the value's supplied offset or timezone representation.
    Preserve,
    /// Normalize to UTC.
    Utc,
}

/// Constraints for decimal values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecimalConstraint {
    /// The total significant-digit precision, if constrained.
    precision: Option<u16>,
    /// The number of decimal places.
    scale: u16,
    /// The required rounding strategy.
    rounding: RoundingMode,
    /// The domain meaning of the decimal value.
    semantic: DecimalSemantic,
}

impl DecimalConstraint {
    /// Creates decimal constraints from precision, scale, rounding, and
    /// semantic meaning.
    ///
    /// # Parameters
    ///
    /// * `precision` - The optional total significant-digit precision.
    /// * `scale` - The number of decimal places.
    /// * `rounding` - The required rounding strategy.
    /// * `semantic` - The domain meaning of the decimal value.
    ///
    /// # Panics
    ///
    /// Panics when `scale` exceeds a supplied `precision`.
    ///
    /// # Returns
    ///
    /// Decimal constraints containing the supplied precision, scale, and
    /// policies.
    #[must_use]
    pub const fn new(
        precision: Option<u16>,
        scale: u16,
        rounding: RoundingMode,
        semantic: DecimalSemantic,
    ) -> Self {
        if let Some(precision) = precision {
            assert!(
                scale <= precision,
                "decimal scale cannot exceed precision"
            );
        }
        Self {
            precision,
            scale,
            rounding,
            semantic,
        }
    }

    /// Returns the total significant-digit precision, if constrained.
    ///
    /// # Returns
    ///
    /// `Some` with the precision when constrained; otherwise, `None`.
    #[must_use]
    #[inline(always)]
    pub const fn precision(self) -> Option<u16> {
        self.precision
    }

    /// Returns the number of decimal places.
    ///
    /// # Returns
    ///
    /// The number of decimal places.
    #[must_use]
    #[inline(always)]
    pub const fn scale(self) -> u16 {
        self.scale
    }

    /// Returns the required rounding strategy.
    ///
    /// # Returns
    ///
    /// The required rounding strategy.
    #[must_use]
    #[inline(always)]
    pub const fn rounding(self) -> RoundingMode {
        self.rounding
    }

    /// Returns whether the value is an ordinary number or money.
    ///
    /// # Returns
    ///
    /// The domain meaning of the decimal value.
    #[must_use]
    #[inline(always)]
    pub const fn semantic(self) -> DecimalSemantic {
        self.semantic
    }
}

/// A rounding strategy for decimal constraints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundingMode {
    /// Round toward zero.
    Down,
    /// Round away from zero.
    Up,
    /// Round to the nearest value, with halves rounded away from zero.
    HalfUp,
    /// Round to the nearest value, with halves rounded to even.
    HalfEven,
}

/// The domain meaning of a decimal value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecimalSemantic {
    /// A general-purpose decimal number.
    Number,
    /// A monetary amount.
    Money,
}
