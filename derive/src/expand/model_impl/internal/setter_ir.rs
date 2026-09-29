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

/// Records the name, method, and input type of one setter.
pub(in crate::expand::model_impl) struct SetterIr {
    /// Consumer-evaluated condition under which this method exists.
    pub(in crate::expand::model_impl) presence: PresenceCondition,
    /// Canonical property name.
    pub(in crate::expand::model_impl) property: String,
    /// Source setter method identifier.
    pub(in crate::expand::model_impl) method: Ident,
    /// Setter input type.
    pub(in crate::expand::model_impl) input: Type,
}
