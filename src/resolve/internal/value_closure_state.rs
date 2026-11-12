// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::any::TypeId;
use std::collections::HashSet;

/// Tracks Value-closure traversal state for one structure-resolution attempt.
///
/// The active set prevents recursive loops, while the closed set stores only
/// complete successful checks that did not depend on a detected back edge.
pub(in crate::resolve) struct ValueClosureState {
    /// Types on the current recursive traversal path.
    active: HashSet<TypeId>,
    /// Types completely checked as valid without encountering a back edge.
    closed: HashSet<TypeId>,
    /// Saturating count of back edges observed during this attempt.
    cycle_epoch: u64,
    #[cfg(test)]
    expansions: usize,
}

impl ValueClosureState {
    /// Creates empty traversal state for one resolution attempt.
    ///
    /// The returned cache is not shared with another resolver or context.
    pub(in crate::resolve) fn new() -> Self {
        Self {
            active: HashSet::new(),
            closed: HashSet::new(),
            cycle_epoch: 0,
            #[cfg(test)]
            expansions: 0,
        }
    }

    /// Reports whether a type has a complete successful closure result.
    ///
    /// # Parameters
    ///
    /// - `type_id`: Concrete Rust type identity whose closure result is
    ///   queried.
    ///
    /// # Returns
    ///
    /// `true` only when this attempt checked the type without errors or a
    /// cycle-dependent result.
    pub(in crate::resolve) fn is_closed(&self, type_id: TypeId) -> bool {
        self.closed.contains(&type_id)
    }

    /// Enters a type or records a back edge to an active type.
    ///
    /// # Parameters
    ///
    /// - `type_id`: Concrete Rust type identity being traversed.
    ///
    /// # Returns
    ///
    /// `true` when this call adds the type to the active set, or `false` when
    /// it is already active and the cycle epoch is advanced.
    pub(in crate::resolve) fn enter(&mut self, type_id: TypeId) -> bool {
        if self.active.contains(&type_id) {
            self.cycle_epoch = self.cycle_epoch.saturating_add(1);
            return false;
        }
        self.active.insert(type_id);
        #[cfg(test)]
        {
            self.expansions += 1;
        }
        true
    }

    /// Returns the current back-edge counter for this resolution attempt.
    ///
    /// # Returns
    ///
    /// The saturating epoch used to exclude cycle-dependent cache entries.
    pub(in crate::resolve) fn epoch(&self) -> u64 {
        self.cycle_epoch
    }

    /// Leaves an active type and records it only when its check is cacheable.
    ///
    /// A result is cacheable when the traversal was clean and no back edge was
    /// observed since `start_epoch`; a saturated epoch disables new entries.
    ///
    /// # Parameters
    ///
    /// - `type_id`: Concrete Rust type identity being removed from the active
    ///   set.
    /// - `start_epoch`: Epoch captured before checking this type.
    /// - `clean`: Whether visible descendants completed without errors.
    pub(in crate::resolve) fn leave(&mut self, type_id: TypeId, start_epoch: u64, clean: bool) {
        self.active.remove(&type_id);
        if clean && self.cycle_epoch == start_epoch && self.cycle_epoch != u64::MAX {
            self.closed.insert(type_id);
        }
    }

    /// Returns the number of types entered by this test traversal.
    ///
    /// # Returns
    ///
    /// The deterministic expansion count used by complexity regressions.
    #[cfg(test)]
    pub(in crate::resolve) fn expansion_count(&self) -> usize {
        self.expansions
    }

    /// Reports whether the test traversal released all active types.
    ///
    /// # Returns
    ///
    /// `true` when no recursive frame remains active.
    #[cfg(test)]
    pub(in crate::resolve) fn active_is_empty(&self) -> bool {
        self.active.is_empty()
    }
}

#[cfg(test)]
mod tests;
