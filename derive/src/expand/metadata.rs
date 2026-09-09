// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generates type metadata providers and generic declaration templates.

use heck::ToSnakeCase;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;
use syn::Error;
use syn::Ident;
use syn::LitStr;
use syn::parse_quote;

use super::fields::expand_field_vector;
use super::fields::expand_generic_field_vector;
use super::role::expand_role;
use crate::compiler::fingerprint::stable_fingerprint;
use crate::ir::MacroKind;
use crate::ir::declaration::DeclarationIr;
use crate::ir::declaration::FieldIr;
use crate::ir::declaration::VariantIr;

/// Generates lazy type metadata and registration implementations.
pub(crate) fn expand_metadata(declaration: &DeclarationIr, item: &DeriveInput, runtime: &TokenStream) -> TokenStream {
    let ident = &item.ident;
    let fields = expand_field_vector(&declaration.fields, quote!(descriptor.fields()), runtime);
    let role = expand_role(declaration, runtime);
    let declared_model_id = declaration.options.id.as_ref().map_or_else(
        || quote!(None),
        |id| quote!(Some(#runtime::metadata::ModelId::new(#id))),
    );
    let has_generics = !item.generics.params.is_empty();
    let mut impl_generics_source = item.generics.clone();
    for parameter in impl_generics_source.type_params_mut() {
        parameter.bounds.push(parse_quote!('static));
    }
    impl_generics_source
        .make_where_clause()
        .predicates
        .push(parse_quote!(Self: #runtime::__private::Reflect));
    let (impl_generics, ty_generics, where_clause) = impl_generics_source.split_for_impl();
    let generic_metadata = format_ident!("__qubit_model_generic_metadata_{}", ident.to_string().to_snake_case());
    let registration = match (declaration.options.id.as_ref(), has_generics) {
        (Some(_), false) => TokenStream::new(),
        (id, true) => expand_generic_registration(
            ident,
            id,
            declaration.kind,
            &generic_metadata,
            &declaration.fields,
            &declaration.variants,
            runtime,
        ),
        (None, false) => TokenStream::new(),
    };
    let model_id = if has_generics {
        quote!(None)
    } else {
        declared_model_id.clone()
    };
    let generic_definition = if has_generics {
        quote! { let metadata = metadata.generic_definition(#generic_metadata()); }
    } else {
        TokenStream::new()
    };
    let build_metadata = quote! {
        let descriptor = #runtime::__private::TypeDescriptor::of::<Self>();
        #fields
        let fields: &'static [#runtime::metadata::FieldMetadata] = #runtime::__private::v6::leak_slice(fields);
        #role
        let properties: ::std::vec::Vec<_> = fields
            .iter()
            .filter_map(|field| field.name().map(|name| {
                #runtime::__private::v6::property_metadata(
                    name,
                    field.type_ref(),
                    Some(field),
                    None,
                    None,
                )
            }))
            .collect();
        let property_fragments: ::std::vec::Vec<_> = fields
            .iter()
            .filter_map(|field| field.name().map(|name| {
                #runtime::__private::v6::property_fragment(
                    name,
                    field.type_ref(),
                    #runtime::metadata::PropertyFragmentSource::Field(field),
                )
            }))
            .collect();
        let properties: &'static [#runtime::metadata::PropertyMetadata] = #runtime::__private::v6::leak_slice(properties);
        let property_fragments: &'static [#runtime::metadata::PropertyFragment] =
            #runtime::__private::v6::leak_slice(property_fragments);
        let metadata = #runtime::__private::v6::GeneratedTypeMetadataBuilder::new(
            descriptor, #model_id, fields, role,
        ).properties(properties).property_fragments(property_fragments);
        #generic_definition
        metadata.finish_unchecked()
    };
    let metadata_body = if has_generics {
        quote! {
            static CACHE: ::std::sync::OnceLock<
                ::std::sync::Mutex<
                    ::std::collections::HashMap<::std::any::TypeId, &'static ::std::sync::OnceLock<#runtime::metadata::TypeMetadata>>,
                >,
            > = ::std::sync::OnceLock::new();
            let cache = CACHE.get_or_init(|| ::std::sync::Mutex::new(::std::collections::HashMap::new()));
            let type_id = ::std::any::TypeId::of::<Self>();
            let cell = {
                let mut guard = cache.lock().unwrap_or_else(|_| panic!("generic model metadata cache poisoned"));
                *guard.entry(type_id).or_insert_with(|| #runtime::__private::v6::leak(::std::sync::OnceLock::new()))
            };
            cell.get_or_init(|| { #build_metadata })
        }
    } else {
        quote! {
            static METADATA: ::std::sync::OnceLock<#runtime::metadata::TypeMetadata> = ::std::sync::OnceLock::new();
            METADATA.get_or_init(|| { #build_metadata })
        }
    };

    quote! {
        impl #impl_generics #runtime::__private::ModelTypeSeal for #ident #ty_generics #where_clause {}

        impl #impl_generics #runtime::__private::TypeMetadataProvider for #ident #ty_generics #where_clause {
            fn __type_metadata() -> &'static #runtime::metadata::TypeMetadata {
                #metadata_body
            }
        }

        #registration
    }
}

/// Generates registration metadata for a generic model definition.
#[allow(clippy::too_many_arguments)]
fn expand_generic_registration(
    ident: &Ident,
    id: Option<&LitStr>,
    kind: MacroKind,
    metadata_fn: &Ident,
    fields: &[FieldIr],
    variants: &[VariantIr],
    runtime: &TokenStream,
) -> TokenStream {
    let model_id = id.map_or_else(
        || quote!(None),
        |id| quote!(Some(#runtime::metadata::ModelId::new(#id))),
    );
    let snake_name = ident.to_string().to_snake_case();
    let definition_fn = format_ident!("__qubit_model_reflect_definition_{}", ident);
    let registration_module = format_ident!("__qubit_model_generic_capability_{}", snake_name);
    let role = match kind {
        MacroKind::Model => quote!(#runtime::metadata::ModelRole::Model),
        MacroKind::Enum => quote!(#runtime::metadata::ModelRole::Enum),
        MacroKind::Value => quote!(#runtime::metadata::ModelRole::Value),
        MacroKind::Entity | MacroKind::Projection | MacroKind::ModelImpl => {
            return Error::new(
                Span::call_site(),
                "only Model, Enum, and Value support generic registration",
            )
            .into_compile_error();
        }
    };
    let fingerprint = stable_fingerprint(&ident.to_string());
    let template_fields = expand_generic_field_vector(
        fields,
        quote!(definition.fields().expect("generic struct fields")),
        runtime,
        false,
    );
    let template_variants = expand_generic_variant_vector(variants, runtime);
    quote! {
        #[doc(hidden)]
        fn #metadata_fn() -> &'static #runtime::generic::GenericModelMetadata {
            static METADATA: ::std::sync::OnceLock<#runtime::generic::GenericModelMetadata> =
                ::std::sync::OnceLock::new();
            METADATA.get_or_init(|| {
                let definition = #definition_fn();
                #template_fields
                let fields: &'static [#runtime::metadata::FieldMetadata] = #runtime::__private::v6::leak_slice(fields);
                #template_variants
                #runtime::__private::v6::generic_model_metadata(
                    #model_id,
                    #role,
                    definition,
                    fields,
                    variants,
                )
            })
        }

        #[doc(hidden)]
        mod #registration_module {
            #runtime::__private::v6::register_generic_model_capability! {
                definition = super::#definition_fn,
                metadata = super::#metadata_fn,
                source = (env!("CARGO_PKG_NAME"), module_path!(), line!(), column!(), #fingerprint),
            }
        }
    }
}

/// Generates complete symbolic overlays for generic enum variants.
fn expand_generic_variant_vector(variants: &[VariantIr], runtime: &TokenStream) -> TokenStream {
    let bodies = variants.iter().enumerate().map(|(variant_index, variant)| {
        let fields = expand_generic_field_vector(
            &variant.fields,
            quote!(definition.variants().expect("generic enum variants")[#variant_index].fields()),
            runtime,
            true,
        );
        let canonical = &variant.canonical_name;
        let serialized = &variant.serialized_name;
        let deserialized = &variant.deserialized_name;
        let default = variant.default;
        quote! {
            {
                #fields
                let fields: &'static [#runtime::metadata::FieldMetadata] =
                    #runtime::__private::v6::leak_slice(fields);
                variants.push(#runtime::__private::v6::generic_enum_variant_metadata(
                    &definition.variants().expect("generic enum variants")[#variant_index],
                    #canonical,
                    #serialized,
                    #deserialized,
                    fields,
                    #default,
                ));
            }
        }
    });
    quote! {
        let mut variants = ::std::vec::Vec::new();
        #(#bodies)*
        let variants: &'static [#runtime::metadata::EnumVariantMetadata] =
            #runtime::__private::v6::leak_slice(variants);
    }
}
