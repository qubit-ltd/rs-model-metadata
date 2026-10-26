// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generates type metadata providers and generic declaration templates.

use std::collections::HashSet;

use heck::ToSnakeCase;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use syn::DeriveInput;
use syn::Error;
use syn::GenericArgument;
use syn::GenericParam;
use syn::Ident;
use syn::LitStr;
use syn::PathArguments;
use syn::ReturnType;
use syn::Type;
use syn::TypePath;
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
///
/// # Parameters
///
/// - `declaration` provides the normalized model description.
/// - `item` provides the original Rust type and its generics.
/// - `runtime` identifies the runtime crate path used in the generated code.
///
/// # Returns
///
/// The implementations and registration items to append to the input type.
#[must_use]
pub(crate) fn expand_metadata(declaration: &DeclarationIr, item: &DeriveInput, runtime: &TokenStream) -> TokenStream {
    let ident = &item.ident;
    let fields = expand_field_vector(&declaration.fields, runtime, ident);
    let nested_metadata = expand_nested_metadata_providers(declaration, runtime);
    let role = expand_role(declaration, item, runtime);
    let declared_model_id = declaration.options.id.as_ref().map_or_else(
        || quote!(None),
        |id| quote!(Some(#runtime::metadata::ModelId::new(#id))),
    );
    let has_generics = !item.generics.params.is_empty();
    let generic_type_names: Vec<Ident> = item
        .generics
        .params
        .iter()
        .filter_map(|parameter| match parameter {
            GenericParam::Type(parameter) => Some(parameter.ident.clone()),
            GenericParam::Const(parameter) => Some(parameter.ident.clone()),
            GenericParam::Lifetime(_) => None,
        })
        .collect();
    let mut impl_generics_source = item.generics.clone();
    for parameter in impl_generics_source.type_params_mut() {
        parameter.bounds.push(parse_quote!('static));
        parameter.bounds.push(parse_quote!(#runtime::__private::Reflect));
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
            &generic_type_names,
            ident,
            runtime,
        ),
        (None, false) => TokenStream::new(),
    };
    let metadata_capability = if has_generics {
        TokenStream::new()
    } else {
        quote! {
            #runtime::__private::v8::submit_model_metadata_capability!(#ident);
            #runtime::__private::v8::submit_model_impl_capability!(#ident);
        }
    };
    let model_id = if has_generics {
        quote!(None)
    } else {
        declared_model_id.clone()
    };
    let has_model_id = declaration.options.id.as_ref().filter(|_| !has_generics).map(|id| {
        quote! {
            impl #runtime::metadata::HasModelId for #ident {
                const MODEL_ID: #runtime::metadata::ModelId = #runtime::metadata::ModelId::new(#id);
            }
        }
    });
    let generic_definition = if has_generics {
        quote! { let metadata = metadata.generic_definition(#generic_metadata()); }
    } else {
        TokenStream::new()
    };
    let build_metadata = quote! {
        let descriptor = #runtime::__private::TypeDescriptor::of::<Self>();
        #fields
        let fields: &'static [#runtime::metadata::FieldMetadata] = #runtime::__private::v8::leak_slice(fields);
        #role
        let properties: ::std::vec::Vec<_> = fields
            .iter()
            .filter_map(|field| field.name().map(|name| {
                #runtime::__private::v8::property_metadata(
                    name,
                    field.type_expression_data(),
                    Some(field),
                    None,
                    None,
                )
            }))
            .collect();
        let property_fragments: ::std::vec::Vec<_> = fields
            .iter()
            .filter_map(|field| field.name().map(|name| {
                #runtime::__private::v8::property_fragment(
                    name,
                    field.type_expression_data(),
                    #runtime::metadata::PropertyFragmentSource::Field(field),
                )
            }))
            .collect();
        let properties: &'static [#runtime::metadata::PropertyMetadata] = #runtime::__private::v8::leak_slice(properties);
        let property_fragments: &'static [#runtime::metadata::PropertyFragment] =
            #runtime::__private::v8::leak_slice(property_fragments);
        #nested_metadata
        let metadata = #runtime::__private::v8::GeneratedTypeMetadataBuilder::new::<Self>(
            #model_id, fields, role,
        ).properties(properties)
            .property_fragments(property_fragments)
            .nested_metadata_providers(nested_metadata_providers);
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
                *guard.entry(type_id).or_insert_with(|| #runtime::__private::v8::leak(::std::sync::OnceLock::new()))
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

        #has_model_id

        #registration
        #metadata_capability
    }
}

/// Emits lazy providers for model metadata on concrete nested field types.
#[must_use]
fn expand_nested_metadata_providers(declaration: &DeclarationIr, runtime: &TokenStream) -> TokenStream {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();
    for field in declaration
        .fields
        .iter()
        .chain(declaration.variants.iter().flat_map(|variant| variant.fields.iter()))
    {
        collect_nested_type_candidates(&field.ty, &mut candidates, &mut seen);
    }
    let providers = candidates.iter().enumerate().map(|(index, ty)| {
        let provider = format_ident!("__qubit_nested_type_metadata_{index}");
        quote! {
            let #provider: fn() -> Option<&'static #runtime::metadata::TypeMetadata> = || {
                use #runtime::__private::v8::NestedTypeMetadataProbe as _;
                let marker = ::std::marker::PhantomData::<#ty>;
                (&&marker).nested_type_metadata()
            };
            nested_metadata_providers.push(#provider);
        }
    });
    quote! {
        let mut nested_metadata_providers = ::std::vec::Vec::new();
        #(#providers)*
        let nested_metadata_providers: &'static [fn() -> Option<&'static #runtime::metadata::TypeMetadata>] =
            #runtime::__private::v8::leak_slice(nested_metadata_providers);
    }
}

/// Retains every concrete type expression node once, including nested generic
/// arguments and container elements.
fn collect_nested_type_candidates(ty: &Type, output: &mut Vec<Type>, seen: &mut HashSet<String>) {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if path.segments.last().is_some_and(|segment| segment.ident == "str") {
                return;
            }
            if path
                .segments
                .iter()
                .any(|segment| matches!(&segment.arguments, PathArguments::AngleBracketed(_)))
            {
                push_type_candidate(ty, output, seen);
            }
            for segment in &path.segments {
                match &segment.arguments {
                    PathArguments::AngleBracketed(arguments) => {
                        for argument in &arguments.args {
                            if let GenericArgument::Type(nested) = argument {
                                collect_nested_type_candidates(nested, output, seen);
                            }
                        }
                    }
                    PathArguments::Parenthesized(arguments) => {
                        for nested in &arguments.inputs {
                            collect_nested_type_candidates(nested, output, seen);
                        }
                        if let ReturnType::Type(_, nested) = &arguments.output {
                            collect_nested_type_candidates(nested, output, seen);
                        }
                    }
                    PathArguments::None => {}
                }
            }
        }
        Type::Array(value) => collect_nested_type_candidates(&value.elem, output, seen),
        Type::Group(value) => collect_nested_type_candidates(&value.elem, output, seen),
        Type::Paren(value) => collect_nested_type_candidates(&value.elem, output, seen),
        Type::Ptr(value) => collect_nested_type_candidates(&value.elem, output, seen),
        Type::Reference(value) => collect_nested_type_candidates(&value.elem, output, seen),
        Type::Slice(value) => collect_nested_type_candidates(&value.elem, output, seen),
        Type::Tuple(value) => {
            for nested in &value.elems {
                collect_nested_type_candidates(nested, output, seen);
            }
        }
        _ => {}
    }
}

/// Adds a type expression only when its tokenized form has not been seen.
fn push_type_candidate(ty: &Type, output: &mut Vec<Type>, seen: &mut HashSet<String>) {
    let key = ty.to_token_stream().to_string();
    if seen.insert(key) {
        output.push(ty.clone());
    }
}

/// Generates registration metadata for a generic model definition.
#[allow(clippy::too_many_arguments)]
#[must_use]
fn expand_generic_registration(
    ident: &Ident,
    id: Option<&LitStr>,
    kind: MacroKind,
    metadata_fn: &Ident,
    fields: &[FieldIr],
    variants: &[VariantIr],
    generic_type_names: &[Ident],
    owner: &Ident,
    runtime: &TokenStream,
) -> TokenStream {
    let model_id = id.map_or_else(
        || quote!(None),
        |id| quote!(Some(#runtime::metadata::ModelId::new(#id))),
    );
    let snake_name = ident.to_string().to_snake_case();
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
    let capability_fn = format_ident!("__qubit_reflect_v4_definition_capability_{}", ident);
    let definition_path = quote!(concat!(module_path!(), "::", stringify!(#ident)));
    let model_impl_capability = quote! {
        #runtime::__private::v8::register_generic_model_impl_capability! {
            definition_path = concat!(module_path!(), "::", stringify!(#ident)),
            capability = #capability_fn,
            source = (env!("CARGO_PKG_NAME"), module_path!(), line!(), column!(), #fingerprint),
        }
    };
    let template_fields = expand_generic_field_vector(
        fields,
        definition_path.clone(),
        runtime,
        false,
        generic_type_names,
        ident,
    );
    let template_variants =
        expand_generic_variant_vector(variants, &definition_path, runtime, generic_type_names, owner);
    quote! {
        #[doc(hidden)]
        fn #metadata_fn() -> &'static #runtime::generic::GenericModelMetadata {
            static METADATA: ::std::sync::OnceLock<#runtime::generic::GenericModelMetadata> =
                ::std::sync::OnceLock::new();
            METADATA.get_or_init(|| {
                #template_fields
                let fields: &'static [#runtime::metadata::FieldMetadata] = #runtime::__private::v8::leak_slice(fields);
                #template_variants
                let definition_path: &'static str = #definition_path;
                #runtime::__private::v8::generic_model_metadata(
                    #model_id,
                    #role,
                    definition_path,
                    fields,
                    variants,
                )
            })
        }

        #[doc(hidden)]
        mod #registration_module {
            #runtime::__private::v8::register_generic_model_capability! {
                definition_path = concat!(module_path!(), "::", stringify!(#ident)),
                capability = #capability_fn,
                metadata = super::#metadata_fn,
                source = (env!("CARGO_PKG_NAME"), module_path!(), line!(), column!(), #fingerprint),
            }
            #model_impl_capability
        }
    }
}

/// Generates complete symbolic overlays for generic enum variants.
#[must_use]
fn expand_generic_variant_vector(
    variants: &[VariantIr],
    definition_path: &TokenStream,
    runtime: &TokenStream,
    generic_type_names: &[Ident],
    owner: &Ident,
) -> TokenStream {
    let bodies = variants.iter().enumerate().map(|(variant_index, variant)| {
        let fields = expand_generic_field_vector(
            &variant.fields,
            definition_path.clone(),
            runtime,
            true,
            generic_type_names,
            owner,
        );
        let canonical = &variant.canonical_name;
        let serialized = &variant.serialized_name;
        let deserialized = &variant.deserialized_name;
        let default = variant.default;
        let rust_name = &variant.rust_name;
        quote! {
            {
                #fields
                let fields: &'static [#runtime::metadata::FieldMetadata] =
                    #runtime::__private::v8::leak_slice(fields);
                variants.push(#runtime::__private::v8::generic_enum_variant_metadata(
                    #definition_path,
                    #variant_index,
                    #rust_name,
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
            #runtime::__private::v8::leak_slice(variants);
    }
}
