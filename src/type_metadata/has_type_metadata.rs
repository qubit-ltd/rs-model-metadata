// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static metadata contract implemented by generated model declarations.

use crate::Reflect;
use crate::TypeMetadata;

/// Exposes the generated metadata overlay for one reflected model type.
///
/// This trait is a public generic bound, but its hidden seal makes manual
/// implementations unsupported. Model role macros implement the seal together
/// with reflection and capability registration, ensuring all three static
/// entry points describe the same type.
pub trait HasTypeMetadata: Reflect + crate::__private::ModelTypeSeal {
    /// Returns the immutable metadata overlay for this model type.
    fn type_metadata() -> &'static TypeMetadata;
}
