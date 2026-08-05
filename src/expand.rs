// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
//! Token generation for normalized static runtime model metadata.

use proc_macro2::{
    Span,
    TokenStream,
};
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    LitStr,
    Result,
};

use crate::attribute::{
    RoundingMode,
    SensitiveHandling,
    TemporalNormalization,
    TemporalPrecision,
    TextFormat,
    TextRepertoire,
};
use crate::input::ModelVariant;
use crate::normalize::{
    DecimalSemantic,
    FieldAttributeIr,
    FieldIr,
    ModelAttributeIr,
    ModelIr,
    ModelShapeIr,
    NamedFieldsIr,
    PrimaryKeyIr,
    UniqueIr,
};

/// Generates static metadata and trait implementations for one normalized
/// model.
///
/// Generated items refer only to public runtime metadata APIs. Errors are
/// reserved for future expansion checks that cannot be represented while
/// building the normalized IR.
pub(crate) fn expand(
    input: &ModelIr,
    runtime: &TokenStream,
) -> Result<TokenStream> {
    let ident = &input.ident;
    let kind =
        expand_type_kind(&input.shape, &input.attributes, ident, runtime);
    let capabilities = match &input.shape {
        ModelShapeIr::Newtype(field) if field.opaque.is_empty() => {
            let ty = &field.ty;
            quote!(<#ty as #runtime::HasTypeShape>::CAPABILITIES)
        }
        ModelShapeIr::NamedStruct(_)
        | ModelShapeIr::UnitStruct
        | ModelShapeIr::Newtype(_)
        | ModelShapeIr::FieldlessEnum(_) => {
            quote!(#runtime::TypeCapabilities::NONE)
        }
    };

    Ok(quote! {
        const _: () = {
            #kind

            impl #runtime::HasTypeShape for #ident {
                const TYPE_SHAPE: #runtime::TypeShape =
                    #runtime::TypeShape::Named(#runtime::NamedTypeRef::of::<Self>());
                const CAPABILITIES: #runtime::TypeCapabilities = #capabilities;
            }

            impl #runtime::HasTypeMetadata for #ident {
                fn type_metadata() -> &'static #runtime::TypeMetadata {
                    &MODEL_METADATA
                }
            }
        };
    })
}

