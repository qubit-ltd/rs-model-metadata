// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use crate::relation::PropertyPath;

/// Selects an entity as a whole or one of its properties.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::PropertyPath;
/// use qubit_model_metadata::metadata::ReferenceSelection;
///
/// let selection = ReferenceSelection::Property(PropertyPath::new(&["owner", "id"]));
/// assert!(matches!(selection, ReferenceSelection::Property(path) if path.to_string() == "owner.id"));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceSelection {
    /// Selects the complete entity.
    Entity,
    /// Selects a property path on the entity.
    Property(PropertyPath<'static>),
}
