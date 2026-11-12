// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Successful codec bindings and declarations skipped during traversal.

use super::CodecBindSkip;
use super::CodecBindings;

/// Successful bindings plus diagnostics for declarations that could not be
/// inspected.
///
/// A successful report does not establish that every field should have a codec,
/// that every declaration was inspectable, or that application configuration
/// is complete. If any inspected declaration has a binding error, the operation
/// returns that error instead of a partial report.
///
/// Reports are returned by [`bind_codecs_with_report`](super::bind_codecs_with_report).
/// The report can be inspected by reference or consumed to retain only its
/// successful bindings.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::codec::{CodecBindReport, CodecBindings};
///
/// fn inspect(report: &CodecBindReport<'_>) {
///     let count = report.bindings().bindings().len();
///     for skip in report.skipped() {
///         assert!(!skip.property().is_empty());
///         let _ = (skip.model().type_id(), skip.location(), count);
///     }
/// }
///
/// fn take_bindings<'a>(report: CodecBindReport<'a>) -> CodecBindings<'a> {
///     report.into_bindings()
/// }
/// ```
#[derive(Debug)]
pub struct CodecBindReport<'a> {
    /// Successfully resolved codec bindings.
    bindings: CodecBindings<'a>,
    /// Declarations whose types could not be inspected.
    skipped: Box<[CodecBindSkip]>,
}

impl<'a> CodecBindReport<'a> {
    /// Constructs a report from successful bindings and collected skips.
    ///
    /// # Parameters
    ///
    /// * `bindings` - The bindings resolved during traversal.
    /// * `skipped` - Field and selector type-resolution diagnostics.
    ///
    /// # Returns
    ///
    /// An immutable report containing both collections.
    pub(super) fn new(bindings: CodecBindings<'a>, skipped: Vec<CodecBindSkip>) -> Self {
        Self {
            bindings,
            skipped: skipped.into_boxed_slice(),
        }
    }

    /// Returns all successfully resolved codec bindings.
    #[must_use]
    pub const fn bindings(&self) -> &CodecBindings<'a> {
        &self.bindings
    }

    /// Returns declarations whose field or selector type could not be resolved.
    #[must_use]
    pub fn skipped(&self) -> &[CodecBindSkip] {
        &self.skipped
    }

    /// Consumes the report and returns its successful bindings.
    #[must_use]
    pub fn into_bindings(self) -> CodecBindings<'a> {
        self.bindings
    }
}