/// Generates type-system diagnostics that remain meaningful even when local
/// semantic validation has already rejected the model.
pub(crate) fn expand_independent_diagnostics(
    input: &ModelIr,
    runtime: &TokenStream,
) -> TokenStream {
    let unique_assertions = expand_unique_capability_assertions(
        &input.shape,
        &input.attributes,
        runtime,
    );
    let field_assertions = model_fields(&input.shape)
        .iter()
        .flat_map(|field| expand_capability_assertions(field, runtime))
        .collect::<Vec<_>>();
    quote!(#(#unique_assertions)* #(#field_assertions)*)
}

/// Returns every field-bearing supported shape as a slice.
fn model_fields(shape: &ModelShapeIr) -> &[FieldIr] {
    match shape {
        ModelShapeIr::NamedStruct(fields) => fields,
        ModelShapeIr::Newtype(field) => std::slice::from_ref(field.as_ref()),
        ModelShapeIr::UnitStruct | ModelShapeIr::FieldlessEnum(_) => &[],
    }
}

/// Generates the static structural metadata for the normalized model shape.
fn expand_type_kind(
    shape: &ModelShapeIr,
    attributes: &[ModelAttributeIr],
    ident: &syn::Ident,
    runtime: &TokenStream,
) -> TokenStream {
    let unique_capability_assertions =
        expand_unique_capability_assertions(shape, attributes, runtime);
    let attributes = expand_model_attributes(attributes, runtime);
    match shape {
        ModelShapeIr::NamedStruct(fields) => {
            let fields = expand_fields(fields, runtime);
            let count = fields.len();
            quote! {
                #(#unique_capability_assertions)*
                static FIELDS: [#runtime::FieldMetadata; #count] = [#(#fields),*];
                static MODEL_METADATA: #runtime::TypeMetadata = #runtime::TypeMetadata::new(
                    #runtime::TypeIdentity::of::<#ident>(),
                    #runtime::TypeKind::Struct(#runtime::StructMetadata::new(&FIELDS)),
                    &[#(#attributes),*],
                );
            }
        }
        ModelShapeIr::UnitStruct => quote! {
            #(#unique_capability_assertions)*
            static MODEL_METADATA: #runtime::TypeMetadata = #runtime::TypeMetadata::new(
                #runtime::TypeIdentity::of::<#ident>(),
                #runtime::TypeKind::Struct(#runtime::StructMetadata::new(&[])),
                &[#(#attributes),*],
            );
        },
        ModelShapeIr::Newtype(field) => {
            let field = expand_field(field, runtime);
            quote! {
                #(#unique_capability_assertions)*
                static FIELDS: [#runtime::FieldMetadata; 1] = [#field];
                static MODEL_METADATA: #runtime::TypeMetadata = #runtime::TypeMetadata::new(
                    #runtime::TypeIdentity::of::<#ident>(),
                    #runtime::TypeKind::Newtype(#runtime::NewtypeMetadata::new(FIELDS[0])),
                    &[#(#attributes),*],
                );
            }
        }
        ModelShapeIr::FieldlessEnum(variants) => {
            let variants = expand_variants(variants, runtime);
            let count = variants.len();
            quote! {
                #(#unique_capability_assertions)*
                static VARIANTS: [#runtime::EnumVariantMetadata; #count] = [#(#variants),*];
                static MODEL_METADATA: #runtime::TypeMetadata = #runtime::TypeMetadata::new(
                    #runtime::TypeIdentity::of::<#ident>(),
                    #runtime::TypeKind::Enum(#runtime::EnumMetadata::new(&VARIANTS)),
                    &[#(#attributes),*],
                );
            }
        }
    }
}

/// Generates text-capability assertions for every normalized `ignore_case`
/// reference.
fn expand_unique_capability_assertions(
    shape: &ModelShapeIr,
    attributes: &[ModelAttributeIr],
    runtime: &TokenStream,
) -> Vec<TokenStream> {
    let fields = model_fields(shape);
    attributes
        .iter()
        .filter_map(|attribute| match attribute {
            ModelAttributeIr::Unique(unique) => Some(unique),
            _ => None,
        })
        .flat_map(|unique| &unique.ignore_case)
        .filter_map(|reference| {
            let field =
                fields.iter().find(|field| field.name == reference.name)?;
            let ty = &field.ty;
            let span = reference.span;
            Some(quote_spanned! {span=>
                const _: () = assert!(
                    <#ty as #runtime::HasTypeShape>::CAPABILITIES
                        .contains(#runtime::TypeCapabilities::TEXT),
                    "`ignore_case` requires a text-capable field",
                );
            })
        })
        .collect()
}

/// Generates canonical model-level runtime attributes in IR order.
fn expand_model_attributes(
    attributes: &[ModelAttributeIr],
    runtime: &TokenStream,
) -> Vec<TokenStream> {
    attributes
        .iter()
        .map(|attribute| match attribute {
            ModelAttributeIr::PrimaryKey(primary_key) => expand_primary_key(primary_key, runtime),
            ModelAttributeIr::Unique(unique) => expand_unique(unique, runtime),
            ModelAttributeIr::Index(index) => expand_named_fields(index, runtime, true),
            ModelAttributeIr::Key(key) => expand_named_fields(key, runtime, false),
            ModelAttributeIr::Ownership(ownership) => {
                let owner = ownership
                    .owner
                    .first()
                    .expect("ownership parser requires an owner");
                let span = ownership.span;
                quote_spanned! {span=>
                    #runtime::AttributeMetadata::Ownership(#runtime::OwnershipMetadata::new(
                        #runtime::NamedTypeRef::of::<#owner>(),
                    ))
                }
            }
        })
        .collect()
}

/// Generates one model-level primary-key attribute.
fn expand_primary_key(
    primary_key: &PrimaryKeyIr,
    runtime: &TokenStream,
) -> TokenStream {
    let fields = primary_key.fields.iter().map(|field| {
        let name = LitStr::new(&field.name, field.span);
        let generated = primary_key
            .generated
            .iter()
            .any(|candidate| candidate.name == field.name);
        quote_spanned! {field.span=>
            #runtime::PrimaryKeyFieldMetadata::new(#name, #generated)
        }
    });
    let span = primary_key.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::PrimaryKey(#runtime::PrimaryKeyMetadata::new(&[
            #(#fields),*
        ]))
    }
}

