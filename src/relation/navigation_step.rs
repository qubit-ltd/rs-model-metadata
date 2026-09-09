// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One step through an explicitly supplied object graph.

/// One relative object-navigation step, distinct from property selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NavigationStep {
    /// Reads the named object property.
    Property(&'static str),
    /// Selects the containing domain object; collections add no parent level.
    Parent,
}
