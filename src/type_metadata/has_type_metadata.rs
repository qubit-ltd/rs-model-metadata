// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static metadata contract implemented by generated model declarations.

use qubit_reflect::Reflect;

use crate::__private::ModelTypeSeal;
use crate::__private::TypeMetadataProvider;

/// Marks a reflected Rust type that has generated model metadata.
///
/// Generated code supplies the hidden provider and seal. Public generic APIs
/// can use this trait as a capability bound without exposing an unchecked
/// metadata-construction hook.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
/// use qubit_model_metadata::type_metadata::HasTypeMetadata;
///
/// #[Model]
/// struct User {
///     name: String,
/// }
///
/// fn accepts_metadata_type<T: HasTypeMetadata>() {}
///
/// # fn main() {
/// accepts_metadata_type::<User>();
/// # }
/// ```
pub trait HasTypeMetadata: Reflect + ModelTypeSeal + TypeMetadataProvider {}

impl<T> HasTypeMetadata for T where T: Reflect + ModelTypeSeal + TypeMetadataProvider {}
