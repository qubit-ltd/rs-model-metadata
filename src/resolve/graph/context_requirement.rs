// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Context required to navigate a resolved structure graph.

/// Instance context that cannot be obtained from a structural registry.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::resolve::ContextRequirement;
///
/// let requirement = ContextRequirement::ParentObject;
/// assert_eq!(requirement, ContextRequirement::ParentObject);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRequirement {
    /// All object navigation is local to the declared root.
    None,
    /// Navigation needs a containing object supplied by the consumer.
    ParentObject,
}
