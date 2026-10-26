// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Normalized validator declaration IR.

use syn::LitStr;

use super::super::StrategyArgumentIr;

/// Normalized validator metadata consumed when generating runtime declarations.
#[derive(Clone)]
pub(crate) struct ValidatorIr {
    /// Validated identifier used to select the runtime validator strategy.
    pub(crate) id: LitStr,
    /// Named, typed arguments forwarded to the runtime validator strategy.
    pub(crate) params: Vec<(String, StrategyArgumentIr)>,
    /// Legacy unbound field paths required before evaluating this validator.
    pub(crate) depends_on: Vec<Vec<String>>,
    /// Named bindings of dependency slots to object navigation and property
    /// paths.
    pub(crate) dependency_bindings: Vec<(String, Vec<String>, Vec<String>)>,
}
