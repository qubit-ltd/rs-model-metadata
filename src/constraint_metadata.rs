// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One declared standard constraint occurrence.

use crate::constraint::DecimalConstraint;
use crate::constraint::MapConstraint;
use crate::constraint::SequenceConstraint;
use crate::constraint::TextConstraint;
use crate::constraint::TimeConstraint;

/// A standard constraint occurrence on a field or selector.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::AllowedChars;
/// use qubit_model_metadata::metadata::ConstraintMetadata;
/// use qubit_model_metadata::metadata::TextConstraint;
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
    Text(
        /// Length, format, and allowed-character policies for text values.
        TextConstraint,
    ),
    /// Decimal or money constraints.
    Decimal(
        /// Precision, range, and rounding policies for decimal values.
        DecimalConstraint,
    ),
    /// Time constraints.
    Time(
        /// Precision and range policies for temporal values.
        TimeConstraint,
    ),
    /// Sequence constraints.
    Sequence(
        /// Length, uniqueness, and element policies for sequences.
        SequenceConstraint,
    ),
    /// Map constraints.
    Map(
        /// Entry-count and key/value policies for maps.
        MapConstraint,
    ),
}
