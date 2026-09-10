// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Ordered validation plan construction diagnostics.

use std::error::Error;
use std::fmt;
use std::ops::Index;
use std::slice::Iter;

use super::validation_build_error::ValidationBuildError;

/// Independent failures collected while building one validation plan.
///
/// Root-level failures precede declaration occurrences. Declaration diagnostics
/// follow occurrence order, and ties retain their original order. Borrowing the
/// collection as a slice or iterator keeps every diagnostic and its source
/// context available without cloning. The aggregate's display reports only the
/// count; inspect individual diagnostics for their typed causes.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Enum;
/// use qubit_model_metadata::metadata::TypeMetadata;
/// use qubit_model_metadata::registry::ModelRegistry;
/// use qubit_model_metadata::resolve::ResolveInputs;
/// use qubit_model_metadata::resolve::StructureResolver;
/// use qubit_model_metadata::validation::ValidationBuildErrorKind;
/// use qubit_model_metadata::validation::ValidationCapabilities;
///
/// #[Enum]
/// enum Choice { Named { #[text(non_blank)] name: String } }
/// # fn main() {
/// let root = TypeMetadata::of::<Choice>();
/// let models = ModelRegistry::from_metadata(&[]).expect("isolated registry");
/// let roots = [root];
/// let graph = StructureResolver::new(ResolveInputs { models: &models, roots: &roots })
///     .resolve().expect("valid enum structure");
/// let errors = ValidationCapabilities::check(root, &graph)
///     .expect_err("enum payload execution is unsupported");
/// assert_eq!(errors.len(), 1);
/// for diagnostic in &errors {
///     assert_eq!(diagnostic.kind(), ValidationBuildErrorKind::UnsupportedExecution);
///     assert_eq!(diagnostic.field_location().expect("payload field").index(), 0);
/// }
/// # }
/// ```
#[must_use]
#[derive(Debug)]
pub struct ValidationBuildErrors {
    /// Owned diagnostics in stable root-before-occurrence order.
    errors: Vec<ValidationBuildError>,
}

impl ValidationBuildErrors {
    /// Orders diagnostics by occurrence ordinal, placing root failures first.
    ///
    /// Stable sorting preserves the input order among diagnostics with the
    /// same ordinal, including failures without an occurrence.
    pub(crate) fn from_errors(mut errors: Vec<ValidationBuildError>) -> Self {
        errors.sort_by_key(ValidationBuildError::occurrence);
        Self { errors }
    }

    /// Returns the total number of independently retained diagnostics.
    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Returns whether the collection contains no diagnostics.
    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns failures in deterministic declaration order.
    #[must_use = "inspect the declaration diagnostics"]
    #[inline(always)]
    pub fn as_slice(&self) -> &[ValidationBuildError] {
        &self.errors
    }

    /// Iterates over failures in declaration order.
    #[must_use = "inspect the declaration diagnostics"]
    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &ValidationBuildError> {
        self.errors.iter()
    }
}

impl Index<usize> for ValidationBuildErrors {
    /// One diagnostic borrowed from this collection.
    type Output = ValidationBuildError;

    /// Borrows the diagnostic at its stable collection position.
    ///
    /// # Panics
    ///
    /// Panics when `index` is greater than or equal to the diagnostic count.
    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.errors[index]
    }
}

impl<'a> IntoIterator for &'a ValidationBuildErrors {
    /// A diagnostic borrowed for the lifetime of the collection reference.
    type Item = &'a ValidationBuildError;
    /// Allocation-free iteration over the retained diagnostic sequence.
    type IntoIter = Iter<'a, ValidationBuildError>;

    /// Borrows each diagnostic in root-before-occurrence order.
    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

impl fmt::Display for ValidationBuildErrors {
    /// Formats a count summary without expanding metadata or nested causes.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if writing the summary fails.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} validation plan binding error(s)", self.errors.len())
    }
}

impl Error for ValidationBuildErrors {}

impl AsRef<[ValidationBuildError]> for ValidationBuildErrors {
    /// Exposes the complete ordered diagnostics without allocating or cloning.
    #[inline(always)]
    fn as_ref(&self) -> &[ValidationBuildError] {
        self.as_slice()
    }
}
