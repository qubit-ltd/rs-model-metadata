// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
//! Token generation for normalized static runtime model metadata.

use std::slice::from_ref;

use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::GenericArgument;
use syn::Ident;
use syn::LitStr;
use syn::PathArguments;
use syn::PathSegment;
use syn::Type;
use syn::TypePath;

use crate::attribute::AllowedChars;
use crate::attribute::LookupRelationAttribute;
use crate::attribute::ReferenceAttribute;
use crate::attribute::ReferencePathSegment;
use crate::attribute::RoundingMode;
use crate::attribute::SequenceAttribute;
use crate::attribute::SpannedValue;
use crate::attribute::StrategyAttribute;
use crate::attribute::TemporalAttribute;
use crate::attribute::TemporalPrecision;
use crate::attribute::TextAttribute;
use crate::attribute::TextFormat;
use crate::normalize::DecimalIr;
use crate::normalize::DecimalSemantic;
use crate::normalize::ElementConstraintIr;
use crate::normalize::ElementIr;
use crate::normalize::FieldAttributeIr;
use crate::normalize::FieldIr;
use crate::normalize::ModelAttributeIr;
use crate::normalize::ModelIr;
use crate::normalize::ModelShapeIr;
use crate::normalize::ModelVariantIr;
use crate::normalize::ModelVariantShapeIr;
use crate::normalize::NamedFieldsIr;
use crate::normalize::PrimaryKeyIr;
use crate::normalize::UniqueIr;

/// Generates static metadata and trait implementations for one normalized
/// model.
///
/// Generated items refer only to public runtime metadata APIs. All expansion
/// preconditions are checked before this function is called.
///
/// # Parameters
///
/// - `input`: The validated normalized model to expand.
/// - `runtime`: The resolved runtime crate path used by generated tokens.
///
/// # Returns
///
/// Returns generated Rust tokens for the validated normalized model.
#[must_use]
pub(crate) fn expand(input: &ModelIr, runtime: &TokenStream) -> TokenStream {
    let ident = &input.ident;
    let id = input.id.first().expect("validated model input requires one model ID");
    let kind = expand_type_kind(&input.shape, &input.attributes, ident, id, runtime);
    let registration = expand_registration(ident, id, runtime);
    let capabilities = match &input.shape {
        ModelShapeIr::Newtype(field) if field.opaque.is_empty() => {
            let ty = &field.ty;
            quote!(<#ty as #runtime::HasTypeShape>::CAPABILITIES)
        }
        ModelShapeIr::NamedStruct(_) if input.textual.is_some() => {
            quote!(#runtime::TypeCapabilities::TEXT)
        }
        ModelShapeIr::NamedStruct(_) | ModelShapeIr::UnitStruct | ModelShapeIr::Newtype(_) | ModelShapeIr::Enum(_) => {
            quote!(#runtime::TypeCapabilities::NONE)
        }
    };

    quote! {
        const _: () = {
            #kind
            #registration

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

            impl #runtime::HasModelRegistration for #ident {
                fn model_registration() -> &'static #runtime::ModelRegistration {
                    &MODEL_REGISTRATION
                }
            }
        };
    }
}

/// Generates the distributed registration for one statically derived model.
fn expand_registration(ident: &Ident, id: &LitStr, runtime: &TokenStream) -> TokenStream {
    quote! {
        #[#runtime::__private::linkme::distributed_slice(
            #runtime::MODEL_REGISTRATIONS
        )]
        #[linkme(crate = #runtime::__private::linkme)]
        static MODEL_REGISTRATION: #runtime::ModelRegistration =
            #runtime::ModelRegistration::new(
                #runtime::ModelId::new(#id),
                &MODEL_METADATA,
                stringify!(#ident),
                module_path!(),
                #runtime::SourceLocation::new(file!(), line!(), column!()),
            );
    }
}

