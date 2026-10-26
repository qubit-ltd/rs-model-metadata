// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Budgets and selection for one validation call.

use std::num::NonZeroUsize;

use super::ValidationMode;
use super::ValidationOptionsBuilder;
use super::ValidationSelection;

/// Limits and selection for one validation call.
///
/// Defaults select all fields, collect failures, and allow depth 64, 100,000
/// operations, 100 retained failures and 1,000,000 selector rule calls.
/// Configure an independent [`ValidationOptionsBuilder`] when overriding them.
///
/// # Examples
///
/// ```
/// use std::num::NonZeroUsize;
/// use qubit_model_metadata::validation::ValidationMode;
/// use qubit_model_metadata::validation::ValidationOptions;
///
/// let options = ValidationOptions::builder()
///     .mode(ValidationMode::FailFast)
///     .max_nodes(NonZeroUsize::new(1_000).expect("positive budget"))
///     .build();
/// assert_ne!(options, ValidationOptions::default());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationOptions {
    /// Global stopping policy shared by every rule.
    pub(super) mode: ValidationMode,
    /// Complete field paths selected for execution.
    pub(super) selection: ValidationSelection,
    /// Maximum property segments and parent hops per access.
    pub(super) max_depth: NonZeroUsize,
    /// Maximum total root, read, element and rule operations.
    pub(super) max_nodes: NonZeroUsize,
    /// Maximum retained failures, including prerequisites.
    pub(super) max_violations: NonZeroUsize,
    /// Maximum selector element rule calls.
    pub(super) max_comparisons: NonZeroUsize,
}

impl Default for ValidationOptions {
    /// Creates options with the documented execution policy and budgets.
    fn default() -> Self {
        Self {
            mode: ValidationMode::CollectAll,
            selection: ValidationSelection::All,
            max_depth: NonZeroUsize::new(64).expect("non-zero"),
            max_nodes: NonZeroUsize::new(100_000).expect("non-zero"),
            max_violations: NonZeroUsize::new(100).expect("non-zero"),
            max_comparisons: NonZeroUsize::new(1_000_000).expect("non-zero"),
        }
    }
}

impl ValidationOptions {
    /// Creates a builder initialized with the default execution policy and
    /// budgets.
    ///
    /// # Returns
    ///
    /// A builder containing the same execution policy and budgets as
    /// [`ValidationOptions::default`].
    #[inline]
    pub fn builder() -> ValidationOptionsBuilder {
        ValidationOptionsBuilder::default()
    }

    /// Returns the selected execution mode for the executor.
    #[must_use]
    #[inline]
    pub(crate) const fn mode(&self) -> ValidationMode {
        self.mode
    }

    /// Returns the field selection for the executor.
    #[must_use]
    #[inline]
    pub(crate) const fn selection(&self) -> &ValidationSelection {
        &self.selection
    }

    /// Returns the maximum traversal depth as a plain integer.
    #[must_use]
    #[inline]
    pub(crate) const fn max_depth(&self) -> usize {
        self.max_depth.get()
    }

    /// Returns the maximum visited-node budget.
    #[must_use]
    #[inline]
    pub(crate) const fn max_nodes(&self) -> usize {
        self.max_nodes.get()
    }

    /// Returns the maximum retained-violation budget.
    #[must_use]
    #[inline]
    pub(crate) const fn max_violations(&self) -> usize {
        self.max_violations.get()
    }

    /// Returns the maximum collection-comparison budget.
    #[must_use]
    #[inline]
    pub(crate) const fn max_comparisons(&self) -> usize {
        self.max_comparisons.get()
    }
}
