// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Generates field metadata, constraints, validators, codecs, and Serde
//! overlays.

use proc_macro2::TokenStream;
use proc_macro2::TokenTree;
use quote::ToTokens;
use quote::format_ident;
use quote::quote;
use quote::quote_spanned;
use syn::Expr;
use syn::GenericArgument;
use syn::Ident;
use syn::Lifetime;
use syn::Lit;
use syn::LitStr;
use syn::PathArguments;
use syn::PathSegment;
use syn::Stmt;
use syn::Type;
use syn::TypeArray;
use syn::TypePath;
use syn::TypePtr;
use syn::TypeReference;
use syn::UnOp;
use syn::Visibility;
use syn::spanned::Spanned;

use crate::ir::declaration::CodecIr;
use crate::ir::declaration::ConstraintIr;
use crate::ir::declaration::FieldIr;
use crate::ir::declaration::FieldOccurrence;
use crate::ir::declaration::IdentifierAssignmentIr;
use crate::ir::declaration::RedactIr;
use crate::ir::declaration::RedactModeIr;
use crate::ir::declaration::ReferenceIr;
use crate::ir::declaration::ReferenceTargetIr;
use crate::ir::declaration::SelectorIr;
use crate::ir::declaration::SelectorPositionIr;
use crate::ir::declaration::SerdeIr;
use crate::ir::declaration::StrategyArgumentIr;
use crate::ir::declaration::ValidatorIr;

/// Converts Rust type syntax to v4 static expression facts.
///
/// Unsupported syntax is emitted as a compile-time diagnostic so metadata
/// never silently loses source type information. Declared generic parameters
/// retain their names in the generated expression facts.
///
/// # Parameters
/// - `ty`: Rust type syntax to represent.
/// - `runtime`: path to the runtime crate.
/// - `generic_type_names`: declared type and const parameter names.
///
/// # Returns
/// Tokens constructing the corresponding expression facts.
#[must_use]
pub(super) fn type_expression_data_with_parameters(
    ty: &Type,
    runtime: &TokenStream,
    generic_type_names: &[Ident],
) -> TokenStream {
    type_expression_data_for_owner(ty, runtime, generic_type_names, None)
}

