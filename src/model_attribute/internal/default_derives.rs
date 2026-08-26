// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Default derive selection for model declarations.

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DataEnum;
use syn::Error;
use syn::Fields;
use syn::Result;
use syn::parse_quote;

use crate::disabled_capabilities::DisabledCapabilities;

/// Builds the standard derives selected by the declaration shape.
///
/// # Parameters
///
/// - `data`: The declaration shape that determines available derives.
/// - `serde`: The resolved path to Serde in the consuming crate.
/// - `disabled`: Capabilities explicitly disabled by the macro invocation.
/// - `redacted`: Whether redaction supplies selected implementations.
///
/// # Returns
///
/// Returns the generated `derive` attribute.
///
/// # Errors
///
/// Returns an error when the declaration is a union or requests an unsupported
/// capability.
pub(crate) fn default_derives(
    data: &Data,
    serde: &TokenStream,
    disabled: &DisabledCapabilities,
    redacted: bool,
) -> Result<Attribute> {
    match data {
        Data::Enum(data) => enum_derives(data, serde, disabled, redacted),
        Data::Struct(_) => struct_derives(serde, disabled, redacted),
        Data::Union(union) => Err(Error::new_spanned(
            union.union_token,
            "Model attribute does not support unions",
        )),
    }
}

/// Builds enum defaults, including shape-dependent `Copy` support.
///
/// # Parameters
///
/// - `data`: The enum declaration being rewritten.
/// - `serde`: The resolved path to Serde in the consuming crate.
/// - `disabled`: Capabilities explicitly disabled by the macro invocation.
/// - `redacted`: Whether redaction supplies selected implementations.
///
/// # Returns
///
/// Returns the generated enum `derive` attribute.
fn enum_derives(
    data: &DataEnum,
    serde: &TokenStream,
    disabled: &DisabledCapabilities,
    redacted: bool,
) -> Result<Attribute> {
    let mut derives = Vec::new();
    if !disabled.clone {
        derives.push(quote!(Clone));
    }
    let fieldless = data
        .variants
        .iter()
        .all(|variant| matches!(variant.fields, Fields::Unit));
    if fieldless && !disabled.copy && !disabled.clone {
        derives.push(quote!(Copy));
    }
    if !disabled.debug && !redacted {
        derives.push(quote!(Debug));
    }
    if !disabled.eq {
        derives.push(quote!(Eq));
    }
    if !disabled.partial_eq {
        derives.push(quote!(PartialEq));
    }
    if !disabled.partial_ord {
        derives.push(quote!(PartialOrd));
    }
    if !disabled.ord {
        derives.push(quote!(Ord));
    }
    if !disabled.hash {
        derives.push(quote!(Hash));
    }
    if !disabled.serialize && !redacted {
        derives.push(quote!(#serde::Serialize));
    }
    if !disabled.deserialize {
        derives.push(quote!(#serde::Deserialize));
    }

    Ok(parse_quote!(#[derive(#(#derives),*)]))
}

/// Builds the standard defaults for a struct declaration.
///
/// # Parameters
///
/// - `serde`: The resolved path to Serde in the consuming crate.
/// - `disabled`: Capabilities explicitly disabled by the macro invocation.
/// - `redacted`: Whether redaction supplies selected implementations.
///
/// # Returns
///
/// Returns the generated struct `derive` attribute.
///
/// # Errors
///
/// Returns an error when `no_copy` is requested, because structs do not opt in
/// to generated `Copy` support.
fn struct_derives(serde: &TokenStream, disabled: &DisabledCapabilities, redacted: bool) -> Result<Attribute> {
    if disabled.copy {
        return Err(Error::new(Span::call_site(), "`no_copy` is only supported on enums"));
    }
    let mut derives = Vec::new();
    if !disabled.clone {
        derives.push(quote!(Clone));
    }
    if !disabled.debug && !redacted {
        derives.push(quote!(Debug));
    }
    if !disabled.eq {
        derives.push(quote!(Eq));
    }
    if !disabled.partial_eq {
        derives.push(quote!(PartialEq));
    }
    if !disabled.hash {
        derives.push(quote!(Hash));
    }
    if !disabled.serialize && !redacted {
        derives.push(quote!(#serde::Serialize));
    }
    if !disabled.deserialize {
        derives.push(quote!(#serde::Deserialize));
    }

    Ok(parse_quote!(#[derive(#(#derives),*)]))
}
