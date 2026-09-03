// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Attribute macros for the five Qubit model roles and model properties.
//!
//! A role macro turns one Rust declaration into reflection metadata plus the
//! model-specific semantics needed by Qubit runtimes. Use [`Entity`] and
//! [`Projection`] for identity-bearing records and views, [`Model`] for
//! structured data, [`Enum`] for domain enumerations, [`Value`] for value
//! objects, and [`ModelImpl`] for getter/setter-backed properties.
//!
//! # Example
//!
//! ```
//! use qubit_id::Id;
//! use qubit_model_derive::{Entity, ModelImpl};
//! use model_runtime::TypeMetadata;
//!
//! #[Entity(id = "example.Document")]
//! pub struct Document {
//!     #[identifier]
//!     id: Id,
//!     title: String,
//! }
//!
//! #[ModelImpl]
//! impl Document {
//!     pub fn title(&self) -> &str {
//!         &self.title
//!     }
//!
//!     pub fn set_title(&mut self, title: String) {
//!         self.title = title;
//!     }
//! }
//!
//! fn main() {
//!     let metadata = TypeMetadata::of::<Document>();
//!     assert!(metadata.field("id").expect("id field").is_identifier());
//!     assert!(metadata
//!         .try_property("title")
//!         .expect("valid properties")
//!         .expect("title property")
//!         .is_writable());
//! }
//! ```
//!
//! Cross-model IDs, references, validators, and codecs are resolved only after
//! all participating crates are linked. See the repository user guide for the
//! complete role, field, and resolver contracts.

mod entry;
mod expand;
mod ir;
mod normalize;
mod parse;
mod validate;

mod compiler;

mod runtime_path;

use ir::MacroKind;
use proc_macro::TokenStream;

/// Compiles an identity-bearing persistent entity declaration.
///
/// The attribute arguments configure the entity's stable model identity and
/// behavior; `input` is the annotated struct declaration. Returns generated
/// Rust tokens or compiler diagnostics when the declaration is invalid.
///
/// An entity must be a non-generic named struct with exactly one
/// `#[identifier]` field whose type is `qubit_id::Id`.
#[allow(non_snake_case)]
#[proc_macro_attribute /* keep separated for the rs-ci aggregation scanner */]
pub fn Entity(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Entity, args, input)
}

/// Compiles an entity projection declaration.
///
/// The attribute arguments configure the projection source and identity;
/// `input` is the annotated struct declaration. Returns generated Rust tokens
/// or compiler diagnostics when the declaration is invalid.
///
/// A projection must be a non-generic named struct with one `Id` identifier.
/// It may be open or declare exactly one fixed source by Rust type or model ID.
#[allow(non_snake_case)]
#[proc_macro_attribute /* keep separated for the rs-ci aggregation scanner */]
pub fn Projection(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Projection, args, input)
}

/// Compiles an ordinary structured model declaration.
///
/// The attribute arguments configure model behavior; `input` is the annotated
/// struct declaration. Returns generated Rust tokens or compiler diagnostics
/// when the declaration is invalid.
///
/// Models accept named and unit structs. Named fields may define an ordered
/// logical key with `#[key_part(order = n)]`.
#[allow(non_snake_case)]
#[proc_macro_attribute /* keep separated for the rs-ci aggregation scanner */]
pub fn Model(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Model, args, input)
}

/// Compiles a domain model enum declaration.
///
/// The attribute arguments configure enum behavior; `input` is the annotated
/// enum declaration. Returns generated Rust tokens or compiler diagnostics when
/// the declaration is invalid.
///
/// Each variant retains distinct Rust, canonical model, and Serde names.
#[allow(non_snake_case)]
#[proc_macro_attribute /* keep separated for the rs-ci aggregation scanner */]
pub fn Enum(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Enum, args, input)
}

/// Compiles a domain value-type declaration.
///
/// The attribute arguments configure value behavior; `input` is the annotated
/// struct declaration. Returns generated Rust tokens or compiler diagnostics
/// when the declaration is invalid.
///
/// Values accept named structs or one-field tuple structs. Only named value
/// fields can participate in an ordered `key_part` logical key.
#[allow(non_snake_case)]
#[proc_macro_attribute /* keep separated for the rs-ci aggregation scanner */]
pub fn Value(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Value, args, input)
}

/// Compiles model-aware getters, setters, and projection producers on an
/// inherent implementation.
///
/// This attribute accepts no configuration arguments. `input` must be an
/// inherent implementation whose public, synchronous methods follow the getter
/// or setter contract. Returns generated Rust tokens or compiler diagnostics.
///
/// Getter output may be owned or one of the supported borrowed forms. Setters
/// require `&mut self`, one owned value parameter, and a unit return type.
#[allow(non_snake_case)]
#[proc_macro_attribute /* keep separated for the rs-ci aggregation scanner */]
pub fn ModelImpl(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::ModelImpl, args, input)
}
