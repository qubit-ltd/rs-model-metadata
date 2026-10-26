// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Declares the target entity used to verify cross-crate model registration.

use qubit_model_derive::Entity;
use qubit_model_metadata::__private::qubit_id::Id;

/// An entity registered by the linked-workspace runtime fixture.
///
/// # Examples
///
/// ```
/// use core::any::TypeId;
/// use model_b::Target;
/// use qubit_model_metadata::TypeMetadata;
///
/// let target = Target { id: 42.into() };
/// assert_eq!(target.id.value(), 42);
/// assert_eq!(TypeMetadata::of::<Target>().type_id(), TypeId::of::<Target>());
/// ```
#[Entity(id = "test.linked.Target")]
pub struct Target {
    /// The identifier used by the generated entity metadata.
    #[identifier]
    pub id: Id,
}
