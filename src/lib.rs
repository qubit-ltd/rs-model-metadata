// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Attribute macros for the five Qubit model roles and model properties.

mod entry;
mod expand;
mod ir;

mod runtime_path;

use ir::MacroKind;
use proc_macro::TokenStream;

/// Declares a persistent entity.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Entity(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Entity, args, input)
}

/// Declares an entity projection.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Projection(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Projection, args, input)
}

/// Declares an ordinary structured model.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Model(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Model, args, input)
}

/// Declares a model enum.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Enum(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Enum, args, input)
}

/// Declares a domain value type.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Value(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::Value, args, input)
}

/// Declares getter/setter-backed model properties on an inherent impl.
#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn ModelProperties(args: TokenStream, input: TokenStream) -> TokenStream {
    entry::expand(MacroKind::ModelProperties, args, input)
}
