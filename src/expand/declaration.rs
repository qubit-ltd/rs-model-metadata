// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Compiles non-property model declarations into reflection and metadata
//! tokens.

// qubit-style: allow multiple-public-types
// The private intermediate representations below are one compiler-stage unit.
// qubit-style: allow explicit-imports
// Generated token streams deliberately retain absolute paths for downstream
// hygiene.

use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;
use syn::Error;
use syn::ItemImpl;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::parse::Parser;
use syn::parse_quote;
use syn::parse2;
use syn::punctuated::Punctuated;

use crate::expand::model_impl::expand_model_impl;
use crate::expand::model_impl::validate_model_impl;
use crate::ir::MacroKind;
use crate::runtime_path::runtime_path;

#[path = "../ir/declaration.rs"]
mod declaration_ir;
#[path = "../normalize/declaration.rs"]
mod declaration_normalize;
#[path = "../parse/declaration.rs"]
mod declaration_parse;
#[path = "../validate/declaration.rs"]
mod declaration_validate;
#[path = "capabilities.rs"]
mod capabilities;
#[path = "declaration_codegen.rs"]
mod declaration_codegen;

use capabilities::apply_default_derives;
use capabilities::apply_serde_defaults;
use capabilities::expand_display;
use declaration_codegen::expand_metadata;
use declaration_ir::DeclarationIr;
use declaration_validate::reject_duplicate_reflect;
use declaration_validate::rewrite_field_helpers;
use declaration_validate::validate_declaration;

/// Expands one declaration and converts all failures to compiler diagnostics.
pub(crate) fn expand(kind: MacroKind, args: TokenStream, input: TokenStream) -> TokenStream {
    expand_result(kind, args, input).unwrap_or_else(Error::into_compile_error)
}

/// Parses, validates, and expands one declaration into generated tokens.
fn expand_result(kind: MacroKind, args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let raw_options = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(args)?;
    if kind == MacroKind::ModelImpl {
        let item: ItemImpl = parse2(input)?;
        validate_model_impl(&item)?;
        let runtime = runtime_path()?;
        return expand_model_impl(item, &runtime);
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
    let mut declaration = DeclarationIr::parse(kind, raw_options, &item)?;
    apply_default_derives(&declaration, &mut item, &runtime)?;
    apply_serde_defaults(&mut declaration, &mut item, &runtime);
    rewrite_field_helpers(&mut item.data, &declaration);
    item.attrs.push(parse_quote!(#[derive(#runtime::Reflect)]));
    item.attrs.push(parse_quote!(#[reflect(crate = #runtime)]));
    item.attrs
        .push(parse_quote!(#[reflect(capabilities(#runtime::__private::v3::model_capability))]));
    let display = expand_display(&declaration, &item, &runtime);
    let metadata = expand_metadata(&declaration, &item, &runtime);
    Ok(quote!(#item #display #metadata))
}
