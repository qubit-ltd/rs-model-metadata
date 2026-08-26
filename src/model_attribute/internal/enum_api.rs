// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generated enum naming and display APIs.

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Fields;
use syn::Ident;
use syn::Index;
use syn::LitStr;
use syn::Result;
use syn::WhereClause;

use crate::attribute_support::serialized_variant_name;

/// Generates the non-redacted display implementation.
///
/// # Parameters
///
/// - `input`: The declaration that receives the generated implementation.
/// - `rename_all`: The configured Serde rename rule for the declaration.
///
/// # Returns
///
/// Returns tokens for the `Display` implementation.
///
/// # Errors
///
/// Returns an error when the declaration cannot provide a serialized enum name.
pub(crate) fn expand_display(input: &DeriveInput, rename_all: LitStr) -> Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    match &input.data {
        Data::Struct(data) => expand_struct_display(name, impl_generics, type_generics, where_clause, &data.fields),
        Data::Enum(data) => {
            let mut arms = Vec::with_capacity(data.variants.len());
            for variant in &data.variants {
                let variant_name = &variant.ident;
                let arm = match &variant.fields {
                    Fields::Unit => quote!(Self::#variant_name => formatter.write_str(self.name())),
                    Fields::Unnamed(fields) => {
                        let bindings = (0..fields.unnamed.len())
                            .map(|index| format_ident!("field_{index}"))
                            .collect::<Vec<_>>();
                        quote! {
                            Self::#variant_name(#(#bindings),*) => {
                                let mut debug = formatter.debug_tuple(self.name());
                                #(debug.field(#bindings);)*
                                debug.finish()
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let bindings = fields
                            .named
                            .iter()
                            .filter_map(|field| field.ident.as_ref())
                            .collect::<Vec<_>>();
                        quote! {
                            Self::#variant_name { #(#bindings),* } => {
                                let mut debug = formatter.debug_struct(self.name());
                                #(debug.field(stringify!(#bindings), #bindings);)*
                                debug.finish()
                            }
                        }
                    }
                };
                arms.push(arm);
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

/// Generates canonical enum name methods and unit-only reverse conversion.
///
/// # Parameters
///
/// - `input`: The enum declaration that receives the generated methods.
///
/// # Returns
///
/// Returns tokens for `name()` and, for unit-only enums, `from_name()`.
///
/// # Errors
///
/// Returns an error when two variants share the same serialized name.
pub(crate) fn expand_enum_names(input: &DeriveInput) -> Result<TokenStream> {
    let Data::Enum(data) = &input.data else {
        return Ok(TokenStream::new());
    };
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let mut name_arms = Vec::with_capacity(data.variants.len());
    let mut unit_variants = Vec::with_capacity(data.variants.len());
    let mut names = Vec::with_capacity(data.variants.len());
    for variant in &data.variants {
        let serialized = serialized_variant_name(variant)?;
        if names.iter().any(|name: &String| name == &serialized) {
            return Err(Error::new_spanned(
                &variant.ident,
                format!("duplicate serialized enum name {serialized:?}"),
            ));
        }
        let variant_name = &variant.ident;
        let pattern = match variant.fields {
            Fields::Unit => {
                unit_variants.push(variant_name);
                quote!(Self::#variant_name)
            }
            Fields::Unnamed(_) => quote!(Self::#variant_name(..)),
            Fields::Named(_) => quote!(Self::#variant_name { .. }),
        };
        name_arms.push((pattern, serialized.clone()));
        names.push(serialized);
    }
    let names = names
        .iter()
        .map(|name| LitStr::new(name, Span::call_site()))
        .collect::<Vec<_>>();
    let patterns = name_arms.iter().map(|(pattern, _)| pattern);
    let name_literals = name_arms
        .iter()
        .map(|(_, name)| LitStr::new(name, Span::call_site()))
        .collect::<Vec<_>>();
    let from_name = (unit_variants.len() == data.variants.len()).then(|| {
        quote! {
            /// Converts a canonical serialized name back into an enum variant.
            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#names => Some(Self::#unit_variants),)*
                    _ => None,
                }
            }
        }
    });
    Ok(quote! {
        impl #impl_generics #name #type_generics #where_clause {
            /// Returns the canonical serialized name of this enum variant.
            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    #(#patterns => #name_literals,)*
                }
            }

            #from_name
        }
    })
}

/// Generates a Debug-shaped `Display` implementation without requiring Debug.
///
/// # Parameters
///
/// - `name`: The declaration identifier.
/// - `impl_generics`: Generic parameters for the generated implementation.
/// - `type_generics`: Generic arguments for the declaration type.
/// - `where_clause`: Optional generic bounds for the declaration.
/// - `fields`: Fields that determine the display structure.
///
/// # Returns
///
/// Returns tokens for the generated `Display` implementation.
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
