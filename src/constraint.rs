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
    min_chars: Option<u32>,
    max_chars: Option<u32>,
    min_bytes: Option<u32>,
    max_bytes: Option<u32>,
    repertoire: TextRepertoire,
    non_blank: bool,
    format: Option<TextFormat>,
}

impl TextConstraint {
    /// Creates text constraints from character, byte, repertoire, and format
    /// limits.
    ///
    /// # Panics
    ///
    /// Panics when either supplied minimum exceeds its corresponding maximum.
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
    #[must_use]
    pub const fn min_chars(self) -> Option<u32> {
        self.min_chars
    }

    /// Returns the maximum number of Unicode scalar values, if constrained.
    #[must_use]
    pub const fn max_chars(self) -> Option<u32> {
        self.max_chars
    }

    /// Returns the minimum UTF-8 byte length, if constrained.
    #[must_use]
    pub const fn min_bytes(self) -> Option<u32> {
        self.min_bytes
    }

    /// Returns the maximum UTF-8 byte length, if constrained.
    #[must_use]
    pub const fn max_bytes(self) -> Option<u32> {
        self.max_bytes
    }

    /// Returns the permitted character repertoire.
    #[must_use]
    pub const fn repertoire(self) -> TextRepertoire {
        self.repertoire
    }

    /// Returns whether whitespace-only values are forbidden.
    #[must_use]
    pub const fn is_non_blank(self) -> bool {
        self.non_blank
    }

    /// Returns the required semantic text format, if any.
    #[must_use]
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
    min_items: Option<u32>,
    max_items: Option<u32>,
    unique_items: bool,
}

impl SequenceConstraint {
    /// Creates sequence constraints from item limits and uniqueness semantics.
    ///
    /// # Panics
    ///
    /// Panics when the supplied minimum item count exceeds the maximum.
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
    #[must_use]
    pub const fn min_items(self) -> Option<u32> {
        self.min_items
    }

    /// Returns the maximum number of items, if constrained.
    #[must_use]
    pub const fn max_items(self) -> Option<u32> {
        self.max_items
    }

    /// Returns whether sequence elements must be unique.
    #[must_use]
    pub const fn has_unique_items(self) -> bool {
        self.unique_items
    }
}

/// Constraints that apply to a map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapConstraint {
    min_entries: Option<u32>,
    max_entries: Option<u32>,
}

impl MapConstraint {
    /// Creates map constraints from entry limits.
    ///
    /// # Panics
    ///
    /// Panics when the supplied minimum entry count exceeds the maximum.
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
    #[must_use]
    pub const fn min_entries(self) -> Option<u32> {
        self.min_entries
    }

    /// Returns the maximum number of entries, if constrained.
    #[must_use]
    pub const fn max_entries(self) -> Option<u32> {
        self.max_entries
    }
}

/// Constraints for temporal values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalConstraint {
    precision: TemporalPrecision,
    normalization: TemporalNormalization,
}

impl TemporalConstraint {
    /// Creates temporal constraints from precision and normalization semantics.
    #[must_use]
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
    #[must_use]
    pub const fn precision(self) -> TemporalPrecision {
        self.precision
    }

    /// Returns the temporal normalization policy.
    #[must_use]
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
    precision: Option<u16>,
    scale: u16,
    rounding: RoundingMode,
    semantic: DecimalSemantic,
}

impl DecimalConstraint {
    /// Creates decimal constraints from precision, scale, rounding, and
    /// semantic meaning.
    ///
    /// # Panics
    ///
    /// Panics when `scale` exceeds a supplied `precision`.
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
    #[must_use]
    pub const fn precision(self) -> Option<u16> {
        self.precision
    }

    /// Returns the number of decimal places.
    #[must_use]
    pub const fn scale(self) -> u16 {
        self.scale
    }

    /// Returns the required rounding strategy.
    #[must_use]
    pub const fn rounding(self) -> RoundingMode {
        self.rounding
    }

    /// Returns whether the value is an ordinary number or money.
    #[must_use]
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
