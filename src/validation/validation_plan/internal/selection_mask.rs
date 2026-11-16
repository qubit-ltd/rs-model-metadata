// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tracks selected validation occurrences and later work.

use qubit_validator::ValidationPath;

use super::selection_entry::SelectionEntry;
use super::selection_index::SelectionIndex;
use crate::validation::internal::field_rule_binding::FieldRuleBinding;
use crate::validation::validation_options::FieldPath;
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
    Fields(
        /// Per-occurrence selection flags and any retained explicit field path.
        Vec<SelectionEntry>,
    ),
}
impl SelectionMask {
    /// Builds occurrence state from the caller's selection and field index.
    ///
    /// # Parameters
    ///
    /// * `selection` - Requested validation selection.
    /// * `model_rule_count` - Number of model-level occurrences before fields.
    /// * `bindings` - Ordered field-rule occurrences in the plan.
    /// * `selection_index` - Plan-owned field selection index and path cache.
    ///
    /// # Returns
    ///
    /// A mask that retains selected occurrence paths and avoids per-occurrence
    /// entries for an all-fields selection, or the first unmatched selection.
    pub(in crate::validation::validation_plan) fn new(
        selection: &ValidationSelection,
        model_rule_count: usize,
        bindings: &[FieldRuleBinding],
        selection_index: &SelectionIndex,
    ) -> Result<Self, FieldPath> {
        let total = model_rule_count + bindings.len();
        if matches!(selection, ValidationSelection::All) {
            return Ok(Self::All { total });
        }
        let ValidationSelection::Fields(fields) = selection else {
            unreachable!("all-fields selection returned above");
        };
        if fields.is_empty() {
            return Err(FieldPath::from_segments(std::iter::empty::<String>()));
        }
        let mut selected = vec![false; total];
        for field in fields {
            let selects_model_rules = field.segments().is_empty() && model_rule_count > 0;
            if selects_model_rules {
                selected[..model_rule_count].fill(true);
            }
            match selection_index.matching(field.segments()) {
                Some(occurrences) => {
                    for &binding_index in occurrences {
                        selected[model_rule_count + binding_index] = true;
                    }
                }
                None if !selects_model_rules => return Err(field.clone()),
                None => {}
            }
        }
        let mut entries: Vec<_> = selected
            .into_iter()
            .enumerate()
            .map(|(occurrence, selected)| SelectionEntry {
                selected,
                later_selected: false,
                field_path: (selected && occurrence >= model_rule_count)
                    .then(|| selection_index.path(occurrence - model_rule_count).clone()),
            })
            .collect();
        let mut later = false;
        for entry in entries.iter_mut().rev() {
            entry.later_selected = later;
            later |= entry.selected;
        }
        Ok(Self::Fields(entries))
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
    pub(in crate::validation::validation_plan) fn take_field_path(
        &mut self,
        index: usize,
    ) -> Option<ValidationPath> {
        match self {
            Self::All { .. } => None,
            Self::Fields(entries) => entries[index].field_path.take(),
        }
    }
}
