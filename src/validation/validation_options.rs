// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runtime controls for model validation.

// qubit-style: allow multiple-public-types

use std::num::NonZeroUsize;

mod validation_options_builder;

pub use validation_options_builder::ValidationOptionsBuilder;

/// Selects whether validation stops at the first failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationMode {
    /// Execute every selected occurrence until a budget is reached.
    CollectAll,
    /// Stop after the first reported violation.
    FailFast,
}

/// An owned field path used to select model validation occurrences.
///
/// Matching uses complete field-name paths, ignoring collection indices. A
/// parent path does not select its descendants. Construction does not check
/// the path against a model graph; a path without a matching occurrence selects
/// no field rules. Model-level rules have the empty field-name path: include
/// an empty segment sequence to select them explicitly.
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldPath {
    /// Complete field-name path; collection indices are not included.
    segments: Vec<String>,
}

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

    /// Returns the owned path segments used for matching.
    #[must_use]
    #[inline(always)]
    pub(crate) fn segments(&self) -> &[String] {
        &self.segments
    }
}

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
    mode: ValidationMode,
    /// Complete field paths selected for execution.
    selection: ValidationSelection,
    /// Maximum property segments and parent hops per access.
    max_depth: NonZeroUsize,
    /// Maximum total root, read, element and rule operations.
    max_nodes: NonZeroUsize,
    /// Maximum retained failures, including prerequisites.
    max_violations: NonZeroUsize,
    /// Maximum selector element rule calls.
    max_comparisons: NonZeroUsize,
}

/// The selected model fields to validate.
///
/// Selection applies during execution, after all declarations have been
/// checked and bound. It cannot bypass a plan construction error. Model-level
/// rules are selected by `All` or an explicitly included empty field path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationSelection {
    /// Validate every bound occurrence.
    All,
    /// Validate occurrences whose complete field-name path is selected.
    /// Collection indices are ignored; selecting a parent is not a prefix
    /// match.
    Fields(Vec<FieldPath>),
}

impl Default for ValidationOptions {
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
    #[must_use = "configure validation and call build"]
    pub fn builder() -> ValidationOptionsBuilder {
        ValidationOptionsBuilder::default()
    }

    /// Returns the selected execution mode for the executor.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn mode(&self) -> ValidationMode {
        self.mode
    }
    /// Returns the field selection for the executor.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn selection(&self) -> &ValidationSelection {
        &self.selection
    }
    /// Returns the maximum traversal depth as a plain integer.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn max_depth(&self) -> usize {
        self.max_depth.get()
    }
    /// Returns the maximum visited-node budget.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn max_nodes(&self) -> usize {
        self.max_nodes.get()
    }
    /// Returns the maximum retained-violation budget.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn max_violations(&self) -> usize {
        self.max_violations.get()
    }
    /// Returns the maximum collection-comparison budget.
    #[must_use]
    #[inline(always)]
    pub(crate) const fn max_comparisons(&self) -> usize {
        self.max_comparisons.get()
    }
}
