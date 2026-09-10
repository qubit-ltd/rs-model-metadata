// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned configuration for one validation call.

use std::num::NonZeroUsize;

use super::ValidationMode;
use super::ValidationOptions;
use super::ValidationSelection;

/// Builds validation options from the documented defaults.
///
/// Obtain a builder with [`ValidationOptions::builder`]. Each configuration
/// method replaces its setting; [`Self::build`] consumes the builder without
/// cloning the selected paths. Nonzero budget types reject zero before it can
/// enter this configuration.
#[must_use = "finish the configuration with build"]
#[derive(Clone, Debug, Default)]
pub struct ValidationOptionsBuilder {
    /// Configuration transferred to the caller on completion.
    options: ValidationOptions,
}

impl ValidationOptionsBuilder {
    /// Sets the global collect-all or fail-fast policy.
    #[must_use = "use the returned builder"]
    pub fn mode(mut self, mode: ValidationMode) -> Self {
        self.options.mode = mode;
        self
    }

    /// Transfers the complete field-path selection into this builder.
    #[must_use = "use the returned builder"]
    pub fn selection(mut self, selection: ValidationSelection) -> Self {
        self.options.selection = selection;
        self
    }

    /// Limits property segments, collection indices and parent hops per access.
    #[must_use = "use the returned builder"]
    pub fn max_depth(mut self, value: NonZeroUsize) -> Self {
        self.options.max_depth = value;
        self
    }

    /// Limits the root, actual reads, selector elements and rule calls
    /// combined.
    #[must_use = "use the returned builder"]
    pub fn max_nodes(mut self, value: NonZeroUsize) -> Self {
        self.options.max_nodes = value;
        self
    }

    /// Caps retained failures, including failed prerequisites, across all
    /// rules.
    #[must_use = "use the returned builder"]
    pub fn max_violations(mut self, value: NonZeroUsize) -> Self {
        self.options.max_violations = value;
        self
    }

    /// Limits selector element rule calls; it cannot count comparisons
    /// performed inside an external validator.
    #[must_use = "use the returned builder"]
    pub fn max_comparisons(mut self, value: NonZeroUsize) -> Self {
        self.options.max_comparisons = value;
        self
    }

    /// Consumes this builder and transfers the completed configuration.
    #[must_use = "use the completed validation options"]
    pub fn build(self) -> ValidationOptions {
        self.options
    }
}
