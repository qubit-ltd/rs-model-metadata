// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field selection applied during validation execution.

use super::FieldPath;

/// The selected model fields to validate.
///
/// Selection applies during execution, after all declarations have been
/// checked and bound. `Fields` rejects an empty list or any path without an
/// exact bound-rule match before a getter runs. It cannot bypass a plan
/// construction error. Model-level rules are selected by `All` or an
/// explicitly included empty field path when a model rule exists.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::validation::{FieldPath, ValidationOptions, ValidationSelection};
///
/// let options = ValidationOptions::builder()
///     .selection(ValidationSelection::Fields(vec![FieldPath::new("address.city")]))
///     .build();
/// assert_ne!(options, ValidationOptions::default());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationSelection {
    /// Validate every bound occurrence.
    All,
    /// Validate occurrences whose complete field-name path is selected.
    /// Collection indices are ignored; selecting a parent is not a prefix
    /// match. Every supplied path must match a bound rule, and this variant
    /// must contain at least one path.
    Fields(Vec<FieldPath>),
}
