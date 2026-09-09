// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Structural trait implementations with bounds on actual field shapes.

use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::GenericArgument;
use syn::Ident;
use syn::Path;
use syn::PathArguments;
use syn::Token;
use syn::Type;
use syn::parse_quote;
use syn::punctuated::Punctuated;

/// Generates Clone, equality and hashing without unnecessary generic bounds.
pub(super) fn expand(item: &DeriveInput, name: &str) -> TokenStream {
    let trait_path = match name {
        "Clone" => quote!(::core::clone::Clone),
        "PartialEq" => quote!(::core::cmp::PartialEq),
        "Eq" => quote!(::core::cmp::Eq),
        "Hash" => quote!(::core::hash::Hash),
        "Copy" => quote!(::core::marker::Copy),
        "Default" => quote!(::core::default::Default),
        "PartialOrd" => quote!(::core::cmp::PartialOrd),
        "Ord" => quote!(::core::cmp::Ord),
        "Debug" => quote!(::core::fmt::Debug),
        _ => unreachable!("supported structural trait"),
    };
    let shapes: Vec<_> = match &item.data {
        Data::Struct(data) => vec![(quote!(Self), &data.fields)],
        Data::Enum(data) => data
            .variants
            .iter()
            .map(|variant| {
                let ident = &variant.ident;
                (quote!(Self::#ident), &variant.fields)
            })
            .collect(),
        Data::Union(_) => unreachable!("validated model shape"),
    };
    let mut generics = item.generics.clone();
    let ident = &item.ident;
    let (_, arguments, _) = item.generics.split_for_impl();
    let recursive_type = quote!(#ident #arguments).to_string();
    for (_, fields) in &shapes {
        for field in *fields {
            for ty in recursive_bounds(&field.ty, &recursive_type).unwrap_or_else(|| vec![&field.ty]) {
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: #trait_path));
            }
        }
    }
    if name == "Copy" {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(Self: ::core::clone::Clone));
    }
    let arms: Vec<_> = shapes
        .iter()
        .map(|(constructor, fields)| {
            let left: Vec<_> = (0..fields.len())
                .map(|index| format_ident!("__model_left_{index}"))
                .collect();
            let right: Vec<_> = (0..fields.len())
                .map(|index| format_ident!("__model_right_{index}"))
                .collect();
            let pattern = pattern_tokens(constructor, fields, &left);
            match name {
                "Clone" => {
                    let values: Vec<_> = left
                        .iter()
                        .map(|value| quote!(::core::clone::Clone::clone(#value)))
                        .collect();
                    let body = construct(constructor, fields, &values);
                    quote!(#pattern => #body)
                }
                "PartialEq" => {
                    let other = pattern_tokens(constructor, fields, &right);
                    quote!((#pattern, #other) => true #(&& ::core::cmp::PartialEq::eq(#left, #right))*)
                }
                "PartialOrd" | "Ord" => {
                    let other = pattern_tokens(constructor, fields, &right);
                    if name == "PartialOrd" {
                        quote!((#pattern, #other) => {
                            #(match ::core::cmp::PartialOrd::partial_cmp(#left, #right) {
                                Some(::core::cmp::Ordering::Equal) => {}, ordering => return ordering,
                            })*
                            Some(::core::cmp::Ordering::Equal)
                        })
                    } else {
                        quote!((#pattern, #other) => {
                            #(match ::core::cmp::Ord::cmp(#left, #right) {
                                ::core::cmp::Ordering::Equal => {}, ordering => return ordering,
                            })*
                            ::core::cmp::Ordering::Equal
                        })
                    }
                }
                "Debug" => {
                    let label = if matches!(item.data, Data::Struct(_)) {
                        item.ident.to_string()
                    } else {
                        constructor.to_string().split(" :: ").last().unwrap().to_owned()
                    };
                    let calls: Vec<_> = fields
                        .iter()
                        .zip(&left)
                        .map(|(field, binding)| {
                            if let Some(ident) = &field.ident {
                                let label = ident.to_string();
                                quote!(builder.field(#label, #binding);)
                            } else {
                                quote!(builder.field(#binding);)
                            }
                        })
                        .collect();
                    let builder = if matches!(fields, Fields::Named(_)) {
                        quote!(debug_struct)
                    } else {
                        quote!(debug_tuple)
                    };
                    quote!(#pattern => { let mut builder = formatter.#builder(#label); #(#calls)* builder.finish() })
                }
                "Hash" => quote!(#pattern => { #(::core::hash::Hash::hash(#left, state);)* }),
                _ => TokenStream::new(),
            }
        })
        .collect();
    let discriminant = matches!(item.data, Data::Enum(_))
        .then(|| quote!(::core::hash::Hash::hash(&::core::mem::discriminant(self), state);));
    let ordering_prefix = if matches!(name, "PartialOrd" | "Ord") {
        enum_ordering_prefix(item, name)
    } else {
        TokenStream::new()
    };
    let ordering_fallback = matches!(&item.data, Data::Enum(data) if data.variants.len() > 1)
        .then(|| quote!(_ => unreachable!("distinct variants have distinct discriminants"),));
    let body = match name {
        "Clone" => {
            quote!(fn clone(&self) -> Self { match self { #(#arms),* } })
        }
        "PartialEq" => {
            let fallback =
                matches!(&item.data, Data::Enum(data) if data.variants.len() > 1).then(|| quote!(_ => false,));
            quote!(fn eq(&self, other: &Self) -> bool { match (self, other) { #(#arms,)* #fallback } })
        }
        "Hash" => {
            quote!(fn hash<__ModelHasher: ::core::hash::Hasher>(&self, state: &mut __ModelHasher) {
                #discriminant
                match self { #(#arms),* }
            })
        }
        "Default" => {
            let (constructor, fields) = &shapes[0];
            let values: Vec<_> = fields
                .iter()
                .map(|_| quote!(::core::default::Default::default()))
                .collect();
            let value = construct(constructor, fields, &values);
            quote!(fn default() -> Self { #value })
        }
        "PartialOrd" => {
            quote!(fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> { #ordering_prefix match (self, other) { #(#arms,)* #ordering_fallback } })
        }
        "Ord" => {
            quote!(fn cmp(&self, other: &Self) -> ::core::cmp::Ordering { #ordering_prefix match (self, other) { #(#arms,)* #ordering_fallback } })
        }
        "Debug" => {
            quote!(fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result { match self { #(#arms),* } })
        }
        "Eq" | "Copy" => TokenStream::new(),
        _ => unreachable!("supported structural trait"),
    };
    let ident = &item.ident;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    quote!(impl #impl_generics #trait_path for #ident #type_generics #where_clause { #body })
}

/// Converts binding names to the same shape used by structural constructors.
fn pattern_tokens(constructor: &TokenStream, fields: &Fields, bindings: &[Ident]) -> TokenStream {
    let values: Vec<_> = bindings.iter().map(|binding| quote!(#binding)).collect();
    construct(constructor, fields, &values)
}

/// Constructs a named, tuple or unit model variant from field expressions.
fn construct(constructor: &TokenStream, fields: &Fields, values: &[TokenStream]) -> TokenStream {
    match fields {
        Fields::Named(fields) => {
            let names = fields.named.iter().map(|field| &field.ident);
            quote!(#constructor { #(#names: #values),* })
        }
        Fields::Unnamed(_) => quote!(#constructor(#(#values),*)),
        Fields::Unit => quote!(#constructor),
    }
}

/// Compares Enum discriminants using their declared integer representation.
fn enum_ordering_prefix(item: &DeriveInput, trait_name: &str) -> TokenStream {
    let Data::Enum(data) = &item.data else {
        return TokenStream::new();
    };
    let mut representation: Type = parse_quote!(isize);
    for attribute in &item.attrs {
        if attribute.path().is_ident("repr")
            && let Ok(paths) = attribute.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)
        {
            for path in paths {
                if path.get_ident().is_some_and(|ident| {
                    matches!(
                        ident.to_string().as_str(),
                        "i8" | "i16"
                            | "i32"
                            | "i64"
                            | "i128"
                            | "isize"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "u128"
                            | "usize"
                    )
                }) {
                    representation = parse_quote!(#path);
                }
            }
        }
    }
    let mut next = quote!(0);
    let arms: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let name = &variant.ident;
            let pattern = match variant.fields {
                Fields::Named(_) => quote!(Self::#name { .. }),
                Fields::Unnamed(_) => quote!(Self::#name(..)),
                Fields::Unit => quote!(Self::#name),
            };
            let value = variant
                .discriminant
                .as_ref()
                .map(|(_, value)| quote!(#value))
                .unwrap_or_else(|| next.clone());
            next = quote!((#value) + 1);
            quote!(#pattern => #value)
        })
        .collect();
    let result = if trait_name == "PartialOrd" {
        quote!(Some(ordering))
    } else {
        quote!(ordering)
    };
    quote! {
        let discriminant = |value: &Self| -> #representation { match value { #(#arms),* } };
        let ordering = ::core::cmp::Ord::cmp(&discriminant(self), &discriminant(other));
        if ordering != ::core::cmp::Ordering::Equal { return #result; }
    }
}

/// Removes self-recursive obligations through standard storage wrappers while
/// retaining bounds on the other stored types.
pub(super) fn recursive_bounds<'a>(ty: &'a Type, recursive_type: &str) -> Option<Vec<&'a Type>> {
    let rendered = ty.to_token_stream().to_string();
    if rendered == recursive_type || rendered == "Self" {
        return Some(Vec::new());
    }
    let children: Vec<&Type> = match ty {
        Type::Path(path) if path.qself.is_none() => {
            let segment = path.path.segments.last()?;
            let names: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            let names: Vec<_> = names.iter().map(String::as_str).collect();
            let standard = crate::compiler::type_path::is_option_path(&path.path)
                || crate::compiler::type_path::is_collection_path(&path.path)
                || matches!(
                    names.as_slice(),
                    ["Box" | "Rc" | "Arc"]
                        | ["std" | "alloc", "boxed", "Box"]
                        | ["std" | "alloc", "rc", "Rc"]
                        | ["std" | "alloc", "sync", "Arc"]
                );
            if !standard {
                return None;
            }
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return None;
            };
            arguments
                .args
                .iter()
                .filter_map(|argument| match argument {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                })
                .collect()
        }
        Type::Tuple(tuple) => tuple.elems.iter().collect(),
        Type::Array(array) => vec![&array.elem],
        Type::Paren(paren) => vec![&paren.elem],
        _ => return None,
    };
    let mut found = false;
    let bounds = children
        .into_iter()
        .flat_map(|child| {
            if let Some(bounds) = recursive_bounds(child, recursive_type) {
                found = true;
                bounds
            } else {
                vec![child]
            }
        })
        .collect();
    found.then_some(bounds)
}
