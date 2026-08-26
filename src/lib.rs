// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Derive macros for `qubit-model-metadata`.

mod attribute;
mod attribute_support;
mod derive_model_impl;
mod disabled_capabilities;
mod enum_attribute;
mod expand;
mod input;
mod model_attribute;
mod model_options;
mod normalize;
mod runtime_path;
mod validate;

#[cfg(test)]
mod tests;

use proc_macro::TokenStream;

/// Declares a struct model and derives its standard capabilities and metadata.
///
/// The macro accepts named and unit structs and single-field tuple newtypes.
/// Use [`Enum`] for enums. It emits compile errors for unsupported
/// declaration shapes or when required runtime dependencies cannot be resolved
/// from the consuming crate's dependencies.
///
/// # Parameters
///
/// - `args`: The type-level model arguments, including the required `id`.
/// - `input`: The token stream containing the model declaration and its
///   standalone field helper attributes such as `#[identifier]` and
///   `#[indexed]`.
///
/// # Returns
///
/// Returns the rewritten declaration, generated implementations, and static
/// metadata, or compile-error tokens when the declaration is invalid.
///
/// # Errors
///
/// Diagnostics are returned as compile-error tokens for unsupported model
/// shapes, invalid attributes, and missing runtime dependencies.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Model;
///
/// #[Model(id = "example.Item")]
/// struct Item {
///     name: String,
/// }
///
/// let item = Item {
///     name: "alpha".to_owned(),
/// };
/// assert_eq!(item.name, "alpha");
/// ```
#[proc_macro_attribute /* required by the style checker */]
#[allow(non_snake_case)]
pub fn Model(args: TokenStream, input: TokenStream) -> TokenStream {
    model_attribute::expand(args.into(), input.into()).into()
}

/// Declares an enum with metadata and canonical serialized names.
///
/// The macro adds `#[must_use]` unless the declaration already has one. Every
/// enum receives `name`, which returns the Serde serialization name of its
/// current variant. An enum whose variants are all unit variants also receives
/// `from_name`.
///
/// # Parameters
///
/// - `args`: The type-level enum arguments, including the required `id`.
/// - `input`: The token stream containing the enum declaration.
///
/// # Returns
///
/// Returns the rewritten declaration, generated implementations, and static
/// metadata, or compile-error tokens when the declaration is invalid.
///
/// # Errors
///
/// Diagnostics are returned as compile-error tokens for invalid payload field
/// attributes, duplicate serialized names, generics, invalid attributes, and
/// missing runtime dependencies.
///
/// # Examples
///
/// ```
/// use qubit_model_derive::Enum;
///
/// #[Enum(id = "example.Status")]
/// enum Status {
///     Active,
///     Suspended,
/// }
///
/// assert_eq!(Status::Active.name(), "ACTIVE");
/// assert_eq!(Status::from_name("SUSPENDED"), Some(Status::Suspended));
/// ```
#[proc_macro_attribute /* required by the style checker */]
#[allow(non_snake_case)]
pub fn Enum(args: TokenStream, input: TokenStream) -> TokenStream {
    enum_attribute::expand(args.into(), input.into()).into()
}