/// Generates type-system diagnostics that remain meaningful even when local
/// semantic validation has already rejected the model.
#[must_use]
pub(crate) fn expand_independent_diagnostics(input: &ModelIr, runtime: &TokenStream) -> TokenStream {
    let unique_assertions = expand_unique_capability_assertions(&input.shape, &input.attributes, runtime);
    let field_assertions = all_fields(&input.shape)
        .into_iter()
        .flat_map(|field| expand_capability_assertions(field, runtime))
        .collect::<Vec<_>>();
    quote!(#(#unique_assertions)* #(#field_assertions)*)
}

/// Returns every field-bearing declaration member for independent type
/// assertions.
///
/// # Parameters
///
/// - `shape`: The normalized model shape to inspect.
///
/// # Returns
///
/// Returns borrowed fields across structs, newtypes, and enum variants.
fn all_fields(shape: &ModelShapeIr) -> Vec<&FieldIr> {
    match shape {
        ModelShapeIr::NamedStruct(fields) => fields.iter().collect(),
        ModelShapeIr::Newtype(field) => vec![field.as_ref()],
        ModelShapeIr::Enum(variants) => variants
            .iter()
            .flat_map(|variant| match &variant.shape {
                ModelVariantShapeIr::Unit => [].as_slice().iter(),
                ModelVariantShapeIr::Tuple(fields) | ModelVariantShapeIr::Struct(fields) => fields.iter(),
            })
            .collect(),
        ModelShapeIr::UnitStruct => Vec::new(),
    }
}

/// Returns every field-bearing supported shape as a slice.
#[must_use]
#[inline(always)]
fn model_fields(shape: &ModelShapeIr) -> &[FieldIr] {
    match shape {
        ModelShapeIr::NamedStruct(fields) => fields,
        ModelShapeIr::Newtype(field) => from_ref(field.as_ref()),
        ModelShapeIr::UnitStruct | ModelShapeIr::Enum(_) => &[],
    }
}

/// Generates the static structural metadata for the normalized model shape.
fn expand_type_kind(
    shape: &ModelShapeIr,
    attributes: &[ModelAttributeIr],
    ident: &Ident,
    id: &LitStr,
    runtime: &TokenStream,
) -> TokenStream {
    let unique_capability_assertions = expand_unique_capability_assertions(shape, attributes, runtime);
    let attributes = expand_model_attributes(attributes, runtime);
    match shape {
        ModelShapeIr::NamedStruct(fields) => {
            let fields = expand_fields(fields, runtime);
            let count = fields.len();
            quote! {
                #(#unique_capability_assertions)*
                static FIELDS: [#runtime::FieldMetadata; #count] = [#(#fields),*];
                static MODEL_METADATA: #runtime::TypeMetadata = #runtime::TypeMetadata::new(
                    #runtime::ModelId::new(#id),
                    #runtime::TypeIdentity::of::<#ident>(),
                    #runtime::TypeKind::Struct(#runtime::StructMetadata::new(&FIELDS)),
                    &[#(#attributes),*],
                );
            }
        }
        ModelShapeIr::UnitStruct => quote! {
            #(#unique_capability_assertions)*
            static MODEL_METADATA: #runtime::TypeMetadata = #runtime::TypeMetadata::new(
                #runtime::ModelId::new(#id),
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
                    #runtime::ModelId::new(#id),
                    #runtime::TypeIdentity::of::<#ident>(),
                    #runtime::TypeKind::Newtype(#runtime::NewtypeMetadata::new(FIELDS[0])),
                    &[#(#attributes),*],
                );
            }
        }
        ModelShapeIr::Enum(variants) => {
            let (field_statics, variants) = expand_variants(variants, runtime);
            let count = variants.len();
            quote! {
                #(#unique_capability_assertions)*
                #(#field_statics)*
                static VARIANTS: [#runtime::EnumVariantMetadata; #count] = [#(#variants),*];
                static MODEL_METADATA: #runtime::TypeMetadata = #runtime::TypeMetadata::new(
                    #runtime::ModelId::new(#id),
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
            let field = fields.iter().find(|field| field.name == reference.name)?;
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
fn expand_model_attributes(attributes: &[ModelAttributeIr], runtime: &TokenStream) -> Vec<TokenStream> {
    attributes
        .iter()
        .map(|attribute| match attribute {
            ModelAttributeIr::PrimaryKey(primary_key) => expand_primary_key(primary_key, runtime),
            ModelAttributeIr::Unique(unique) => expand_unique(unique, runtime),
            ModelAttributeIr::Index(index) => expand_named_fields(index, runtime, true),
            ModelAttributeIr::Key(key) => expand_named_fields(key, runtime, false),
            ModelAttributeIr::Ownership(ownership) => {
                let owner = ownership.owner.first().expect("ownership parser requires an owner");
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
fn expand_primary_key(primary_key: &PrimaryKeyIr, runtime: &TokenStream) -> TokenStream {
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
        let ignore_case = unique.ignore_case.iter().any(|candidate| candidate.name == field.name);
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
fn expand_named_fields(value: &NamedFieldsIr, runtime: &TokenStream, index: bool) -> TokenStream {
    let name = expand_optional_name(value.name.first());
    let fields = value.fields.iter().map(|(name, span)| LitStr::new(name, *span));
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
fn expand_fields(fields: &[FieldIr], runtime: &TokenStream) -> Vec<TokenStream> {
    fields.iter().map(|field| expand_field(field, runtime)).collect()
}

/// Generates one static field metadata value and its canonical attributes.
fn expand_field(field: &FieldIr, runtime: &TokenStream) -> TokenStream {
    let ordinal = field.ordinal;
    let name = LitStr::new(&field.name, Span::call_site());
    let ty = &field.ty;
    let attributes = expand_field_attributes(&field.attributes, runtime);
    let capability_assertions = expand_capability_assertions(field, runtime);
    let field_type = expand_field_type(field, runtime);

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

/// Returns whether a field declares a direct reference constraint.
fn field_declares_reference(field: &FieldIr) -> bool {
    field
        .attributes
        .iter()
        .any(|attribute| matches!(attribute, FieldAttributeIr::Reference(_)))
}

/// Generates the runtime `TypeRef` expression for one field.
fn expand_field_type(field: &FieldIr, runtime: &TokenStream) -> TokenStream {
    let ty = &field.ty;
    if let Some(span) = field.opaque.first().copied() {
        let opaque_shape = expand_opaque_shape(ty, runtime);
        return quote_spanned!(span=> #runtime::TypeRef::opaque_with_shape::<#ty>(|| #opaque_shape));
    }
    if field_declares_reference(field) {
        return expand_reference_field_type(field, runtime);
    }
    quote!(#runtime::TypeRef::of::<#ty>())
}

/// Generates a `TypeRef` for a reference field without an explicit `opaque`.
///
/// Reference fields whose Rust type implements [`HasTypeShape`] keep their
/// structural metadata. Foreign-key scalars and other opaque leaves fall back
/// to an opaque reference that preserves visible container syntax.
fn expand_reference_field_type(field: &FieldIr, runtime: &TokenStream) -> TokenStream {
    let ty = &field.ty;
    if reference_field_type_requires_opaque(ty) {
        let opaque_shape = expand_opaque_shape(ty, runtime);
        quote!(#runtime::TypeRef::opaque_with_shape::<#ty>(|| #opaque_shape))
    } else {
        quote!(#runtime::TypeRef::of::<#ty>())
    }
}

/// Returns whether a reference field type should use an opaque `TypeRef`.
fn reference_field_type_requires_opaque(ty: &Type) -> bool {
    match unwrap_type(ty) {
        Type::Path(path) => reference_path_type_requires_opaque(path),
        Type::Array(array) => reference_field_type_requires_opaque(&array.elem),
        Type::Slice(slice) => reference_field_type_requires_opaque(&slice.elem),
        Type::Tuple(tuple) => tuple.elems.iter().any(reference_field_type_requires_opaque),
        Type::Reference(reference) => reference_field_type_requires_opaque(&reference.elem),
        _ => true,
    }
}

/// Returns whether a path type used by a reference field should be opaque.
fn reference_path_type_requires_opaque(path: &TypePath) -> bool {
    let Some(segment) = path.path.segments.last() else {
        return true;
    };
    let name = segment.ident.to_string();
    if matches!(
        name.as_str(),
        "Id" | "Path" | "PathBuf" | "OsString" | "CString" | "CStr" | "OsStr"
    ) {
        return true;
    }
    if is_builtin_has_type_shape_type(name.as_str()) {
        return false;
    }
    let arguments = type_path_arguments(segment);
    match name.as_str() {
        "Option" | "Vec" | "HashSet" | "BTreeSet" | "LinkedList" | "VecDeque" | "BinaryHeap" => arguments
            .iter()
            .any(|argument| reference_field_type_requires_opaque(argument)),
        "HashMap" | "BTreeMap" => arguments
            .iter()
            .any(|argument| reference_field_type_requires_opaque(argument)),
        _ => {
            if path.path.segments.len() > 1 {
                let crate_root = path.path.segments.first().map(|segment| segment.ident.to_string());
                matches!(crate_root.as_deref(), Some("std" | "core" | "alloc" | "os" | "ffi"))
            } else {
                false
            }
        }
    }
}

/// Returns whether a path segment names a type with a built-in [`HasTypeShape`]
/// implementation.
fn is_builtin_has_type_shape_type(name: &str) -> bool {
    matches!(
        name,
        "bool"
            | "char"
            | "str"
            | "String"
            | "i8"
            | "i16"
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
            | "f32"
            | "f64"
            | "NaiveDate"
            | "NaiveTime"
            | "NaiveDateTime"
            | "DateTime"
            | "BigDecimal"
    )
}

/// Returns the type arguments from the final segment of a path type.
fn type_path_arguments(segment: &PathSegment) -> Vec<&Type> {
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Vec::new();
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

/// Strips grouping and parenthesis wrappers from a field type.
fn unwrap_type(ty: &Type) -> &Type {
    match ty {
        Type::Group(group) => unwrap_type(&group.elem),
        Type::Paren(paren) => unwrap_type(&paren.elem),
        other => other,
    }
}

/// Generates the visible container structure for an opaque field type.
///
/// # Parameters
///
/// * `ty` - The source-level field type whose opaque leaves are retained.
/// * `runtime` - The resolved runtime crate path used by generated tokens.
///
/// # Returns
///
/// A `TypeShape` expression that preserves recognized standard containers and
/// uses `TypeRef::opaque` for uninterpreted leaves.
fn expand_opaque_shape(ty: &Type, runtime: &TokenStream) -> TokenStream {
    match ty {
        Type::Array(array) => {
            let element = expand_opaque_type_ref(&array.elem, runtime);
            let length = &array.len;
            quote!(#runtime::TypeShape::Array {
                element: #element,
                length: #length,
            })
        }
        Type::Path(path) => expand_opaque_path_shape(path, runtime),
        _ => quote!(#runtime::TypeShape::Opaque),
    }
}

/// Generates the visible shape for a path type, when its final segment is a
/// supported standard container.
fn expand_opaque_path_shape(path: &TypePath, runtime: &TokenStream) -> TokenStream {
    let Some(segment) = path.path.segments.last() else {
        return quote!(#runtime::TypeShape::Opaque);
    };
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return quote!(#runtime::TypeShape::Opaque);
    };
    let type_arguments = arguments
        .args
        .iter()
        .filter_map(|argument| {
            if let GenericArgument::Type(ty) = argument {
                Some(ty)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    match (segment.ident.to_string().as_str(), type_arguments.as_slice()) {
        ("Option", [inner]) => {
            let inner = expand_opaque_type_ref(inner, runtime);
            quote!(#runtime::TypeShape::Optional(#inner))
        }
        ("Vec", [inner]) => {
            let inner = expand_opaque_type_ref(inner, runtime);
            quote!(#runtime::TypeShape::Sequence(#inner))
        }
        ("HashSet" | "BTreeSet", [inner]) => {
            let inner = expand_opaque_type_ref(inner, runtime);
            quote!(#runtime::TypeShape::Set(#inner))
        }
        ("HashMap" | "BTreeMap", [key, value]) => {
            let key = expand_opaque_type_ref(key, runtime);
            let value = expand_opaque_type_ref(value, runtime);
            quote!(#runtime::TypeShape::Map {
                key: #key,
                value: #value,
            })
        }
        _ => quote!(#runtime::TypeShape::Opaque),
    }
}

/// Generates a TypeRef for an opaque nested container or opaque leaf.
fn expand_opaque_type_ref(ty: &Type, runtime: &TokenStream) -> TokenStream {
    let shape = expand_opaque_shape(ty, runtime);
    quote!(#runtime::TypeRef::opaque_with_shape::<#ty>(|| #shape))
}

/// Generates const assertions for capabilities that must be resolved through
/// Rust's type system.
fn expand_capability_assertions(field: &FieldIr, runtime: &TokenStream) -> Vec<TokenStream> {
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
                FieldAttributeIr::Sequence(value) => expand_sequence_capability_assertion(ty, value, runtime),
                FieldAttributeIr::Map(value) => expand_capability_assertion(
                    ty,
                    value.span,
                    quote!(#runtime::TypeCapabilities::MAP),
                    "`map` requires a map-capable field",
                    runtime,
                ),
                FieldAttributeIr::Temporal(value) => expand_capability_assertion(
                    ty,
                    value.span,
                    quote!(#runtime::TypeCapabilities::TEMPORAL),
                    "`time` requires a temporal-capable field",
                    runtime,
                ),
                FieldAttributeIr::Decimal(value) => {
                    let message = match value.semantic {
                        DecimalSemantic::Number => "`decimal` requires a decimal-capable field",
                        DecimalSemantic::Money => "`money` requires a decimal-capable field",
                    };
                    expand_capability_assertion(
                        ty,
                        value.value.span,
                        quote!(#runtime::TypeCapabilities::DECIMAL),
                        message,
                        runtime,
                    )
                }
                FieldAttributeIr::Element(value) => expand_element_capability_assertion(ty, value, runtime),
                FieldAttributeIr::Reference(_)
                | FieldAttributeIr::LookupRelation(_)
                | FieldAttributeIr::Codec(_)
                | FieldAttributeIr::Generator(_) => return None,
            };
            Some(assertion)
        })
        .collect()
}

/// Generates one const assertion for a required type capability.
fn expand_capability_assertion(
    ty: &Type,
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
fn expand_sequence_capability_assertion(ty: &Type, value: &SequenceAttribute, runtime: &TokenStream) -> TokenStream {
    let span = value.span;
    let unique_items_check = value.unique_items.first().map(|span| {
        quote_spanned! {*span=>
            if capabilities.contains(#runtime::TypeCapabilities::SET) {
                panic!("`unique_items` is redundant for Set fields");
            }
        }
    });
    let repeats_length = !value.min_items.is_empty() || !value.max_items.is_empty();
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

/// Generates capability assertions for a field's element constraints.
fn expand_element_capability_assertion(ty: &Type, value: &ElementIr, runtime: &TokenStream) -> TokenStream {
    let checks = value.attributes.iter().map(|attribute| match attribute {
        ElementConstraintIr::Text(value) => {
            let span = value.span;
            quote_spanned! {span=>
                assert!(
                    capabilities.contains(#runtime::TypeCapabilities::TEXT),
                    "`element(text(...))` requires a text-capable element",
                );
            }
        }
        ElementConstraintIr::Decimal(value) => {
            let span = value.value.span;
            quote_spanned! {span=>
                assert!(
                    capabilities.contains(#runtime::TypeCapabilities::DECIMAL),
                    "`element(decimal(...))` requires a decimal-capable element",
                );
            }
        }
    });
    let span = value.span;
    quote_spanned! {span=>
        const _: () = {
            let capabilities = match <#ty as #runtime::HasTypeShape>::ELEMENT_CAPABILITIES {
                Some(capabilities) => capabilities,
                None => panic!("`element` requires a sequence field"),
            };
            #(#checks)*
        };
    }
}

/// Generates canonical field-level runtime attributes in IR order.
fn expand_field_attributes(attributes: &[FieldAttributeIr], runtime: &TokenStream) -> Vec<TokenStream> {
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
            FieldAttributeIr::Element(value) => expand_element(value, runtime),
            FieldAttributeIr::Reference(value) => expand_reference(value, runtime),
            FieldAttributeIr::LookupRelation(value) => expand_lookup_relation(value, runtime),
            FieldAttributeIr::Codec(value) => expand_strategy(value, runtime, true),
            FieldAttributeIr::Generator(value) => expand_strategy(value, runtime, false),
        })
        .collect()
}

/// Generates one text constraint attribute.
fn expand_text(value: &TextAttribute, runtime: &TokenStream) -> TokenStream {
    let min_chars = expand_optional_u32(value.min_chars.first());
    let max_chars = expand_optional_u32(value.max_chars.first());
    let min_bytes = expand_optional_u32(value.min_bytes.first());
    let max_bytes = expand_optional_u32(value.max_bytes.first());
    let non_blank = !value.non_blank.is_empty();
    let allowed_chars = value.allowed_chars.first();
    let allowed_chars_value = match allowed_chars.map(|occurrence| occurrence.value) {
        None | Some(AllowedChars::Unicode) => {
            quote!(#runtime::AllowedChars::Unicode)
        }
        Some(AllowedChars::Ascii) => quote!(#runtime::AllowedChars::Ascii),
    };
    let allowed_chars = if let Some(allowed_chars) = allowed_chars {
        let allowed_chars_span = allowed_chars.span;
        quote_spanned!(allowed_chars_span=> #allowed_chars_value)
    } else {
        allowed_chars_value
    };
    let format = match value.format.first() {
        Some(format) => {
            let format_value = match format.value {
                TextFormat::Email => quote!(#runtime::TextFormat::Email),
                TextFormat::Mobile => quote!(#runtime::TextFormat::Mobile),
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
            #allowed_chars,
            #non_blank,
            #format,
        ))
    }
}

/// Generates one temporal constraint attribute.
fn expand_temporal(value: &TemporalAttribute, runtime: &TokenStream) -> TokenStream {
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
    let span = value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Temporal(#runtime::TemporalConstraint::new(
            #precision,
        ))
    }
}

/// Generates one normalized decimal constraint attribute.
fn expand_decimal(value: &DecimalIr, runtime: &TokenStream) -> TokenStream {
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

/// Generates nested static metadata for sequence elements.
fn expand_element(value: &ElementIr, runtime: &TokenStream) -> TokenStream {
    let attributes = value.attributes.iter().map(|attribute| match attribute {
        ElementConstraintIr::Text(value) => expand_text(value, runtime),
        ElementConstraintIr::Decimal(value) => expand_decimal(value, runtime),
    });
    let span = value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Element(#runtime::ElementMetadata::new(
            &[#(#attributes),*],
        ))
    }
}

/// Generates an `Option<u32>` expression without relying on `Option<T>` token
/// flattening.
fn expand_optional_u32(value: Option<&SpannedValue<u32>>) -> TokenStream {
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
fn expand_optional_u16(value: Option<&SpannedValue<u16>>) -> TokenStream {
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
fn expand_u16_or_default(value: Option<&SpannedValue<u16>>) -> TokenStream {
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
fn expand_reference(value: &ReferenceAttribute, runtime: &TokenStream) -> TokenStream {
    let entity = value.entity.first().expect("reference parser requires an entity");
    let target = match value.property.first() {
        Some(property) => {
            let fields = property.iter().map(|field| LitStr::new(&field.name, field.span));
            quote!(#runtime::ReferenceTarget::Property(#runtime::FieldPath::new(&[#(#fields),*])))
        }
        None => quote!(#runtime::ReferenceTarget::WholeModel),
    };
    let existing = if let Some(existing) = value.existing.first() {
        let existing_value = existing.value;
        let existing_span = existing.span;
        quote_spanned!(existing_span=> #existing_value)
    } else {
        quote!(true)
    };
    let path = match value.path.first() {
        Some(segments) => {
            let segments = segments.iter().map(|segment| match segment {
                ReferencePathSegment::Parent(span) => {
                    quote_spanned!(*span=> #runtime::ReferencePathSegment::Parent)
                }
                ReferencePathSegment::Field(field) => {
                    let field = LitStr::new(&field.name, field.span);
                    quote!(#runtime::ReferencePathSegment::Field(#field))
                }
            });
            quote!(Some(#runtime::ReferencePath::new(&[#(#segments),*])))
        }
        None => quote!(None),
    };
    let span = value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::Reference(#runtime::ReferenceMetadata::new(
            #runtime::ModelId::new(#entity),
            #target,
            #existing,
            #path,
        ))
    }
}

/// Generates one lookup-relation attribute and its static target field path.
fn expand_lookup_relation(value: &LookupRelationAttribute, runtime: &TokenStream) -> TokenStream {
    let target = value.target.first().expect("lookup_relation parser requires a target");
    let target_field = value
        .target_field
        .first()
        .expect("lookup_relation parser requires a target field")
        .iter()
        .map(|field| LitStr::new(&field.name, field.span));
    let span = value.span;
    quote_spanned! {span=>
        #runtime::AttributeMetadata::LookupRelation(
            #runtime::LookupRelationMetadata::new(
                #runtime::NamedTypeRef::of::<#target>(),
                #runtime::FieldPath::new(&[#(#target_field),*]),
            )
        )
    }
}

/// Generates one codec or generator strategy attribute.
fn expand_strategy(value: &StrategyAttribute, runtime: &TokenStream, codec: bool) -> TokenStream {
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
fn expand_variants(variants: &[ModelVariantIr], runtime: &TokenStream) -> (Vec<TokenStream>, Vec<TokenStream>) {
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
                let expanded_fields = expand_fields(fields, runtime);
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
