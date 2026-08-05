// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for constraint construction invariants.

use qubit_model_metadata::{
    DecimalConstraint,
    DecimalSemantic,
    MapConstraint,
    RoundingMode,
    SequenceConstraint,
    TemporalConstraint,
    TemporalNormalization,
    TemporalPrecision,
    TextConstraint,
    TextFormat,
    TextRepertoire,
};

const VALID_TEXT: TextConstraint = TextConstraint::new(
    Some(1),
    Some(8),
    Some(1),
    Some(16),
    TextRepertoire::Ascii,
    true,
    Some(TextFormat::Email),
);
const VALID_MOBILE_TEXT: TextConstraint = TextConstraint::new(
    None,
    None,
    None,
    None,
    TextRepertoire::Unicode,
    false,
    Some(TextFormat::Mobile),
);
const VALID_SEQUENCE: SequenceConstraint =
    SequenceConstraint::new(Some(1), Some(8), true);
const VALID_MAP: MapConstraint = MapConstraint::new(Some(1), Some(8));
const VALID_DECIMAL: DecimalConstraint = DecimalConstraint::new(
    Some(8),
    3,
    RoundingMode::HalfEven,
    DecimalSemantic::Number,
);
const VALID_TEMPORAL: TemporalConstraint = TemporalConstraint::new(
    TemporalPrecision::Millisecond,
    TemporalNormalization::Utc,
);

#[test]
fn test_constraint_constructors_remain_const_compatible() {
    assert_eq!(VALID_TEXT.min_chars(), Some(1));
    assert_eq!(VALID_TEXT.max_chars(), Some(8));
    assert_eq!(VALID_TEXT.min_bytes(), Some(1));
    assert_eq!(VALID_TEXT.max_bytes(), Some(16));
    assert_eq!(VALID_TEXT.repertoire(), TextRepertoire::Ascii);
    assert!(VALID_TEXT.is_non_blank());
    assert_eq!(VALID_TEXT.format(), Some(TextFormat::Email));
    assert_eq!(VALID_MOBILE_TEXT.format(), Some(TextFormat::Mobile));

    assert_eq!(VALID_SEQUENCE.min_items(), Some(1));
    assert_eq!(VALID_SEQUENCE.max_items(), Some(8));
    assert!(VALID_SEQUENCE.has_unique_items());

    assert_eq!(VALID_MAP.min_entries(), Some(1));
    assert_eq!(VALID_MAP.max_entries(), Some(8));

    assert_eq!(VALID_TEMPORAL.precision(), TemporalPrecision::Millisecond);
    assert_eq!(VALID_TEMPORAL.normalization(), TemporalNormalization::Utc);

    assert_eq!(VALID_DECIMAL.precision(), Some(8));
    assert_eq!(VALID_DECIMAL.scale(), 3);
    assert_eq!(VALID_DECIMAL.rounding(), RoundingMode::HalfEven);
    assert_eq!(VALID_DECIMAL.semantic(), DecimalSemantic::Number);
}

#[test]
#[should_panic(
    expected = "minimum character count cannot exceed maximum character count"
)]
fn test_text_constraint_rejects_reversed_character_range() {
    let _ = TextConstraint::new(
        Some(2),
        Some(1),
        None,
        None,
        TextRepertoire::Unicode,
        false,
        None,
    );
}

#[test]
#[should_panic(
    expected = "minimum byte count cannot exceed maximum byte count"
)]
fn test_text_constraint_rejects_reversed_byte_range() {
    let _ = TextConstraint::new(
        None,
        None,
        Some(2),
        Some(1),
        TextRepertoire::Unicode,
        false,
        None,
    );
}

#[test]
#[should_panic(
    expected = "minimum item count cannot exceed maximum item count"
)]
fn test_sequence_constraint_rejects_reversed_range() {
    let _ = SequenceConstraint::new(Some(2), Some(1), false);
}

#[test]
#[should_panic(
    expected = "minimum entry count cannot exceed maximum entry count"
)]
fn test_map_constraint_rejects_reversed_range() {
    let _ = MapConstraint::new(Some(2), Some(1));
}

#[test]
#[should_panic(expected = "decimal scale cannot exceed precision")]
fn test_decimal_constraint_rejects_scale_above_precision() {
    let _ = DecimalConstraint::new(
        Some(2),
        3,
        RoundingMode::HalfEven,
        DecimalSemantic::Number,
    );
}
