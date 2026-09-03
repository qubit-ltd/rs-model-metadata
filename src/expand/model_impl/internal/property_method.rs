// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Validated property method variants.

use super::GetterIr;
use super::SetterIr;

/// Represents one validated property method.
pub(in crate::expand::model_impl) enum PropertyMethod {
    /// Validated getter.
    Getter(GetterIr),
    /// Validated setter.
    Setter(SetterIr),
}
