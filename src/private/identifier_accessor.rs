// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed access to an identifier field in generated role metadata.

use qubit_id::Id;
use qubit_reflect::error::TypeMismatch;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

/// Typed accessor used only for the identifier field of Entity and Projection
/// metadata.
#[doc(hidden)]
pub type IdentifierAccessor = for<'a> fn(DynamicRef<'a, Local>) -> Result<&'a Id, TypeMismatch>;
