// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Static metadata for Rust domain models, with declaration overlays generated
//! by `qubit-model-derive` and backed by type shape information from
//! `qubit-reflect`.
//!
//! Reflection describes Rust types and their fields. The derive crate adds
//! model declarations such as roles, constraints, and property metadata. This
//! runtime crate exposes those declarations and resolves them against explicit
//! registries; it does not validate a model instance merely because metadata
//! contains a validation declaration.
//!
//! A typical application follows this path:
//!
//! 1. Read declaration metadata with [`metadata::TypeMetadata`].
//! 2. Build an explicit [`registry::ModelRegistry`] when registrations are
//!    needed.
//! 3. Resolve declarations with [`resolve::StructureResolver`].
//! 4. Optionally bind validation rules or codecs, then use a compiled
//!    [`PropertyAccessPath`] to access values.
//!
//! | Feature | Purpose |
//! | --- | --- |
//! | default (empty) | Declaration metadata, registries, structural resolution, and access paths. |
//! | `validation` | Build and execute validation plans using registered rules. |
//! | `codec` | Bind codec declarations to registered codecs. |
//! | `generic` | Inspect generic model definitions and concrete generic metadata. |
//!
//! The default feature set is empty. Validation and codec support are optional;
//! enable each feature only when the application uses that capability. A
//! declaration such as a uniqueness constraint describes intent. Reading it
//! does not validate an instance or issue a database query.
//!
//! # Read a declaration
//!
//! This example uses only the default runtime features. It reads metadata
//! emitted by `qubit-model-derive`; it does not construct a validation plan or
//! validate a `Profile` value.
//!
//! ```
//! use qubit_model_derive::Model;
//! use qubit_model_metadata::metadata::TypeMetadata;
//!
//! #[Model]
//! struct Profile {
//!     #[text(non_blank)]
//!     name: String,
//! }
//!
//! fn main() {
//!     let metadata = TypeMetadata::of::<Profile>();
//!     let name = metadata.field("name").expect("declared field");
//!     assert!(name.text_constraint().is_some());
//! }
//! ```
//!
//! The [runtime user guide](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.md)
//! explains registration, resolution, and optional features. For declaring
//! models, see the [derive user guide](https://github.com/qubit-ltd/rs-model-metadata/blob/main/derive/doc/user_guide.md).

// Gives derives the same absolute path inside this crate and in its doctests.
pub extern crate self as qubit_model_metadata;

#[doc(hidden)]
pub mod __private;
mod abi_violation;
/// Compiled dynamic property access paths.
pub mod access_path;
pub use access_path::PropertyAccessPath;
pub use access_path::PropertyAccessPathError;
pub use access_path::PropertyAccessWriteFailure;
#[cfg(feature = "codec")]
pub mod codec;
mod constraint;
mod constraint_metadata;
mod field_metadata;
#[cfg(feature = "generic")]
pub mod generic;
mod local_property_set;
/// Declaration-side metadata types grouped under one stable namespace.
pub mod metadata;
mod metadata_vocabulary;
mod model_impl_metadata;
mod model_metadata_error;
pub mod prelude;
mod property;
mod property_accessor_conflict;
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
mod resolved_properties;
mod resolved_property_fragments;
mod role;
mod structure;
#[cfg(test)]
mod tests;
mod transparent_descriptor;
mod type_metadata;
#[cfg(feature = "validation")]
pub mod validation;