/// Generates one model-level unique constraint.
fn expand_unique(unique: &UniqueIr, runtime: &TokenStream) -> TokenStream {
    let name = expand_optional_name(unique.name.first());
    let fields = unique.fields.iter().map(|field| {
        let name = LitStr::new(&field.name, field.span);
        let ignore_case = unique
            .ignore_case
            .iter()
            .any(|candidate| candidate.name == field.name);
        let comparison = if ignore_case {
            quote!(#runtime::UniqueComparison::IgnoreCase)
        } else {
            quote!(#runtime::UniqueComparison::Exact)
        };
        quote_spanned! {field.span=>
            #runtime::UniqueFieldMetadata::new(#name, #comparison)
        }
    });
    let span = unique.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Unique(#runtime::UniqueMetadata::new(
            #name,
            &[#(#fields),*],
        ))
    }
}

/// Generates an index or logical-key attribute from ordered field names.
fn expand_named_fields(
    value: &NamedFieldsIr,
    runtime: &TokenStream,
    index: bool,
) -> TokenStream {
    let name = expand_optional_name(value.name.first());
    let fields = value
        .fields
        .iter()
        .map(|(name, span)| LitStr::new(name, *span));
    let span = value.span;
    if index {
        quote_spanned! {span=>
            #runtime::AttributeMetadata::Index(#runtime::IndexMetadata::new(
                #name,
                &[#(#fields),*],
            ))
        }
    } else {
        quote_spanned! {span=>
            #runtime::AttributeMetadata::Key(#runtime::KeyMetadata::new(
                #name,
                &[#(#fields),*],
            ))
        }
    }
}

/// Generates an `Option<&'static str>` expression for an optional logical name.
fn expand_optional_name(name: Option<&LitStr>) -> TokenStream {
    match name {
        Some(name) => quote!(Some(#name)),
        None => quote!(None),
    }
}

/// Generates field metadata values in declaration order.
fn expand_fields(
    fields: &[FieldIr],
    runtime: &TokenStream,
) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| expand_field(field, runtime))
        .collect()
}

