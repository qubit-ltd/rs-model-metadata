// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Token generation for enum-variant metadata.

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::LitStr;

use crate::normalize::ModelVariantIr;
use crate::normalize::ModelVariantShapeIr;

/// Generates enum-variant metadata values in declaration order.
///
/// # Parameters
///
/// - `variants`: Validated enum variants in declaration order.
/// - `runtime`: The resolved runtime crate path used by generated tokens.
///
/// # Returns
///
/// Returns `(field_statics, variants)`, where the first vector contains field
/// metadata declarations and the second contains enum-variant metadata values.
pub(crate) fn expand_variants(
    variants: &[ModelVariantIr],
    runtime: &TokenStream,
) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let mut field_statics = Vec::new();
    let mut expanded = Vec::with_capacity(variants.len());
    for variant in variants {
        let ordinal = variant.ordinal;
        let name = LitStr::new(&variant.name, Span::call_site());
        match &variant.shape {
            ModelVariantShapeIr::Unit => {
                expanded.push(quote!(#runtime::EnumVariantMetadata::new(#ordinal, #name)));
            }
            ModelVariantShapeIr::Tuple(fields) | ModelVariantShapeIr::Struct(fields) => {
                let field_static = format_ident!("VARIANT_{ordinal}_FIELDS");
                let expanded_fields = super::super::expand_fields(fields, runtime);
                let count = expanded_fields.len();
                field_statics.push(quote! {
                    static #field_static: [#runtime::FieldMetadata; #count] = [#(#expanded_fields),*];
                });
                let constructor = match variant.shape {
                    ModelVariantShapeIr::Tuple(_) => quote!(tuple),
                    ModelVariantShapeIr::Struct(_) => quote!(structure),
                    ModelVariantShapeIr::Unit => unreachable!("unit variants are handled separately"),
                };
                expanded.push(quote! {
                    #runtime::EnumVariantMetadata::#constructor(
                        #ordinal,
                        #name,
                        &#field_static,
                    )
                });
            }
        }
    }
    (field_statics, expanded)
}
