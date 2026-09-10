// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static metadata overlays for Rust domain models.

// Gives derives the same absolute path inside this crate and in its doctests.
extern crate self as qubit_model_metadata;

#[doc(hidden)]
pub mod __private;
mod abi_violation;
#[cfg(feature = "codec")]
pub mod codec;
mod constraint;
mod field_metadata;
mod metadata_vocabulary;
// Keep the implementation namespace private while allowing internal modules
// to refer to the vocabulary without coupling them to its file layout.
mod field_semantics {
    pub use super::metadata_vocabulary::*;
}
#[cfg(feature = "generic")]
pub mod generic;
mod local_property_set;
/// Declaration-side metadata types grouped under one stable namespace.
pub mod metadata;
mod model_id;
mod model_impl_metadata;
mod model_metadata_error;
pub mod prelude;
mod property;
mod property_build_error;
mod property_build_error_kind;
mod property_build_errors;
mod property_fragment;
mod property_fragment_source;
mod property_resolution_error;
mod reflect_facade;
/// Frozen model registrations grouped under one stable namespace.
pub mod registry;
mod relation;
/// Cross-model resolution types grouped under one stable namespace.
pub mod resolve;
mod role;
mod type_metadata;
#[cfg(feature = "validation")]
pub mod validation;
