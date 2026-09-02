// =============================================================================

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Expr;
use syn::ExprLit;
use syn::Fields;
use syn::Lit;
use syn::LitStr;
use syn::Type;
use syn::parse_quote;

use super::MacroKind;
use super::declaration_ir::CodecIr;
use super::declaration_ir::ConstraintIr;
use super::declaration_ir::DeclarationIr;
use super::declaration_ir::FieldIr;
use super::declaration_ir::FieldOccurrence;
use super::declaration_ir::IdentifierAssignmentIr;
use super::declaration_ir::RedactIr;
use super::declaration_ir::RedactModeIr;
use super::declaration_ir::ReferenceIr;
use super::declaration_ir::ReferenceTargetIr;
use super::declaration_ir::SelectorIr;
use super::declaration_ir::SelectorPositionIr;
use super::declaration_ir::SerdeIr;
use super::declaration_ir::StrategyArgumentIr;
use super::declaration_ir::ValidatorIr;
use super::declaration_ir::VariantIr;
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Generates lazy type metadata and registration implementations.
pub(super) fn expand_metadata(declaration: &DeclarationIr, item: &DeriveInput, runtime: &TokenStream) -> TokenStream {
    let ident = &item.ident;
    let fields = expand_field_vector(&declaration.fields, quote!(descriptor.fields()), runtime);
    let role = expand_role(declaration, runtime);
    let declared_model_id = declaration
        .options
        .id
        .as_ref()
        .map_or_else(|| quote!(None), |id| quote!(Some(#runtime::ModelId::new(#id))));
    let has_generics = !item.generics.params.is_empty();
    let mut impl_generics_source = item.generics.clone();
    for parameter in impl_generics_source.type_params_mut() {
        parameter.bounds.push(parse_quote!(#runtime::Reflect));
        parameter.bounds.push(parse_quote!('static));
    }
    let (impl_generics, ty_generics, where_clause) = impl_generics_source.split_for_impl();
    let generic_metadata = format_ident!("__qubit_model_generic_metadata_{}", ident.to_string().to_snake_case());
    let registration = match (declaration.options.id.as_ref(), has_generics) {
        (Some(_), false) => expand_registration(ident, runtime),
        (Some(id), true) => expand_generic_registration(
            ident,
            id,
            declaration.kind,
            &generic_metadata,
            &declaration.fields,
            &declaration.variants,
            &item.data,
            &item.generics,
            runtime,
        ),
        (None, _) => TokenStream::new(),
    };
    let model_id = if has_generics {
        quote!(None)
    } else {
        declared_model_id.clone()
    };
    let generic_definition = if has_generics {
        declaration.options.id.as_ref().map_or_else(TokenStream::new, |_| {
            quote! {
                let metadata = metadata.generic_definition(#generic_metadata());
            }
        })
    } else {
        TokenStream::new()
    };
    let build_metadata = quote! {
        let descriptor = #runtime::TypeDescriptor::of::<Self>();
        #fields
        let fields: &'static [#runtime::FieldMetadata] = #runtime::__private::v3::leak_slice(fields);
        #role
        let properties: ::std::vec::Vec<_> = fields
            .iter()
            .filter_map(|field| field.name().map(|name| {
                #runtime::__private::v3::property_metadata(
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
                #runtime::__private::v3::property_fragment(
                    name,
                    field.type_ref(),
                    #runtime::PropertyFragmentSource::Field(field),
                )
            }))
            .collect();
        let properties: &'static [#runtime::PropertyMetadata] = #runtime::__private::v3::leak_slice(properties);
        let property_fragments: &'static [#runtime::PropertyFragment] =
            #runtime::__private::v3::leak_slice(property_fragments);
        let metadata = #runtime::__private::v3::GeneratedTypeMetadataBuilder::new(
            descriptor, #model_id, fields, role,
        ).properties(properties).property_fragments(property_fragments);
        #generic_definition
        metadata.finish_unchecked()
    };
    let metadata_body = if has_generics {
        quote! {
            static CACHE: ::std::sync::OnceLock<
                ::std::sync::Mutex<
                    ::std::collections::HashMap<::std::any::TypeId, &'static #runtime::TypeMetadata>,
                >,
            > = ::std::sync::OnceLock::new();
            let cache = CACHE.get_or_init(|| ::std::sync::Mutex::new(::std::collections::HashMap::new()));
            let type_id = ::std::any::TypeId::of::<Self>();
            let mut guard = cache.lock().unwrap_or_else(::std::sync::PoisonError::into_inner);
            if let Some(metadata) = guard.get(&type_id).copied() {
                return metadata;
            }
            let metadata: &'static #runtime::TypeMetadata = #runtime::__private::v3::leak({ #build_metadata });
            guard.insert(type_id, metadata);
            metadata
        }
    } else {
        quote! {
            static METADATA: ::std::sync::OnceLock<#runtime::TypeMetadata> = ::std::sync::OnceLock::new();
            METADATA.get_or_init(|| { #build_metadata })
        }
    };

    quote! {
        impl #impl_generics #runtime::__private::ModelTypeSeal for #ident #ty_generics #where_clause {}

        impl #impl_generics #runtime::__private::TypeMetadataProvider for #ident #ty_generics #where_clause {
            fn __type_metadata() -> &'static #runtime::TypeMetadata {
                #metadata_body
            }
        }

        #registration
    }
}

/// Generates registration metadata for a generic model definition.
#[allow(clippy::too_many_arguments)]
fn expand_generic_registration(
    ident: &syn::Ident,
    id: &LitStr,
    kind: MacroKind,
    metadata_fn: &syn::Ident,
    fields: &[FieldIr],
    variants: &[VariantIr],
    data: &Data,
    generics: &syn::Generics,
    runtime: &TokenStream,
) -> TokenStream {
    let snake_name = ident.to_string().to_snake_case();
    let definition_fn = format_ident!("__qubit_reflect_generic_definition_{}", snake_name);
    let source_fn = format_ident!("__qubit_model_generic_source_{}", snake_name);
    let registration_fn = format_ident!("__qubit_model_generic_registration_{}", snake_name);
    let role = match kind {
        MacroKind::Model => quote!(#runtime::ModelRole::Model),
        MacroKind::Enum => quote!(#runtime::ModelRole::Enum),
        MacroKind::Value => quote!(#runtime::ModelRole::Value),
        MacroKind::Entity | MacroKind::Projection | MacroKind::ModelImpl => {
            return Error::new(
                proc_macro2::Span::call_site(),
                "only Model, Enum, and Value support generic registration",
            )
            .into_compile_error();
        }
    };
    let fingerprint = stable_fingerprint(&ident.to_string());
    let template = expand_generic_template(ident, fields, data, generics, runtime);
    let template_root = format_ident!("__qubit_model_generic_template_{}", snake_name);
    let template_fields = expand_field_vector(fields, quote!(template_descriptor.fields()), runtime);
    let template_variants = expand_generic_variant_vector(variants, runtime);
    quote! {
        #template

        #[doc(hidden)]
        fn #metadata_fn() -> &'static #runtime::GenericModelMetadata {
            static METADATA: ::std::sync::OnceLock<#runtime::GenericModelMetadata> =
                ::std::sync::OnceLock::new();
            METADATA.get_or_init(|| {
                let template_descriptor = #template_root();
                #template_fields
                let fields: &'static [#runtime::FieldMetadata] = #runtime::__private::v3::leak_slice(fields);
                #template_variants
                #runtime::__private::v3::generic_model_metadata(
                    #runtime::ModelId::new(#id),
                    #role,
                    #definition_fn(),
                    fields,
                    variants,
                )
            })
        }

        #[doc(hidden)]
        fn #source_fn() -> &'static #runtime::identity::FragmentIdentity {
            static SOURCE: ::std::sync::OnceLock<#runtime::identity::FragmentIdentity> =
                ::std::sync::OnceLock::new();
            SOURCE.get_or_init(|| #runtime::identity::FragmentIdentity::new(
                env!("CARGO_PKG_NAME"), module_path!(), line!(), column!(), "generic-model", #fingerprint,
            ))
        }

        #[doc(hidden)]
        fn #registration_fn() -> #runtime::ModelRegistration {
            #runtime::__private::v3::generic_registration(#metadata_fn(), #source_fn())
        }

        #runtime::__private::inventory::submit! {
            #runtime::ModelRegistrationFactory(#registration_fn)
        }
    }
}

/// Generates the descriptor template used by generic model instances.
fn expand_generic_template(
    ident: &syn::Ident,
    fields: &[FieldIr],
    data: &Data,
    generics: &syn::Generics,
    runtime: &TokenStream,
) -> TokenStream {
    if let Data::Enum(data) = data {
        return expand_generic_enum_template(ident, data, generics, runtime);
    }
    let snake_name = ident.to_string().to_snake_case();
    let marker = format_ident!("__QubitModelGenericTemplate{}", ident);
    let root = format_ident!("__qubit_model_generic_template_{}", snake_name);
    let type_parameters: std::collections::HashSet<_> = generics
        .type_params()
        .map(|parameter| parameter.ident.to_string())
        .collect();
    let const_parameters: std::collections::HashSet<_> = generics
        .const_params()
        .map(|parameter| parameter.ident.to_string())
        .collect();
    let declared_fields: Vec<_> = match data {
        Data::Struct(data) => data.fields.iter().collect(),
        Data::Enum(_) | Data::Union(_) => Vec::new(),
    };
    let field_descriptors = fields.iter().zip(declared_fields).map(|(field, declared)| {
        let index = *field.index.value();
        let name = declared
            .ident
            .as_ref()
            .map(|name| LitStr::new(&name.to_string(), name.span()));
        let rust_name = name.as_ref().map_or_else(|| quote!(None), |name| quote!(Some(#name)));
        let query_name = rust_name.clone();
        let declared_visibility = &declared.vis;
        let visibility = LitStr::new(
            &quote!(#declared_visibility).to_string().replace(' ', ""),
            proc_macro2::Span::call_site(),
        );
        let expression = expand_type_expression(&field.ty, &type_parameters, &const_parameters, runtime);
        quote! {
            {
                let field_type: &'static #runtime::descriptor::TypeRef = #runtime::__private::v3::leak(
                    #runtime::descriptor::TypeRef::Symbolic(#expression),
                );
                descriptors.push(#runtime::__private::reflect_codegen_v1::descriptor::field(
                    #root,
                    #index,
                    #rust_name,
                    #query_name,
                    field_type,
                    #runtime::identity::Visibility::from_source(#visibility),
                ));
            }
        }
    });
    let struct_kind = match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) => quote!(#runtime::descriptor::StructKind::Named),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                quote!(#runtime::descriptor::StructKind::Newtype)
            }
            Fields::Unnamed(_) => {
                quote!(#runtime::descriptor::StructKind::Tuple)
            }
            Fields::Unit => quote!(#runtime::descriptor::StructKind::Unit),
        },
        Data::Enum(_) | Data::Union(_) => {
            quote!(#runtime::descriptor::StructKind::Unit)
        }
    };
    let query_name = LitStr::new(&ident.to_string(), ident.span());
    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #marker;

        #[doc(hidden)]
        fn #root() -> &'static #runtime::TypeDescriptor {
            static DESCRIPTOR: ::std::sync::OnceLock<#runtime::TypeDescriptor> =
                ::std::sync::OnceLock::new();
            DESCRIPTOR.get_or_init(|| {
                let mut descriptors = ::std::vec::Vec::new();
                #(#field_descriptors)*
                let descriptors = #runtime::__private::v3::leak_slice(descriptors);
                #runtime::__private::reflect_codegen_v1::descriptor::struct_type::<#marker>(
                    #query_name,
                    #struct_kind,
                    descriptors,
                )
            })
        }
    }
}

/// Generates the symbolic reflection template for a generic enum definition.
fn expand_generic_enum_template(
    ident: &syn::Ident,
    data: &syn::DataEnum,
    generics: &syn::Generics,
    runtime: &TokenStream,
) -> TokenStream {
    let snake_name = ident.to_string().to_snake_case();
    let marker = format_ident!("__QubitModelGenericTemplate{}", ident);
    let root = format_ident!("__qubit_model_generic_template_{}", snake_name);
    let active = format_ident!("__qubit_model_generic_template_active_{}", snake_name);
    let type_parameters: std::collections::HashSet<_> = generics
        .type_params()
        .map(|parameter| parameter.ident.to_string())
        .collect();
    let const_parameters: std::collections::HashSet<_> = generics
        .const_params()
        .map(|parameter| parameter.ident.to_string())
        .collect();
    let variants = data.variants.iter().enumerate().map(|(variant_index, variant)| {
        let rust_name = LitStr::new(&variant.ident.to_string(), variant.ident.span());
        let kind = match &variant.fields {
            Fields::Unit => quote!(#runtime::descriptor::VariantKind::Unit),
            Fields::Unnamed(_) => quote!(#runtime::descriptor::VariantKind::Tuple),
            Fields::Named(_) => quote!(#runtime::descriptor::VariantKind::Struct),
        };
        let fields = variant.fields.iter().enumerate().map(|(field_index, field)| {
            let name = field
                .ident
                .as_ref()
                .map(|name| LitStr::new(&name.to_string(), name.span()));
            let rust_field_name = name.as_ref().map_or_else(|| quote!(None), |name| quote!(Some(#name)));
            let query_name = rust_field_name.clone();
            let declared_visibility = &field.vis;
            let visibility = LitStr::new(
                &quote!(#declared_visibility).to_string().replace(' ', ""),
                proc_macro2::Span::call_site(),
            );
            let expression = expand_type_expression(&field.ty, &type_parameters, &const_parameters, runtime);
            quote! {
                {
                    let field_type: &'static #runtime::descriptor::TypeRef =
                        #runtime::__private::v3::leak(#runtime::descriptor::TypeRef::Symbolic(#expression));
                    fields.push(
                        #runtime::__private::reflect_codegen_v1::descriptor::field(
                            #root,
                            #field_index,
                            #rust_field_name,
                            #query_name,
                            field_type,
                            #runtime::identity::Visibility::from_source(#visibility),
                        )
                        .with_variant(#variant_index, #rust_name),
                    );
                }
            }
        });
        quote! {
            {
                let mut fields = ::std::vec::Vec::new();
                #(#fields)*
                let fields = #runtime::__private::v3::leak_slice(fields);
                variants.push(#runtime::__private::reflect_codegen_v1::descriptor::variant(
                    #root,
                    #variant_index,
                    #rust_name,
                    #rust_name,
                    #kind,
                    fields,
                    #active,
                ));
            }
        }
    });
    let query_name = LitStr::new(&ident.to_string(), ident.span());
    quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #marker;

        #[doc(hidden)]
        fn #active(
            _value: #runtime::value::DynamicRef<'_, #runtime::value::Local>,
        ) -> ::std::result::Result<bool, #runtime::TypeMismatch> {
            Ok(false)
        }

        #[doc(hidden)]
        fn #root() -> &'static #runtime::TypeDescriptor {
            static DESCRIPTOR: ::std::sync::OnceLock<#runtime::TypeDescriptor> =
                ::std::sync::OnceLock::new();
            DESCRIPTOR.get_or_init(|| {
                let mut variants = ::std::vec::Vec::new();
                #(#variants)*
                let variants = #runtime::__private::v3::leak_slice(variants);
                #runtime::__private::reflect_codegen_v1::descriptor::enum_type::<#marker>(#query_name, variants)
            })
        }
    }
}

/// Generates complete symbolic overlays for generic enum variants.
fn expand_generic_variant_vector(variants: &[VariantIr], runtime: &TokenStream) -> TokenStream {
    let bodies = variants.iter().enumerate().map(|(variant_index, variant)| {
        let fields = expand_field_vector(
            &variant.fields,
            quote!(template_descriptor.variants()[#variant_index].fields()),
            runtime,
        );
        let canonical = &variant.canonical_name;
        let serialized = &variant.serialized_name;
        let deserialized = &variant.deserialized_name;
        let default = variant.default;
        quote! {
            {
                #fields
                let fields: &'static [#runtime::FieldMetadata] =
                    #runtime::__private::v3::leak_slice(fields);
                variants.push(#runtime::__private::v3::enum_variant_metadata(
                    &template_descriptor.variants()[#variant_index],
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
        let variants: &'static [#runtime::EnumVariantMetadata] =
            #runtime::__private::v3::leak_slice(variants);
    }
}

/// Converts a Rust type into the runtime's symbolic type expression.
fn expand_type_expression(
    ty: &Type,
    type_parameters: &std::collections::HashSet<String>,
    const_parameters: &std::collections::HashSet<String>,
    runtime: &TokenStream,
) -> TokenStream {
    match ty {
        Type::Path(path) if path.qself.is_none() => {
            if path.path.segments.len() == 1 {
                let name = path.path.segments[0].ident.to_string();
                if type_parameters.contains(&name) {
                    return quote!(#runtime::__private::reflect_codegen_v1::expression::parameter(#name));
                }
            }
            let segments: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|segment| LitStr::new(&segment.ident.to_string(), segment.ident.span()))
                .collect();
            let arguments = path.path.segments.last().map_or_else(Vec::new, |segment| {
                let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return Vec::new();
                };
                arguments
                    .args
                    .iter()
                    .filter_map(|argument| match argument {
                        syn::GenericArgument::Type(ty) => {
                            let expression = expand_type_expression(ty, type_parameters, const_parameters, runtime);
                            Some(quote!(#runtime::expression::GenericArgument::Type(#expression)))
                        }
                        syn::GenericArgument::Const(value) => {
                            let expression = expand_const_expression(value, const_parameters, runtime);
                            let diagnostic = LitStr::new(&quote!(#value).to_string(), proc_macro2::Span::call_site());
                            Some(quote!(#runtime::expression::GenericArgument::Const(
                                #runtime::__private::reflect_codegen_v1::expression::const_argument(
                                    #runtime::expression::TypeExpression::Concrete(
                                        #runtime::__private::reflect_codegen_v1::expression::concrete(
                                            ::std::boxed::Box::new(["_".into()]),
                                            ::std::boxed::Box::new([]),
                                            #runtime::expression::DiagnosticText::default(),
                                        ),
                                    ),
                                    #expression,
                                    #diagnostic,
                                ),
                            )))
                        }
                        _ => None,
                    })
                    .collect()
            });
            quote!(#runtime::expression::TypeExpression::Concrete(
                #runtime::__private::reflect_codegen_v1::expression::concrete(
                    ::std::boxed::Box::new([#(#segments.into()),*]),
                    ::std::boxed::Box::new([#(#arguments),*]),
                    #runtime::expression::DiagnosticText::default(),
                ),
            ))
        }
        Type::Slice(slice) => {
            let element = expand_type_expression(&slice.elem, type_parameters, const_parameters, runtime);
            quote!(#runtime::expression::TypeExpression::Slice(::std::boxed::Box::new(#element)))
        }
        Type::Array(array) => {
            let element = expand_type_expression(&array.elem, type_parameters, const_parameters, runtime);
            let length = expand_const_expression(&array.len, const_parameters, runtime);
            quote!(#runtime::expression::TypeExpression::Array(
                #runtime::__private::reflect_codegen_v1::expression::array(
                    #element,
                    #length,
                ),
            ))
        }
        Type::Tuple(tuple) => {
            let elements = tuple
                .elems
                .iter()
                .map(|element| expand_type_expression(element, type_parameters, const_parameters, runtime));
            quote!(#runtime::expression::TypeExpression::Tuple(
                ::std::boxed::Box::new([#(#elements),*]),
            ))
        }
        Type::Reference(reference) => {
            let target = expand_type_expression(&reference.elem, type_parameters, const_parameters, runtime);
            let lifetime = match reference
                .lifetime
                .as_ref()
                .map(|value| value.ident.to_string())
                .as_deref()
            {
                Some("static") => {
                    quote!(#runtime::expression::LifetimeExpression::Static)
                }
                Some("_") => {
                    quote!(#runtime::expression::LifetimeExpression::Placeholder)
                }
                Some(name) => {
                    quote!(#runtime::__private::reflect_codegen_v1::expression::named_lifetime(#name))
                }
                None => {
                    quote!(#runtime::expression::LifetimeExpression::Elided)
                }
            };
            let mutable = reference.mutability.is_some();
            quote!(#runtime::expression::TypeExpression::Reference(
                #runtime::__private::reflect_codegen_v1::expression::reference(
                    #lifetime,
                    #mutable,
                    #target,
                ),
            ))
        }
        _ => {
            let source = LitStr::new(&quote!(#ty).to_string(), proc_macro2::Span::call_site());
            quote!(#runtime::expression::TypeExpression::Concrete(
                #runtime::__private::reflect_codegen_v1::expression::concrete(
                    ::std::boxed::Box::new([#source.into()]),
                    ::std::boxed::Box::new([]),
                    #runtime::expression::DiagnosticText::from(#source),
                ),
            ))
        }
    }
}

/// Converts a const expression into the runtime's symbolic representation.
fn expand_const_expression(
    value: &Expr,
    const_parameters: &std::collections::HashSet<String>,
    runtime: &TokenStream,
) -> TokenStream {
    match value {
        Expr::Path(path) => {
            let segments: Vec<_> = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            if segments.len() == 1 && const_parameters.contains(&segments[0]) {
                let name = &segments[0];
                quote!(#runtime::__private::reflect_codegen_v1::expression::const_parameter(#name))
            } else {
                quote!(#runtime::__private::reflect_codegen_v1::expression::const_path([#(#segments),*]))
            }
        }
        Expr::Lit(ExprLit {
            lit: Lit::Int(value), ..
        }) => {
            let value = match value.base10_parse::<u128>() {
                Ok(value) => value,
                Err(_) => {
                    return Error::new(value.span(), "const integer literal exceeds u128").into_compile_error();
                }
            };
            quote!(#runtime::expression::ConstExpression::UnsignedInteger(#value))
        }
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value), ..
        }) => {
            let value = value.value;
            quote!(#runtime::expression::ConstExpression::Boolean(#value))
        }
        Expr::Lit(ExprLit {
            lit: Lit::Char(value), ..
        }) => {
            let value = value.value();
            quote!(#runtime::expression::ConstExpression::Character(#value))
        }
        _ => {
            let source = quote!(#value).to_string();
            quote!(#runtime::__private::reflect_codegen_v1::expression::const_path([#source]))
        }
    }
}

/// Generates the runtime field vector for a declaration.
fn expand_field_vector(fields: &[FieldIr], descriptor_fields: TokenStream, runtime: &TokenStream) -> TokenStream {
    let bodies = fields
        .iter()
        .map(|field| expand_field(field, &descriptor_fields, runtime));
    quote! {
        let mut fields = ::std::vec::Vec::new();
        #(#bodies)*
    }
}

/// Generates one field descriptor and its normalized attributes.
fn expand_field(field: &FieldIr, descriptor_fields: &TokenStream, runtime: &TokenStream) -> TokenStream {
    let index = *field.index.value();
    let field_type = &field.ty;
    let validator_irs: Vec<_> = field
        .occurrences
        .iter()
        .filter_map(|value| match value {
            FieldOccurrence::Validator(value) => Some(value),
            _ => None,
        })
        .collect();
    let validators = validator_irs.iter().map(|value| expand_validator(value, runtime));
    let identifier_assignment = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Identifier(value) => Some(*value),
        _ => None,
    });
    let has_identifier = identifier_assignment.is_some();
    let has_indexed = field
        .occurrences
        .iter()
        .any(|value| matches!(value, FieldOccurrence::Indexed));
    let unique_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Unique(value) => Some(value),
        _ => None,
    });
    let reference_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Reference(value) => Some(value),
        _ => None,
    });
    let key_part_order = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::KeyPart(value) => Some(*value),
        _ => None,
    });
    let codec_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Codec(value) => Some(value),
        _ => None,
    });
    let redact_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Redact(value) => Some(value),
        _ => None,
    });
    let serde_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Serde(value) => Some(value),
        _ => None,
    });
    let element_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::Element) => Some(value),
        _ => None,
    });
    let map_key_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::MapKey) => Some(value),
        _ => None,
    });
    let map_value_ir = field.occurrences.iter().find_map(|value| match value {
        FieldOccurrence::Selector(value) if matches!(value.position, SelectorPositionIr::MapValue) => Some(value),
        _ => None,
    });
    let element_selector = element_ir.map(|value| {
        let value_type = quote!(
            <#field_type as #runtime::__private::v3::SequenceConstraintTarget>::Element
        );
        expand_selector_metadata(value, &value_type, format_ident!("element_selector"), runtime)
    });
    let map_key_selector = map_key_ir.map(|value| {
        let value_type = quote!(
            <#field_type as #runtime::__private::v3::MapConstraintTarget>::Key
        );
        expand_selector_metadata(value, &value_type, format_ident!("map_key_selector"), runtime)
    });
    let map_value_selector = map_value_ir.map(|value| {
        let value_type = quote!(
            <#field_type as #runtime::__private::v3::MapConstraintTarget>::Value
        );
        expand_selector_metadata(value, &value_type, format_ident!("map_value_selector"), runtime)
    });
    let constraint_irs: Vec<_> = field
        .occurrences
        .iter()
        .filter_map(|value| match value {
            FieldOccurrence::Constraint(value) => Some(value),
            _ => None,
        })
        .collect();
    let constraints = constraint_irs.iter().map(|value| {
        expand_constraint(
            value,
            element_ir.is_some(),
            map_key_ir.is_some(),
            map_value_ir.is_some(),
            runtime,
        )
    });
    let constraint_assertions = expand_constraint_assertions(&constraint_irs, quote!(#field_type), runtime);
    let requires_sequence = element_ir.is_some()
        || constraint_irs
            .iter()
            .any(|value| matches!(value, ConstraintIr::Sequence { .. }));
    let requires_map = map_key_ir.is_some()
        || map_value_ir.is_some()
        || constraint_irs
            .iter()
            .any(|value| matches!(value, ConstraintIr::Map { .. }));
    let sequence_assertion = requires_sequence.then(|| {
        quote! {
            fn assert_sequence_target<T: #runtime::__private::v3::SequenceConstraintTarget>() {}
            assert_sequence_target::<#field_type>();
        }
    });
    let map_assertion = requires_map.then(|| {
        quote! {
            fn assert_map_target<T: #runtime::__private::v3::MapConstraintTarget>() {}
            assert_map_target::<#field_type>();
        }
    });
    let identifier = identifier_assignment.map(|assignment| {
        let assignment = match assignment {
            IdentifierAssignmentIr::Application => {
                quote!(#runtime::IdentifierAssignment::Application)
            }
            IdentifierAssignmentIr::Database => quote!(#runtime::IdentifierAssignment::Database),
        };
        quote! {
            fn assert_identifier_type<T: #runtime::__private::v3::IdentifierType>() {}
            assert_identifier_type::<#field_type>();
            let identifier: &'static #runtime::IdentifierMetadata = #runtime::__private::v3::leak(
                #runtime::IdentifierMetadata::new(#assignment),
            );
        }
    });
    let unique = unique_ir.map(|unique| {
        let paths = unique.respect_to.iter().map(|path| expand_field_path(path, runtime));
        let ignore_case = unique.ignore_case;
        quote! {
            let unique_paths: &'static [#runtime::PropertyPath] = #runtime::__private::v3::leak_slice(::std::vec![#(#paths),*]);
            let unique: &'static #runtime::FieldUniqueMetadata = #runtime::__private::v3::leak(
                #runtime::FieldUniqueMetadata::new(unique_paths, #ignore_case),
            );
        }
    });
    let reference = reference_ir.map(|value| expand_reference(value, runtime));
    let key_part = key_part_order.map(|order| {
        quote! {
            let key_part: &'static #runtime::KeyPartMetadata = #runtime::__private::v3::leak(
                #runtime::KeyPartMetadata::new(#order),
            );
        }
    });
    let codec = codec_ir.map(|codec| {
        let value = match codec {
            CodecIr::DeclaredId(id) => quote!(#runtime::CodecReference::DeclaredId(#id)),
            CodecIr::RustType(ty) => {
                let value_type = codec_value_type(&field.ty);
                quote!(#runtime::CodecReference::RustType(#runtime::__private::v3::leak(
                    #runtime::__private::v3::ValueCodecDescriptor::of::<#ty, #value_type>(),
                )))
            }
        };
        quote! {
            let codec_reference: &'static #runtime::CodecReference = #runtime::__private::v3::leak(#value);
            let codec: &'static #runtime::CodecMetadata = #runtime::__private::v3::leak(
                #runtime::CodecMetadata::new(codec_reference, #runtime::CodecSource::Field),
            );
        }
    });
    let redact = redact_ir.map(|value| expand_redact(value, quote!(#runtime::RedactPosition::Field), runtime));
    let serde = serde_ir.map_or_else(
        || quote!(let serde: &'static #runtime::SerdeFieldMetadata = &#runtime::SerdeFieldMetadata::DEFAULT;),
        |value| expand_serde(value, runtime),
    );
    let mut occurrence_tokens = Vec::new();
    let mut validator_index = 0usize;
    let mut constraint_index = 0usize;
    for occurrence in &field.occurrences {
        occurrence_tokens.push(match occurrence {
            FieldOccurrence::Identifier(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Identifier(identifier));)
            }
            FieldOccurrence::Indexed => TokenStream::new(),
            FieldOccurrence::Unique(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Unique(unique));),
            FieldOccurrence::Reference(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Reference(reference));)
            }
            FieldOccurrence::KeyPart(_) => {
                quote!(attributes.push(#runtime::FieldAttributeMetadata::KeyPart(key_part));)
            }
            FieldOccurrence::Constraint(_) => {
                let current = constraint_index;
                constraint_index += 1;
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Constraint(&constraints[#current]));)
            }
            FieldOccurrence::Selector(_) => TokenStream::new(),
            FieldOccurrence::Validator(_) => {
                let current = validator_index;
                validator_index += 1;
                quote!(attributes.push(#runtime::FieldAttributeMetadata::Validator(&validators[#current]));)
            }
            FieldOccurrence::Codec(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Codec(codec));),
            FieldOccurrence::Redact(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Redact(redact));),
            FieldOccurrence::Serde(_) => quote!(attributes.push(#runtime::FieldAttributeMetadata::Serde(serde));),
            FieldOccurrence::Opaque => quote!(attributes.push(#runtime::FieldAttributeMetadata::Opaque);),
        });
    }
    let mut reason_parts = Vec::new();
    if has_indexed {
        reason_parts.push(quote!(#runtime::IndexingReasons::EXPLICIT));
    }
    if has_identifier {
        reason_parts.push(quote!(#runtime::IndexingReasons::IDENTIFIER));
    }
    if unique_ir.is_some() {
        reason_parts.push(quote!(#runtime::IndexingReasons::UNIQUE));
    }
    if reference_ir.is_some() {
        reason_parts.push(quote!(#runtime::IndexingReasons::REFERENCE));
    }
    let indexed = reason_parts
        .into_iter()
        .reduce(|left, right| quote!(#left | #right))
        .map(|reasons| {
            quote! {
                attributes.push(#runtime::FieldAttributeMetadata::Indexed(#reasons));
            }
        });
    quote! {
        {
            #sequence_assertion
            #map_assertion
            #constraint_assertions
            #identifier
            #unique
            #reference
            #key_part
            let validators: &'static [#runtime::ValidatorMetadata] =
                #runtime::__private::v3::leak_slice(::std::vec![#(#validators),*]);
            #codec
            #redact
            #serde
            #element_selector
            #map_key_selector
            #map_value_selector
            let constraints: &'static [#runtime::ConstraintMetadata] =
                #runtime::__private::v3::leak_slice(::std::vec![#(#constraints),*]);
            let mut attributes = ::std::vec::Vec::new();
            #(#occurrence_tokens)*
            #indexed
            let attributes: &'static [#runtime::FieldAttributeMetadata] =
                #runtime::__private::v3::leak_slice(attributes);
            fields.push(#runtime::__private::v3::field_metadata(
                &#descriptor_fields[#index],
                attributes,
                constraints,
                validators,
                serde,
            ));
        }
    }
}

/// Generates runtime metadata for one normalized constraint.
fn expand_constraint(
    value: &ConstraintIr,
    has_element: bool,
    has_map_key: bool,
    has_map_value: bool,
    runtime: &TokenStream,
) -> TokenStream {
    match value {
        ConstraintIr::Text(value) => {
            let min_chars = option_number(value.min_chars);
            let max_chars = option_number(value.max_chars);
            let min_bytes = option_number(value.min_bytes);
            let max_bytes = option_number(value.max_bytes);
            let allowed = match value.allowed_chars.as_deref().unwrap_or("unicode") {
                "unicode" => quote!(#runtime::AllowedChars::Unicode),
                "printable_unicode" => {
                    quote!(#runtime::AllowedChars::PrintableUnicode)
                }
                "ascii" => quote!(#runtime::AllowedChars::Ascii),
                "printable_ascii" => {
                    quote!(#runtime::AllowedChars::PrintableAscii)
                }
                "code" => quote!(#runtime::AllowedChars::Code),
                _ => quote!(compile_error!("invalid allowed_chars value")),
            };
            let non_blank = value.non_blank;
            let format = value.format.as_deref().map_or_else(
                || quote!(None),
                |value| {
                    let value = match value {
                        "email" => quote!(#runtime::TextFormat::Email),
                        "cn_mobile" => quote!(#runtime::TextFormat::Mobile),
                        "uri" => quote!(#runtime::TextFormat::Uri),
                        "uuid" => quote!(#runtime::TextFormat::Uuid),
                        _ => quote!(compile_error!("invalid text format")),
                    };
                    quote!(Some(#value))
                },
            );
            quote!(#runtime::ConstraintMetadata::Text(#runtime::TextConstraint::new(
                #min_chars, #max_chars, #min_bytes, #max_bytes, #allowed, #non_blank, #format,
            )))
        }
        ConstraintIr::Decimal(value) => {
            let precision = option_number(value.precision);
            let scale = value.scale;
            let rounding = rounding_tokens(&value.rounding, runtime);
            let semantic = if value.money {
                quote!(#runtime::DecimalSemantic::Money)
            } else {
                quote!(#runtime::DecimalSemantic::Number)
            };
            let min = option_lit_str(&value.min);
            let max = option_lit_str(&value.max);
            let min_inclusive = value.min_inclusive;
            let max_inclusive = value.max_inclusive;
            quote!(#runtime::ConstraintMetadata::Decimal(
                #runtime::DecimalConstraint::new(#precision, #scale, #rounding, #semantic)
                    .with_bounds(#min, #max, #min_inclusive, #max_inclusive)
            ))
        }
        ConstraintIr::Time(value) => {
            let precision = match value.as_str() {
                "second" => quote!(#runtime::TemporalPrecision::Second),
                "millisecond" => {
                    quote!(#runtime::TemporalPrecision::Millisecond)
                }
                "microsecond" => {
                    quote!(#runtime::TemporalPrecision::Microsecond)
                }
                "nanosecond" => quote!(#runtime::TemporalPrecision::Nanosecond),
                _ => quote!(compile_error!("invalid time precision")),
            };
            quote!(#runtime::ConstraintMetadata::Time(#runtime::TimeConstraint::new(#precision)))
        }
        ConstraintIr::Sequence { min, max, unique } => {
            let min = option_number(*min);
            let max = option_number(*max);
            let base = quote!(#runtime::SequenceConstraint::new(#min, #max, #unique));
            let value = if has_element {
                quote!(#base.with_element(element_selector))
            } else {
                base
            };
            quote!(#runtime::ConstraintMetadata::Sequence(#value))
        }
        ConstraintIr::Map { min, max } => {
            let min = option_number(*min);
            let max = option_number(*max);
            let key = if has_map_key {
                quote!(Some(map_key_selector))
            } else {
                quote!(None)
            };
            let value = if has_map_value {
                quote!(Some(map_value_selector))
            } else {
                quote!(None)
            };
            quote!(#runtime::ConstraintMetadata::Map(#runtime::MapConstraint::new(#min, #max).with_selectors(#key, #value)))
        }
    }
}

/// Generates compile-time type-capability assertions for constraints.
fn expand_constraint_assertions(
    constraints: &[&ConstraintIr],
    target: TokenStream,
    runtime: &TokenStream,
) -> TokenStream {
    let assertions = constraints.iter().map(|constraint| match constraint {
        ConstraintIr::Text(_) => quote! {
            fn assert_text_target<T: #runtime::__private::v3::TextConstraintTarget + ?Sized>() {}
            assert_text_target::<#target>();
        },
        ConstraintIr::Decimal(_) => quote! {
            fn assert_decimal_target<T: #runtime::__private::v3::DecimalConstraintTarget>() {}
            assert_decimal_target::<#target>();
        },
        ConstraintIr::Time(_) => quote! {
            fn assert_temporal_target<T: #runtime::__private::v3::TemporalConstraintTarget>() {}
            assert_temporal_target::<#target>();
        },
        ConstraintIr::Sequence { min, max, unique } => {
            let length = (min.is_some() || max.is_some()).then(|| {
                quote! {
                    fn assert_variable_sequence<T: #runtime::__private::v3::VariableLengthSequenceTarget>() {}
                    assert_variable_sequence::<#target>();
                }
            });
            let uniqueness = unique.then(|| {
                quote! {
                    fn assert_unique_items_target<T: #runtime::__private::v3::UniqueItemsConstraintTarget>() {}
                    assert_unique_items_target::<#target>();
                }
            });
            quote! {
                fn assert_sequence_constraint<T: #runtime::__private::v3::SequenceConstraintTarget>() {}
                assert_sequence_constraint::<#target>();
                #length
                #uniqueness
            }
        }
        ConstraintIr::Map { .. } => quote! {
            fn assert_map_constraint<T: #runtime::__private::v3::MapConstraintTarget>() {}
            assert_map_constraint::<#target>();
        },
    });
    quote!(#(#assertions)*)
}

/// Generates runtime metadata for a collection selector.
fn expand_selector_metadata(
    value: &SelectorIr,
    value_type: &TokenStream,
    name: syn::Ident,
    runtime: &TokenStream,
) -> TokenStream {
    let position = match value.position {
        SelectorPositionIr::Element => {
            quote!(#runtime::SelectorPosition::Element)
        }
        SelectorPositionIr::MapKey => {
            quote!(#runtime::SelectorPosition::MapKey)
        }
        SelectorPositionIr::MapValue => {
            quote!(#runtime::SelectorPosition::MapValue)
        }
    };
    let constraints = value
        .constraints
        .iter()
        .map(|constraint| expand_constraint(constraint, false, false, false, runtime));
    let constraint_refs: Vec<_> = value.constraints.iter().collect();
    let constraint_assertions = expand_constraint_assertions(&constraint_refs, value_type.clone(), runtime);
    let validators = value
        .validators
        .iter()
        .map(|validator| expand_validator(validator, runtime));
    let codec = value.codec.as_ref().map_or_else(
        || quote!(None),
        |codec| {
            let reference = codec_reference_expression(codec, value_type, runtime);
            quote!({
                let reference: &'static #runtime::CodecReference = #runtime::__private::v3::leak(#reference);
                Some(#runtime::__private::v3::leak(
                    #runtime::CodecMetadata::new(reference, #runtime::CodecSource::Selector(#position)),
                ) as &'static #runtime::CodecMetadata)
            })
        },
    );
    let redact = value.redact.as_ref().map_or_else(
        || quote!(None),
        |redact| {
            let expression = redact_expression(
                redact,
                match value.position {
                    SelectorPositionIr::Element => quote!(#runtime::RedactPosition::Element),
                    SelectorPositionIr::MapKey => quote!(#runtime::RedactPosition::MapKey),
                    SelectorPositionIr::MapValue => quote!(#runtime::RedactPosition::MapValue),
                },
                runtime,
            );
            quote!(Some(#runtime::__private::v3::leak(#expression) as &'static #runtime::RedactMetadata))
        },
    );
    quote! {
        #constraint_assertions
        let selector_constraints: &'static [#runtime::ConstraintMetadata] = #runtime::__private::v3::leak_slice(::std::vec![#(#constraints),*]);
        let selector_validators: &'static [#runtime::ValidatorMetadata] = #runtime::__private::v3::leak_slice(::std::vec![#(#validators),*]);
        let selector_codec = #codec;
        let selector_redact = #redact;
        let #name: &'static #runtime::SelectorMetadata = #runtime::__private::v3::leak(
            #runtime::SelectorMetadata::new(#position, selector_constraints, selector_validators, selector_codec, selector_redact),
        );
    }
}

/// Converts an optional tokenizable number into runtime option tokens.
fn option_number<T: quote::ToTokens>(value: Option<T>) -> TokenStream {
    value.map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

/// Maps a validated rounding name to runtime enum tokens.
fn rounding_tokens(value: &str, runtime: &TokenStream) -> TokenStream {
    match value {
        "down" => quote!(#runtime::RoundingMode::Down),
        "up" => quote!(#runtime::RoundingMode::Up),
        "ceiling" => quote!(#runtime::RoundingMode::Ceiling),
        "floor" => quote!(#runtime::RoundingMode::Floor),
        "half_up" => quote!(#runtime::RoundingMode::HalfUp),
        "half_down" => quote!(#runtime::RoundingMode::HalfDown),
        "half_even" => quote!(#runtime::RoundingMode::HalfEven),
        "unnecessary" => quote!(#runtime::RoundingMode::Unnecessary),
        _ => quote!(compile_error!("invalid rounding mode")),
    }
}

/// Generates runtime metadata for one validator declaration.
fn expand_validator(validator: &ValidatorIr, runtime: &TokenStream) -> TokenStream {
    let id = &validator.id;
    let params = validator.params.iter().map(|(name, value)| {
        let value = expand_strategy_argument(value, runtime);
        quote!(#runtime::NamedValidationArgument::new(#name, #value))
    });
    let depends_on = validator.depends_on.iter().map(|path| expand_field_path(path, runtime));
    quote!({
        let params: &'static [#runtime::NamedValidationArgument<'static>] = #runtime::__private::v3::leak_slice(::std::vec![#(#params),*]);
        let depends_on: &'static [#runtime::PropertyPath<'static>] = #runtime::__private::v3::leak_slice(::std::vec![#(#depends_on),*]);
        #runtime::ValidatorMetadata::new(#id, params, depends_on)
    })
}

/// Generates runtime tokens for one validator strategy argument.
fn expand_strategy_argument(value: &StrategyArgumentIr, runtime: &TokenStream) -> TokenStream {
    match value {
        StrategyArgumentIr::Bool(value) => {
            quote!(#runtime::ValidationArgument::Bool(#value))
        }
        StrategyArgumentIr::Integer(value) => {
            quote!(#runtime::ValidationArgument::Integer(#value))
        }
        StrategyArgumentIr::Unsigned(value) => {
            quote!(#runtime::ValidationArgument::Unsigned(#value))
        }
        StrategyArgumentIr::String(value) => {
            quote!(#runtime::ValidationArgument::String(#value))
        }
        StrategyArgumentIr::BoolList(values) => {
            quote!(#runtime::ValidationArgument::BoolList(&[#(#values),*]))
        }
        StrategyArgumentIr::IntegerList(values) => {
            quote!(#runtime::ValidationArgument::IntegerList(&[#(#values),*]))
        }
        StrategyArgumentIr::UnsignedList(values) => {
            quote!(#runtime::ValidationArgument::UnsignedList(&[#(#values),*]))
        }
        StrategyArgumentIr::StringList(values) => {
            quote!(#runtime::ValidationArgument::StringList(&[#(#values),*]))
        }
    }
}

/// Generates runtime metadata for one relationship declaration.
fn expand_reference(reference: &ReferenceIr, runtime: &TokenStream) -> TokenStream {
    let target = match &reference.target {
        ReferenceTargetIr::RustType(ty) => {
            quote!(#runtime::DeclaredEntityTarget::RustType(#runtime::TypeMetadata::of::<#ty>))
        }
        ReferenceTargetIr::ModelId(id) => {
            quote!(#runtime::DeclaredEntityTarget::ModelId(#runtime::ModelId::new(#id)))
        }
    };
    let selection = reference.property.as_ref().map_or_else(
        || quote!(#runtime::ReferenceSelection::Entity),
        |path| {
            let path = expand_field_path(path, runtime);
            quote!(#runtime::ReferenceSelection::Property(#path))
        },
    );
    let same_as = reference.same_as.as_ref().map_or_else(
        || quote!(None),
        |path| {
            let path = expand_field_path(path, runtime);
            quote!(Some(#runtime::__private::v3::leak(#path) as &'static #runtime::PropertyPath<'static>))
        },
    );
    let existing = reference.existing;
    quote! {
        let reference_target: &'static #runtime::DeclaredEntityTarget = #runtime::__private::v3::leak(#target);
        let reference_selection: &'static #runtime::ReferenceSelection = #runtime::__private::v3::leak(#selection);
        let reference: &'static #runtime::FieldReferenceMetadata = #runtime::__private::v3::leak(
            #runtime::FieldReferenceMetadata::new(reference_target, reference_selection, #existing, #same_as),
        );
    }
}

/// Generates runtime metadata for one redaction declaration.
fn expand_redact(redact: &RedactIr, position: TokenStream, runtime: &TokenStream) -> TokenStream {
    let expression = redact_expression(redact, position, runtime);
    quote! {
        let redact: &'static #runtime::RedactMetadata = #runtime::__private::v3::leak(#expression);
    }
}

/// Generates the redaction expression associated with a redaction mode.
fn redact_expression(redact: &RedactIr, position: TokenStream, runtime: &TokenStream) -> TokenStream {
    let (sensitivity, mode) = match &redact.mode {
        RedactModeIr::Level(level) => {
            let sensitivity = match level.as_str() {
                "low" => quote!(#runtime::Sensitivity::Low),
                "medium" => quote!(#runtime::Sensitivity::Medium),
                "high" => quote!(#runtime::Sensitivity::High),
                "secret" => quote!(#runtime::Sensitivity::Secret),
                _ => quote!(compile_error!("redact level must be low, medium, high, or secret")),
            };
            (quote!(Some(#sensitivity)), quote!(#runtime::RedactModeMetadata::Level))
        }
        RedactModeIr::Skip => (quote!(None), quote!(#runtime::RedactModeMetadata::Skip)),
        RedactModeIr::Nested => (quote!(None), quote!(#runtime::RedactModeMetadata::Nested)),
        RedactModeIr::Map => (quote!(None), quote!(#runtime::RedactModeMetadata::Map)),
        RedactModeIr::KeyedBy(field) => (quote!(None), quote!(#runtime::RedactModeMetadata::KeyedBy(#field))),
        RedactModeIr::Json => (quote!(None), quote!(#runtime::RedactModeMetadata::Json)),
    };
    quote!(#runtime::RedactMetadata::new(#sensitivity, #mode, #position))
}

/// Generates runtime metadata for a declared value codec.
fn codec_reference_expression<T: quote::ToTokens>(
    codec: &CodecIr,
    value_type: &T,
    runtime: &TokenStream,
) -> TokenStream {
    match codec {
        CodecIr::DeclaredId(id) => {
            quote!(#runtime::CodecReference::DeclaredId(#id))
        }
        CodecIr::RustType(ty) => {
            quote!(#runtime::CodecReference::RustType(#runtime::__private::v3::leak(
                #runtime::__private::v3::ValueCodecDescriptor::of::<#ty, #value_type>(),
            )))
        }
    }
}

/// Generates runtime metadata for one Serde behavior declaration.
fn expand_serde(value: &SerdeIr, runtime: &TokenStream) -> TokenStream {
    let serialize_name = option_lit_str(&value.serialize_name);
    let deserialize_name = option_lit_str(&value.deserialize_name);
    let skip_serializing = value.skip_serializing;
    let skip_deserializing = value.skip_deserializing;
    let flatten = value.flatten;
    let with = option_lit_str(&value.with);
    let default = value.default;
    let default_source = if value.default_from_model {
        quote!(#runtime::SerdeBehaviorSource::ModelDefault)
    } else if value.default {
        quote!(#runtime::SerdeBehaviorSource::Explicit)
    } else {
        quote!(#runtime::SerdeBehaviorSource::None)
    };
    let omit_source = if value.omit_from_model {
        quote!(#runtime::SerdeBehaviorSource::ModelDefault)
    } else if value.omit_suppressed {
        quote!(#runtime::SerdeBehaviorSource::Suppressed)
    } else if value.explicit_skip_serializing_if {
        quote!(#runtime::SerdeBehaviorSource::Explicit)
    } else {
        quote!(#runtime::SerdeBehaviorSource::None)
    };
    quote! {
        let serde: &'static #runtime::SerdeFieldMetadata = #runtime::__private::v3::leak(
            #runtime::SerdeFieldMetadata::new(#serialize_name, #deserialize_name, #skip_serializing, #skip_deserializing, #flatten, #with, #default)
                .with_sources(#default_source, #omit_source),
        );
    }
}

/// Converts an owned field path into runtime path tokens.
fn expand_field_path(path: &[String], runtime: &TokenStream) -> TokenStream {
    quote!(#runtime::PropertyPath::new(&[#(#path),*]))
}

/// Converts an optional string literal into runtime option tokens.
fn option_lit_str(value: &Option<LitStr>) -> TokenStream {
    value
        .as_ref()
        .map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

/// Generates role-specific model metadata for a declaration.
fn expand_role(declaration: &DeclarationIr, runtime: &TokenStream) -> TokenStream {
    match declaration.kind {
        MacroKind::Entity => {
            let Some(index) = identifier_index(&declaration.fields) else {
                return Error::new(
                    proc_macro2::Span::call_site(),
                    "Entity requires exactly one identifier field",
                )
                .into_compile_error();
            };
            quote! {
                let role: &'static #runtime::RoleMetadata =
                    #runtime::__private::v3::leak(#runtime::__private::v3::entity_role(&fields[#index]));
            }
        }
        MacroKind::Projection => {
            let Some(index) = identifier_index(&declaration.fields) else {
                return Error::new(
                    proc_macro2::Span::call_site(),
                    "Projection requires exactly one identifier field",
                )
                .into_compile_error();
            };
            let source = if let Some(source) = declaration.options.source.as_ref() {
                quote!(Some(#runtime::__private::v3::leak(
                    #runtime::DeclaredEntityTarget::RustType(#runtime::TypeMetadata::of::<#source>),
                ) as &'static #runtime::DeclaredEntityTarget))
            } else if let Some(id) = declaration.options.source_id.as_ref() {
                quote!(Some(#runtime::__private::v3::leak(
                    #runtime::DeclaredEntityTarget::ModelId(#runtime::ModelId::new(#id)),
                ) as &'static #runtime::DeclaredEntityTarget))
            } else {
                quote!(None)
            };
            quote! {
                let source = #source;
                let role: &'static #runtime::RoleMetadata =
                    #runtime::__private::v3::leak(#runtime::__private::v3::projection_role(&fields[#index], source));
            }
        }
        MacroKind::Model => quote! {
            let role: &'static #runtime::RoleMetadata =
                #runtime::__private::v3::leak(#runtime::__private::v3::model_role());
        },
        MacroKind::Value => {
            let transparent = if declaration.options.transparent {
                quote!(Some(&fields[0]))
            } else {
                quote!(None)
            };
            let canonical_codec = declaration.options.codec.as_ref().map_or_else(
                || quote!(None),
                |codec_type| {
                    quote!({
                        let reference: &'static #runtime::CodecReference = #runtime::__private::v3::leak(
                            #runtime::CodecReference::RustType(#runtime::__private::v3::leak(
                                #runtime::__private::v3::ValueCodecDescriptor::of::<#codec_type, Self>(),
                            )),
                        );
                        Some(#runtime::__private::v3::leak(
                            #runtime::CodecMetadata::new(reference, #runtime::CodecSource::CanonicalValue),
                        ) as &'static #runtime::CodecMetadata)
                    })
                },
            );
            quote! {
                let canonical_codec = #canonical_codec;
                let role: &'static #runtime::RoleMetadata = #runtime::__private::v3::leak(
                    #runtime::__private::v3::value_role(#transparent, canonical_codec),
                );
            }
        }
        MacroKind::Enum => expand_enum_role(&declaration.variants, runtime),
        MacroKind::ModelImpl => Error::new(
            proc_macro2::Span::call_site(),
            "ModelImpl does not produce role metadata",
        )
        .into_compile_error(),
    }
}

/// Returns the value type encoded by a codec type declaration.
fn codec_value_type(ty: &Type) -> &Type {
    let Type::Path(path) = ty else { return ty };
    let Some(segment) = path.path.segments.last() else {
        return ty;
    };
    if segment.ident != "Option" {
        return ty;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return ty;
    };
    arguments
        .args
        .iter()
        .find_map(|argument| match argument {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .unwrap_or(ty)
}

/// Generates role metadata for all enum variants.
fn expand_enum_role(variants: &[VariantIr], runtime: &TokenStream) -> TokenStream {
    let variants = variants.iter().enumerate().map(|(variant_index, variant)| {
        let fields = expand_field_vector(
            &variant.fields,
            quote!(descriptor.variants()[#variant_index].fields()),
            runtime,
        );
        let canonical = &variant.canonical_name;
        let serialized = &variant.serialized_name;
        let deserialized = &variant.deserialized_name;
        let default = variant.default;
        let rust_name = &variant.rust_name;
        quote! {
            {
                #fields
                let fields: &'static [#runtime::FieldMetadata] = #runtime::__private::v3::leak_slice(fields);
                let reflect = &descriptor.variants()[#variant_index];
                debug_assert_eq!(reflect.rust_name(), #rust_name);
                variants.push(#runtime::__private::v3::enum_variant_metadata(
                    reflect,
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
        #(#variants)*
        let variants: &'static [#runtime::EnumVariantMetadata] = #runtime::__private::v3::leak_slice(variants);
        let role: &'static #runtime::RoleMetadata =
            #runtime::__private::v3::leak(#runtime::__private::v3::enum_role(variants));
        let fields: &'static [#runtime::FieldMetadata] = &[];
    }
}

/// Returns the index of the declaration's identifier field.
fn identifier_index(fields: &[FieldIr]) -> Option<usize> {
    fields.iter().position(|field| {
        field
            .occurrences
            .iter()
            .any(|value| matches!(value, FieldOccurrence::Identifier(_)))
    })
}

/// Generates inventory registration for a concrete model declaration.
fn expand_registration(ident: &syn::Ident, runtime: &TokenStream) -> TokenStream {
    let snake_name = ident.to_string().to_snake_case();
    let source_fn = format_ident!("__qubit_model_source_{}", snake_name);
    let registration_fn = format_ident!("__qubit_model_registration_{}", snake_name);
    let fingerprint = stable_fingerprint(&ident.to_string());
    quote! {
        #[doc(hidden)]
        fn #source_fn() -> &'static #runtime::identity::FragmentIdentity {
            static SOURCE: ::std::sync::OnceLock<#runtime::identity::FragmentIdentity> = ::std::sync::OnceLock::new();
            SOURCE.get_or_init(|| #runtime::identity::FragmentIdentity::new(
                env!("CARGO_PKG_NAME"),
                module_path!(),
                line!(),
                column!(),
                "model",
                #fingerprint,
            ))
        }

        #[doc(hidden)]
        fn #registration_fn() -> #runtime::ModelRegistration {
            #runtime::__private::v3::concrete_registration(
                #runtime::TypeMetadata::of::<#ident>(),
                #source_fn(),
            )
        }

        #runtime::__private::inventory::submit! {
            #runtime::ModelRegistrationFactory(#registration_fn)
        }
    }
}

/// Computes the stable registration fingerprint for `value`.
fn stable_fingerprint(value: &str) -> u64 {
    value.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}
