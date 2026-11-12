// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
// =============================================================================

use std::any::TypeId;
use std::collections::HashSet;

/// Per-resolution state for checking value type closures.
pub(in crate::resolve) struct ValueClosureState {
    active: HashSet<TypeId>,
    closed: HashSet<TypeId>,
    cycle_epoch: u64,
    #[cfg(test)]
    expansions: usize,
    #[cfg(test)]
    cache_enabled: bool,
}

impl ValueClosureState {
    pub(in crate::resolve) fn new() -> Self {
        Self {
            active: HashSet::new(),
            closed: HashSet::new(),
            cycle_epoch: 0,
            #[cfg(test)]
            expansions: 0,
            #[cfg(test)]
            cache_enabled: true,
        }
    }

    pub(in crate::resolve) fn is_closed(&self, type_id: TypeId) -> bool {
        #[cfg(test)]
        if !self.cache_enabled {
            return false;
        }
        self.closed.contains(&type_id)
    }

    #[cfg(test)]
    pub(in crate::resolve) fn new_legacy_reference() -> Self {
        Self {
            cache_enabled: false,
            ..Self::new()
        }
    }

    /// Enters a type, returning false when it is already active (a back edge).
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

    pub(in crate::resolve) fn epoch(&self) -> u64 {
        self.cycle_epoch
    }

    pub(in crate::resolve) fn leave(&mut self, type_id: TypeId, start_epoch: u64, clean: bool) {
        self.active.remove(&type_id);
        if clean && self.cycle_epoch == start_epoch && self.cycle_epoch != u64::MAX {
            #[cfg(not(test))]
            self.closed.insert(type_id);
            #[cfg(test)]
            if self.cache_enabled {
                self.closed.insert(type_id);
            }
        }
    }

    #[cfg(test)]
    pub(in crate::resolve) fn expansion_count(&self) -> usize {
        self.expansions
    }

    #[cfg(test)]
    pub(in crate::resolve) fn active_is_empty(&self) -> bool {
        self.active.is_empty()
    }
}

#[cfg(test)]
#[path = "value_closure_state/tests.rs"]
mod tests;
