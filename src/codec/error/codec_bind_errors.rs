// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use std::error::Error;

use crate::codec::CodecBindError;

/// All deterministically ordered codec binding failures.
///
/// The binding process returns this collection when
/// [`bind_codecs`](crate::codec::bind_codecs) reports one or more failures.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::codec::CodecBindErrors;
///
/// // `bind_codecs` provides this value as the error in its `Err` result.
/// fn inspect(errors: &CodecBindErrors) {
///     assert!(!errors.errors().is_empty());
/// }
/// # let _ = inspect;
/// ```
#[must_use]
#[derive(Debug)]
pub struct CodecBindErrors(Box<[CodecBindError]>);

impl CodecBindErrors {
    /// Creates a deterministically ordered collection of binding diagnostics.
    ///
    /// # Parameters
    ///
    /// * `errors` - Diagnostics gathered during one codec binding pass.
    ///
    /// # Returns
    ///
    /// A collection sorted by occurrence identity and then failure class.
    pub(crate) fn new(mut errors: Vec<CodecBindError>) -> Self {
        errors.sort_by(|left, right| {
            left.occurrence()
                .cmp(right.occurrence())
                .then_with(|| left.kind().cmp(&right.kind()))
        });
        Self(errors.into_boxed_slice())
    }

    /// Returns all failures.
    ///
    /// # Returns
    ///
    /// The diagnostics in deterministic occurrence and failure-class order.
    #[must_use]
    #[inline]
    pub fn errors(&self) -> &[CodecBindError] {
        &self.0
    }

    /// Consumes the collection.
    ///
    /// # Returns
    ///
    /// The owned diagnostics in their deterministic order.
    #[must_use]
    #[inline]
    pub fn into_vec(self) -> Vec<CodecBindError> {
        self.0.into_vec()
    }
}

impl Display for CodecBindErrors {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "{} codec binding error(s)", self.0.len())
    }
}

impl Error for CodecBindErrors {}
