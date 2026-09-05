// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generates default trait capabilities, display behavior, and Serde defaults.

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Field;
use syn::Fields;
use syn::Ident;
use syn::Index;
use syn::LitStr;
use syn::Meta;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parse_quote;
use syn::punctuated::Punctuated;

use crate::compiler::type_path::is_collection_path;
use crate::compiler::type_path::is_option_path;
use crate::ir::MacroKind;
use crate::ir::declaration::DeclarationIr;
use crate::ir::declaration::FieldIr;
use crate::ir::declaration::FieldOccurrence;
use crate::ir::declaration::SerdeIr;
/// Adds role-dependent default derives while preserving user opt-outs.
pub(super) fn apply_default_derives(
    declaration: &DeclarationIr,
    item: &mut DeriveInput,
    runtime: &TokenStream,
) -> Result<()> {
    let options = &declaration.options;
    if options.copy && options.no_clone {
        return Err(Error::new_spanned(
            &item.ident,
            "`copy` requires Clone; remove `no_clone`",
        ));
    }
    if options.partial_ord && options.no_partial_eq {
        return Err(Error::new_spanned(&item.ident, "`partial_ord` requires PartialEq"));
    }
    if options.ord && (options.no_partial_eq || options.no_eq) {
        return Err(Error::new_spanned(&item.ident, "`ord` requires PartialEq and Eq"));
    }
    let existing = existing_derive_names(&item.attrs)?;
    let conflicts = [
        (options.no_clone, "Clone"),
        (options.no_copy, "Copy"),
        (options.no_partial_eq, "PartialEq"),
        (options.no_partial_eq || options.no_eq, "Eq"),
        (options.no_partial_eq || options.no_eq || options.no_hash, "Hash"),
        (options.no_partial_eq, "PartialOrd"),
        (options.no_partial_eq || options.no_eq, "Ord"),
    ];
    if let Some((_, capability)) = conflicts
        .into_iter()
        .find(|(disabled, capability)| *disabled && existing.iter().any(|name| name == capability))
    {
        return Err(Error::new_spanned(
            &item.ident,
            format!("explicit `{capability}` derive conflicts with the model capability switches"),
        ));
    }
    if !options.no_redact && !options.no_debug && existing.iter().any(|name| name == "Debug") {
        return Err(Error::new_spanned(
            &item.ident,
            "explicit Debug would bypass model redaction; use the generated safe implementation",
        ));
    }
    if !options.no_redact && !options.no_serialize && existing.iter().any(|name| name == "Serialize") {
        return Err(Error::new_spanned(
            &item.ident,
            "explicit Serialize would bypass model redaction; use the generated safe implementation",
        ));
    }
    let mut derives = Vec::new();
    let mut add = |name: &str, tokens: TokenStream| {
        if !existing.iter().any(|existing| existing == name) {
            derives.push(tokens);
        }
    };
    if !options.no_clone {
        add("Clone", quote!(Clone));
    }
    if options.no_redact && !options.no_debug {
        add("Debug", quote!(Debug));
    }
    if !options.no_partial_eq {
        add("PartialEq", quote!(PartialEq));
    }
    if !options.no_partial_eq && !options.no_eq {
        add("Eq", quote!(Eq));
    }
    if !options.no_partial_eq && !options.no_eq && !options.no_hash {
        add("Hash", quote!(Hash));
    }
    if options.ord {
        add("PartialOrd", quote!(PartialOrd));
        add("Ord", quote!(Ord));
    } else if options.partial_ord && !options.no_partial_eq {
        add("PartialOrd", quote!(PartialOrd));
    }
    let default_copy = !options.no_clone
        && !options.no_copy
        && declaration.kind == MacroKind::Enum
        && declaration.variants.iter().all(|variant| variant.fields.is_empty());
    if options.copy || default_copy {
        add("Copy", quote!(Copy));
    }
    if options.default {
        add("Default", quote!(Default));
    }
    if options.no_redact && !options.no_serialize {
        add("Serialize", quote!(#runtime::__private::serde::Serialize));
    }
    if !options.no_deserialize {
        add("Deserialize", quote!(#runtime::__private::serde::Deserialize));
    }
    if !options.no_redact {
        add("Redact", quote!(#runtime::__private::v4::Redact));
    }
    if !derives.is_empty() {
        item.attrs.push(parse_quote!(#[derive(#(#derives),*)]));
    }
    if !options.no_serialize || !options.no_deserialize {
        let path = format!("{}::__private::serde", runtime.to_string().replace(' ', ""));
        let path = LitStr::new(&path, Span::call_site());
        item.attrs.push(parse_quote!(#[serde(crate = #path)]));
        if declaration.kind == MacroKind::Enum && !has_serde_rename_all(&item.attrs)? {
            item.attrs
                .push(parse_quote!(#[serde(rename_all = "SCREAMING_SNAKE_CASE")]));
        }
        if options.transparent {
            item.attrs.push(parse_quote!(#[serde(transparent)]));
        }
    }
    if !options.no_redact {
        let mut flags = Vec::new();
        if !options.no_debug && !existing.iter().any(|value| value == "Debug") {
            flags.push(quote!(debug));
        }
        if !options.no_display && !options.transparent {
            flags.push(quote!(display));
        }
        if !options.no_serialize && !existing.iter().any(|value| value == "Serialize") {
            flags.push(quote!(serde));
        }
        if options.transparent {
            flags.push(quote!(transparent));
        }
        item.attrs.push(parse_quote!(#[redact(
            crate = #runtime::__private::qubit_redact,
            #(#flags),*
        )]));
    }
    Ok(())
}

/// Reports whether a declaration already supplies `serde(rename_all = ...)`.
fn has_serde_rename_all(attributes: &[Attribute]) -> Result<bool> {
    for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("serde")) {
        let entries = attribute.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        if entries.iter().any(|entry| entry.path().is_ident("rename_all")) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Generates the redaction-aware display implementation for a declaration.
pub(super) fn expand_display(declaration: &DeclarationIr, item: &DeriveInput, runtime: &TokenStream) -> TokenStream {
    let options = &declaration.options;
    if options.no_display || (!options.no_redact && !options.transparent) {
        return TokenStream::new();
    }
    let name = &item.ident;
    let mut generics = item.generics.clone();
    let transparent_field = if options.transparent {
        match &item.data {
            Data::Struct(data) => data.fields.iter().next(),
            Data::Enum(_) | Data::Union(_) => None,
        }
    } else {
        None
    };
    if options.no_redact {
        let where_clause = generics.make_where_clause();
        if let Some(field) = transparent_field {
            let ty = &field.ty;
            where_clause.predicates.push(parse_quote!(#ty: ::core::fmt::Display));
        } else {
            let fields: Vec<_> = match &item.data {
                Data::Struct(data) => data.fields.iter().collect(),
                Data::Enum(data) => data.variants.iter().flat_map(|variant| variant.fields.iter()).collect(),
                Data::Union(_) => Vec::new(),
            };
            for field in fields {
                let ty = &field.ty;
                where_clause.predicates.push(parse_quote!(#ty: ::core::fmt::Debug));
            }
        }
    }
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let body = if !options.no_redact {
        let Some(transparent_field) = transparent_field else {
            return Error::new_spanned(
                &item.ident,
                "redaction-aware transparent display requires exactly one field",
            )
            .into_compile_error();
        };
        let (prefix, suffix) = match transparent_field.ident.as_ref() {
            Some(field) => (format!("{} {{ {}: ", name, field), " }".to_owned()),
            None => (format!("{}(", name), ")".to_owned()),
        };
        quote! {
            let output = #runtime::__private::v4::Redactor::application_default().redact_text(self);
            let text = output.text().as_str();
            let text = text
                .strip_prefix(#prefix)
                .and_then(|text| text.strip_suffix(#suffix))
                .unwrap_or(text);
            let text = text
                .strip_prefix('"')
                .and_then(|text| text.strip_suffix('"'))
                .unwrap_or(text);
            formatter.write_str(text)
        }
    } else if let Some(field) = transparent_field {
        let access = field
            .ident
            .as_ref()
            .map_or_else(|| quote!(self.0), |field| quote!(self.#field));
        quote!(::core::fmt::Display::fmt(&#access, formatter))
    } else {
        plain_structured_display_body(name, &item.data)
    };
    quote! {
        impl #impl_generics ::core::fmt::Display for #name #type_generics #where_clause {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #body
            }
        }
    }
}

/// Generates a plain structured display body for non-redacted output.
fn plain_structured_display_body(name: &Ident, data: &Data) -> TokenStream {
    match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let names = fields.named.iter().filter_map(|field| field.ident.as_ref());
                quote! {
                    let mut debug = formatter.debug_struct(stringify!(#name));
                    #(debug.field(stringify!(#names), &self.#names);)*
                    debug.finish()
                }
            }
            Fields::Unnamed(fields) => {
                let indexes = (0..fields.unnamed.len()).map(Index::from);
                quote! {
                    let mut debug = formatter.debug_tuple(stringify!(#name));
                    #(debug.field(&self.#indexes);)*
                    debug.finish()
                }
            }
            Fields::Unit => quote!(formatter.write_str(stringify!(#name))),
        },
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unit => quote!(Self::#variant_name => formatter.write_str(stringify!(#variant_name))),
                    Fields::Unnamed(fields) => {
                        let bindings: Vec<_> = (0..fields.unnamed.len())
                            .map(|index| format_ident!("field_{index}"))
                            .collect();
                        quote! {
                            Self::#variant_name(#(#bindings),*) => {
                                let mut debug = formatter.debug_tuple(stringify!(#variant_name));
                                #(debug.field(#bindings);)*
                                debug.finish()
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let bindings: Vec<_> = fields.named.iter().filter_map(|field| field.ident.as_ref()).collect();
                        quote! {
                            Self::#variant_name { #(#bindings),* } => {
                                let mut debug = formatter.debug_struct(stringify!(#variant_name));
                                #(debug.field(stringify!(#bindings), #bindings);)*
                                debug.finish()
                            }
                        }
                    }
                }
            });
            quote!(match self { #(#arms,)* })
        }
        Data::Union(_) => TokenStream::new(),
    }
}

/// Adds default Serde attributes required by the selected role options.
pub(super) fn apply_serde_defaults(declaration: &mut DeclarationIr, item: &mut DeriveInput, runtime: &TokenStream) {
    match (&mut item.data, declaration.kind) {
        (Data::Struct(data), _) => {
            for (field, ir) in data.fields.iter_mut().zip(&mut declaration.fields) {
                apply_field_serde_default(field, ir, runtime);
            }
        }
        (Data::Enum(data), MacroKind::Enum) => {
            for (variant, variant_ir) in data.variants.iter_mut().zip(&mut declaration.variants) {
                if !matches!(variant.fields, Fields::Named(_)) {
                    continue;
                }
                for (field, ir) in variant.fields.iter_mut().zip(&mut variant_ir.fields) {
                    apply_field_serde_default(field, ir, runtime);
                }
            }
        }
        _ => {}
    }
}

/// Applies the role's Serde default policy to one field.
fn apply_field_serde_default(field: &mut Field, ir: &mut FieldIr, runtime: &TokenStream) {
    if field.ident.is_none() {
        return;
    }
    let keep_serializing = ir.keep_serializing;
    let kind = omission_kind(&field.ty);
    let Some(kind) = kind else {
        return;
    };
    let serde = match ir.occurrences.iter_mut().find_map(|occurrence| match occurrence {
        FieldOccurrence::Serde(value) => Some(value),
        _ => None,
    }) {
        Some(value) => value,
        None => {
            ir.occurrences.push(FieldOccurrence::Serde(SerdeIr::default()));
            let Some(FieldOccurrence::Serde(value)) = ir.occurrences.last_mut() else {
                return;
            };
            value
        }
    };
    if !serde.default {
        field.attrs.push(parse_quote!(#[serde(default)]));
        serde.default = true;
        serde.default_from_model = true;
    }
    if keep_serializing {
        serde.omit_suppressed = true;
        return;
    }
    if serde.skip_serializing || serde.explicit_skip_serializing_if {
        return;
    }
    let suffix = match kind {
        OmissionKind::Option => "is_none",
        OmissionKind::Collection => "is_empty",
    };
    let path = format!(
        "{}::__private::serde_helpers::{suffix}",
        runtime.to_string().replace(' ', ""),
    );
    let path = LitStr::new(&path, Span::call_site());
    field.attrs.push(parse_quote!(#[serde(skip_serializing_if = #path)]));
    serde.omit_from_model = true;
}

/// Identifies container kinds that support omission-on-default behavior.
pub(crate) enum OmissionKind {
    Option,
    Collection,
}

/// Returns the omission policy supported by `ty`, if any.
pub(crate) fn omission_kind(ty: &Type) -> Option<OmissionKind> {
    let Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() {
        return None;
    }
    if is_option_path(&path.path) {
        return Some(OmissionKind::Option);
    }
    is_collection_path(&path.path).then_some(OmissionKind::Collection)
}

/// Collects derive names already present on a declaration.
fn existing_derive_names(attributes: &[Attribute]) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for attribute in attributes {
        if !attribute.path().is_ident("derive") {
            continue;
        }
        let paths = attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)?;
        result.extend(
            paths
                .iter()
                .filter_map(|path| path.segments.last().map(|segment| segment.ident.to_string())),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use syn::Type;
    use syn::parse_quote;

    use super::OmissionKind;
    use super::omission_kind;

    /// Ensures similarly named domain types do not acquire standard omission
    /// behavior.
    #[test]
    fn test_omission_kind_rejects_lookalike_paths() {
        let option: Type = parse_quote!(domain::Option<String>);
        let vector: Type = parse_quote!(domain::Vec<String>);
        let map: Type = parse_quote!(domain::HashMap<String, String>);
        let standard: Type = parse_quote!(std::collections::HashMap<String, String>);

        assert!(omission_kind(&option).is_none());
        assert!(omission_kind(&vector).is_none());
        assert!(omission_kind(&map).is_none());
        assert!(matches!(omission_kind(&standard), Some(OmissionKind::Collection)));
    }
}
