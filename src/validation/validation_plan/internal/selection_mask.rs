// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tracks selected validation occurrences and later work.

use qubit_validator::ValidationPath;

use super::super::execute::field_matches;
use super::selection_entry::SelectionEntry;
use crate::validation::internal::field_rule_binding::FieldRuleBinding;
use crate::validation::internal::path_reader::path_for;
use crate::validation::validation_options::ValidationSelection;

/// Tracks which planned rule occurrences are selected and whether later work
/// remains.
#[must_use]
pub(in crate::validation::validation_plan) enum SelectionMask {
    /// Represents an all-fields selection without allocating per-occurrence
    /// entries.
    All {
        /// Number of planned occurrences represented by this mask.
        total: usize,
    },
    /// Stores selection and path state for each planned occurrence.
    Fields(Vec<SelectionEntry>),
}

impl SelectionMask {
    /// Builds occurrence state from the caller's selection and compiled
    /// bindings.
    ///
    /// # Parameters
    ///
    /// * `selection` - Requested validation selection.
    /// * `model_rule_count` - Number of model-level occurrences before fields.
    /// * `bindings` - Ordered field-rule occurrences in the plan.
    ///
    /// # Returns
    ///
    /// A mask that retains per-occurrence paths for explicit field selections
    /// and avoids those entries for an all-fields selection.
    pub(in crate::validation::validation_plan) fn new(
        selection: &ValidationSelection,
        model_rule_count: usize,
        bindings: &[FieldRuleBinding],
    ) -> Self {
        let total = model_rule_count + bindings.len();
        if matches!(selection, ValidationSelection::All) {
            return Self::All { total };
        }
        let model_path = ValidationPath::root();
        let mut entries = Vec::with_capacity(total);
        entries.extend((0..model_rule_count).map(|_| SelectionEntry {
            selected: selected(selection, &model_path),
            later_selected: false,
            field_path: None,
        }));
        for binding in bindings {
            let path = path_for(binding.value());
            let is_selected = selected(selection, &path);
            entries.push(SelectionEntry {
                selected: is_selected,
                later_selected: false,
                field_path: is_selected.then_some(path),
            });
        }
        let mut later = false;
        for entry in entries.iter_mut().rev() {
            entry.later_selected = later;
            later |= entry.selected;
        }
        Self::Fields(entries)
    }

    /// Returns whether the indexed occurrence is selected.
    ///
    /// # Parameters
    ///
    /// * `index` - Position of the occurrence in the combined plan order.
    ///
    /// # Returns
    ///
    /// `true` when the indexed occurrence is included in the caller's
    /// selection.
    ///
    /// # Panics
    ///
    /// Panics when this mask stores field entries and `index` is out of
    /// bounds.
    #[must_use]
    #[inline]
    pub(in crate::validation::validation_plan) fn selected(&self, index: usize) -> bool {
        match self {
            Self::All { total } => index < *total,
            Self::Fields(entries) => entries[index].selected,
        }
    }

    /// Returns whether any occurrence after the indexed one is selected.
    ///
    /// # Parameters
    ///
    /// * `index` - Position of the occurrence in the combined plan order.
    ///
    /// # Returns
    ///
    /// `true` when a later occurrence is included in the caller's selection.
    ///
    /// # Panics
    ///
    /// Panics when this mask stores field entries and `index` is out of
    /// bounds. For an all-fields mask, `index + 1` can overflow.
    #[must_use]
    #[inline]
    pub(in crate::validation::validation_plan) fn later_selected(&self, index: usize) -> bool {
        match self {
            Self::All { total } => index + 1 < *total,
            Self::Fields(entries) => entries[index].later_selected,
        }
    }

    /// Takes the cached field path for an occurrence, if it has one.
    ///
    /// # Parameters
    ///
    /// * `index` - Position of the occurrence in the combined plan order.
    ///
    /// # Returns
    ///
    /// The cached path for an explicitly selected field occurrence, or `None`
    /// for an all-fields mask or an occurrence without a cached path.
    ///
    /// # Panics
    ///
    /// Panics when this mask stores field entries and `index` is out of
    /// bounds.
    #[inline]
    pub(in crate::validation::validation_plan) fn take_field_path(&mut self, index: usize) -> Option<ValidationPath> {
        match self {
            Self::All { .. } => None,
            Self::Fields(entries) => entries[index].field_path.take(),
        }
    }
}

/// Reports whether a validation path is selected by the caller.
#[must_use]
fn selected(selection: &ValidationSelection, path: &ValidationPath) -> bool {
    match selection {
        ValidationSelection::All => true,
        ValidationSelection::Fields(fields) => fields.iter().any(|field| field_matches(field, path)),
    }
}
