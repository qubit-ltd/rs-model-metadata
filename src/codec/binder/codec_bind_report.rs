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

/// Successful bindings plus diagnostics for declarations that could not be inspected.
///
/// A successful report does not establish that every field should have a codec,
/// that every declaration was inspectable, or that application configuration
/// is complete. If any inspected declaration has a binding error, the operation
/// returns that error instead of a partial report.
#[derive(Debug)]
pub struct CodecBindReport<'a> {
    bindings: CodecBindings<'a>,
    skipped: Box<[CodecBindSkip]>,
}

impl<'a> CodecBindReport<'a> {
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
