// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generates role-specific metadata for entities, projections, models, values,
//! and enums.

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

use super::fields::expand_field_vector;
use crate::ir::MacroKind;
use crate::ir::declaration::DeclarationIr;
use crate::ir::declaration::FieldIr;
use crate::ir::declaration::FieldOccurrence;
use crate::ir::declaration::VariantIr;

/// Generates role-specific model metadata for a declaration.
pub(super) fn expand_role(
    declaration: &DeclarationIr,
    runtime: &TokenStream,
) -> TokenStream {
    match declaration.kind {
        MacroKind::Entity => {
            let Some(index) = identifier_index(&declaration.fields) else {
                return Error::new(
                    Span::call_site(),
                    "Entity requires exactly one identifier field",
                )
                .into_compile_error();
            };
            quote! {
                let role: &'static #runtime::metadata::RoleMetadata =
                    #runtime::__private::v5::leak(#runtime::__private::v5::entity_role(&fields[#index]));
            }
        }
        MacroKind::Projection => {
            let Some(index) = identifier_index(&declaration.fields) else {
                return Error::new(
                    Span::call_site(),
                    "Projection requires exactly one identifier field",
                )
                .into_compile_error();
            };
            let source = if let Some(source) =
                declaration.options.source.as_ref()
            {
                quote!(Some(#runtime::__private::v5::leak(
                    #runtime::metadata::DeclaredEntityTarget::RustType(#runtime::metadata::TypeMetadata::of::<#source>),
                ) as &'static #runtime::metadata::DeclaredEntityTarget))
            } else if let Some(id) = declaration.options.source_id.as_ref() {
                quote!(Some(#runtime::__private::v5::leak(
                    #runtime::metadata::DeclaredEntityTarget::ModelId(#runtime::metadata::ModelId::new(#id)),
                ) as &'static #runtime::metadata::DeclaredEntityTarget))
            } else {
                quote!(None)
            };
            quote! {
                let source = #source;
                let role: &'static #runtime::metadata::RoleMetadata =
                    #runtime::__private::v5::leak(#runtime::__private::v5::projection_role(&fields[#index], source));
            }
        }
        MacroKind::Model => quote! {
            let role: &'static #runtime::metadata::RoleMetadata =
                #runtime::__private::v5::leak(#runtime::__private::v5::model_role());
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
                        let reference: &'static #runtime::metadata::CodecReference = #runtime::__private::v5::leak(
                            #runtime::metadata::CodecReference::RustType(
                                #runtime::__private::v5::RustTypeReference::of::<#codec_type>(),
                            ),
                        );
                        Some(#runtime::__private::v5::leak(
                            #runtime::metadata::CodecMetadata::new(reference, #runtime::metadata::CodecSource::CanonicalValue),
                        ) as &'static #runtime::metadata::CodecMetadata)
                    })
                },
            );
            quote! {
                let canonical_codec = #canonical_codec;
                let role: &'static #runtime::metadata::RoleMetadata = #runtime::__private::v5::leak(
                    #runtime::__private::v5::value_role(#transparent, canonical_codec),
                );
            }
        }
        MacroKind::Enum => expand_enum_role(&declaration.variants, runtime),
        MacroKind::ModelImpl => Error::new(
            Span::call_site(),
            "ModelImpl does not produce role metadata",
        )
        .into_compile_error(),
    }
}

/// Generates role metadata for all enum variants.
fn expand_enum_role(
    variants: &[VariantIr],
    runtime: &TokenStream,
) -> TokenStream {
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
                let fields: &'static [#runtime::metadata::FieldMetadata] = #runtime::__private::v5::leak_slice(fields);
                let reflect = &descriptor.variants()[#variant_index];
                debug_assert_eq!(reflect.rust_name(), #rust_name);
                variants.push(#runtime::__private::v5::enum_variant_metadata(
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
        let variants: &'static [#runtime::metadata::EnumVariantMetadata] = #runtime::__private::v5::leak_slice(variants);
        let role: &'static #runtime::metadata::RoleMetadata =
            #runtime::__private::v5::leak(#runtime::__private::v5::enum_role(variants));
        let fields: &'static [#runtime::metadata::FieldMetadata] = &[];
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
