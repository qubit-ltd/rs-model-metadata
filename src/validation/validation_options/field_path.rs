// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned field paths used to select model validation occurrences.

use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

/// An owned field path used to select model validation occurrences.
///
/// Matching uses complete field-name paths, ignoring collection indices. A
/// parent path does not select its descendants. The path constructor does not
/// validate names; at execution time, every selected path must match a bound
/// rule or the complete selection returns `InvalidSelection` before any getter
/// runs. Model-level rules have the empty field-name path: include an empty
/// segment sequence to select them explicitly when the plan has a model rule.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::validation::FieldPath;
/// use qubit_model_metadata::validation::ValidationOptions;
/// use qubit_model_metadata::validation::ValidationSelection;
///
/// let path = FieldPath::from_segments(["address", "city"]);
/// assert_eq!(path, FieldPath::new("address.city"));
/// assert_ne!(path, FieldPath::from_segments(["address.city"]));
/// let _options = ValidationOptions::builder()
///     .selection(ValidationSelection::Fields(vec![path]))
///     .build();
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct FieldPath {
    /// Complete field-name path; collection indices are not included.
    segments: Vec<String>,
}

impl Debug for FieldPath {
    /// Formats the path without revealing caller-supplied field names.
    ///
    /// The `segments` field is represented by a redaction marker, while
    /// formatter failures are returned unchanged.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FieldPath")
            .field("segments", &"<redacted>")
            .finish()
    }
}

impl Display for FieldPath {
    /// Writes a stable, non-sensitive message for an unmatched field path.
    ///
    /// The caller-supplied segments are omitted so diagnostics do not expose
    /// field names; formatter failures are returned unchanged.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("field path did not match validation rules")
    }
}

impl Error for FieldPath {}

impl FieldPath {
    /// Creates a path from dot-separated field names.
    ///
    /// # Parameters
    ///
    /// * `path` - Field names separated by literal dots. Whitespace and empty
    ///   segments are retained; this constructor does not validate names.
    ///
    /// # Returns
    ///
    /// An independently owned path containing each separated field name.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            segments: path.into().split('.').map(str::to_owned).collect(),
        }
    }

    /// Creates a path from individual field names.
    ///
    /// # Parameters
    ///
    /// * `segments` - Names in traversal order. Each item becomes an owned
    ///   string; dots inside an item are not split into further segments.
    ///
    /// # Returns
    ///
    /// An independently owned path preserving the supplied segment boundaries.
    /// An empty iterator produces an empty path. No graph lookup or name
    /// validation is performed.
    #[must_use]
    pub fn from_segments(segments: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            segments: segments.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the retained path segments in their original order.
    ///
    /// This can expose caller-supplied field names. Callers must decide
    /// whether those names are trusted before displaying or logging them.
    ///
    /// # Returns
    ///
    /// Returns the path segments in order, without joining or normalizing
    /// them.
    #[must_use]
    #[inline]
    pub fn segments(&self) -> &[String] {
        &self.segments
    }
}
