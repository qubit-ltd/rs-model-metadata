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
use syn::GenericArgument;
use syn::PathArguments;
use syn::Type;

use super::fields::expand_field_vector;
use crate::compiler::type_path::is_option_path;
use crate::ir::MacroKind;
use crate::ir::declaration::DeclarationIr;
use crate::ir::declaration::FieldIr;
use crate::ir::declaration::FieldOccurrence;
use crate::ir::declaration::VariantIr;

/// Generates role-specific model metadata for a declaration.
pub(super) fn expand_role(declaration: &DeclarationIr, runtime: &TokenStream) -> TokenStream {
    match declaration.kind {
        MacroKind::Entity => {
            let Some(index) = identifier_index(&declaration.fields) else {
                return Error::new(Span::call_site(), "Entity requires exactly one identifier field")
                    .into_compile_error();
            };
            quote! {
                let role: &'static #runtime::RoleMetadata =
                    #runtime::__private::v4::leak(#runtime::__private::v4::entity_role(&fields[#index]));
            }
        }
        MacroKind::Projection => {
            let Some(index) = identifier_index(&declaration.fields) else {
                return Error::new(Span::call_site(), "Projection requires exactly one identifier field")
                    .into_compile_error();
            };
            let source = if let Some(source) = declaration.options.source.as_ref() {
                quote!(Some(#runtime::__private::v4::leak(
                    #runtime::DeclaredEntityTarget::RustType(#runtime::TypeMetadata::of::<#source>),
                ) as &'static #runtime::DeclaredEntityTarget))
            } else if let Some(id) = declaration.options.source_id.as_ref() {
                quote!(Some(#runtime::__private::v4::leak(
                    #runtime::DeclaredEntityTarget::ModelId(#runtime::ModelId::new(#id)),
                ) as &'static #runtime::DeclaredEntityTarget))
            } else {
                quote!(None)
            };
            quote! {
                let source = #source;
                let role: &'static #runtime::RoleMetadata =
                    #runtime::__private::v4::leak(#runtime::__private::v4::projection_role(&fields[#index], source));
            }
        }
        MacroKind::Model => quote! {
            let role: &'static #runtime::RoleMetadata =
                #runtime::__private::v4::leak(#runtime::__private::v4::model_role());
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
                        let reference: &'static #runtime::CodecReference = #runtime::__private::v4::leak(
                            #runtime::CodecReference::RustType(#runtime::__private::v4::leak(
                                #runtime::__private::v4::ValueCodecDescriptor::of::<#codec_type, Self>(),
                            )),
                        );
                        Some(#runtime::__private::v4::leak(
                            #runtime::CodecMetadata::new(reference, #runtime::CodecSource::CanonicalValue),
                        ) as &'static #runtime::CodecMetadata)
                    })
                },
            );
            quote! {
                let canonical_codec = #canonical_codec;
                let role: &'static #runtime::RoleMetadata = #runtime::__private::v4::leak(
                    #runtime::__private::v4::value_role(#transparent, canonical_codec),
                );
            }
        }
        MacroKind::Enum => expand_enum_role(&declaration.variants, runtime),
        MacroKind::ModelImpl => {
            Error::new(Span::call_site(), "ModelImpl does not produce role metadata").into_compile_error()
        }
    }
}

/// Returns the value type encoded by a codec type declaration.
pub(super) fn codec_value_type(ty: &Type) -> &Type {
    let Type::Path(path) = ty else { return ty };
    if path.qself.is_some() || !is_option_path(&path.path) {
        return ty;
    }
    let Some(segment) = path.path.segments.last() else {
        return ty;
    };
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return ty;
    };
    arguments
        .args
        .iter()
        .find_map(|argument| match argument {
            GenericArgument::Type(ty) => Some(ty),
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
                let fields: &'static [#runtime::FieldMetadata] = #runtime::__private::v4::leak_slice(fields);
                let reflect = &descriptor.variants()[#variant_index];
                debug_assert_eq!(reflect.rust_name(), #rust_name);
                variants.push(#runtime::__private::v4::enum_variant_metadata(
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
        let variants: &'static [#runtime::EnumVariantMetadata] = #runtime::__private::v4::leak_slice(variants);
        let role: &'static #runtime::RoleMetadata =
            #runtime::__private::v4::leak(#runtime::__private::v4::enum_role(variants));
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

#[cfg(test)]
mod tests {
    use syn::Type;
    use syn::parse_quote;

    use super::codec_value_type;

    #[test]
    fn test_codec_value_type_unwraps_only_standard_option_paths() {
        let plain: Type = parse_quote!(String);
        let option: Type = parse_quote!(Option<String>);
        let qualified: Type = parse_quote!(::core::option::Option<&'static str>);
        let lookalike: Type = parse_quote!(domain::Option<String>);
        let reference: Type = parse_quote!(&'static str);

        assert_eq!(codec_value_type(&plain), &plain);
        assert_eq!(codec_value_type(&option), &parse_quote!(String));
        assert_eq!(codec_value_type(&qualified), &parse_quote!(&'static str),);
        assert_eq!(codec_value_type(&lookalike), &lookalike);
        assert_eq!(codec_value_type(&reference), &reference);
    }
}
