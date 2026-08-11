// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Structural classifications for named model types.

use crate::type_metadata::EnumMetadata;
use crate::type_metadata::NewtypeMetadata;
use crate::type_metadata::StructMetadata;

/// The structural form of a named model type.
#[must_use]
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum TypeKind {
    /// A type with named fields.
    Struct(StructMetadata),
    /// A fieldless enum.
    Enum(EnumMetadata),
    /// A tuple newtype with one inner field.
    Newtype(NewtypeMetadata),
}
