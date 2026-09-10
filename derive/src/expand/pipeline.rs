// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Runs the parse, normalization, validation, and token-expansion pipeline.

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;
use syn::ItemImpl;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::parse::Parser;
use syn::parse_quote;
use syn::parse2;
use syn::punctuated::Punctuated;

use crate::expand::metadata::expand_metadata;
use crate::expand::model_impl::expand_model_impl;
use crate::ir::MacroKind;
use crate::normalize::declaration::normalize_declaration;
use crate::normalize::declaration::validate_declaration_ir;
use crate::parse::declaration::parse_declaration;
use crate::runtime_path::runtime_path;
use crate::validate::declaration::reject_duplicate_reflect;
use crate::validate::declaration::rewrite_field_helpers;
use crate::validate::declaration::validate_declaration;

/// Parses, validates, and expands one declaration into generated tokens.
pub(crate) fn run(kind: MacroKind, args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let raw_options = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(args)?;
    if kind == MacroKind::ModelImpl {
        let item: ItemImpl = parse2(input)?;
        let runtime = runtime_path()?;
        return expand_model_impl(item, &runtime, quote!(#raw_options));
    }

    let mut item: DeriveInput = parse2(input)?;
    let runtime = runtime_path();
    if let Err(mut validation) = validate_declaration(kind, &item) {
        if let Err(runtime_error) = runtime {
            validation.combine(runtime_error);
        }
        return Err(validation);
    }
    let runtime = runtime?;
    reject_duplicate_reflect(&item.attrs)?;
    let mut declaration = parse_declaration(kind, raw_options, &item)?;
    normalize_declaration(&mut declaration);
    validate_declaration_ir(&declaration, &item)?;
    let output = super::output::prepare(&mut item, &declaration, &runtime)?;
    rewrite_field_helpers(&mut item.data, &declaration);
    item.attrs.push(parse_quote!(#[derive(#runtime::__private::Reflect)]));
    item.attrs.push(parse_quote!(#[reflect(crate = #runtime)]));
    if !item.generics.params.is_empty() {
        let provider = format_ident!("__qubit_model_reflect_definition_{}", item.ident);
        item.attrs
            .push(parse_quote!(#[reflect(definition_provider_v2 = #provider)]));
    }
    item.attrs
        .push(parse_quote!(#[reflect(capabilities(#runtime::__private::v7::model_capability))]));
    let metadata = expand_metadata(&declaration, &item, &runtime);
    let expanded = quote!(#item #metadata #output);
    if item.generics.params.is_empty() {
        Ok(expanded)
    } else {
        Ok(quote!(#runtime::__private::v7::with_generic_feature! { #expanded }))
    }
}
