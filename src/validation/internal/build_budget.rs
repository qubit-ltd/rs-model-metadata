// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Per-build counters and depth checks.

mod counter;

pub(crate) use self::counter::Counter;

/// Maximum model, wrapper, and type-expression traversal depth.
pub(crate) const MAX_DEPTH: usize = 64;
/// Maximum charged discovery and binding work units per build.
pub(crate) const MAX_WORK: usize = 100_000;
/// Maximum discovered declaration occurrences per build.
pub(crate) const MAX_OCCURRENCES: usize = 10_000;
/// Maximum bound rules per build.
pub(crate) const MAX_BINDINGS: usize = 10_000;
/// Maximum ordinary diagnostics retained before a terminal limit error.
pub(crate) const MAX_DIAGNOSTICS: usize = 100;

/// Shared counters for one plan-discovery and binding attempt.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BuildBudget {
    /// Charged discovery, traversal, and binding attempts.
    work: usize,
    /// Discovered executable declaration occurrences.
    occurrences: usize,
    /// Rules emitted by occurrence binders.
    bindings: usize,
    /// Ordinary diagnostics retained for the caller.
    diagnostics: usize,
    /// Private limits ordered by depth, work, occurrences, bindings,
    /// diagnostics.
    limits: [usize; 5],
}

impl BuildBudget {
    /// Creates a counter set with the production limits.
    ///
    /// # Returns
    ///
    /// Returns a fresh build budget with every counter set to zero.
    pub(crate) const fn new() -> Self {
        Self {
            work: 0,
            occurrences: 0,
            bindings: 0,
            diagnostics: 0,
            limits: [MAX_DEPTH, MAX_WORK, MAX_OCCURRENCES, MAX_BINDINGS, MAX_DIAGNOSTICS],
        }
    }

    /// Creates a budget with test-only limits.
    ///
    /// # Parameters
    ///
    /// - `depth`: Maximum accepted traversal depth.
    /// - `work`: Maximum accepted work charges.
    /// - `occurrences`: Maximum accepted declaration occurrences.
    /// - `bindings`: Maximum accepted emitted rules.
    /// - `diagnostics`: Maximum accepted ordinary diagnostics.
    ///
    /// # Returns
    ///
    /// Returns a fresh test budget with the supplied limits and zero counters.
    #[cfg(test)]
    pub(crate) const fn with_limits(
        depth: usize,
        work: usize,
        occurrences: usize,
        bindings: usize,
        diagnostics: usize,
    ) -> Self {
        Self {
            work: 0,
            occurrences: 0,
            bindings: 0,
            diagnostics: 0,
            limits: [depth, work, occurrences, bindings, diagnostics],
        }
    }

    /// Charges entry into one model usage position at `depth`.
    ///
    /// # Parameters
    ///
    /// - `depth`: Zero-based usage depth of the model being entered.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when both depth and work capacity are available.
    ///
    /// # Errors
    ///
    /// Returns `Err(())` when either limit is exhausted.
    pub(crate) fn enter(&mut self, depth: usize) -> Result<(), ()> {
        self.check_depth(depth)?;
        self.charge(Counter::Work)
    }

    /// Checks whether `depth` is within the configured maximum.
    ///
    /// # Parameters
    ///
    /// - `depth`: Zero-based depth of the item being inspected.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when `depth` is within the configured limit.
    ///
    /// # Errors
    ///
    /// Returns `Err(())` when `depth` exceeds the configured limit.
    pub(crate) fn check_depth(&self, depth: usize) -> Result<(), ()> {
        if depth > self.limits[0] { Err(()) } else { Ok(()) }
    }

    /// Charges one item against the selected finite counter.
    ///
    /// # Parameters
    ///
    /// - `counter`: Counter category associated with the inspected item.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when capacity remains after the charge.
    ///
    /// # Errors
    ///
    /// Returns `Err(())` when the counter has reached its configured limit.
    pub(crate) fn charge(&mut self, counter: Counter) -> Result<(), ()> {
        let (used, limit) = match counter {
            Counter::Work => (&mut self.work, self.limits[1]),
            Counter::Occurrence => (&mut self.occurrences, self.limits[2]),
            Counter::Binding => (&mut self.bindings, self.limits[3]),
            Counter::Diagnostic => (&mut self.diagnostics, self.limits[4]),
        };
        if *used >= limit {
            return Err(());
        }
        *used += 1;
        Ok(())
    }

    /// Returns the number of work charges accepted by this test budget.
    ///
    /// # Returns
    ///
    /// Returns the current work counter.
    #[cfg(test)]
    fn work_used(&self) -> usize {
        self.work
    }
}

#[cfg(test)]
mod tests {
    use super::BuildBudget;
    use super::Counter;

    #[test]
    fn test_capacity_allows_exact_limit_then_rejects_next_without_overflow() {
        let mut budget = BuildBudget::with_limits(usize::MAX, 2, 2, 2, 2);
        assert!(budget.charge(Counter::Work).is_ok());
        assert!(budget.charge(Counter::Work).is_ok());
        assert!(budget.charge(Counter::Work).is_err());
        assert_eq!(budget.work_used(), 2);

        let mut maxed = BuildBudget::with_limits(usize::MAX, usize::MAX, 1, 1, 1);
        assert!(maxed.charge(Counter::Work).is_ok());
        assert_eq!(maxed.work_used(), 1);

        for counter in [Counter::Occurrence, Counter::Binding, Counter::Diagnostic] {
            let mut budget = BuildBudget::with_limits(1, 1, 1, 1, 1);
            assert!(budget.charge(counter).is_ok());
            assert!(budget.charge(counter).is_err());

            let mut empty = BuildBudget::with_limits(1, 0, 0, 0, 0);
            assert!(empty.charge(counter).is_err());
        }

        let mut depth = BuildBudget::with_limits(0, usize::MAX, 1, 1, 1);
        assert!(depth.enter(0).is_ok());
        assert!(depth.enter(1).is_err());
    }
}
