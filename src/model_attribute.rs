// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Expansion of the `Model` attribute macro.

mod internal;

use internal::add_default_serde_field_attributes;
use internal::default_derives;
use internal::expand_display;
use internal::expand_enum_names;
use internal::has_redact_fields;
use internal::remove_field_attributes;
use internal::remove_serde_attributes;
use proc_macro_crate::FoundCrate;
use proc_macro_crate::crate_name;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Ident;
use syn::LitStr;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::parse::Parser;
use syn::parse_quote;
use syn::parse2;
use syn::punctuated::Punctuated;

use crate::attribute_support::has_must_use;
use crate::derive_model_impl::derive_model_tokens;
use crate::model_options::ModelOptions;
use crate::runtime_path::runtime_path;

/// Expands one model declaration and converts failures to compiler diagnostics.
#[must_use]
pub(crate) fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    match expand_result(args, input, false) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error(),
    }
}

/// Expands the public `Enum` attribute macro.
#[must_use]
pub(crate) fn expand_enum(args: TokenStream, input: TokenStream) -> TokenStream {
    match expand_result(args, input, true) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error(),
    }
}

/// Expands one model declaration with default traits and metadata.
fn expand_result(args: TokenStream, input: TokenStream, enum_declaration: bool) -> Result<TokenStream> {
    let attributes = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(args)?;
    let options = ModelOptions::parse(attributes)?;
    let mut item: DeriveInput = parse2(input)?;
    match (&item.data, enum_declaration) {
        (Data::Struct(_), false) => {}
        (Data::Enum(_), true) => {}
        (Data::Enum(_), false) => {
            return Err(Error::new_spanned(
                &item.ident,
                "#[Model] only supports structs; use #[Enum] for enums",
            ));
        }
        (Data::Struct(_), true) => {
            return Err(Error::new_spanned(&item.ident, "#[Enum] only supports enums"));
        }
        (Data::Union(union), _) => {
            return Err(Error::new_spanned(
                union.union_token,
                "model declaration attributes do not support unions",
            ));
        }
    }
    let serde = dependency_path("serde", "Model attribute requires the `serde` dependency")?;
    let rename_all = rename_all_rule(&item.data)?;
    let mut metadata_input = item.clone();
    let metadata_attributes = &options.metadata;
    let redacted = options.redact || has_redact_fields(&item.data);

    metadata_input
        .attrs
        .push(parse_quote!(#[model(#(#metadata_attributes),*)]));
    add_default_serde_field_attributes(
        &mut item.data,
        !options.disabled.serialize,
        !options.disabled.deserialize,
    )?;
    remove_field_attributes(&mut item.data);
    if options.disabled.serialize && options.disabled.deserialize && !redacted {
        remove_serde_attributes(&mut item);
    }

    let derives = default_derives(&item.data, &serde, &options.disabled, redacted)?;
    item.attrs.push(derives);
    if enum_declaration && !has_must_use(&item.attrs) {
        item.attrs.push(parse_quote!(#[must_use]));
    }
    if redacted {
        let redact = dependency_path("qubit-redact", "Model redaction requires the `qubit-redact` dependency")?;
        item.attrs.push(parse_quote!(#[derive(#redact::Redact)]));
        if !options.disabled.serialize {
            item.attrs.push(parse_quote!(#[redact(serde)]));
        }
        if !options.disabled.debug {
            item.attrs.push(parse_quote!(#[redact(debug)]));
        }
        if !options.disabled.display {
            item.attrs.push(parse_quote!(#[redact(display)]));
        }
    }
    if !options.disabled.serialize || !options.disabled.deserialize {
        item.attrs.push(parse_quote!(#[serde(rename_all = #rename_all)]));
    }
    let enum_names = enum_declaration
        .then(|| expand_enum_names(&metadata_input))
        .transpose()?
        .unwrap_or_default();
    let metadata = derive_model_tokens(metadata_input.into_token_stream(), runtime_path());
    let display = (!redacted && !options.disabled.display)
        .then(|| expand_display(&item, rename_all))
        .transpose()?
        .unwrap_or_default();
    Ok(quote!(#item #metadata #enum_names #display))
}

/// Resolves one dependency path in the consuming crate.
fn dependency_path(package: &str, diagnostic: &str) -> Result<TokenStream> {
    match crate_name(package) {
        Ok(FoundCrate::Itself) => Ok(quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            Ok(quote!(::#ident))
        }
        Err(_) => Err(Error::new(Span::call_site(), diagnostic)),
    }
}

/// Returns the enforced Serde rename rule for one supported item shape.
fn rename_all_rule(data: &Data) -> Result<LitStr> {
    match data {
        Data::Enum(_) => Ok(LitStr::new("SCREAMING_SNAKE_CASE", Span::call_site())),
        Data::Struct(_) => Ok(LitStr::new("snake_case", Span::call_site())),
        Data::Union(union) => Err(Error::new_spanned(
            union.union_token,
            "Model attribute does not support unions",
        )),
    }
}
