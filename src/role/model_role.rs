// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// The semantic role assigned by a model macro.
///
/// # Examples
///
/// ```
/// use qubit_model_metadata::metadata::ModelRole;
///
/// let role = ModelRole::Entity;
/// assert_eq!(role, ModelRole::Entity);
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ModelRole {
    /// Persisted entity with an identifier.
    Entity,
    /// Open or fixed view over entity data.
    Projection,
    /// General structured model.
    Model,
    /// Enumerated model.
    Enum,
    /// Value object.
    Value,
}