/// Builds expression facts while applying declaration-owner and generic-name
/// rules.
///
/// Exact owner paths become `SelfType`, while declared type and const
/// parameters retain their names. Unsupported syntax remains represented by a
/// generated compile-time diagnostic.
#[must_use]
fn type_expression_data_for_owner(
    ty: &Type,
    runtime: &TokenStream,
    generic_type_names: &[Ident],
    owner: Option<&Ident>,
) -> TokenStream {
    let expression = quote!(#runtime::__private::codegen_v4::expression::ExpressionData);
    let leak = quote!(#runtime::__private::v8::leak);
    match ty {
        Type::Path(path) => {
            if let Some(qself) = &path.qself {
                let associated = path.path.segments.last().expect("qualified path has a final segment");
                let name = LitStr::new(&associated.ident.to_string(), associated.ident.span());
                let self_type = type_expression_data_for_owner(&qself.ty, runtime, generic_type_names, owner);
                let arguments = trait_generic_arguments(&[associated], runtime, generic_type_names, owner);
                let trait_ = if qself.position > 0 {
                    let trait_segments = path.path.segments.iter().take(qself.position).collect::<Vec<_>>();
                    let mut trait_name = trait_segments
                        .iter()
                        .map(|segment| segment.to_token_stream().to_string())
                        .collect::<Vec<_>>()
                        .join(" :: ");
                    if path.path.leading_colon.is_some() {
                        trait_name.insert_str(0, "::");
                    }
                    let trait_name = LitStr::new(&trait_name, path.path.span());
                    let arguments = trait_generic_arguments(&trait_segments, runtime, generic_type_names, owner);
                    let lifetimes = trait_generic_lifetimes(&trait_segments, runtime);
                    let data = quote!(#runtime::__private::codegen_v4::expression::TraitExpressionData::new(
                        #trait_name,
                        #runtime::__private::v8::leak_slice(::std::vec![#(#arguments),*]),
                        #runtime::__private::v8::leak_slice(::std::vec![#(#lifetimes),*]),
                    ));
                    quote!(Some(#leak(#data)))
                } else {
                    quote!(None)
                };
                return quote!(#leak(#expression::AssociatedType {
                    self_type: #self_type,
                    trait_: #trait_,
                    name: #name,
                    arguments: #runtime::__private::v8::leak_slice(::std::vec![#(#arguments),*]),
                }));
            }
            if path.path.segments.len() == 1 && path.path.segments[0].ident == "Self" {
                return quote!(#leak(#expression::SelfType));
            }
            if path.path.segments.len() == 1
                && owner.is_some_and(|owner| path.path.segments[0].ident == *owner)
                && path.path.segments[0].arguments.is_empty()
            {
                return quote!(#leak(#expression::SelfType));
            }
            if path.path.segments.len() == 1
                && path.path.segments[0].arguments.is_empty()
                && generic_type_names
                    .iter()
                    .any(|name| name == &path.path.segments[0].ident)
            {
                let name = LitStr::new(
                    &path.path.segments[0].ident.to_string(),
                    path.path.segments[0].ident.span(),
                );
                return quote!(#leak(#expression::ParameterName(#name)));
            }
            let first_segment = path.path.segments.first().expect("type path has at least one segment");
            let is_projection = path.path.segments.len() > 1
                && (first_segment.ident == "Self"
                    || generic_type_names.iter().any(|name| name == &first_segment.ident))
                && first_segment.arguments.is_empty();
            if is_projection {
                let associated = path.path.segments.last().expect("projection has a final segment");
                let name = LitStr::new(&associated.ident.to_string(), associated.ident.span());
                let mut self_path = path.path.clone();
                self_path.segments.pop();
                let self_type = type_expression_data_for_owner(
                    &Type::Path(TypePath {
                        attrs: Vec::new(),
                        qself: None,
                        path: self_path,
                    }),
                    runtime,
                    generic_type_names,
                    owner,
                );
                let arguments = trait_generic_arguments(&[associated], runtime, generic_type_names, owner);
                return quote!(#leak(#expression::AssociatedType {
                    self_type: #self_type,
                    trait_: None,
                    name: #name,
                    arguments: #runtime::__private::v8::leak_slice(::std::vec![#(#arguments),*]),
                }));
            }
            let mut path_name = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            if path.path.leading_colon.is_some() {
                path_name.insert_str(0, "::");
            }
            let name = LitStr::new(&path_name, path.path.span());
            if path
                .path
                .segments
                .iter()
                .any(|segment| matches!(segment.arguments, PathArguments::Parenthesized(_)))
            {
                return quote!(compile_error!(
                    "parenthesized path arguments are unsupported in model metadata expressions"
                ));
            }
            let values = path
                .path
                .segments
                .iter()
                .flat_map(|segment| match &segment.arguments {
                    PathArguments::AngleBracketed(args) => args.args.iter().collect::<Vec<_>>(),
                    PathArguments::None | PathArguments::Parenthesized(_) => Vec::new(),
                })
                .map(|arg| match arg {
                    GenericArgument::Type(inner) => {
                        let inner = type_expression_data_for_owner(inner, runtime, generic_type_names, owner);
                        quote!(#runtime::__private::codegen_v4::expression::GenericExpressionArgument::Type(#inner))
                    }
                    GenericArgument::Lifetime(lifetime) => {
                        let value = lifetime_expression(lifetime, runtime);
                        quote!(#runtime::__private::codegen_v4::expression::GenericExpressionArgument::Lifetime(#value))
                    }
                    GenericArgument::Const(value) => {
                        let value = const_expression(value, runtime, generic_type_names);
                        quote!(#runtime::__private::codegen_v4::expression::GenericExpressionArgument::Const(
                            #value
                        ))
                    }
                    _ => quote!(compile_error!(
                        "unsupported generic argument in model metadata expression"
                    )),
                })
                .collect::<Vec<_>>();
            let args = quote!(#runtime::__private::v8::leak_slice(::std::vec![#(#values),*]));
            quote!(#leak(#expression::Named { path: #name, definition: None, arguments: #args }))
        }
        Type::Reference(TypeReference {
            lifetime,
            mutability,
            elem,
            ..
        }) => {
            let target = type_expression_data_for_owner(elem, runtime, generic_type_names, owner);
            let lifetime = lifetime.as_ref().map_or_else(
                || quote!(None),
                |value| {
                    let expression = lifetime_expression(value, runtime);
                    quote!(Some(#expression))
                },
            );
            let mutable = mutability.is_some();
            quote!(#leak(#expression::Reference { lifetime: #lifetime, mutable: #mutable, target: #target }))
        }
        Type::Ptr(TypePtr { mutability, elem, .. }) => {
            let target = type_expression_data_for_owner(elem, runtime, generic_type_names, owner);
            let mutable = matches!(mutability, syn::PointerMutability::Mut(_));
            quote!(#leak(#expression::RawPointer { mutable: #mutable, target: #target }))
        }
        Type::Slice(slice) => {
            let element = type_expression_data_for_owner(&slice.elem, runtime, generic_type_names, owner);
            quote!(#leak(#expression::Slice(#element)))
        }
        Type::Array(TypeArray { elem, len, .. }) => {
            let element = type_expression_data_for_owner(elem, runtime, generic_type_names, owner);
            let length = const_expression(len, runtime, generic_type_names);
            quote!(#leak(#expression::Array { element: #element, length: #length }))
        }
        Type::Tuple(tuple) => {
            let values = tuple
                .elems
                .iter()
                .map(|item| type_expression_data_for_owner(item, runtime, generic_type_names, owner));
            let values = quote!(#runtime::__private::v8::leak_slice(::std::vec![#(#values),*]));
            quote!(#leak(#expression::Tuple(#values)))
        }
        Type::Never(_) => quote!(#leak(#expression::Never)),
        Type::Paren(paren) => type_expression_data_for_owner(&paren.elem, runtime, generic_type_names, owner),
        Type::Group(group) => type_expression_data_for_owner(&group.elem, runtime, generic_type_names, owner),
        _ => {
            let message = LitStr::new("unsupported Rust type syntax in model metadata expression", ty.span());
            quote!(compile_error!(#message); #leak(#expression::Opaque(&#runtime::__private::codegen_v4::expression::OpaqueTypeData::new(&[]))))
        }
    }
}

/// Converts a Rust type into expression tokens for the declaration owner.
#[must_use]
fn type_expression_for_owner(
    ty: &Type,
    runtime: &TokenStream,
    generic_type_names: &[Ident],
    owner: Option<&Ident>,
) -> TokenStream {
    type_expression_data_for_owner(ty, runtime, generic_type_names, owner)
}

/// Converts generic arguments from associated-type trait path segments.
#[must_use]
fn trait_generic_arguments(
    segments: &[&PathSegment],
    runtime: &TokenStream,
    generic_type_names: &[Ident],
    owner: Option<&Ident>,
) -> Vec<TokenStream> {
    segments
        .iter()
        .flat_map(|segment| match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments.args.iter().collect::<Vec<_>>(),
            PathArguments::None | PathArguments::Parenthesized(_) => Vec::new(),
        })
        .map(|argument| match argument {
            GenericArgument::Type(ty) => {
                let ty = type_expression_data_for_owner(ty, runtime, generic_type_names, owner);
                quote!(#runtime::__private::codegen_v4::expression::GenericExpressionArgument::Type(#ty))
            }
            GenericArgument::Lifetime(lifetime) => {
                let lifetime = lifetime_expression(lifetime, runtime);
                quote!(#runtime::__private::codegen_v4::expression::GenericExpressionArgument::Lifetime(#lifetime))
            }
            GenericArgument::Const(value) => {
                let value = const_expression(value, runtime, generic_type_names);
                quote!(#runtime::__private::codegen_v4::expression::GenericExpressionArgument::Const(#value))
            }
            _ => quote!(compile_error!(
                "unsupported trait generic argument in associated type expression"
            )),
        })
        .collect()
}

/// Collects lifetime arguments from associated-type trait path segments.
#[must_use]
fn trait_generic_lifetimes(segments: &[&PathSegment], runtime: &TokenStream) -> Vec<TokenStream> {
    segments
        .iter()
        .flat_map(|segment| match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments.args.iter().collect::<Vec<_>>(),
            PathArguments::None | PathArguments::Parenthesized(_) => Vec::new(),
        })
        .filter_map(|argument| match argument {
            GenericArgument::Lifetime(lifetime) => Some(lifetime_expression(lifetime, runtime)),
            _ => None,
        })
        .collect()
}

/// Converts a Rust lifetime into its runtime expression representation.
#[must_use]
fn lifetime_expression(lifetime: &Lifetime, runtime: &TokenStream) -> TokenStream {
    match lifetime.ident.to_string().as_str() {
        "static" => quote!(#runtime::__private::codegen_v4::expression::LifetimeExpression::Static),
        "_" => quote!(#runtime::__private::codegen_v4::expression::LifetimeExpression::Placeholder),
        name => {
            let name = LitStr::new(name, lifetime.span());
            quote!(#runtime::__private::codegen_v4::expression::LifetimeExpression::Named(#name))
        }
    }
}

/// Converts a Rust const expression into a supported runtime expression fact.
#[must_use]
fn const_expression(expression: &Expr, runtime: &TokenStream, generic_type_names: &[Ident]) -> TokenStream {
    let path = |value: &Expr| {
        let text = LitStr::new(&value.to_token_stream().to_string(), value.span());
        quote!(#runtime::__private::codegen_v4::expression::ConstExpression::Path(#text))
    };
    let signed =
        |value: i128| quote!(#runtime::__private::codegen_v4::expression::ConstExpression::SignedInteger(#value));
    let unsigned =
        |value: u128| quote!(#runtime::__private::codegen_v4::expression::ConstExpression::UnsignedInteger(#value));
    match expression {
        Expr::Block(value) if value.block.stmts.len() == 1 => {
            if let Stmt::Expr(inner, None) = &value.block.stmts[0] {
                return const_expression(inner, runtime, generic_type_names);
            }
            path(expression)
        }
        Expr::Group(value) => const_expression(&value.expr, runtime, generic_type_names),
        Expr::Paren(value) => const_expression(&value.expr, runtime, generic_type_names),
        Expr::Lit(value) => match &value.lit {
            Lit::Int(value) => value
                .base10_parse::<u128>()
                .map(unsigned)
                .unwrap_or_else(|_| path(expression)),
            Lit::Bool(value) => {
                let value = value.value;
                quote!(#runtime::__private::codegen_v4::expression::ConstExpression::Boolean(#value))
            }
            Lit::Char(value) => {
                let value = value.value();
                quote!(#runtime::__private::codegen_v4::expression::ConstExpression::Character(#value))
            }
            _ => path(expression),
        },
        Expr::Unary(value) if matches!(value.op, UnOp::Neg(_)) => {
            if let Expr::Lit(literal) = value.expr.as_ref()
                && let Lit::Int(integer) = &literal.lit
                && let Ok(number) = integer.base10_parse::<i128>()
            {
                return signed(-number);
            }
            path(expression)
        }
        Expr::Path(value)
            if value.path.segments.len() == 1
                && generic_type_names
                    .iter()
                    .any(|name| name == &value.path.segments[0].ident) =>
        {
            let name = LitStr::new(&value.path.segments[0].ident.to_string(), value.path.span());
            quote!(#runtime::__private::codegen_v4::expression::ConstExpression::Parameter(#name))
        }
        _ => path(expression),
    }
}

/// Generates the runtime field vector for a declaration.
///
/// # Parameters
/// - `fields`: normalized fields to expand.
/// - `runtime`: path to the runtime crate.
/// - `owner`: declaration type name used to identify recursive fields.
///
/// # Returns
/// Tokens that build the declaration's runtime field vector.
#[must_use]
pub(super) fn expand_field_vector(fields: &[FieldIr], runtime: &TokenStream, owner: &Ident) -> TokenStream {
    let bodies = fields
        .iter()
        .map(|field| expand_field_with_parameters(field, runtime, None, None, &[], Some(owner)));
    quote! {
        let mut fields = ::std::vec::Vec::new();
        #(#bodies)*
    }
}

/// Generates field overlays for a source-level generic declaration.
///
/// # Parameters
/// - `fields`: normalized fields to expand.
/// - `definition_path`: tokens naming the generic declaration.
/// - `runtime`: path to the runtime crate.
/// - `variant_inherited`: whether fields inherit variant-level metadata.
/// - `generic_type_names`: declared type and const parameter names.
/// - `owner`: declaration type name used to identify recursive fields.
///
/// # Returns
/// Tokens that build the generic declaration's runtime field vector.
#[must_use]
pub(super) fn expand_generic_field_vector(
    fields: &[FieldIr],
    definition_path: TokenStream,
    runtime: &TokenStream,
    variant_inherited: bool,
    generic_type_names: &[Ident],
    owner: &Ident,
) -> TokenStream {
    let bodies = fields.iter().map(|field| {
        expand_field_with_parameters(
            field,
            runtime,
            Some(variant_inherited),
            Some(&definition_path),
            generic_type_names,
            Some(owner),
        )
    });
    quote! {
        let mut fields = ::std::vec::Vec::new();
        #(#bodies)*
    }
}

/// Generates one field descriptor and its normalized attributes.
///
/// # Parameters
/// - `field`: normalized field metadata.
/// - `runtime`: path to the runtime crate.
/// - `generic_variant_inherited`: generic variant inheritance state, when
///   expanding a generic declaration.
///
/// # Returns
/// Tokens that construct the field metadata and append it to the field vector.
#[must_use]
#[cfg(test)]
fn expand_field(
    field: &FieldIr,
    _descriptor_fields: &TokenStream,
    runtime: &TokenStream,
    generic_variant_inherited: Option<bool>,
    definition_path: Option<&TokenStream>,
) -> TokenStream {
    expand_field_with_parameters(field, runtime, generic_variant_inherited, definition_path, &[], None)
}

/// Generates the ordered metadata bindings and field-vector update for one
/// field.
///
/// The emitted tokens preserve normalized occurrence order, build supporting
/// selector and constraint metadata before attaching attributes, then append
/// the completed descriptor to the surrounding `fields` vector.
///
/// `generic_variant_inherited` controls definition-specific metadata,
/// `generic_type_names` identifies declared parameters, and `owner` identifies
/// recursive fields.
///
/// # Parameters
/// - `field`: normalized field metadata.
/// - `runtime`: path to the runtime crate.
/// - `generic_variant_inherited`: generic variant inheritance state.
/// - `definition_path`: generic declaration path when expanding a definition.
/// - `generic_type_names`: declared type and const parameter names.
/// - `owner`: declaration type name used to identify recursive fields.
///
/// # Returns
/// Tokens that construct the field metadata and append it to the field vector.
#[must_use]
fn expand_field_with_parameters(
    field: &FieldIr,
    runtime: &TokenStream,
    generic_variant_inherited: Option<bool>,
    definition_path: Option<&TokenStream>,
    generic_type_names: &[Ident],
    owner: Option<&Ident>,
) -> TokenStream {
    let index = *field.index.value();
    let variant = field
        .variant_index
        .map_or_else(|| quote!(None), |index| quote!(Some(#index)));
    let source_name = field.rust_name.as_ref().map_or_else(
        || quote!(None),
        |name| {
            let name = LitStr::new(&name.to_string(), name.span());
            quote!(Some(#name))
        },
    );
    let visibility = match &field.visibility {
        Visibility::Public(_) => quote!(#runtime::__private::v8::Visibility::Public),
        Visibility::Restricted(value) if value.path.is_ident("crate") => {
            quote!(#runtime::__private::v8::Visibility::Crate)
        }
        Visibility::Restricted(value) => {
            let name = LitStr::new(&value.path.to_token_stream().to_string(), value.path.span());
            quote!(#runtime::__private::v8::Visibility::Restricted(#name))
        }
        Visibility::Inherited => quote!(#runtime::__private::v8::Visibility::Private),
    };
    let declaration_owner = quote!(None);
    let span = field.index.span();
    let location = quote_spanned!(span=>
        let field_declaration = #runtime::__private::v8::leak(#runtime::metadata::DeclarationLocation {
            file: Some(file!()), line: Some(line!()), column: Some(column!()), owner: #declaration_owner,
            variant: #variant, field: Some(#index), selector: None,
        });
    );
    let field_type = &field.ty;
    let type_expression = type_expression_for_owner(field_type, runtime, generic_type_names, owner);
    let opaque_type_id = if generic_variant_inherited.is_none()
        && field
            .occurrences
            .iter()
            .any(|occurrence| matches!(occurrence, FieldOccurrence::Opaque))
        && type_is_static_concrete(field_type, generic_type_names)
    {
        quote!(Some(::std::any::TypeId::of::<#field_type>()))
    } else {
        quote!(None)
    };
    let validator_irs: Vec<_> = field
        .occurrences
        .iter()
        .filter_map(|value| match value {
            FieldOccurrence::Validator(value) => Some(value),
            _ => None,
        })
        .collect();
    let validators = validator_irs
        .iter()
        .map(|value| expand_validator(value, runtime, quote!(field_declaration)));
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
    let (collection_assertions, selector_metadata, constraints_binding, collection_ops, optional_ops) =
        expand_collection_metadata(field, runtime, generic_variant_inherited.is_some());
    let identifier = identifier_assignment.map(|assignment| {
        let assignment = match assignment {
            IdentifierAssignmentIr::Application => {
                quote!(#runtime::metadata::IdentifierAssignment::Application)
            }
            IdentifierAssignmentIr::Database => quote!(#runtime::metadata::IdentifierAssignment::Database),
        };
        quote! {
            fn assert_identifier_type<T: #runtime::__private::v8::IdentifierType>() {}
            assert_identifier_type::<#field_type>();
            let identifier: &'static #runtime::metadata::IdentifierMetadata = #runtime::__private::v8::leak(
                #runtime::metadata::IdentifierMetadata::new(#assignment),
            );
        }
    });
    let unique = unique_ir.map(|unique| {
        let paths = unique.respect_to.iter().map(|path| expand_field_path(path, runtime));
        let ignore_case = unique
            .ignore_case
            .map_or_else(|| quote!(None), |value| quote!(Some(#value)));
        let constructor = if generic_variant_inherited.is_some() {
            quote!(#runtime::metadata::FieldUniqueMetadata::for_definition(unique_paths, #ignore_case))
        } else {
            quote!(#runtime::metadata::FieldUniqueMetadata::for_type(
                unique_paths,
                #ignore_case,
            ))
        };
        let assertion = (unique.ignore_case.is_some() && generic_variant_inherited.is_none()).then(|| {
            quote! {
                fn assert_text_unique<T: #runtime::__private::v8::TextConstraintTarget + ?Sized>() {}
                assert_text_unique::<#field_type>();
            }
        });
        quote! {
            #assertion
            let unique_paths: &'static [#runtime::metadata::PropertyPath] =
                #runtime::__private::v8::leak_slice(::std::vec![#(#paths),*]);
            let unique: &'static #runtime::metadata::FieldUniqueMetadata = #runtime::__private::v8::leak(
                #constructor,
            );
        }
    });
    let reference = reference_ir.map(|value| expand_reference(value, runtime));
    let key_part = key_part_order.map(|order| {
        quote! {
            let key_part: &'static #runtime::metadata::KeyPartMetadata = #runtime::__private::v8::leak(
                #runtime::metadata::KeyPartMetadata::new(#order),
            );
        }
    });
    let codec = codec_ir.map(|codec| {
        let value = match codec {
            CodecIr::DeclaredId(id) => quote!(#runtime::metadata::CodecReference::DeclaredId(#id)),
            CodecIr::RustType(ty) => {
                quote!(#runtime::metadata::CodecReference::RustType(
                    #runtime::__private::v8::RustTypeReference::of::<#ty>(),
                ))
            }
        };
        quote! {
            let codec_reference: &'static #runtime::metadata::CodecReference = #runtime::__private::v8::leak(#value);
            let codec: &'static #runtime::metadata::CodecMetadata = #runtime::__private::v8::leak(
                #runtime::metadata::CodecMetadata::new(codec_reference, #runtime::metadata::CodecSource::Field),
            );
        }
    });
    let redact =
        redact_ir.map(|value| expand_redact(value, quote!(#runtime::metadata::RedactPosition::Field), runtime));
    let serde = serde_ir.map_or_else(
        || {
            quote!(
                let serde: &'static #runtime::metadata::SerdeFieldMetadata =
                    &#runtime::metadata::SerdeFieldMetadata::DEFAULT;
            )
        },
        |value| expand_serde(value, runtime),
    );
    let (occurrence_tokens, indexed) = expand_field_attribute_tokens(
        field,
        has_indexed,
        has_identifier,
        unique_ir.is_some(),
        reference_ir.is_some(),
        runtime,
    );
    let metadata = match generic_variant_inherited {
        Some(variant_inherited) => quote! {
            #runtime::__private::v8::generic_field_metadata(
                #definition_path,
                #index,
                #variant,
                #source_name,
                #visibility,
                _field_type_expression_data,
                #variant_inherited,
                attributes,
                constraints,
                validators,
                serde,
            )
        },
        None => quote! {
            #runtime::__private::v8::field_metadata_with_opaque_type_id(
                ::std::any::TypeId::of::<Self>(),
                #variant,
                #index,
                #source_name,
                #visibility,
                _field_type_expression_data,
                opaque_type_id,
                attributes,
                constraints,
                validators,
                serde,
            )
        },
    };
    quote! {
        {
            #location
            let _field_type_expression_data = #type_expression;
            let opaque_type_id: Option<::std::any::TypeId> = #opaque_type_id;
            #collection_assertions
            #identifier
            #unique
            #reference
            #key_part
            let validators: &'static [#runtime::metadata::ValidatorMetadata] =
                #runtime::__private::v8::leak_slice(::std::vec![#(#validators),*]);
            #codec
            #redact
            #serde
            #selector_metadata
            #constraints_binding
            let mut attributes = ::std::vec::Vec::new();
            #(#occurrence_tokens)*
            #indexed
            let attributes: &'static [#runtime::metadata::FieldAttributeMetadata] =
                #runtime::__private::v8::leak_slice(attributes);
            #collection_ops
            #optional_ops
            let field_metadata = (#metadata).with_declaration(*field_declaration);
            let field_metadata = match collection_ops {
                Some(ops) => field_metadata.with_collection_ops(ops),
                None => field_metadata,
            };
            let field_metadata = match optional_ops {
                Some(ops) => field_metadata.with_optional_ops(ops),
                None => field_metadata,
            };
            fields.push(field_metadata);
        }
    }
}

/// Builds selector, constraint, and collection metadata tokens for a field.
///
/// The returned fragments are inserted at their original binding points so
/// references to declaration metadata remain initialized in the same order.
#[must_use]
fn expand_collection_metadata(
    field: &FieldIr,
    runtime: &TokenStream,
    generic_definition: bool,
) -> (TokenStream, TokenStream, TokenStream, TokenStream, TokenStream) {
    let field_type = &field.ty;
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
            <#field_type as #runtime::__private::v8::SequenceConstraintTarget>::Element
        );
        expand_selector_metadata(value, &value_type, format_ident!("element_selector"), runtime)
    });
    let map_key_selector = map_key_ir.map(|value| {
        let value_type = quote!(
            <#field_type as #runtime::__private::v8::MapConstraintTarget>::Key
        );
        expand_selector_metadata(value, &value_type, format_ident!("map_key_selector"), runtime)
    });
    let map_value_selector = map_value_ir.map(|value| {
        let value_type = quote!(
            <#field_type as #runtime::__private::v8::MapConstraintTarget>::Value
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
            fn assert_sequence_target<T: #runtime::__private::v8::SequenceConstraintTarget>() {}
            assert_sequence_target::<#field_type>();
        }
    });
    let map_assertion = requires_map.then(|| {
        quote! {
            fn assert_map_target<T: #runtime::__private::v8::MapConstraintTarget>() {}
            assert_map_target::<#field_type>();
        }
    });
    let selector_metadata = quote! {
        #element_selector
        #map_key_selector
        #map_value_selector
    };
    let constraints_binding = quote! {
        let constraints: &'static [#runtime::metadata::ConstraintMetadata] =
            #runtime::__private::v8::leak_slice(::std::vec![#(#constraints),*]);
    };
    let collection_ops = expand_collection_ops(field, &constraint_irs, runtime, generic_definition);
    let optional_ops = expand_optional_ops(field, runtime, generic_definition);
    (
        quote!(#sequence_assertion #map_assertion #constraint_assertions),
        selector_metadata,
        constraints_binding,
        collection_ops,
        optional_ops,
    )
}

/// Builds field-attribute pushes in normalized occurrence order and appends the
/// synthesized indexing reason after those explicit attributes.
///
/// Constraint and validator indices advance only when their matching
/// occurrences are visited, so the generated references stay aligned with the
/// arrays assembled by the caller.
#[must_use]
fn expand_field_attribute_tokens(
    field: &FieldIr,
    has_indexed: bool,
    has_identifier: bool,
    has_unique: bool,
    has_reference: bool,
    runtime: &TokenStream,
) -> (Vec<TokenStream>, Option<TokenStream>) {
    let mut occurrence_tokens = Vec::new();
    let mut validator_index = 0usize;
    let mut constraint_index = 0usize;
    for occurrence in &field.occurrences {
        occurrence_tokens.push(match occurrence {
            FieldOccurrence::Identifier(_) => {
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Identifier(identifier));)
            }
            FieldOccurrence::Indexed => TokenStream::new(),
            FieldOccurrence::Unique(_) => {
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Unique(unique));)
            }
            FieldOccurrence::Reference(_) => {
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Reference(reference));)
            }
            FieldOccurrence::KeyPart(_) => {
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::KeyPart(key_part));)
            }
            FieldOccurrence::Constraint(_) => {
                let current = constraint_index;
                constraint_index += 1;
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Constraint(&constraints[#current]));)
            }
            FieldOccurrence::Selector(_) => TokenStream::new(),
            FieldOccurrence::Validator(_) => {
                let current = validator_index;
                validator_index += 1;
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Validator(&validators[#current]));)
            }
            FieldOccurrence::Codec(_) => {
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Codec(codec));)
            }
            FieldOccurrence::Redact(_) => {
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Redact(redact));)
            }
            FieldOccurrence::Serde(_) => {
                quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Serde(serde));)
            }
            FieldOccurrence::Opaque => quote!(attributes.push(#runtime::metadata::FieldAttributeMetadata::Opaque);),
        });
    }
    let mut reason_parts = Vec::new();
    if has_indexed {
        reason_parts.push(quote!(#runtime::metadata::IndexingReasons::EXPLICIT));
    }
    if has_identifier {
        reason_parts.push(quote!(#runtime::metadata::IndexingReasons::IDENTIFIER));
    }
    if has_unique {
        reason_parts.push(quote!(#runtime::metadata::IndexingReasons::UNIQUE));
    }
    if has_reference {
        reason_parts.push(quote!(#runtime::metadata::IndexingReasons::REFERENCE));
    }
    let indexed = reason_parts
        .into_iter()
        .reduce(|left, right| quote!(#left | #right))
        .map(|reasons| {
            quote! {
                attributes.push(#runtime::metadata::FieldAttributeMetadata::Indexed(#reasons));
            }
        });
    (occurrence_tokens, indexed)
}

/// Reports whether a type contains neither lifetimes nor declared type
/// parameters.
#[must_use]
fn type_is_static_concrete(ty: &Type, generic_type_names: &[Ident]) -> bool {
    /// Recursively checks whether the token stream contains static concrete
    /// syntax.
    fn is_eligible(stream: TokenStream, generic_type_names: &[Ident]) -> bool {
        stream.into_iter().all(|token| match token {
            TokenTree::Group(group) => is_eligible(group.stream(), generic_type_names),
            TokenTree::Ident(ident) => !generic_type_names.iter().any(|name| name == &ident),
            TokenTree::Punct(punct) => !matches!(punct.as_char(), '&' | '\''),
            TokenTree::Literal(_) => true,
        })
    }

    is_eligible(ty.to_token_stream(), generic_type_names)
}

/// Emits an optional-element adapter for a concrete `Option<T>` field.
///
/// Generic definitions and opaque fields intentionally receive no adapter;
/// other fields receive one only when their declared type has the supported
/// `Option<T>` shape.
///
/// # Parameters
/// - `field`: normalized field metadata.
/// - `runtime`: path to the runtime crate.
/// - `generic_definition`: whether expansion targets a generic definition.
///
/// # Returns
/// Tokens binding `optional_ops` to an adapter or `None`.
#[must_use]
fn expand_optional_ops(field: &FieldIr, runtime: &TokenStream, generic_definition: bool) -> TokenStream {
    if generic_definition
        || field
            .occurrences
            .iter()
            .any(|occurrence| matches!(occurrence, FieldOccurrence::Opaque))
    {
        return quote!(let optional_ops = None;);
    }
    let Some(element_type) = option_inner_type(&field.ty) else {
        return quote!(let optional_ops = None;);
    };
    quote! {
        let optional_ops = Some(#runtime::metadata::OptionalOps::for_element::<#element_type>());
    }
}

/// Emits exact-type collection readers for a concrete field declaration.
///
/// # Parameters
/// - `field`: normalized field metadata.
/// - `constraints`: constraints attached to the field.
/// - `runtime`: path to the runtime crate.
/// - `generic_definition`: whether expansion targets a generic definition.
///
/// # Returns
/// Tokens that build collection readers, or no readers when none apply.
#[must_use]
fn expand_collection_ops(
    field: &FieldIr,
    constraints: &[&ConstraintIr],
    runtime: &TokenStream,
    generic_definition: bool,
) -> TokenStream {
    let map = constraints
        .iter()
        .any(|value| matches!(value, ConstraintIr::Map { .. }));
    let has_selector = field
        .occurrences
        .iter()
        .any(|value| matches!(value, FieldOccurrence::Selector(_)));
    let unique = constraints
        .iter()
        .any(|value| matches!(value, ConstraintIr::Sequence { unique: true, .. }));
    if unique && known_non_vec_unique_shape(&field.ty) {
        return quote! {
            compile_error!("unique_items requires Vec<T> or [T; N] with a borrowed slice getter");
            let collection_ops = None;
        };
    }
    if generic_definition || (!map && !unique && !has_selector) {
        return quote!(let collection_ops = None;);
    }
    let field_type = &field.ty;
    if unique
        && (collection_name(field_type) == Some("Vec") || matches!(field_type, Type::Array(_)))
        && collection_element_type(field_type).is_none()
    {
        return quote!(compile_error!("unique_items requires a concrete element type"); let collection_ops = None;);
    }
    let optional_map = collection_element_type(field_type)
        .filter(|_| {
            matches!(field_type, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Option"))
        })
        .filter(|inner| matches!(collection_name(inner), Some("HashMap" | "BTreeMap")));
    let map_adapter = expand_map_length_adapter(field_type, map, optional_map, runtime);
    let item_adapter = expand_unique_item_adapter(field_type, unique, runtime);
    let items_adapter = if has_selector {
        expand_collection_items(field_type, runtime)
    } else {
        TokenStream::new()
    };
    let map_value = if map_adapter.is_empty() {
        quote!(None)
    } else {
        quote!(Some(map_len))
    };
    let item_value = if item_adapter.is_empty() {
        quote!(None)
    } else {
        quote!(Some(item_eq))
    };
    if map_adapter.is_empty() && item_adapter.is_empty() && items_adapter.is_empty() {
        return quote!(let collection_ops = None;);
    }
    let collection_ops = if items_adapter.is_empty() {
        quote!(#runtime::metadata::CollectionOps::new(#map_value, #item_value))
    } else {
        quote!(#runtime::metadata::CollectionOps::new(#map_value, #item_value).with_items(collection_items))
    };
    quote! {
        #map_adapter
        #item_adapter
        #items_adapter
        let collection_ops = Some(#collection_ops);
    }
}

/// Generates a map-length callback for supported concrete map field shapes.
///
/// The callback accepts borrowed map values, reports absence for optional
/// values, and preserves the existing type-mismatch and adapter-unavailable
/// errors for incompatible inputs.
#[must_use]
fn expand_map_length_adapter(
    field_type: &Type,
    map: bool,
    optional_map: Option<&Type>,
    runtime: &TokenStream,
) -> TokenStream {
    if map && matches!(collection_name(field_type), Some("HashMap" | "BTreeMap")) {
        quote! {
            fn map_len(value: &#runtime::metadata::PropertyValue<'_>) -> ::core::result::Result<::core::option::Option<usize>, #runtime::metadata::PropertyAccessError> {
                match value {
                    #runtime::metadata::PropertyValue::Borrowed(value) => {
                        let map = value.downcast_ref::<#field_type>().ok_or_else(||
                            #runtime::metadata::PropertyAccessError::value_type_mismatch::<#field_type>((*value).clone())
                        )?;
                        Ok(Some(map.len()))
                    }
                    #runtime::metadata::PropertyValue::OptionalBorrowed(value) => {
                        let Some(value) = value else { return Ok(None); };
                        let map = value.downcast_ref::<#field_type>().ok_or_else(||
                            #runtime::metadata::PropertyAccessError::value_type_mismatch::<#field_type>((*value).clone())
                        )?;
                        Ok(Some(map.len()))
                    }
                    _ => Err(#runtime::metadata::PropertyAccessError::AdapterUnavailable),
                }
            }
        }
    } else if map && let Some(inner_type) = optional_map {
        quote! {
            fn map_len(value: &#runtime::metadata::PropertyValue<'_>) -> ::core::result::Result<::core::option::Option<usize>, #runtime::metadata::PropertyAccessError> {
                match value {
                    #runtime::metadata::PropertyValue::Borrowed(value) => {
                        let map = value.downcast_ref::<#field_type>().ok_or_else(||
                            #runtime::metadata::PropertyAccessError::value_type_mismatch::<#field_type>((*value).clone())
                        )?;
                        Ok(map.as_ref().map(|map| map.len()))
                    }
                    #runtime::metadata::PropertyValue::OptionalBorrowed(value) => {
                        let Some(value) = value else { return Ok(None); };
                        let map = value.downcast_ref::<#inner_type>().ok_or_else(||
                            #runtime::metadata::PropertyAccessError::value_type_mismatch::<#inner_type>((*value).clone())
                        )?;
                        Ok(Some(map.len()))
                    }
                    _ => Err(#runtime::metadata::PropertyAccessError::AdapterUnavailable),
                }
            }
        }
    } else {
        TokenStream::new()
    }
}

/// Generates an equality callback for unique items in supported sequences.
///
/// The callback downcasts each dynamic reference to the declared element type
/// and reports a type mismatch through the existing property-access error.
#[must_use]
fn expand_unique_item_adapter(field_type: &Type, unique: bool, runtime: &TokenStream) -> TokenStream {
    if unique && (collection_name(field_type) == Some("Vec") || matches!(field_type, Type::Array(_))) {
        let element_type = collection_element_type(field_type).expect("sequence element shape was checked");
        quote! {
            fn assert_item_eq<T: ::core::cmp::PartialEq + 'static>() {}
            assert_item_eq::<#element_type>();
            fn item_eq(
                left: #runtime::__private::DynamicRef<'_, #runtime::__private::Local>,
                right: #runtime::__private::DynamicRef<'_, #runtime::__private::Local>,
            ) -> ::core::result::Result<bool, #runtime::metadata::PropertyAccessError> {
                let left = left.downcast_ref::<#element_type>().ok_or_else(||
                    #runtime::metadata::PropertyAccessError::value_type_mismatch::<#element_type>(left.clone())
                )?;
                let right = right.downcast_ref::<#element_type>().ok_or_else(||
                    #runtime::metadata::PropertyAccessError::value_type_mismatch::<#element_type>(right.clone())
                )?;
                Ok(left == right)
            }
        }
    } else {
        TokenStream::new()
    }
}

/// Generates an iterator over borrowed values for a known sequence or map.
#[must_use]
fn expand_collection_items(field_type: &Type, runtime: &TokenStream) -> TokenStream {
    let inner = option_inner_type(field_type).unwrap_or(field_type);
    let kind = collection_name(inner);
    let is_array = matches!(inner, Type::Array(_));
    if !is_array && !matches!(kind, Some("Vec" | "HashMap" | "BTreeMap")) {
        return TokenStream::new();
    }
    let has_concrete_members = match kind {
        Some("Vec") => collection_element_type(inner).is_some(),
        Some("HashMap" | "BTreeMap") => {
            let Type::Path(path) = inner else {
                return TokenStream::new();
            };
            let Some(PathArguments::AngleBracketed(arguments)) =
                path.path.segments.last().map(|segment| &segment.arguments)
            else {
                return TokenStream::new();
            };
            arguments
                .args
                .iter()
                .filter(|argument| matches!(argument, GenericArgument::Type(_)))
                .count()
                >= 2
        }
        _ => true,
    };
    if !has_concrete_members {
        return TokenStream::new();
    }
    let field_collection = if option_inner_type(field_type).is_some() {
        quote! {
            let field = value.clone().downcast::<#field_type>().map_err(|failure| {
                let mismatch = failure.into_parts().0;
                #runtime::metadata::PropertyAccessError::ValueTypeMismatch(
                    #runtime::metadata::PropertyTypeMismatch {
                        expected: mismatch.expected(), actual: mismatch.actual(),
                    },
                )
            })?;
            let Some(collection) = field.as_ref() else { return Ok(None); };
        }
    } else {
        quote! {
            let collection = value.clone().downcast::<#field_type>().map_err(|failure| {
                let mismatch = failure.into_parts().0;
                #runtime::metadata::PropertyAccessError::ValueTypeMismatch(
                    #runtime::metadata::PropertyTypeMismatch {
                        expected: mismatch.expected(), actual: mismatch.actual(),
                    },
                )
            })?;
        }
    };
    let inner_collection = if option_inner_type(field_type).is_some() {
        quote! {
            let collection = value.clone().downcast::<#inner>().map_err(|failure| {
                let mismatch = failure.into_parts().0;
                #runtime::metadata::PropertyAccessError::ValueTypeMismatch(
                    #runtime::metadata::PropertyTypeMismatch {
                        expected: mismatch.expected(), actual: mismatch.actual(),
                    },
                )
            })?;
        }
    } else {
        field_collection.clone()
    };
    let members = if is_array || kind == Some("Vec") {
        quote! {
            {
                if position != #runtime::metadata::SelectorPosition::Element {
                    return Err(#runtime::metadata::PropertyAccessError::AdapterUnavailable);
                }
                Box::new(collection.iter().map(#runtime::__private::DynamicRef::<#runtime::__private::Local>::new))
                    as Box<dyn Iterator<Item = #runtime::__private::DynamicRef<'_, #runtime::__private::Local>> + '_>
            }
        }
    } else {
        quote! {
            match position {
                #runtime::metadata::SelectorPosition::MapKey => Box::new(collection.keys().map(
                    #runtime::__private::DynamicRef::<#runtime::__private::Local>::new
                )) as Box<dyn Iterator<Item = #runtime::__private::DynamicRef<'_, #runtime::__private::Local>> + '_>,
                #runtime::metadata::SelectorPosition::MapValue => Box::new(collection.values().map(
                    #runtime::__private::DynamicRef::<#runtime::__private::Local>::new
                )) as Box<dyn Iterator<Item = #runtime::__private::DynamicRef<'_, #runtime::__private::Local>> + '_>,
                #runtime::metadata::SelectorPosition::Element => {
                    return Err(#runtime::metadata::PropertyAccessError::AdapterUnavailable);
                }
            }
        }
    };
    quote! {
        fn collection_items<'a>(
            value: &#runtime::metadata::PropertyValue<'a>,
            position: #runtime::metadata::SelectorPosition,
        ) -> ::core::result::Result<::core::option::Option<#runtime::metadata::BorrowedCollection<'a>>, #runtime::metadata::PropertyAccessError> {
            match value {
                #runtime::metadata::PropertyValue::Borrowed(value) => {
                    #field_collection
                    Ok(Some(#runtime::metadata::BorrowedCollection::new(#members)))
                }
                #runtime::metadata::PropertyValue::OptionalBorrowed(Some(value)) => {
                    #inner_collection
                    Ok(Some(#runtime::metadata::BorrowedCollection::new(#members)))
                }
                #runtime::metadata::PropertyValue::OptionalBorrowed(None) => Ok(None),
                #runtime::metadata::PropertyValue::BorrowedSlice(values) => {
                    if position != #runtime::metadata::SelectorPosition::Element {
                        return Err(#runtime::metadata::PropertyAccessError::AdapterUnavailable);
                    }
                    Ok(Some(#runtime::metadata::BorrowedCollection::new(values.iter())))
                }
                _ => Err(#runtime::metadata::PropertyAccessError::AdapterUnavailable),
            }
        }
    }
}

/// Returns the inner type of `Option<T>` when the declaration uses that shape.
fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    arguments.args.iter().find_map(|argument| match argument {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })
}

/// Returns the standard collection name for supported spelling forms.
///
/// # Parameters
/// - `ty`: the declared collection type.
///
/// # Returns
/// `Some` with a supported collection name, or `None` for other types.
fn collection_name(ty: &Type) -> Option<&str> {
    let Type::Path(path) = ty else {
        return None;
    };
    if !crate::compiler::type_path::is_collection_path(&path.path) {
        return None;
    }
    let name = &path.path.segments.last()?.ident;
    if name == "HashMap" {
        Some("HashMap")
    } else if name == "BTreeMap" {
        Some("BTreeMap")
    } else if name == "Vec" {
        Some("Vec")
    } else {
        None
    }
}

/// Returns the first concrete type argument of a collection declaration.
///
/// # Parameters
/// - `ty`: the declared collection type.
///
/// # Returns
/// The first concrete element or value type argument, when present.
fn collection_element_type(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Array(array) => Some(&array.elem),
        Type::Path(path) => {
            let PathArguments::AngleBracketed(args) = &path.path.segments.last()?.arguments else {
                return None;
            };
            args.args.iter().find_map(|arg| match arg {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
        }
        _ => None,
    }
}

/// Identifies sequence shapes accepted by declaration traits but lacking a
/// slice adapter.
///
/// # Parameters
/// - `ty`: the declared sequence type.
///
/// # Returns
/// `true` for known unsupported wrappers and sequence containers.
#[must_use]
#[inline]
fn known_non_vec_unique_shape(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => {
            let Some(name) = path.path.segments.last().map(|segment| &segment.ident) else {
                return false;
            };
            ["VecDeque", "LinkedList", "BinaryHeap", "Option", "Box", "Rc", "Arc"]
                .iter()
                .any(|candidate| name == candidate)
        }
        _ => false,
    }
}

/// Generates runtime metadata for one normalized constraint.
///
/// # Parameters
/// - `value`: normalized constraint data.
/// - `has_element`: whether an element selector is present.
/// - `has_map_key`: whether a map-key selector is present.
/// - `has_map_value`: whether a map-value selector is present.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens that construct the matching runtime constraint metadata.
#[must_use]
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
                "unicode" => quote!(#runtime::metadata::AllowedChars::Unicode),
                "printable_unicode" => {
                    quote!(#runtime::metadata::AllowedChars::PrintableUnicode)
                }
                "ascii" => quote!(#runtime::metadata::AllowedChars::Ascii),
                "printable_ascii" => {
                    quote!(#runtime::metadata::AllowedChars::PrintableAscii)
                }
                "code" => quote!(#runtime::metadata::AllowedChars::Code),
                _ => quote!(compile_error!("invalid allowed_chars value")),
            };
            let non_blank = value.non_blank;
            let format = value.format.as_deref().map_or_else(
                || quote!(None),
                |value| {
                    let value = match value {
                        "email_ascii" => {
                            quote!(#runtime::metadata::TextFormat::EmailAscii)
                        }
                        "cn_mobile" => {
                            quote!(#runtime::metadata::TextFormat::Mobile)
                        }
                        "uri" => quote!(#runtime::metadata::TextFormat::Uri),
                        "uuid" => quote!(#runtime::metadata::TextFormat::Uuid),
                        _ => quote!(compile_error!("invalid text format")),
                    };
                    quote!(Some(#value))
                },
            );
            quote!(#runtime::metadata::ConstraintMetadata::Text(#runtime::metadata::TextConstraint::new(
                #min_chars, #max_chars, #min_bytes, #max_bytes, #allowed, #non_blank, #format,
            )))
        }
        ConstraintIr::Decimal(value) => {
            let precision = option_number(value.precision);
            let scale = value.scale;
            let rounding = rounding_tokens(&value.rounding, runtime);
            let semantic = if value.money {
                quote!(#runtime::metadata::DecimalSemantic::Money)
            } else {
                quote!(#runtime::metadata::DecimalSemantic::Number)
            };
            let min = option_lit_str(&value.min);
            let max = option_lit_str(&value.max);
            let min_inclusive = value.min_inclusive;
            let max_inclusive = value.max_inclusive;
            quote!(#runtime::metadata::ConstraintMetadata::Decimal(
                #runtime::metadata::DecimalConstraint::new(#precision, #scale, #rounding, #semantic)
                    .with_bounds(#min, #max, #min_inclusive, #max_inclusive)
            ))
        }
        ConstraintIr::Time(value) => {
            let precision = match value.as_str() {
                "second" => {
                    quote!(#runtime::metadata::TemporalPrecision::Second)
                }
                "millisecond" => {
                    quote!(#runtime::metadata::TemporalPrecision::Millisecond)
                }
                "microsecond" => {
                    quote!(#runtime::metadata::TemporalPrecision::Microsecond)
                }
                "nanosecond" => {
                    quote!(#runtime::metadata::TemporalPrecision::Nanosecond)
                }
                _ => quote!(compile_error!("invalid time precision")),
            };
            quote!(#runtime::metadata::ConstraintMetadata::Time(#runtime::metadata::TimeConstraint::new(#precision)))
        }
        ConstraintIr::Sequence { min, max, unique } => {
            let min = option_number(*min);
            let max = option_number(*max);
            let base = quote!(#runtime::metadata::SequenceConstraint::new(#min, #max, #unique));
            let value = if has_element {
                quote!(#base.with_element(element_selector))
            } else {
                base
            };
            quote!(#runtime::metadata::ConstraintMetadata::Sequence(#value))
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
            quote!(#runtime::metadata::ConstraintMetadata::Map(#runtime::metadata::MapConstraint::new(#min, #max).with_selectors(#key, #value)))
        }
    }
}

/// Generates compile-time type-capability assertions for constraints.
///
/// # Parameters
/// - `constraints`: normalized constraints to check.
/// - `target`: tokens naming the asserted Rust type.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens containing the assertions required by those constraints.
#[must_use]
fn expand_constraint_assertions(
    constraints: &[&ConstraintIr],
    target: TokenStream,
    runtime: &TokenStream,
) -> TokenStream {
    let assertions = constraints.iter().map(|constraint| match constraint {
        ConstraintIr::Text(_) => quote! {
            fn assert_text_target<T: #runtime::__private::v8::TextConstraintTarget + ?Sized>() {}
            assert_text_target::<#target>();
        },
        ConstraintIr::Decimal(_) => quote! {
            fn assert_decimal_target<T: #runtime::__private::v8::DecimalConstraintTarget>() {}
            assert_decimal_target::<#target>();
        },
        ConstraintIr::Time(_) => quote! {
            fn assert_temporal_target<T: #runtime::__private::v8::TemporalConstraintTarget>() {}
            assert_temporal_target::<#target>();
        },
        ConstraintIr::Sequence { min, max, unique } => {
            let length = (min.is_some() || max.is_some()).then(|| {
                quote! {
                    fn assert_variable_sequence<T: #runtime::__private::v8::VariableLengthSequenceTarget>() {}
                    assert_variable_sequence::<#target>();
                }
            });
            let uniqueness = unique.then(|| {
                quote! {
                    fn assert_unique_items_target<T: #runtime::__private::v8::UniqueItemsConstraintTarget>() {}
                    assert_unique_items_target::<#target>();
                }
            });
            quote! {
                fn assert_sequence_constraint<T: #runtime::__private::v8::SequenceConstraintTarget>() {}
                assert_sequence_constraint::<#target>();
                #length
                #uniqueness
            }
        }
        ConstraintIr::Map { .. } => quote! {
            fn assert_map_constraint<T: #runtime::__private::v8::MapConstraintTarget>() {}
            assert_map_constraint::<#target>();
        },
    });
    quote!(#(#assertions)*)
}

/// Generates runtime metadata for a collection selector.
///
/// # Parameters
/// - `value`: normalized selector metadata.
/// - `value_type`: tokens naming the selected Rust type.
/// - `name`: local identifier for the generated selector binding.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens that construct selector metadata and bind it to `name`.
#[must_use]
fn expand_selector_metadata(
    value: &SelectorIr,
    value_type: &TokenStream,
    name: Ident,
    runtime: &TokenStream,
) -> TokenStream {
    let position = match value.position {
        SelectorPositionIr::Element => {
            quote!(#runtime::metadata::SelectorPosition::Element)
        }
        SelectorPositionIr::MapKey => {
            quote!(#runtime::metadata::SelectorPosition::MapKey)
        }
        SelectorPositionIr::MapValue => {
            quote!(#runtime::metadata::SelectorPosition::MapValue)
        }
    };
    let constraints = value
        .constraints
        .iter()
        .map(|constraint| expand_constraint(constraint, false, false, false, runtime));
    let constraint_refs: Vec<_> = value.constraints.iter().collect();
    let constraint_assertions = expand_constraint_assertions(&constraint_refs, value_type.clone(), runtime);
    let validators = value.validators.iter().map(|validator| {
        expand_validator(
            validator,
            runtime,
            quote!(#runtime::__private::v8::leak((*field_declaration).with_selector(#position))),
        )
    });
    let codec = value.codec.as_ref().map_or_else(
        || quote!(None),
        |codec| {
            let reference = codec_reference_expression(codec, value_type, runtime);
            quote!({
                let reference: &'static #runtime::metadata::CodecReference = #runtime::__private::v8::leak(#reference);
                Some(#runtime::__private::v8::leak(
                    #runtime::metadata::CodecMetadata::new(reference, #runtime::metadata::CodecSource::Selector(#position)),
                ) as &'static #runtime::metadata::CodecMetadata)
            })
        },
    );
    let redact = value.redact.as_ref().map_or_else(
        || quote!(None),
        |redact| {
            let expression = redact_expression(
                redact,
                match value.position {
                    SelectorPositionIr::Element => quote!(#runtime::metadata::RedactPosition::Element),
                    SelectorPositionIr::MapKey => quote!(#runtime::metadata::RedactPosition::MapKey),
                    SelectorPositionIr::MapValue => quote!(#runtime::metadata::RedactPosition::MapValue),
                },
                runtime,
            );
            quote!(Some(#runtime::__private::v8::leak(#expression) as &'static #runtime::metadata::RedactMetadata))
        },
    );
    quote! {
        #constraint_assertions
        let selector_constraints: &'static [#runtime::metadata::ConstraintMetadata] = #runtime::__private::v8::leak_slice(::std::vec![#(#constraints),*]);
        let selector_validators: &'static [#runtime::metadata::ValidatorMetadata] = #runtime::__private::v8::leak_slice(::std::vec![#(#validators),*]);
        let selector_codec = #codec;
        let selector_redact = #redact;
        let #name: &'static #runtime::metadata::SelectorMetadata = #runtime::__private::v8::leak(
            #runtime::metadata::SelectorMetadata::new(#position, selector_constraints, selector_validators, selector_codec, selector_redact),
        );
    }
}

/// Converts an optional tokenizable number into runtime option tokens.
///
/// # Parameters
/// - `value`: the optional value to quote.
///
/// # Returns
/// Tokens representing `None` or `Some(value)`.
#[must_use]
#[inline]
fn option_number<T: ToTokens>(value: Option<T>) -> TokenStream {
    value.map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

/// Maps a validated rounding name to runtime enum tokens.
///
/// # Parameters
/// - `value`: the normalized rounding mode name.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens naming the corresponding runtime rounding mode.
#[must_use]
#[inline]
fn rounding_tokens(value: &str, runtime: &TokenStream) -> TokenStream {
    match value {
        "down" => quote!(#runtime::metadata::RoundingMode::Down),
        "up" => quote!(#runtime::metadata::RoundingMode::Up),
        "ceiling" => quote!(#runtime::metadata::RoundingMode::Ceiling),
        "floor" => quote!(#runtime::metadata::RoundingMode::Floor),
        "half_up" => quote!(#runtime::metadata::RoundingMode::HalfUp),
        "half_down" => quote!(#runtime::metadata::RoundingMode::HalfDown),
        "half_even" => quote!(#runtime::metadata::RoundingMode::HalfEven),
        "unnecessary" => quote!(#runtime::metadata::RoundingMode::Unnecessary),
        _ => quote!(compile_error!("invalid rounding mode")),
    }
}

/// Generates runtime metadata for one validator declaration.
///
/// # Parameters
/// - `validator`: normalized validator ID, arguments, and dependencies.
/// - `runtime`: path to the runtime crate.
/// - `declaration`: tokens naming the source declaration location.
///
/// # Returns
/// Tokens that construct the runtime validator metadata.
#[must_use]
fn expand_validator(validator: &ValidatorIr, runtime: &TokenStream, declaration: TokenStream) -> TokenStream {
    let id = &validator.id;
    let params = validator.params.iter().map(|(name, value)| {
        let value = expand_strategy_argument(value, runtime);
        quote!(#runtime::__private::NamedValidationArgument::new(#name, #value))
    });
    let depends_on = validator.depends_on.iter().map(|path| expand_field_path(path, runtime));
    let dependency_bindings = validator.dependency_bindings.iter().map(|(name, navigation, path)| {
        let navigation = expand_object_path(navigation, runtime);
        let path = expand_field_path(path, runtime);
        quote!(#runtime::metadata::DependencyBindingMetadata::new(#name, #path).with_object_path(#navigation).with_declaration(#declaration))
    });
    let constructor = if validator.dependency_bindings.is_empty() {
        quote!(#runtime::metadata::ValidatorMetadata::new(#id, params, depends_on))
    } else {
        quote!(#runtime::metadata::ValidatorMetadata::new_bound(
            #id,
            params,
            depends_on,
            dependency_bindings,
            #runtime::metadata::TargetMode::Value,
            #runtime::metadata::OnNone::Skip,
        ))
    };
    quote!({
        let params: &'static [#runtime::__private::NamedValidationArgument<'static>] = #runtime::__private::v8::leak_slice(::std::vec![#(#params),*]);
        let depends_on: &'static [#runtime::metadata::PropertyPath<'static>] = #runtime::__private::v8::leak_slice(::std::vec![#(#depends_on),*]);
        let dependency_bindings: &'static [#runtime::metadata::DependencyBindingMetadata] = #runtime::__private::v8::leak_slice(::std::vec![#(#dependency_bindings),*]);
        #constructor
    })
}

/// Generates runtime tokens for one validator strategy argument.
///
/// # Parameters
/// - `value`: the normalized argument value.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens representing the matching runtime validation argument.
#[must_use]
fn expand_strategy_argument(value: &StrategyArgumentIr, runtime: &TokenStream) -> TokenStream {
    match value {
        StrategyArgumentIr::Bool(value) => {
            quote!(#runtime::__private::ValidationArgument::Bool(#value))
        }
        StrategyArgumentIr::Integer(value) => {
            quote!(#runtime::__private::ValidationArgument::Integer(#value))
        }
        StrategyArgumentIr::Unsigned(value) => {
            quote!(#runtime::__private::ValidationArgument::Unsigned(#value))
        }
        StrategyArgumentIr::String(value) => {
            quote!(#runtime::__private::ValidationArgument::String(#value))
        }
        StrategyArgumentIr::BoolList(values) => {
            quote!(#runtime::__private::ValidationArgument::BoolList(&[#(#values),*]))
        }
        StrategyArgumentIr::IntegerList(values) => {
            quote!(#runtime::__private::ValidationArgument::IntegerList(&[#(#values),*]))
        }
        StrategyArgumentIr::UnsignedList(values) => {
            quote!(#runtime::__private::ValidationArgument::UnsignedList(&[#(#values),*]))
        }
        StrategyArgumentIr::StringList(values) => {
            quote!(#runtime::__private::ValidationArgument::StringList(&[#(#values),*]))
        }
    }
}

/// Generates runtime metadata for one relationship declaration.
///
/// # Parameters
/// - `reference`: normalized target, property, and navigation metadata.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens that construct the field reference metadata.
#[must_use]
fn expand_reference(reference: &ReferenceIr, runtime: &TokenStream) -> TokenStream {
    let target = match &reference.target {
        ReferenceTargetIr::RustType(ty) => {
            quote!(#runtime::metadata::DeclaredEntityTarget::RustType(#runtime::metadata::TypeMetadata::of::<#ty>))
        }
        ReferenceTargetIr::ModelId(id) => {
            quote!(#runtime::metadata::DeclaredEntityTarget::ModelId(#runtime::metadata::ModelId::new(#id)))
        }
    };
    let selection = reference.property.as_ref().map_or_else(
        || quote!(#runtime::metadata::ReferenceSelection::Entity),
        |path| {
            let path = expand_field_path(path, runtime);
            quote!(#runtime::metadata::ReferenceSelection::Property(#path))
        },
    );
    let same_as = reference.same_as.as_ref().map_or_else(
        || quote!(None),
        |path| {
            let path = expand_object_path(path, runtime);
            quote!(Some(#runtime::__private::v8::leak(#path) as &'static #runtime::metadata::ObjectPath))
        },
    );
    let existing = reference.existing;
    quote! {
        let reference_target: &'static #runtime::metadata::DeclaredEntityTarget = #runtime::__private::v8::leak(#target);
        let reference_selection: &'static #runtime::metadata::ReferenceSelection = #runtime::__private::v8::leak(#selection);
        let reference: &'static #runtime::metadata::FieldReferenceMetadata = #runtime::__private::v8::leak(
            #runtime::metadata::FieldReferenceMetadata::new(reference_target, reference_selection, #existing, #same_as),
        );
    }
}

/// Generates runtime metadata for one redaction declaration.
///
/// # Parameters
/// - `redact`: normalized redaction mode.
/// - `position`: tokens naming the field or selector position.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens that bind the redaction metadata.
#[must_use]
fn expand_redact(redact: &RedactIr, position: TokenStream, runtime: &TokenStream) -> TokenStream {
    let expression = redact_expression(redact, position, runtime);
    quote! {
        let redact: &'static #runtime::metadata::RedactMetadata = #runtime::__private::v8::leak(#expression);
    }
}

/// Generates the redaction expression associated with a redaction mode.
///
/// # Parameters
/// - `redact`: normalized redaction mode.
/// - `position`: tokens naming the field or selector position.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens constructing the runtime redaction value.
#[must_use]
fn redact_expression(redact: &RedactIr, position: TokenStream, runtime: &TokenStream) -> TokenStream {
    let (sensitivity, mode) = match &redact.mode {
        RedactModeIr::Level(level) => {
            let sensitivity = match level.as_str() {
                "public" | "low" => {
                    quote!(#runtime::__private::Sensitivity::Public)
                }
                "personal" | "medium" => {
                    quote!(#runtime::__private::Sensitivity::Personal)
                }
                "confidential" | "high" => {
                    quote!(#runtime::__private::Sensitivity::Confidential)
                }
                "secret" => quote!(#runtime::__private::Sensitivity::Secret),
                _ => quote!(compile_error!(
                    "redact level must be public, personal, confidential, or secret"
                )),
            };
            (
                quote!(Some(#sensitivity)),
                quote!(#runtime::metadata::RedactModeMetadata::Level),
            )
        }
        RedactModeIr::Skip => (quote!(None), quote!(#runtime::metadata::RedactModeMetadata::Skip)),
        RedactModeIr::Nested => (quote!(None), quote!(#runtime::metadata::RedactModeMetadata::Nested)),
        RedactModeIr::Map => (quote!(None), quote!(#runtime::metadata::RedactModeMetadata::Map)),
        RedactModeIr::KeyedBy(field) => (
            quote!(None),
            quote!(#runtime::metadata::RedactModeMetadata::KeyedBy(#field)),
        ),
        RedactModeIr::Json => (quote!(None), quote!(#runtime::metadata::RedactModeMetadata::Json)),
    };
    quote!(#runtime::metadata::RedactMetadata::new(#sensitivity, #mode, #position))
}

/// Generates runtime metadata for a declared value codec.
///
/// # Parameters
/// - `codec`: normalized Rust codec type or declared codec ID.
/// - `_value_type`: selected Rust type, retained for caller symmetry.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens naming the corresponding runtime codec reference.
#[must_use]
#[inline]
fn codec_reference_expression<T: ToTokens>(codec: &CodecIr, _value_type: &T, runtime: &TokenStream) -> TokenStream {
    match codec {
        CodecIr::DeclaredId(id) => {
            quote!(#runtime::metadata::CodecReference::DeclaredId(#id))
        }
        CodecIr::RustType(ty) => {
            quote!(#runtime::metadata::CodecReference::RustType(
                #runtime::__private::v8::RustTypeReference::of::<#ty>(),
            ))
        }
    }
}

/// Generates runtime metadata for one Serde behavior declaration.
///
/// # Parameters
/// - `value`: normalized Serde names and behavior flags.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens that construct the field's Serde metadata.
#[must_use]
fn expand_serde(value: &SerdeIr, runtime: &TokenStream) -> TokenStream {
    let serialize_name = option_lit_str(&value.serialize_name);
    let deserialize_name = option_lit_str(&value.deserialize_name);
    let skip_serializing = value.skip_serializing;
    let skip_deserializing = value.skip_deserializing;
    let flatten = value.flatten;
    let with = option_lit_str(&value.with);
    let default = value.default;
    let default_source = if value.default_from_model {
        quote!(#runtime::metadata::SerdeBehaviorSource::ModelDefault)
    } else if value.default {
        quote!(#runtime::metadata::SerdeBehaviorSource::Explicit)
    } else {
        quote!(#runtime::metadata::SerdeBehaviorSource::None)
    };
    let omit_source = if value.omit_from_model {
        quote!(#runtime::metadata::SerdeBehaviorSource::ModelDefault)
    } else if value.omit_suppressed {
        quote!(#runtime::metadata::SerdeBehaviorSource::Suppressed)
    } else if value.explicit_skip_serializing_if {
        quote!(#runtime::metadata::SerdeBehaviorSource::Explicit)
    } else {
        quote!(#runtime::metadata::SerdeBehaviorSource::None)
    };
    quote! {
        let serde: &'static #runtime::metadata::SerdeFieldMetadata = #runtime::__private::v8::leak(
            #runtime::metadata::SerdeFieldMetadata::new(#serialize_name, #deserialize_name, #skip_serializing, #skip_deserializing, #flatten, #with, #default)
                .with_sources(#default_source, #omit_source),
        );
    }
}

/// Converts an owned field path into runtime path tokens.
///
/// # Parameters
/// - `path`: ordered property name segments.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens constructing a runtime property path.
#[must_use]
fn expand_field_path(path: &[String], runtime: &TokenStream) -> TokenStream {
    quote!(#runtime::metadata::PropertyPath::new(&[#(#path),*]))
}

/// Converts an optional string literal into runtime option tokens.
///
/// # Parameters
/// - `value`: optional string literal to quote.
///
/// # Returns
/// Tokens representing `None` or `Some(value)`.
#[must_use]
#[inline]
fn option_lit_str(value: &Option<LitStr>) -> TokenStream {
    value
        .as_ref()
        .map_or_else(|| quote!(None), |value| quote!(Some(#value)))
}

/// Emits checked object navigation with distinct parent steps.
///
/// # Parameters
/// - `steps`: ordered property and parent navigation steps.
/// - `runtime`: path to the runtime crate.
///
/// # Returns
/// Tokens constructing the runtime object path.
///
/// # Panics
/// The generated code panics only if a validated navigation path is rejected
/// when constructing its runtime representation.
#[must_use]
fn expand_object_path(steps: &[String], runtime: &TokenStream) -> TokenStream {
    let steps = steps.iter().map(|step| {
        if step == ".." {
            quote!(#runtime::metadata::NavigationStep::Parent)
        } else {
            quote!(#runtime::metadata::NavigationStep::Property(#step))
        }
    });
    quote!(#runtime::metadata::ObjectPath::new(#runtime::__private::v8::leak_slice(::std::vec![#(#steps),*])).expect("validated object navigation"))
}

#[cfg(test)]
mod tests {
    use proc_macro2::Span;
    use proc_macro2::TokenStream;
    use proc_macro2::TokenTree;
    use quote::ToTokens;
    use quote::format_ident;
    use quote::quote;
    use syn::Ident;
    use syn::LitStr;
    use syn::Type;
    use syn::Visibility;
    use syn::parse_quote;

    use super::codec_reference_expression;
    use super::expand_collection_ops;
    use super::expand_constraint;
    use super::expand_field;
    use super::expand_field_with_parameters;
    use super::expand_redact;
    use super::expand_reference;
    use super::expand_serde;
    use super::expand_strategy_argument;
    use super::expand_validator;
    use super::rounding_tokens;
    use super::type_expression_data_with_parameters;
    use super::type_expression_for_owner;
    use super::type_is_static_concrete;
    use crate::ir::Located;
    use crate::ir::declaration::CodecIr;
    use crate::ir::declaration::ConstraintIr;
    use crate::ir::declaration::DecimalConstraintIr;
    use crate::ir::declaration::FieldIr;
    use crate::ir::declaration::FieldOccurrence;
    use crate::ir::declaration::RedactIr;
    use crate::ir::declaration::RedactModeIr;
    use crate::ir::declaration::ReferenceIr;
    use crate::ir::declaration::ReferenceTargetIr;
    use crate::ir::declaration::SerdeIr;
    use crate::ir::declaration::StrategyArgumentIr;
    use crate::ir::declaration::TextConstraintIr;
    use crate::ir::declaration::ValidatorIr;

    #[test]
    fn test_type_expression_data_preserves_nested_paths_and_references() {
        let ty: Type = parse_quote!(std::option::Option<&'static crate::model::Value<u32>>);
        let tokens = type_expression_data_with_parameters(&ty, &quote!(runtime), &[]).to_string();
        assert!(tokens.contains("ExpressionData :: Named"), "{tokens}");
        assert!(tokens.contains("GenericExpressionArgument :: Type"), "{tokens}");
        assert!(tokens.contains("ExpressionData :: Reference"), "{tokens}");
        assert!(!tokens.contains("ExpressionData :: Opaque"), "{tokens}");
    }

    #[test]
    fn test_type_expression_data_preserves_const_paths_and_special_types() {
        let runtime = quote!(runtime);
        for (ty, expected) in [
            (
                parse_quote!(::std::option::Option<u8>),
                "path : \"::std::option::Option\"",
            ),
            (parse_quote!(Self), "ExpressionData :: SelfType"),
            (parse_quote!(&'_ str), "LifetimeExpression :: Placeholder"),
            (parse_quote!([u8; 4]), "ConstExpression :: UnsignedInteger (4u128)"),
            (parse_quote!([u8; N]), "ConstExpression :: Path (\"N\")"),
            (parse_quote!(Container<4>), "ConstExpression :: UnsignedInteger (4u128)"),
            (parse_quote!(Container<{ true }>), "ConstExpression :: Boolean (true)"),
            (parse_quote!(Container<{ 'x' }>), "ConstExpression :: Character ('x')"),
            (parse_quote!(<T as Iterator>::Item), "ExpressionData :: AssociatedType"),
        ] {
            let tokens = type_expression_data_with_parameters(&ty, &runtime, &[]).to_string();
            assert!(
                tokens.contains(expected),
                "{}: expected `{expected}` in `{tokens}`",
                ty.to_token_stream()
            );
        }
    }

    #[test]
    fn test_type_expression_data_links_declared_type_and_const_parameters_by_name() {
        let runtime = quote!(runtime);
        let parameters = [format_ident!("T"), format_ident!("N")];
        let type_parameter = type_expression_data_with_parameters(&parse_quote!(T), &runtime, &parameters).to_string();
        let const_parameter =
            type_expression_data_with_parameters(&parse_quote!([u8; N]), &runtime, &parameters).to_string();
        assert!(
            type_parameter.contains("ExpressionData :: ParameterName (\"T\")"),
            "{type_parameter}"
        );
        assert!(!type_parameter.contains("ExpressionData :: Named"), "{type_parameter}");
        assert!(
            const_parameter.contains("ConstExpression :: Parameter (\"N\")"),
            "{const_parameter}"
        );
        let projection =
            type_expression_data_with_parameters(&parse_quote!(<T as Iterator<u8>>::Item), &runtime, &parameters)
                .to_string();
        assert!(projection.contains("ExpressionData :: AssociatedType"), "{projection}");
        assert!(projection.contains("ParameterName (\"T\")"), "{projection}");
        assert!(projection.contains("TraitExpressionData :: new"), "{projection}");
        assert!(projection.contains("GenericExpressionArgument :: Type"), "{projection}");
        assert!(projection.contains("Iterator < u8 >"), "{projection}");
        let absolute_projection = type_expression_data_with_parameters(
            &parse_quote!(<T as ::crate::api::Trait<u8>>::Assoc),
            &runtime,
            &parameters,
        )
        .to_string();
        assert!(
            absolute_projection.contains("::crate :: api :: Trait < u8 >"),
            "{absolute_projection}"
        );
        let associated_arguments =
            type_expression_data_with_parameters(&parse_quote!(<T as Iterator<u8>>::Item<'a>), &runtime, &parameters)
                .to_string();
        assert!(
            associated_arguments.contains("arguments : runtime :: __private :: v8 :: leak_slice"),
            "{associated_arguments}"
        );
        assert!(
            associated_arguments.contains("GenericExpressionArgument :: Lifetime"),
            "{associated_arguments}"
        );
        assert!(
            associated_arguments.contains("LifetimeExpression :: Named (\"a\")"),
            "{associated_arguments}"
        );
        let unqualified_projection =
            type_expression_data_with_parameters(&parse_quote!(T::Assoc<'a>), &runtime, &parameters).to_string();
        assert!(
            unqualified_projection.contains("ExpressionData :: AssociatedType"),
            "{unqualified_projection}"
        );
        assert!(
            unqualified_projection.contains("ParameterName (\"T\")"),
            "{unqualified_projection}"
        );
        assert!(
            unqualified_projection.contains("LifetimeExpression :: Named (\"a\")"),
            "{unqualified_projection}"
        );
    }

    #[test]
    fn test_type_expression_data_marks_only_exact_owner_paths_as_self() {
        let owner = format_ident!("RecursiveValue");
        let runtime = quote!(runtime);
        let recursive =
            type_expression_for_owner(&parse_quote!(Option<Box<RecursiveValue>>), &runtime, &[], Some(&owner))
                .to_string();
        assert_eq!(recursive.matches("ExpressionData :: SelfType").count(), 1);

        let qualified_other =
            type_expression_for_owner(&parse_quote!(other::RecursiveValue), &runtime, &[], Some(&owner)).to_string();
        assert!(qualified_other.contains("ExpressionData :: Named"));
        assert!(!qualified_other.contains("ExpressionData :: SelfType"));
    }

    #[test]
    fn test_opaque_type_identity_skips_lifetimes_and_generic_parameters() {
        let generic_names = [format_ident!("T")];
        assert!(type_is_static_concrete(&parse_quote!(Option<Inner>), &[]));
        assert!(!type_is_static_concrete(&parse_quote!(&'a Inner), &[]));
        assert!(!type_is_static_concrete(&parse_quote!(Option<T>), &generic_names));
    }

    #[test]
    fn test_opaque_field_codegen_retains_only_concrete_static_type_ids() {
        let mut opaque = FieldIr {
            variant_index: None,
            index: Located::new(0, Span::call_site()),
            ty: parse_quote!(Inner),
            occurrences: vec![FieldOccurrence::Opaque],
            keep_serializing: true,
            named: true,
            rust_name: Some(format_ident!("hidden")),
            visibility: Visibility::Inherited,
        };
        let concrete = expand_field_with_parameters(&opaque, &quote!(runtime), None, None, &[], None).to_string();
        assert!(
            concrete.contains("opaque_type_id : Option < :: std :: any :: TypeId > = Some")
                && concrete.contains("TypeId :: of :: < Inner >"),
            "monomorphic opaque fields retain exact TypeId: {concrete}"
        );

        opaque.ty = parse_quote!(&'a Inner);
        let borrowed = expand_field_with_parameters(&opaque, &quote!(runtime), None, None, &[], None).to_string();
        assert!(borrowed.contains("let opaque_type_id : Option < :: std :: any :: TypeId > = None"));

        opaque.ty = parse_quote!(T);
        let generic_parameter = format_ident!("T");
        let generic = expand_field_with_parameters(
            &opaque,
            &quote!(runtime),
            Some(false),
            Some(&quote!("fixture::Generic")),
            &[generic_parameter],
            None,
        )
        .to_string();
        assert!(generic.contains("let opaque_type_id : Option < :: std :: any :: TypeId > = None"));
    }

    #[test]
    fn test_generated_field_metadata_uses_v4_expression_facts_without_legacy_handles() {
        let field = FieldIr {
            variant_index: None,
            index: Located::new(0, Span::call_site()),
            ty: parse_quote!(Option<&'static crate::model::Value<u32>>),
            occurrences: vec![],
            keep_serializing: true,
            named: true,
            rust_name: Some(Ident::new("value", Span::call_site())),
            visibility: Visibility::Inherited,
        };
        let generated = expand_field(&field, &quote!(descriptor_fields), &quote!(runtime), None, None).to_string();
        assert!(
            generated.contains("codegen_v4 :: expression :: ExpressionData"),
            "{generated}"
        );
        assert!(!generated.contains("TypeRef"), "{generated}");
        assert!(!generated.contains("codegen_v3"), "{generated}");
    }
    fn assert_tokens_contain(tokens: &TokenStream, expected: &str) {
        fn token_matches(left: &TokenTree, right: &TokenTree) -> bool {
            match (left, right) {
                (TokenTree::Group(left), TokenTree::Group(right)) => {
                    left.delimiter() == right.delimiter()
                        && token_slices_match(
                            &left.stream().into_iter().collect::<Vec<_>>(),
                            &right.stream().into_iter().collect::<Vec<_>>(),
                        )
                }
                (TokenTree::Ident(left), TokenTree::Ident(right)) => left == right,
                (TokenTree::Punct(left), TokenTree::Punct(right)) => {
                    left.as_char() == right.as_char() && left.spacing() == right.spacing()
                }
                (TokenTree::Literal(left), TokenTree::Literal(right)) => left.to_string() == right.to_string(),
                _ => false,
            }
        }

        fn token_slices_match(left: &[TokenTree], right: &[TokenTree]) -> bool {
            left.len() == right.len() && left.iter().zip(right).all(|(left, right)| token_matches(left, right))
        }

        fn contains_token_sequence(actual: &[TokenTree], expected: &[TokenTree]) -> bool {
            if expected.is_empty() {
                return true;
            }
            actual.windows(expected.len()).any(|window| {
                window
                    .iter()
                    .zip(expected)
                    .all(|(actual, expected)| token_matches(actual, expected))
            }) || actual.iter().any(|token| {
                let TokenTree::Group(group) = token else {
                    return false;
                };
                let nested: Vec<_> = group.stream().into_iter().collect();
                contains_token_sequence(&nested, expected)
            })
        }

        let expected_tokens: TokenStream = expected
            .parse()
            .unwrap_or_else(|error| panic!("invalid expected token pattern `{expected}`: {error}"));
        let actual: Vec<_> = tokens.clone().into_iter().collect();
        let expected: Vec<_> = expected_tokens.into_iter().collect();
        assert!(
            contains_token_sequence(&actual, &expected),
            "expected token sequence `{}` in tokens: {tokens}",
            expected.iter().cloned().collect::<TokenStream>()
        );
    }

    /// Known unsupported unique sequence shapes fail at derive expansion.
    #[test]
    fn test_unique_items_adapter_rejects_known_non_vec_shape() {
        let runtime = quote!(runtime);
        let field = FieldIr {
            variant_index: None,
            index: Located::new(0, Span::call_site()),
            ty: parse_quote!(std::collections::VecDeque<String>),
            occurrences: vec![],
            keep_serializing: true,
            named: true,
            rust_name: Some(format_ident!("field")),
            visibility: Visibility::Inherited,
        };
        let constraint = ConstraintIr::Sequence {
            min: None,
            max: None,
            unique: true,
        };
        let tokens = expand_collection_ops(&field, &[&constraint], &runtime, false).to_string();
        assert!(
            tokens.contains("compile_error"),
            "expected a derive-time diagnostic: {tokens}"
        );
        assert!(
            tokens.contains("unique_items"),
            "diagnostic identifies the declaration: {tokens}"
        );
    }

    /// Exercises all code-generation branches for field metadata primitives.
    #[test]
    fn test_expand_field_metadata_branches() {
        let runtime: TokenStream = quote!(runtime);
        let literal = |value: &str| LitStr::new(value, Span::call_site());

        for allowed in [
            "unicode",
            "printable_unicode",
            "ascii",
            "printable_ascii",
            "code",
            "invalid",
        ] {
            for format in [
                None,
                Some("email_ascii"),
                Some("cn_mobile"),
                Some("uri"),
                Some("uuid"),
                Some("invalid"),
            ] {
                let value = ConstraintIr::Text(TextConstraintIr {
                    min_chars: Some(1),
                    max_chars: Some(2),
                    min_bytes: Some(1),
                    max_bytes: Some(4),
                    allowed_chars: Some(allowed.to_owned()),
                    non_blank: true,
                    format: format.map(str::to_owned),
                });
                let tokens = expand_constraint(&value, false, false, false, &runtime);
                let expected_allowed = match allowed {
                    "unicode" => "AllowedChars::Unicode",
                    "printable_unicode" => "AllowedChars::PrintableUnicode",
                    "ascii" => "AllowedChars::Ascii",
                    "printable_ascii" => "AllowedChars::PrintableAscii",
                    "code" => "AllowedChars::Code",
                    _ => "compile_error!",
                };
                assert_tokens_contain(&tokens, expected_allowed);
                let expected_format = match format {
                    None => "None",
                    Some("email_ascii") => "TextFormat::EmailAscii",
                    Some("cn_mobile") => "TextFormat::Mobile",
                    Some("uri") => "TextFormat::Uri",
                    Some("uuid") => "TextFormat::Uuid",
                    _ => "compile_error!",
                };
                assert_tokens_contain(&tokens, expected_format);
            }
        }
        for rounding in [
            "down",
            "up",
            "ceiling",
            "floor",
            "half_up",
            "half_down",
            "half_even",
            "unnecessary",
            "invalid",
        ] {
            let expected_rounding = match rounding {
                "down" => "RoundingMode::Down",
                "up" => "RoundingMode::Up",
                "ceiling" => "RoundingMode::Ceiling",
                "floor" => "RoundingMode::Floor",
                "half_up" => "RoundingMode::HalfUp",
                "half_down" => "RoundingMode::HalfDown",
                "half_even" => "RoundingMode::HalfEven",
                "unnecessary" => "RoundingMode::Unnecessary",
                _ => "compile_error!",
            };
            assert_tokens_contain(&rounding_tokens(rounding, &runtime), expected_rounding);
            let value = ConstraintIr::Decimal(DecimalConstraintIr {
                precision: Some(8),
                scale: 2,
                rounding: rounding.to_owned(),
                money: rounding == "unnecessary",
                min: Some(literal("1.0")),
                max: Some(literal("2.0")),
                min_inclusive: false,
                max_inclusive: true,
            });
            let tokens = expand_constraint(&value, false, false, false, &runtime);
            assert_tokens_contain(&tokens, expected_rounding);
            assert_tokens_contain(
                &tokens,
                if rounding == "unnecessary" {
                    "DecimalSemantic::Money"
                } else {
                    "DecimalSemantic::Number"
                },
            );
        }
        for precision in ["second", "millisecond", "microsecond", "nanosecond", "invalid"] {
            let value = ConstraintIr::Time(precision.to_owned());
            let tokens = expand_constraint(&value, false, false, false, &runtime);
            let expected_precision = match precision {
                "second" => "TemporalPrecision::Second",
                "millisecond" => "TemporalPrecision::Millisecond",
                "microsecond" => "TemporalPrecision::Microsecond",
                "nanosecond" => "TemporalPrecision::Nanosecond",
                _ => "compile_error!",
            };
            assert_tokens_contain(&tokens, expected_precision);
        }
        let sequence = ConstraintIr::Sequence {
            min: Some(1),
            max: Some(2),
            unique: true,
        };
        let sequence_tokens = expand_constraint(&sequence, true, false, false, &runtime);
        assert_tokens_contain(&sequence_tokens, "SequenceConstraint::new");
        assert_tokens_contain(&sequence_tokens, "Some(1usize)");
        assert_tokens_contain(&sequence_tokens, "Some(2usize)");
        assert_tokens_contain(&sequence_tokens, "true");
        let map = ConstraintIr::Map {
            min: Some(1),
            max: Some(2),
        };
        let map_tokens = expand_constraint(&map, false, true, true, &runtime);
        assert_tokens_contain(&map_tokens, "MapConstraint::new");
        assert_tokens_contain(&map_tokens, "with_selectors");
        assert_tokens_contain(&map_tokens, "Some(map_key_selector)");
        assert_tokens_contain(&map_tokens, "Some(map_value_selector)");

        let strategy_values = [
            StrategyArgumentIr::Bool(true),
            StrategyArgumentIr::Integer(-1),
            StrategyArgumentIr::Unsigned(1),
            StrategyArgumentIr::String(literal("value")),
            StrategyArgumentIr::BoolList(vec![true]),
            StrategyArgumentIr::IntegerList(vec![-1]),
            StrategyArgumentIr::UnsignedList(vec![1]),
            StrategyArgumentIr::StringList(vec![literal("value")]),
        ];
        for value in &strategy_values {
            let expected = match value {
                StrategyArgumentIr::Bool(_) => "ValidationArgument::Bool",
                StrategyArgumentIr::Integer(_) => "ValidationArgument::Integer",
                StrategyArgumentIr::Unsigned(_) => "ValidationArgument::Unsigned",
                StrategyArgumentIr::String(_) => "ValidationArgument::String",
                StrategyArgumentIr::BoolList(_) => "ValidationArgument::BoolList",
                StrategyArgumentIr::IntegerList(_) => "ValidationArgument::IntegerList",
                StrategyArgumentIr::UnsignedList(_) => "ValidationArgument::UnsignedList",
                StrategyArgumentIr::StringList(_) => "ValidationArgument::StringList",
            };
            assert_tokens_contain(&expand_strategy_argument(value, &runtime), expected);
        }
        let validator = ValidatorIr {
            id: literal("example.rule"),
            params: strategy_values
                .into_iter()
                .enumerate()
                .map(|(index, value)| (format!("value_{index}"), value))
                .collect(),
            depends_on: vec![vec!["owner".to_owned(), "id".to_owned()]],
            dependency_bindings: Vec::new(),
        };
        let validator_tokens = expand_validator(
            &validator,
            &runtime,
            quote!(&model_runtime::metadata::DeclarationLocation::unknown()),
        );
        assert_tokens_contain(&validator_tokens, "\"example.rule\"");
        assert_tokens_contain(&validator_tokens, "NamedValidationArgument::new");
        assert_tokens_contain(&validator_tokens, "PropertyPath::new");
        assert_tokens_contain(&validator_tokens, "\"owner\"");
        assert_tokens_contain(&validator_tokens, "\"id\"");

        let ty: Type = parse_quote!(Codec);
        let references = [
            ReferenceIr {
                target: ReferenceTargetIr::RustType(Box::new(parse_quote!(Owner))),
                property: None,
                existing: true,
                same_as: None,
            },
            ReferenceIr {
                target: ReferenceTargetIr::ModelId(literal("example.Owner")),
                property: Some(vec!["id".to_owned()]),
                existing: false,
                same_as: Some(vec!["owner".to_owned(), "id".to_owned()]),
            },
        ];
        for reference in &references {
            let tokens = expand_reference(reference, &runtime);
            assert_tokens_contain(
                &tokens,
                match &reference.target {
                    ReferenceTargetIr::RustType(_) => "DeclaredEntityTarget::RustType",
                    ReferenceTargetIr::ModelId(_) => "DeclaredEntityTarget::ModelId",
                },
            );
            assert_tokens_contain(
                &tokens,
                match &reference.property {
                    Some(_) => "ReferenceSelection::Property",
                    None => "ReferenceSelection::Entity",
                },
            );
        }
        let rust_type_codec = codec_reference_expression(&CodecIr::RustType(Box::new(ty)), &quote!(String), &runtime);
        assert_tokens_contain(&rust_type_codec, "CodecReference::RustType");
        let declared_id_codec = codec_reference_expression(
            &CodecIr::DeclaredId(literal("example.codec")),
            &quote!(String),
            &runtime,
        );
        assert_tokens_contain(&declared_id_codec, "CodecReference::DeclaredId");

        for mode in [
            RedactModeIr::Level("public".to_owned()),
            RedactModeIr::Level("personal".to_owned()),
            RedactModeIr::Level("confidential".to_owned()),
            RedactModeIr::Level("secret".to_owned()),
            RedactModeIr::Level("invalid".to_owned()),
            RedactModeIr::Skip,
            RedactModeIr::Nested,
            RedactModeIr::Map,
            RedactModeIr::KeyedBy("owner".to_owned()),
            RedactModeIr::Json,
        ] {
            let expected = match &mode {
                RedactModeIr::Level(level) if level == "invalid" => "compile_error!",
                RedactModeIr::Level(_) => "RedactModeMetadata::Level",
                RedactModeIr::Skip => "RedactModeMetadata::Skip",
                RedactModeIr::Nested => "RedactModeMetadata::Nested",
                RedactModeIr::Map => "RedactModeMetadata::Map",
                RedactModeIr::KeyedBy(_) => "RedactModeMetadata::KeyedBy",
                RedactModeIr::Json => "RedactModeMetadata::Json",
            };
            assert_tokens_contain(&expand_redact(&RedactIr { mode }, quote!(position), &runtime), expected);
        }

        let serde_values = [
            SerdeIr::default(),
            SerdeIr {
                serialize_name: Some(literal("out")),
                deserialize_name: Some(literal("in")),
                skip_serializing: true,
                skip_deserializing: true,
                flatten: true,
                with: Some(literal("helper")),
                default: true,
                explicit_skip_serializing_if: false,
                default_from_model: true,
                omit_from_model: true,
                omit_suppressed: false,
            },
            SerdeIr {
                default: true,
                explicit_skip_serializing_if: true,
                omit_suppressed: true,
                ..SerdeIr::default()
            },
        ];
        let default_serde = expand_serde(&serde_values[0], &runtime);
        assert_tokens_contain(&default_serde, "SerdeFieldMetadata::new");
        let model_default_serde = expand_serde(&serde_values[1], &runtime);
        assert_tokens_contain(
            &model_default_serde,
            "SerdeFieldMetadata::new(Some(\"out\"),Some(\"in\"),true,true,true,Some(\"helper\"),true).with_sources(runtime::metadata::SerdeBehaviorSource::ModelDefault,runtime::metadata::SerdeBehaviorSource::ModelDefault)",
        );
        let suppressed_serde = expand_serde(&serde_values[2], &runtime);
        assert_tokens_contain(
            &suppressed_serde,
            "SerdeFieldMetadata::new(None,None,false,false,false,None,true).with_sources(runtime::metadata::SerdeBehaviorSource::Explicit,runtime::metadata::SerdeBehaviorSource::Suppressed)",
        );
    }
}
