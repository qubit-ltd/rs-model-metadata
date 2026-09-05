// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Intermediate representation for a validated property getter.

use syn::Ident;

use super::GetterReturn;

/// Records the name, method, and return classification of one getter.
pub(in crate::expand::model_impl) struct GetterIr {
    /// Canonical property name.
    pub(in crate::expand::model_impl) property: String,
    /// Source getter method identifier.
    pub(in crate::expand::model_impl) method: Ident,
    /// Classified getter output.
    pub(in crate::expand::model_impl) output: GetterReturn,
}
