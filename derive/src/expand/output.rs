// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Role-default Rust capabilities and delegated redacted output.

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Fields;
use syn::Generics;
use syn::LitStr;
use syn::Member;
use syn::Meta;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use crate::ir::MacroKind;
use crate::ir::declaration::DeclarationIr;
use crate::ir::declaration::FieldOccurrence;

/// Adds role defaults while preserving visible explicit derives.
///
/// Returns additional implementations, or a targeted error for conflicting
/// switches and output declarations. Field output is delegated to qubit-redact.
pub(crate) fn prepare(
    item: &mut DeriveInput,
    declaration: &DeclarationIr,
    runtime: &TokenStream,
) -> Result<TokenStream> {
    if !declaration.options.behavior.contains("no_deserialize") {
        add_deserialize_bounds(item, runtime)?;
    }
    let options = &declaration.options.behavior;
    let explicit = explicit_derives(item)?;
    if options.contains("no_copy") && !matches!(item.data, Data::Enum(_)) {
        return Err(Error::new_spanned(&item.ident, "no_copy is only supported by Enum"));
    }
    let all_unit = matches!(&item.data, Data::Enum(data) if data.variants.iter().all(|variant| matches!(variant.fields, Fields::Unit)));
    if (options.contains("ord") && (options.contains("no_eq") || options.contains("no_partial_eq")))
        || (options.contains("copy") && (options.contains("no_clone") || options.contains("no_copy")))
        || (options.contains("partial_ord") && options.contains("no_partial_eq"))
    {
        return Err(Error::new_spanned(&item.ident, "conflicting model capability options"));
    }
    let fields: Vec<_> = match &item.data {
        Data::Struct(data) => data.fields.iter().collect(),
        Data::Enum(data) => data.variants.iter().flat_map(|variant| variant.fields.iter()).collect(),
        Data::Union(_) => Vec::new(),
    };
    let has_redaction = fields
        .iter()
        .any(|field| field.attrs.iter().any(|attr| attr.path().is_ident("redact")))
        || declaration
            .fields
            .iter()
            .chain(declaration.variants.iter().flat_map(|variant| &variant.fields))
            .any(|field| {
                field.occurrences.iter().any(|occurrence| {
                    matches!(occurrence,
                FieldOccurrence::Selector(selector) if selector.redact.is_some())
                })
            });
    if options.contains("no_redact") && has_redaction {
        return Err(Error::new_spanned(
            &item.ident,
            "no_redact conflicts with field or selector redaction",
        ));
    }
    if has_redaction
        && ["Debug", "Display", "Serialize"]
            .iter()
            .any(|name| explicit.contains(*name))
    {
        return Err(Error::new_spanned(
            &item.ident,
            "explicit output derive conflicts with model redaction; use model output or an explicit opt-out and handwritten implementation",
        ));
    }
    let enabled = |name: &str| !options.contains(&format!("no_{name}"));
    let mut derives: Vec<TokenStream> = Vec::new();
    let mut implementations = Vec::new();
    let source = item.clone();
    let mut add = |name: &str, path: TokenStream, wanted: bool| {
        if wanted && !explicit.contains(name) {
            if !source.generics.params.is_empty()
                && (matches!(
                    name,
                    "Clone" | "PartialEq" | "Eq" | "Hash" | "Copy" | "Debug" | "PartialOrd" | "Ord"
                ) || (matches!(source.data, Data::Struct(_)) && name == "Default"))
            {
                implementations.push(super::structural_traits::expand(&source, name));
            } else {
                derives.push(path);
            }
        }
    };
    add("Clone", quote!(::core::clone::Clone), enabled("clone"));
    add("PartialEq", quote!(::core::cmp::PartialEq), enabled("partial_eq"));
    add("Eq", quote!(::core::cmp::Eq), enabled("eq") && enabled("partial_eq"));
    add(
        "Hash",
        quote!(::core::hash::Hash),
        enabled("hash") && enabled("eq") && enabled("partial_eq"),
    );
    add(
        "Copy",
        quote!(::core::marker::Copy),
        ((all_unit && enabled("clone")) || options.contains("copy")) && enabled("copy"),
    );
    add("Default", quote!(::core::default::Default), options.contains("default"));
    add(
        "PartialOrd",
        quote!(::core::cmp::PartialOrd),
        options.contains("partial_ord") || options.contains("ord"),
    );
    add("Ord", quote!(::core::cmp::Ord), options.contains("ord"));
    let redact = enabled("redact");
    let mut controls = vec![quote!(crate = #runtime::__private::redact)];
    if redact {
        add("Redact", quote!(#runtime::__private::redact::Redact), true);
        if enabled("debug") && !explicit.contains("Debug") {
            controls.push(quote!(debug));
        }
        if enabled("display") && !explicit.contains("Display") {
            controls.push(quote!(display));
        }
        if enabled("serialize") && !explicit.contains("Serialize") {
            controls.push(quote!(serde));
        }
        if declaration.options.transparent {
            controls.push(quote!(transparent));
        }
        item.attrs.push(parse_quote!(#[redact(#(#controls),*)]));
    } else {
        add("Debug", quote!(::core::fmt::Debug), enabled("debug"));
        add(
            "Serialize",
            quote!(#runtime::__private::serde::Serialize),
            enabled("serialize"),
        );
    }
    add(
        "Deserialize",
        quote!(#runtime::__private::serde::Deserialize),
        enabled("deserialize"),
    );
    if !derives.is_empty() {
        item.attrs.insert(0, parse_quote!(#[derive(#(#derives),*)]));
    }
    if enabled("serialize") || enabled("deserialize") || redact {
        let serde_path = LitStr::new(&quote!(#runtime::__private::serde).to_string(), item.ident.span());
        if !has_serde_control(item, "crate")? {
            item.attrs.push(parse_quote!(#[serde(crate = #serde_path)]));
        }
        if declaration.options.transparent && !has_serde_control(item, "transparent")? {
            item.attrs.push(parse_quote!(#[serde(transparent)]));
        }
        apply_enum_default_serde_wire_names(item, declaration)?;
    }
    if !redact && enabled("display") && !explicit.contains("Display") {
        implementations.push(plain_display(item, declaration.options.transparent)?);
    }
    Ok(quote!(#(#implementations)*))
}

/// Collects visible derive names; independent impls remain caller-managed.
fn explicit_derives(item: &DeriveInput) -> Result<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    for attribute in &item.attrs {
        if attribute.path().is_ident("derive") {
            for path in attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)? {
                if let Some(segment) = path.segments.last() {
                    result.insert(segment.ident.to_string());
                }
            }
        }
    }
    Ok(result)
}

/// Builds non-redacted Display without requiring a public Debug impl on Self.
fn plain_display(item: &DeriveInput, transparent: bool) -> Result<TokenStream> {
    let name = &item.ident;
    let mut generics = item.generics.clone();
    let body = match &item.data {
        Data::Struct(data) => {
            let mut fields = Vec::new();
            for (index, field) in data.fields.iter().enumerate() {
                let ty = &field.ty;
                let member = field
                    .ident
                    .clone()
                    .map_or_else(|| Member::Unnamed(index.into()), Member::Named);
                if transparent {
                    generics
                        .make_where_clause()
                        .predicates
                        .push(parse_quote!(#ty: ::core::fmt::Display));
                    return Ok(display_impl(
                        item,
                        &generics,
                        quote!(::core::fmt::Display::fmt(&self.#member, formatter)),
                    ));
                }
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: ::core::fmt::Debug));
                let label = field
                    .ident
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| index.to_string());
                fields.push(quote!(builder.field(#label, &self.#member);));
            }
            quote! { let mut builder = formatter.debug_struct(stringify!(#name)); #(#fields)* builder.finish() }
        }
        Data::Enum(data) => {
            let arms = data
                .variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    let label = variant_name.to_string();
                    let bindings: Vec<_> = (0..variant.fields.len())
                        .map(|index| format_ident!("__model_field_{index}"))
                        .collect();
                    for field in &variant.fields {
                        let ty = &field.ty;
                        generics
                            .make_where_clause()
                            .predicates
                            .push(parse_quote!(#ty: ::core::fmt::Debug));
                    }
                    match &variant.fields {
                        Fields::Named(fields) => {
                            let names: Vec<_> =
                                fields.named.iter().map(|field| field.ident.as_ref().unwrap()).collect();
                            let labels: Vec<_> = names.iter().map(ToString::to_string).collect();
                            quote!(Self::#variant_name { #(#names: #bindings),* } => {
                                let mut builder = formatter.debug_struct(#label);
                                #(builder.field(#labels, #bindings);)* builder.finish()
                            })
                        }
                        Fields::Unnamed(_) => quote!(Self::#variant_name(#(#bindings),*) => {
                            let mut builder = formatter.debug_tuple(#label);
                            #(builder.field(#bindings);)* builder.finish()
                        }),
                        Fields::Unit => quote!(Self::#variant_name => formatter.write_str(#label)),
                    }
                })
                .collect::<Vec<_>>();
            quote!(match self { #(#arms),* })
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(item, "model output does not support unions"));
        }
    };
    Ok(display_impl(item, &generics, body))
}

/// Emits the standalone formatting implementation with precise field bounds.
fn display_impl(item: &DeriveInput, generics: &Generics, body: TokenStream) -> TokenStream {
    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result { #body }
        }
    }
}

/// Uses field-shaped bounds for generic deserialization, including const
/// arrays.
fn add_deserialize_bounds(item: &mut DeriveInput, runtime: &TokenStream) -> Result<()> {
    if item.generics.params.is_empty() {
        return Ok(());
    }
    let name = &item.ident;
    let (_, arguments, _) = item.generics.split_for_impl();
    let recursive_type = quote!(#name #arguments).to_string();
    let fields: Vec<_> = match &mut item.data {
        Data::Struct(data) => data.fields.iter_mut().collect(),
        Data::Enum(data) => data
            .variants
            .iter_mut()
            .flat_map(|variant| variant.fields.iter_mut())
            .collect(),
        Data::Union(_) => Vec::new(),
    };
    for field in fields {
        let mut custom = false;
        for attr in &field.attrs {
            if attr.path().is_ident("serde") {
                let values = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                custom |= values.iter().any(|value| {
                    ["bound", "with", "deserialize_with", "skip", "skip_deserializing"]
                        .iter()
                        .any(|name| value.path().is_ident(name))
                });
            }
        }
        if custom {
            continue;
        }
        let types =
            super::structural_traits::recursive_bounds(&field.ty, &recursive_type).unwrap_or_else(|| vec![&field.ty]);
        let predicates: Vec<_> = types
            .iter()
            .map(|ty| quote!(#ty: #runtime::__private::serde::Deserialize<'de>))
            .collect();
        let bound = LitStr::new(&quote!(#(#predicates),*).to_string(), field.ty.span());
        field.attrs.push(parse_quote!(#[serde(bound(deserialize = #bound))]));
    }
    Ok(())
}

/// Installs default variant Serde names for Enum role declarations.
///
/// Wire names match metadata `serialized_name` / `deserialized_name` unless the
/// enum or variant already declares Serde renaming.
fn apply_enum_default_serde_wire_names(item: &mut DeriveInput, declaration: &DeclarationIr) -> Result<()> {
    if declaration.kind != MacroKind::Enum {
        return Ok(());
    }
    if has_serde_control(item, "rename_all")? {
        return Ok(());
    }
    let Data::Enum(data) = &mut item.data else {
        return Ok(());
    };
    for variant in &mut data.variants {
        if variant_has_serde_rename(&variant.attrs)? {
            continue;
        }
        let rust_name = variant.ident.to_string();
        let Some(variant_ir) = declaration.variants.iter().find(|candidate| candidate.rust_name == rust_name) else {
            continue;
        };
        let serialized = &variant_ir.serialized_name;
        let deserialized = &variant_ir.deserialized_name;
        if serialized == deserialized {
            let name = LitStr::new(serialized, variant.ident.span());
            variant.attrs.push(parse_quote!(#[serde(rename = #name)]));
        } else {
            let serialize = LitStr::new(serialized, variant.ident.span());
            let deserialize = LitStr::new(deserialized, variant.ident.span());
            variant
                .attrs
                .push(parse_quote!(#[serde(rename(serialize = #serialize, deserialize = #deserialize))]));
        }
    }
    Ok(())
}

/// Returns whether a variant already declares Serde rename options.
fn variant_has_serde_rename(attributes: &[syn::Attribute]) -> Result<bool> {
    for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("serde")) {
        for meta in attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
            if meta.path().is_ident("rename") {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Detects explicit container controls before adding model defaults.
fn has_serde_control(item: &DeriveInput, name: &str) -> Result<bool> {
    for attribute in &item.attrs {
        if attribute.path().is_ident("serde") {
            for meta in attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)? {
                if meta.path().is_ident(name) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}
