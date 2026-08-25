// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// A static identifier for an external strategy implemented by another crate.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::StrategyRef;
///
/// let strategy = StrategyRef::new("redact-email");
/// assert_eq!(strategy.name(), "redact-email");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrategyRef {
    /// The stable logical strategy name.
    name: &'static str,
}

impl StrategyRef {
    /// Creates a strategy reference with a stable logical name.
    ///
    /// # Parameters
    ///
    /// * `name` - The stable logical strategy name.
    ///
    /// # Returns
    ///
    /// A strategy reference containing the supplied name.
    ///
    /// # Panics
    ///
    /// Panics when `name` is empty.
    #[must_use]
    #[inline(always)]
    pub const fn new(name: &'static str) -> Self {
        assert!(!name.is_empty(), "strategy names cannot be empty");
        Self { name }
    }

    /// Returns the stable logical strategy name.
    ///
    /// # Returns
    ///
    /// The stable logical strategy name.
    #[must_use]
    #[inline(always)]
    pub const fn name(self) -> &'static str {
        self.name
    }
}
