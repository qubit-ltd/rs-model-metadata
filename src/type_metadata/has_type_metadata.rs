// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static metadata contract implemented by generated model declarations.

use crate::Reflect;

/// Marks a reflected Rust type that has generated model metadata.
///
/// Generated code supplies the hidden provider and seal. Public generic APIs
/// can use this trait as a capability bound without exposing an unchecked
/// metadata-construction hook.
pub trait HasTypeMetadata: Reflect + crate::__private::ModelTypeSeal + crate::__private::TypeMetadataProvider {}

impl<T> HasTypeMetadata for T where T: Reflect + crate::__private::ModelTypeSeal + crate::__private::TypeMetadataProvider
{}
