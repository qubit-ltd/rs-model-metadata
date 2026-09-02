// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Attribute macros for the five Qubit model roles and model properties.

mod entry;
mod expand;
mod ir;

mod compiler;

mod runtime_path;

use ir::MacroKind;
use proc_macro::TokenStream;

/// Compiles an identity-bearing persistent entity declaration.
///
/// The attribute arguments configure the entity's stable model identity and
/// behavior; `input` is the annotated struct declaration. Returns generated
/// Rust tokens or compiler diagnostics when the declaration is invalid.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Entity(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Entity, args, input)
}

/// Compiles an entity projection declaration.
///
/// The attribute arguments configure the projection source and identity;
/// `input` is the annotated struct declaration. Returns generated Rust tokens
/// or compiler diagnostics when the declaration is invalid.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Projection(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Projection, args, input)
}

/// Compiles an ordinary structured model declaration.
///
/// The attribute arguments configure model behavior; `input` is the annotated
/// struct declaration. Returns generated Rust tokens or compiler diagnostics
/// when the declaration is invalid.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Model(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Model, args, input)
}

/// Compiles a domain model enum declaration.
///
/// The attribute arguments configure enum behavior; `input` is the annotated
/// enum declaration. Returns generated Rust tokens or compiler diagnostics when
/// the declaration is invalid.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Enum(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Enum, args, input)
}

/// Compiles a domain value-type declaration.
///
/// The attribute arguments configure value behavior; `input` is the annotated
/// struct declaration. Returns generated Rust tokens or compiler diagnostics
/// when the declaration is invalid.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Value(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Value, args, input)
}

/// Compiles model-aware getters, setters, and projection producers on an
/// inherent implementation.
///
/// This attribute accepts no configuration arguments. `input` must be an
/// inherent implementation whose public, synchronous methods follow the getter
/// or setter contract. Returns generated Rust tokens or compiler diagnostics.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn ModelImpl(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::ModelImpl, args, input)
}
