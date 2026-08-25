// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Expansion of the `Model` attribute macro.

use proc_macro_crate::FoundCrate;
use proc_macro_crate::crate_name;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::quote;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Fields;
use syn::Ident;
use syn::Index;
use syn::LitStr;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::WhereClause;
use syn::parse::Parser;
use syn::parse_quote;
use syn::parse2;
use syn::punctuated::Punctuated;

use crate::attribute_support::has_must_use;
use crate::attribute_support::serialized_variant_name;
use crate::derive_model_impl::derive_model_tokens;
use crate::model_options::DisabledCapabilities;
use crate::model_options::ModelOptions;
use crate::runtime_path::runtime_path;

/// Expands one model declaration and converts failures to compiler diagnostics.
pub(crate) fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    match expand_result(args, input, false) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error(),
    }
}

/// Expands the public `Enum` attribute macro.
pub(crate) fn expand_enum(args: TokenStream, input: TokenStream) -> TokenStream {
    match expand_result(args, input, true) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error(),
    }
}

/// Expands one model declaration with default traits and metadata.
fn expand_result(
    args: TokenStream,
    input: TokenStream,
    enum_declaration: bool,
) -> Result<TokenStream> {
    let attributes = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(args)?;
    let options = ModelOptions::parse(attributes)?;
    let mut item: DeriveInput = parse2(input)?;
    match (&item.data, enum_declaration) {
        (Data::Struct(_), false) => {}
        (Data::Enum(data), true)
            if data
                .variants
                .iter()
                .all(|variant| matches!(variant.fields, Fields::Unit)) => {}
        (Data::Enum(_), false) => {
            return Err(Error::new_spanned(
                &item.ident,
                "#[Model] only supports structs; use #[Enum] for fieldless enums",
            ));
        }
        (Data::Enum(_), true) | (Data::Struct(_), true) => {
            return Err(Error::new_spanned(
                &item.ident,
                "#[Enum] only supports fieldless enums",
            ));
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
    rename_field_attributes(&mut metadata_input.data);
    add_default_serde_field_attributes(
        &mut item.data,
        !options.disabled.serialize,
        !options.disabled.deserialize,
    )?;
    remove_field_attributes(&mut item.data);

    let derives = default_derives(&item.data, &serde, &options.disabled, redacted)?;
    item.attrs.push(derives);
    if enum_declaration && !has_must_use(&item.attrs) {
        item.attrs.push(parse_quote!(#[must_use]));
    }
    if redacted {
        let redact = dependency_path(
            "qubit-redact",
            "Model redaction requires the `qubit-redact` dependency",
        )?;
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
        item.attrs
            .push(parse_quote!(#[serde(rename_all = #rename_all)]));
    }
    let metadata = derive_model_tokens(metadata_input.into_token_stream(), runtime_path());
    let enum_names = enum_declaration
        .then(|| expand_enum_names(&item))
        .transpose()?
        .unwrap_or_default();
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

/// Builds the standard derives selected by the declaration shape.
fn default_derives(
    data: &Data,
    serde: &TokenStream,
    disabled: &DisabledCapabilities,
    redacted: bool,
) -> Result<Attribute> {
    match data {
        Data::Enum(_) => enum_derives(serde, disabled, redacted),
        Data::Struct(_) => struct_derives(serde, disabled, redacted),
        Data::Union(union) => Err(Error::new_spanned(
            union.union_token,
            "Model attribute does not support unions",
        )),
    }
}

/// Builds the defaults for a fieldless enum.
fn enum_derives(
    serde: &TokenStream,
    disabled: &DisabledCapabilities,
    redacted: bool,
) -> Result<Attribute> {
    let mut derives = Vec::new();
    if !disabled.clone {
        derives.push(quote!(Clone));
    }
    if !disabled.copy && !disabled.clone {
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

/// Builds the defaults for a struct.
fn struct_derives(
    serde: &TokenStream,
    disabled: &DisabledCapabilities,
    redacted: bool,
) -> Result<Attribute> {
    if disabled.copy {
        return Err(Error::new(
            Span::call_site(),
            "`no_copy` is only supported on enums",
        ));
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

/// Rewrites field helper attributes for the metadata-only derive input.
fn rename_field_attributes(data: &mut Data) {
    visit_fields(data, |field| {
        for attribute in &mut field.attrs {
            if attribute.path().is_ident("field")
                && let Meta::List(list) = &mut attribute.meta
            {
                list.path = parse_quote!(model);
            }
        }
    });
}

/// Removes field helper attributes from the item returned to the compiler.
fn remove_field_attributes(data: &mut Data) {
    visit_fields(data, |field| {
        field
            .attrs
            .retain(|attribute| !attribute.path().is_ident("field"));
    });
}

/// Adds Serde defaults supported by the model's enabled serialization capabilities.
fn add_default_serde_field_attributes(
    data: &mut Data,
    serialize: bool,
    deserialize: bool,
) -> Result<()> {
    let mut error: Option<Error> = None;
    visit_fields(data, |field| {
        let result = field_keeps_serializing(&field.attrs).and_then(|keep_serializing| {
            if keep_serializing {
                Ok(())
            } else if serialize && is_direct_type(&field.ty, "Option") {
                add_serde_attribute_if_absent(
                    &mut field.attrs,
                    &["skip_serializing_if", "skip_serializing", "skip"],
                    parse_quote!(#[serde(skip_serializing_if = "::core::option::Option::is_none")]),
                )
            } else if let Some(is_empty) = collection_is_empty_function(&field.ty) {
                let serialization = if serialize {
                    add_serde_attribute_if_absent(
                        &mut field.attrs,
                        &["skip_serializing_if", "skip_serializing", "skip"],
                        serde_skip_serializing_if_attribute(is_empty),
                    )
                } else {
                    Ok(())
                };
                let deserialization = if deserialize {
                    add_serde_attribute_if_absent(
                        &mut field.attrs,
                        &["default", "skip_deserializing", "skip"],
                        parse_quote!(#[serde(default)]),
                    )
                } else {
                    Ok(())
                };
                serialization.and(deserialization)
            } else {
                Ok(())
            }
        });
        if let Err(current) = result {
            match &mut error {
                Some(error) => error.combine(current),
                None => error = Some(current),
            }
        }
    });
    error.map_or(Ok(()), Err)
}

/// Returns whether a field opts out of the model's automatic serialization omission.
fn field_keeps_serializing(attributes: &[Attribute]) -> Result<bool> {
    has_attribute_option(attributes, "field", &["keep_serializing"])
}

/// Adds an attribute unless a field's existing Serde attribute has any option.
fn add_serde_attribute_if_absent(
    attributes: &mut Vec<Attribute>,
    options: &[&str],
    default_attribute: Attribute,
) -> Result<()> {
    if !has_attribute_option(attributes, "serde", options)? {
        attributes.push(default_attribute);
    }
    Ok(())
}

/// Returns whether an attribute list contains any one of the supplied options.
fn has_attribute_option(
    attributes: &[Attribute],
    attribute_name: &str,
    options: &[&str],
) -> Result<bool> {
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident(attribute_name))
    {
        let Meta::List(list) = &attribute.meta else {
            continue;
        };
        let serde_options =
            list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        if serde_options.iter().any(|serde_option| {
            options
                .iter()
                .any(|option| serde_option.path().is_ident(option))
        }) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Builds a Serde field attribute calling an inherent `is_empty` method.
fn serde_skip_serializing_if_attribute(is_empty: &str) -> Attribute {
    let is_empty = LitStr::new(is_empty, Span::call_site());
    parse_quote!(#[serde(skip_serializing_if = #is_empty)])
}

/// Returns the `is_empty` function for a directly declared standard collection.
fn collection_is_empty_function(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Path(type_path) => {
            let type_name = type_path.path.segments.last()?.ident.to_string();
            match type_name.as_str() {
                "Vec" => Some("::std::vec::Vec::is_empty"),
                "LinkedList" => Some("::std::collections::LinkedList::is_empty"),
                "VecDeque" => Some("::std::collections::VecDeque::is_empty"),
                "HashMap" => Some("::std::collections::HashMap::is_empty"),
                "BTreeMap" => Some("::std::collections::BTreeMap::is_empty"),
                "HashSet" => Some("::std::collections::HashSet::is_empty"),
                "BTreeSet" => Some("::std::collections::BTreeSet::is_empty"),
                "BinaryHeap" => Some("::std::collections::BinaryHeap::is_empty"),
                _ => None,
            }
        }
        Type::Array(_) => Some("<[_]>::is_empty"),
        Type::Group(group) => collection_is_empty_function(&group.elem),
        Type::Paren(paren) => collection_is_empty_function(&paren.elem),
        _ => None,
    }
}

/// Returns whether a type is a direct declaration of a named standard type.
fn is_direct_type(ty: &Type, type_name: &str) -> bool {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == type_name),
        Type::Group(group) => is_direct_type(&group.elem, type_name),
        Type::Paren(paren) => is_direct_type(&paren.elem, type_name),
        _ => false,
    }
}

/// Returns whether any struct field declares a redaction rule.
fn has_redact_fields(data: &Data) -> bool {
    let Data::Struct(data) = data else {
        return false;
    };
    data.fields.iter().any(|field| {
        field
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("redact"))
    })
}

/// Visits all struct fields in a declaration.
fn visit_fields(data: &mut Data, mut visit: impl FnMut(&mut syn::Field)) {
    if let Data::Struct(data) = data {
        for field in &mut data.fields {
            visit(field);
        }
    }
}

/// Generates the non-redacted display implementation.
fn expand_display(input: &DeriveInput, rename_all: LitStr) -> Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    match &input.data {
        Data::Struct(data) => expand_struct_display(
            name,
            impl_generics,
            type_generics,
            where_clause,
            &data.fields,
        ),
        Data::Enum(data) => {
            let mut arms = Vec::with_capacity(data.variants.len());
            for variant in &data.variants {
                if !matches!(variant.fields, Fields::Unit) {
                    return Ok(TokenStream::new());
                }
                let variant_name = &variant.ident;
                arms.push(quote!(Self::#variant_name => formatter.write_str(self.name())));
            }
            let _ = rename_all;
            Ok(quote! {
                impl #impl_generics ::core::fmt::Display for #name #type_generics #where_clause {
                    fn fmt(
                        &self,
                        formatter: &mut ::core::fmt::Formatter<'_>,
                    ) -> ::core::fmt::Result {
                        match self {
                            #(#arms,)*
                        }
                    }
                }
            })
        }
        Data::Union(_) => Ok(TokenStream::new()),
    }
}

/// Generates canonical name conversion methods for a fieldless enum.
fn expand_enum_names(input: &DeriveInput) -> Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Ok(TokenStream::new());
    };
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let mut variants = Vec::with_capacity(data.variants.len());
    let mut names = Vec::with_capacity(data.variants.len());
    for variant in &data.variants {
        let serialized = serialized_variant_name(variant)?;
        if names.iter().any(|name: &String| name == &serialized) {
            return Err(Error::new_spanned(
                &variant.ident,
                format!("duplicate serialized enum name {serialized:?}"),
            ));
        }
        variants.push(&variant.ident);
        names.push(serialized);
    }
    let names = names
        .iter()
        .map(|name| LitStr::new(name, Span::call_site()))
        .collect::<Vec<_>>();
    Ok(quote! {
        impl #impl_generics #name #type_generics #where_clause {
            /// Returns the canonical serialized name of this enum variant.
            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    #(Self::#variants => #names,)*
                }
            }

            /// Converts a canonical serialized name back into an enum variant.
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#names => Some(Self::#variants),)*
                    _ => None,
                }
            }
        }
    })
}

/// Generates a Debug-shaped `Display` implementation without requiring Debug.
fn expand_struct_display(
    name: &Ident,
    impl_generics: impl ToTokens,
    type_generics: impl ToTokens,
    where_clause: Option<&WhereClause>,
    fields: &Fields,
) -> Result<TokenStream> {
    let body = match fields {
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
    };
    Ok(quote! {
        impl #impl_generics ::core::fmt::Display for #name #type_generics #where_clause {
            fn fmt(
                &self,
                formatter: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                #body
            }
        }
    })
}