/// Generates one static field metadata value and its canonical attributes.
fn expand_field(field: &FieldIr, runtime: &TokenStream) -> TokenStream {
    let ordinal = field.ordinal;
    let name = LitStr::new(&field.name, Span::call_site());
    let ty = &field.ty;
    let attributes = expand_field_attributes(&field.attributes, runtime);
    let capability_assertions = expand_capability_assertions(field, runtime);
    let field_type = if let Some(span) = field.opaque.first().copied() {
        quote_spanned!(span=> #runtime::TypeRef::opaque::<#ty>())
    } else {
        quote!(#runtime::TypeRef::of::<#ty>())
    };

    quote! {{
        #(#capability_assertions)*
        #runtime::FieldMetadata::new(
            #ordinal,
            #name,
            stringify!(#ty),
            #field_type,
            &[#(#attributes),*],
        )
    }}
}

/// Generates const assertions for capabilities that must be resolved through
/// Rust's type system.
fn expand_capability_assertions(
    field: &FieldIr,
    runtime: &TokenStream,
) -> Vec<TokenStream> {
    if !field.opaque.is_empty() {
        return Vec::new();
    }
    let ty = &field.ty;
    field
        .attributes
        .iter()
        .filter_map(|attribute| {
            let assertion = match attribute {
                FieldAttributeIr::Text(value) => expand_capability_assertion(
                    ty,
                    value.span,
                    quote!(#runtime::TypeCapabilities::TEXT),
                    "`text` requires a text-capable field",
                    runtime,
                ),
                FieldAttributeIr::Sequence(value) => {
                    expand_sequence_capability_assertion(ty, value, runtime)
                }
                FieldAttributeIr::Map(value) => expand_capability_assertion(
                    ty,
                    value.span,
                    quote!(#runtime::TypeCapabilities::MAP),
                    "`map` requires a map-capable field",
                    runtime,
                ),
                FieldAttributeIr::Temporal(value) => {
                    expand_capability_assertion(
                        ty,
                        value.span,
                        quote!(#runtime::TypeCapabilities::TEMPORAL),
                        "`time` requires a temporal-capable field",
                        runtime,
                    )
                }
                FieldAttributeIr::Decimal(value) => {
                    let message = match value.semantic {
                        DecimalSemantic::Number => {
                            "`decimal` requires a decimal-capable field"
                        }
                        DecimalSemantic::Money => {
                            "`money` requires a decimal-capable field"
                        }
                    };
                    expand_capability_assertion(
                        ty,
                        value.value.span,
                        quote!(#runtime::TypeCapabilities::DECIMAL),
                        message,
                        runtime,
                    )
                }
                FieldAttributeIr::Reference(_)
                | FieldAttributeIr::Sensitive(_)
                | FieldAttributeIr::Codec(_)
                | FieldAttributeIr::Generator(_) => return None,
            };
            Some(assertion)
        })
        .collect()
}

/// Generates one const assertion for a required type capability.
fn expand_capability_assertion(
    ty: &syn::Type,
    span: Span,
    capability: TokenStream,
    message: &str,
    runtime: &TokenStream,
) -> TokenStream {
    quote_spanned! {span=>
        const _: () = assert!(
            <#ty as #runtime::HasTypeShape>::CAPABILITIES.contains(#capability),
            #message,
        );
    }
}

/// Generates the authoritative sequence capability and redundancy checks.
fn expand_sequence_capability_assertion(
    ty: &syn::Type,
    value: &crate::attribute::SequenceAttribute,
    runtime: &TokenStream,
) -> TokenStream {
    let span = value.span;
    let unique_items_check = value.unique_items.first().map(|span| {
        quote_spanned! {*span=>
            if capabilities.contains(#runtime::TypeCapabilities::SET) {
                panic!("`unique_items` is redundant for Set fields");
            }
        }
    });
    let repeats_length =
        !value.min_items.is_empty() || !value.max_items.is_empty();
    quote_spanned! {span=>
        const _: () = {
            let capabilities = <#ty as #runtime::HasTypeShape>::CAPABILITIES;
            #unique_items_check
            if #repeats_length && capabilities.contains(#runtime::TypeCapabilities::ARRAY) {
                panic!("array length is fixed by its type; remove sequence length arguments");
            }
            assert!(
                capabilities.contains(#runtime::TypeCapabilities::SEQUENCE),
                "`sequence` requires a sequence-capable field",
            );
        };
    }
}

/// Generates canonical field-level runtime attributes in IR order.
fn expand_field_attributes(
    attributes: &[FieldAttributeIr],
    runtime: &TokenStream,
) -> Vec<TokenStream> {
    attributes
        .iter()
        .map(|attribute| match attribute {
            FieldAttributeIr::Text(value) => expand_text(value, runtime),
            FieldAttributeIr::Sequence(value) => {
                let min_items = expand_optional_u32(value.min_items.first());
                let max_items = expand_optional_u32(value.max_items.first());
                let unique_items = !value.unique_items.is_empty();
                let span = value.span;
                quote_spanned! {span=>
                    #runtime::AttributeMetadata::Sequence(#runtime::SequenceConstraint::new(
                        #min_items,
                        #max_items,
                        #unique_items,
                    ))
                }
            }
            FieldAttributeIr::Map(value) => {
                let min_entries = expand_optional_u32(value.min_entries.first());
                let max_entries = expand_optional_u32(value.max_entries.first());
                let span = value.span;
                quote_spanned! {span=>
                    #runtime::AttributeMetadata::Map(#runtime::MapConstraint::new(
                        #min_entries,
                        #max_entries,
                    ))
                }
            }
            FieldAttributeIr::Temporal(value) => expand_temporal(value, runtime),
            FieldAttributeIr::Decimal(value) => expand_decimal(value, runtime),
            FieldAttributeIr::Reference(value) => expand_reference(value, runtime),
            FieldAttributeIr::Sensitive(value) => expand_sensitive(value, runtime),
            FieldAttributeIr::Codec(value) => expand_strategy(value, runtime, true),
            FieldAttributeIr::Generator(value) => expand_strategy(value, runtime, false),
        })
        .collect()
}

/// Generates one text constraint attribute.
fn expand_text(
    value: &crate::attribute::TextAttribute,
    runtime: &TokenStream,
) -> TokenStream {
    let min_chars = expand_optional_u32(value.min_chars.first());
    let max_chars = expand_optional_u32(value.max_chars.first());
    let min_bytes = expand_optional_u32(value.min_bytes.first());
    let max_bytes = expand_optional_u32(value.max_bytes.first());
    let non_blank = !value.non_blank.is_empty();
    let repertoire = value.repertoire.first();
    let repertoire_value = match repertoire.map(|occurrence| occurrence.value) {
        None | Some(TextRepertoire::Unicode) => {
            quote!(#runtime::TextRepertoire::Unicode)
        }
        Some(TextRepertoire::Ascii) => quote!(#runtime::TextRepertoire::Ascii),
    };
    let repertoire = if let Some(repertoire) = repertoire {
        let repertoire_span = repertoire.span;
        quote_spanned!(repertoire_span=> #repertoire_value)
    } else {
        repertoire_value
    };
    let format = match value.format.first() {
        Some(format) => {
            let format_value = match format.value {
                TextFormat::Email => quote!(#runtime::TextFormat::Email),
                TextFormat::Uri => quote!(#runtime::TextFormat::Uri),
                TextFormat::Uuid => quote!(#runtime::TextFormat::Uuid),
            };
            let span = format.span;
            quote_spanned!(span=> Some(#format_value))
        }
        None => quote!(None),
    };
    let span = value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Text(#runtime::TextConstraint::new(
            #min_chars,
            #max_chars,
            #min_bytes,
            #max_bytes,
            #repertoire,
            #non_blank,
            #format,
        ))
    }
}

/// Generates one temporal constraint attribute.
fn expand_temporal(
    value: &crate::attribute::TemporalAttribute,
    runtime: &TokenStream,
) -> TokenStream {
    let precision = value.precision.first();
    let precision_value = match precision.map(|occurrence| occurrence.value) {
        None | Some(TemporalPrecision::Second) => {
            quote!(#runtime::TemporalPrecision::Second)
        }
        Some(TemporalPrecision::Millisecond) => {
            quote!(#runtime::TemporalPrecision::Millisecond)
        }
        Some(TemporalPrecision::Microsecond) => {
            quote!(#runtime::TemporalPrecision::Microsecond)
        }
        Some(TemporalPrecision::Nanosecond) => {
            quote!(#runtime::TemporalPrecision::Nanosecond)
        }
    };
    let precision = if let Some(precision) = precision {
        let precision_span = precision.span;
        quote_spanned!(precision_span=> #precision_value)
    } else {
        precision_value
    };
    let normalization = value.normalization.first();
    let normalization_value =
        match normalization.map(|occurrence| occurrence.value) {
            None | Some(TemporalNormalization::Preserve) => {
                quote!(#runtime::TemporalNormalization::Preserve)
            }
            Some(TemporalNormalization::Utc) => {
                quote!(#runtime::TemporalNormalization::Utc)
            }
        };
    let normalization = if let Some(normalization) = normalization {
        let normalization_span = normalization.span;
        quote_spanned!(normalization_span=> #normalization_value)
    } else {
        normalization_value
    };
    let span = value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Temporal(#runtime::TemporalConstraint::new(
            #precision,
            #normalization,
        ))
    }
}

/// Generates one normalized decimal constraint attribute.
fn expand_decimal(
    value: &crate::normalize::DecimalIr,
    runtime: &TokenStream,
) -> TokenStream {
    let precision = expand_optional_u16(value.value.precision.first());
    let scale = expand_u16_or_default(value.value.scale.first());
    let rounding = value.value.rounding.first();
    let rounding_value = match rounding.map(|occurrence| occurrence.value) {
        Some(RoundingMode::Down) => quote!(#runtime::RoundingMode::Down),
        Some(RoundingMode::Up) => quote!(#runtime::RoundingMode::Up),
        Some(RoundingMode::HalfUp) => quote!(#runtime::RoundingMode::HalfUp),
        None | Some(RoundingMode::HalfEven) => {
            quote!(#runtime::RoundingMode::HalfEven)
        }
    };
    let rounding = if let Some(rounding) = rounding {
        let rounding_span = rounding.span;
        quote_spanned!(rounding_span=> #rounding_value)
    } else {
        rounding_value
    };
    let semantic = match value.semantic {
        DecimalSemantic::Number => quote!(#runtime::DecimalSemantic::Number),
        DecimalSemantic::Money => quote!(#runtime::DecimalSemantic::Money),
    };
    let span = value.value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Decimal(#runtime::DecimalConstraint::new(
            #precision,
            #scale,
            #rounding,
            #semantic,
        ))
    }
}

/// Generates an `Option<u32>` expression without relying on `Option<T>` token
/// flattening.
fn expand_optional_u32(
    value: Option<&crate::attribute::SpannedValue<u32>>,
) -> TokenStream {
    match value {
        Some(value) => {
            let number = value.value;
            let span = value.span;
            quote_spanned!(span=> Some(#number))
        }
        None => quote!(None),
    }
}

/// Generates an `Option<u16>` expression without relying on `Option<T>` token
/// flattening.
fn expand_optional_u16(
    value: Option<&crate::attribute::SpannedValue<u16>>,
) -> TokenStream {
    match value {
        Some(value) => {
            let number = value.value;
            let span = value.span;
            quote_spanned!(span=> Some(#number))
        }
        None => quote!(None),
    }
}

/// Generates a required `u16` expression, using zero for syntax deferred to
/// validation.
fn expand_u16_or_default(
    value: Option<&crate::attribute::SpannedValue<u16>>,
) -> TokenStream {
    match value {
        Some(value) => {
            let number = value.value;
            let span = value.span;
            quote_spanned!(span=> #number)
        }
        None => quote!(0_u16),
    }
}

/// Generates one direct-reference attribute and its static field paths.
fn expand_reference(
    value: &crate::attribute::ReferenceAttribute,
    runtime: &TokenStream,
) -> TokenStream {
    let target = value
        .target
        .first()
        .expect("reference parser requires a target");
    let target_field = value
        .target_field
        .first()
        .expect("reference parser requires a target field")
        .iter()
        .map(|field| LitStr::new(&field.name, field.span));
    let must_exist = if let Some(must_exist) = value.must_exist.first() {
        let must_exist_value = must_exist.value;
        let must_exist_span = must_exist.span;
        quote_spanned!(must_exist_span=> #must_exist_value)
    } else {
        quote!(true)
    };
    let same_as = match value.same_as.first() {
        Some(fields) => {
            let fields = fields
                .iter()
                .map(|field| LitStr::new(&field.name, field.span));
            quote!(Some(#runtime::FieldPath::new(&[#(#fields),*])))
        }
        None => quote!(None),
    };
    let span = value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Reference(#runtime::ReferenceMetadata::new(
            #runtime::NamedTypeRef::of::<#target>(),
            #runtime::FieldPath::new(&[#(#target_field),*]),
            #must_exist,
            #same_as,
        ))
    }
}

/// Generates one sensitive-data handling attribute.
fn expand_sensitive(
    value: &crate::attribute::SensitiveAttribute,
    runtime: &TokenStream,
) -> TokenStream {
    let handling = value.handling.first();
    let handling_value = match handling.map(|occurrence| occurrence.value) {
        None | Some(SensitiveHandling::Redact) => {
            quote!(#runtime::SensitiveHandling::Redact)
        }
        Some(SensitiveHandling::Mask) => {
            quote!(#runtime::SensitiveHandling::Mask)
        }
    };
    let handling = if let Some(handling) = handling {
        let handling_span = handling.span;
        quote_spanned!(handling_span=> #handling_value)
    } else {
        handling_value
    };
    let span = value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Sensitive(#runtime::SensitiveMetadata::new(#handling))
    }
}

/// Generates one codec or generator strategy attribute.
fn expand_strategy(
    value: &crate::attribute::StrategyAttribute,
    runtime: &TokenStream,
    codec: bool,
) -> TokenStream {
    let name = value.name.first().expect("strategy parser requires a name");
    let span = value.span;
    if codec {
        quote_spanned! {span=>
            #runtime::AttributeMetadata::Codec(#runtime::StrategyRef::new(#name))
        }
    } else {
        quote_spanned! {span=>
            #runtime::AttributeMetadata::Generator(#runtime::StrategyRef::new(#name))
        }
    }
}

/// Generates enum-variant metadata values in declaration order.
fn expand_variants(
    variants: &[ModelVariant],
    runtime: &TokenStream,
) -> Vec<TokenStream> {
    variants
        .iter()
        .map(|variant| {
            let ordinal = variant.ordinal;
            let name = LitStr::new(&variant.name, Span::call_site());
            quote!(#runtime::EnumVariantMetadata::new(#ordinal, #name))
        })
        .collect()
}
