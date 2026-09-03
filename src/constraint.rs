// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Strongly typed value objects for field constraints.
// qubit-style: allow type-file-name

mod allowed_chars;
mod decimal_constraint;
mod decimal_semantic;
mod map_constraint;
mod rounding_mode;
mod sequence_constraint;
mod temporal_constraint;
mod temporal_precision;
mod text_constraint;
mod text_format;

pub use self::allowed_chars::AllowedChars;
pub use self::decimal_constraint::DecimalConstraint;
pub use self::decimal_semantic::DecimalSemantic;
pub use self::map_constraint::MapConstraint;
pub use self::rounding_mode::RoundingMode;
pub use self::sequence_constraint::SequenceConstraint;
pub use self::temporal_constraint::TemporalConstraint;
pub use self::temporal_constraint::TemporalConstraint as TimeConstraint;
pub use self::temporal_precision::TemporalPrecision;
pub use self::text_constraint::TextConstraint;
pub use self::text_format::TextFormat;

/// A standard constraint occurrence on a field or selector.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::AllowedChars;
/// use qubit_model_metadata::ConstraintMetadata;
/// use qubit_model_metadata::TextConstraint;
///
/// let constraint = ConstraintMetadata::Text(TextConstraint::new(
///     None,
///     Some(80),
///     None,
///     None,
///     AllowedChars::Unicode,
///     false,
///     None,
/// ));
/// assert!(matches!(constraint, ConstraintMetadata::Text(_)));
/// ```
#[derive(Clone, Copy, Debug)]
pub enum ConstraintMetadata {
    /// Text constraints.
    Text(TextConstraint),
    /// Decimal or money constraints.
    Decimal(DecimalConstraint),
    /// Time constraints.
    Time(TimeConstraint),
    /// Sequence constraints.
    Sequence(SequenceConstraint),
    /// Map constraints.
    Map(MapConstraint),
}
