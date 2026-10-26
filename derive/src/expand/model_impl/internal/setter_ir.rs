// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Intermediate representation for a validated property setter.

use syn::Ident;
use syn::Type;

use super::presence_condition::PresenceCondition;

/// Stores the validated metadata needed to generate one property setter
/// adapter.
///
/// The property name is canonicalized during validation, while the method
/// identifier and input type are retained from the source signature.
pub(in crate::expand::model_impl) struct SetterIr {
    /// Conditional-compilation predicate controlling whether the method exists
    /// in the consuming crate.
    pub(in crate::expand::model_impl) presence: PresenceCondition,
    /// Canonical property name used when associating the setter with a
    /// property.
    pub(in crate::expand::model_impl) property: String,
    /// Identifier of the setter method in the source model.
    pub(in crate::expand::model_impl) method: Ident,
    /// Input type accepted by the setter method.
    pub(in crate::expand::model_impl) input: Type,
}
