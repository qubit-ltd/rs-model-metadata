// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared accounting for actual validation execution operations.

use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;

use crate::validation::ValidationOptions;

/// One invocation's node, depth and selector-comparison budgets.
pub(crate) struct ExecutionBudget<'options> {
    /// Caller-specified immutable limits.
    options: &'options ValidationOptions,
    /// Nodes already consumed; the root is the first node.
    nodes: usize,
    /// Selector rule invocations already consumed.
    comparisons: usize,
}

impl<'options> ExecutionBudget<'options> {
    /// Starts accounting with the root node already visited.
    pub(crate) const fn new(options: &'options ValidationOptions) -> Self {
        Self {
            options,
            nodes: 1,
            comparisons: 0,
        }
    }

    /// Checks depth without charging a read that has not occurred.
    pub(crate) fn check_depth(&self, depth: usize) -> Result<(), ExecutionError> {
        if depth > self.options.max_depth() {
            Err(limit())
        } else {
            Ok(())
        }
    }

    /// Reserves one actual property or collection-element read.
    pub(crate) fn read(&mut self, depth: usize) -> Result<(), ExecutionError> {
        self.check_depth(depth)?;
        if self.nodes >= self.options.max_nodes() {
            return Err(limit());
        }
        self.nodes += 1;
        Ok(())
    }

    /// Reserves one element comparison without charging another node.
    #[allow(dead_code)] // T6 consumes this reservation when unique-items execution is wired.
    pub(crate) fn compare(&mut self, depth: usize) -> Result<(), ExecutionError> {
        self.check_depth(depth)?;
        if self.comparisons >= self.options.max_comparisons() {
            return Err(limit());
        }
        self.comparisons += 1;
        Ok(())
    }

    /// Reserves one rule invocation and, for selectors, one comparison.
    pub(crate) fn invoke(&mut self, depth: usize, selector: bool) -> Result<(), ExecutionError> {
        self.check_depth(depth)?;
        if self.nodes >= self.options.max_nodes() || selector && self.comparisons >= self.options.max_comparisons() {
            return Err(limit());
        }
        self.nodes += 1;
        if selector {
            self.comparisons += 1;
        }
        Ok(())
    }
}

/// Creates the common budget-exhaustion error.
fn limit() -> ExecutionError {
    ExecutionError::new(ExecutionErrorKind::TraversalLimit)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use qubit_validator::ExecutionErrorKind;

    use super::ExecutionBudget;
    use crate::validation::ValidationOptions;

    /// Reaching the largest representable node count must not wrap on the next
    /// operation.
    #[test]
    fn test_node_budget_does_not_wrap_at_usize_max() {
        let options = ValidationOptions::builder()
            .max_nodes(NonZeroUsize::new(usize::MAX).unwrap())
            .build();
        let mut budget = ExecutionBudget {
            options: &options,
            nodes: usize::MAX - 1,
            comparisons: 0,
        };
        budget.read(0).unwrap();
        assert_eq!(budget.read(0).unwrap_err().kind(), ExecutionErrorKind::TraversalLimit);
        assert_eq!(
            budget.invoke(0, false).unwrap_err().kind(),
            ExecutionErrorKind::TraversalLimit
        );
    }

    /// The selector comparison budget has the same non-wrapping boundary.
    #[test]
    fn test_comparison_budget_does_not_wrap_at_usize_max() {
        let options = ValidationOptions::builder()
            .max_comparisons(NonZeroUsize::new(usize::MAX).unwrap())
            .build();
        let mut budget = ExecutionBudget {
            options: &options,
            nodes: 1,
            comparisons: usize::MAX - 1,
        };
        budget.invoke(0, true).unwrap();
        assert_eq!(
            budget.invoke(0, true).unwrap_err().kind(),
            ExecutionErrorKind::TraversalLimit
        );
    }

    /// A collection comparison consumes one comparison without a rule node.
    #[test]
    fn test_compare_stops_before_second_comparison() {
        let options = ValidationOptions::builder()
            .max_comparisons(NonZeroUsize::new(1).expect("positive limit"))
            .build();
        let mut budget = ExecutionBudget::new(&options);
        budget.compare(0).expect("first comparison");
        assert_eq!(budget.comparisons, 1);
        assert_eq!(budget.nodes, 1);
        assert_eq!(
            budget.compare(0).expect_err("second comparison blocked").kind(),
            ExecutionErrorKind::TraversalLimit
        );
        assert_eq!(budget.comparisons, 1);
    }

    /// Saturated comparison accounting cannot wrap at the machine limit.
    #[test]
    fn test_compare_does_not_wrap_at_usize_max() {
        let options = ValidationOptions::builder()
            .max_comparisons(NonZeroUsize::new(usize::MAX).expect("positive limit"))
            .build();
        let mut budget = ExecutionBudget {
            options: &options,
            nodes: 1,
            comparisons: usize::MAX - 1,
        };
        budget.compare(0).expect("last comparison");
        assert_eq!(
            budget.compare(0).expect_err("saturated comparison blocked").kind(),
            ExecutionErrorKind::TraversalLimit
        );
        assert_eq!(budget.comparisons, usize::MAX);
    }
}
