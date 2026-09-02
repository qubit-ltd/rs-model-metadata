// =============================================================================

use super::declaration_ir::DeclarationIr;
use super::declaration_ir::FieldOccurrence;
use super::declaration_ir::RedactIr;
use super::declaration_ir::RedactModeIr;
use super::declaration_ir::SelectorIr;
use super::declaration_ir::SelectorPositionIr;
use super::MacroKind;
use syn::Attribute;
use syn::Data;
use syn::DeriveInput;
use syn::Error;
use syn::Fields;
use syn::GenericParam;
use syn::LitStr;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parse_quote;
use syn::punctuated::Punctuated;
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Validates the declaration shape before constructing intermediate metadata.
pub(super) fn validate_declaration(kind: MacroKind, item: &DeriveInput) -> Result<()> {
    let mut errors = None;
    for parameter in &item.generics.params {
        if matches!(parameter, GenericParam::Lifetime(_)) {
            combine(
                &mut errors,
                Error::new_spanned(parameter, "model roles do not support lifetime parameters"),
            );
        }
        if let GenericParam::Const(parameter) = parameter {
            let supported = matches!(&parameter.ty, Type::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && matches!(
                    path.path.segments[0].ident.to_string().as_str(),
                    "bool" | "char" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
                        | "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
                ));
            if !supported {
                combine(
                    &mut errors,
                    Error::new_spanned(
                        &parameter.ty,
                        "model const parameters require a primitive integer, bool, or char type",
                    ),
                );
            }
        }
    }
    if matches!(kind, MacroKind::Entity | MacroKind::Projection)
        && (!item.generics.params.is_empty() || item.generics.where_clause.is_some())
    {
        combine(
            &mut errors,
            Error::new_spanned(
                &item.generics,
                "Entity and Projection declarations cannot be generic",
            ),
        );
    }

    match (kind, &item.data) {
        (MacroKind::Entity | MacroKind::Projection, Data::Struct(data)) => {
            if !matches!(data.fields, Fields::Named(_)) {
                combine(
                    &mut errors,
                    Error::new_spanned(&data.fields, "Entity and Projection require named fields"),
                );
            }
        }
        (MacroKind::Model, Data::Struct(data)) => {
            if matches!(data.fields, Fields::Unnamed(_)) {
                combine(
                    &mut errors,
                    Error::new_spanned(&data.fields, "Model does not support tuple structs"),
                );
            }
        }
        (MacroKind::Enum, Data::Enum(_)) => {}
        (MacroKind::Value, Data::Struct(data)) => {
            let valid_shape = match &data.fields {
                Fields::Named(fields) => !fields.named.is_empty(),
                Fields::Unnamed(fields) => fields.unnamed.len() == 1,
                Fields::Unit => false,
            };
            if !valid_shape {
                combine(
                    &mut errors,
                    Error::new_spanned(
                        &data.fields,
                        "Value requires named fields or one tuple field",
                    ),
                );
            }
        }
        (_, Data::Union(data)) => combine(
            &mut errors,
            Error::new_spanned(data.union_token, "model macros do not support unions"),
        ),
        (MacroKind::Enum, _) => combine(
            &mut errors,
            Error::new_spanned(&item.ident, "Enum only supports enum declarations"),
        ),
        (_, Data::Enum(_)) => combine(
            &mut errors,
            Error::new_spanned(&item.ident, "this model role requires a struct declaration"),
        ),
        _ => {}
    }

    if let Some(error) = errors {
        Err(error)
    } else {
        Ok(())
    }
}

/// Rejects user reflection derives that would duplicate generated metadata.
pub(super) fn reject_duplicate_reflect(attributes: &[Attribute]) -> Result<()> {
    for attribute in attributes {
        if !attribute.path().is_ident("derive") {
            continue;
        }
        let derives =
            attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)?;
        if let Some(path) = derives.iter().find(|path| {
            path.segments
                .last()
                .is_some_and(|segment| segment.ident == "Reflect")
        }) {
            return Err(Error::new_spanned(
                path,
                "model macros generate Reflect; remove the duplicate derive",
            ));
        }
    }
    Ok(())
}

/// Rewrites helper attributes used to expose nested field metadata.
pub(super) fn rewrite_field_helpers(data: &mut Data, declaration: &DeclarationIr) {
    let fields: Vec<_> = match data {
        Data::Struct(data) => data.fields.iter_mut().zip(&declaration.fields).collect(),
        Data::Enum(data) => data
            .variants
            .iter_mut()
            .zip(&declaration.variants)
            .flat_map(|(variant, ir)| variant.fields.iter_mut().zip(&ir.fields))
            .collect(),
        Data::Union(_) => Vec::new(),
    };
    for (field, ir) in fields {
        let opaque = field
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("opaque"));
        let element_level = ir
            .occurrences
            .iter()
            .find_map(|occurrence| match occurrence {
                FieldOccurrence::Selector(SelectorIr {
                    position: SelectorPositionIr::Element,
                    redact:
                        Some(RedactIr {
                            mode: RedactModeIr::Level(level),
                        }),
                    ..
                }) => Some(level.clone()),
                _ => None,
            });
        let map_key_level = ir
            .occurrences
            .iter()
            .find_map(|occurrence| match occurrence {
                FieldOccurrence::Selector(SelectorIr {
                    position: SelectorPositionIr::MapKey,
                    redact:
                        Some(RedactIr {
                            mode: RedactModeIr::Level(level),
                        }),
                    ..
                }) => Some(level.clone()),
                _ => None,
            });
        let map_value_level = ir
            .occurrences
            .iter()
            .find_map(|occurrence| match occurrence {
                FieldOccurrence::Selector(SelectorIr {
                    position: SelectorPositionIr::MapValue,
                    redact:
                        Some(RedactIr {
                            mode: RedactModeIr::Level(level),
                        }),
                    ..
                }) => Some(level.clone()),
                _ => None,
            });
        field.attrs.retain(|attribute| {
            if attribute.path().is_ident("redact") {
                !declaration.options.no_redact
            } else {
                !is_model_field_helper(attribute)
            }
        });
        if let Some(level) =
            element_level.or_else(|| map_value_level.clone().filter(|_| map_key_level.is_none()))
        {
            let level = LitStr::new(&level, proc_macro2::Span::call_site());
            field.attrs.push(parse_quote!(#[redact(level = #level)]));
        }
        if let Some(key_level) = map_key_level {
            let key_level = LitStr::new(&key_level, proc_macro2::Span::call_site());
            if let Some(value_level) = map_value_level {
                let value_level = LitStr::new(&value_level, proc_macro2::Span::call_site());
                field
                    .attrs
                    .push(parse_quote!(#[redact(map_key_level = #key_level, map_value_level = #value_level)]));
            } else {
                field
                    .attrs
                    .push(parse_quote!(#[redact(map_key_level = #key_level)]));
            }
        }
        if opaque {
            field.attrs.push(parse_quote!(#[reflect(opaque)]));
        }
    }
    if let Data::Enum(data) = data {
        for variant in &mut data.variants {
            variant.attrs.retain(|attribute| !attribute.path().is_ident("variant"));
        }
    }
}

/// Reports whether an attribute is an internal model-field helper.
fn is_model_field_helper(attribute: &Attribute) -> bool {
    let Some(name) = attribute.path().get_ident().map(ToString::to_string) else {
        return false;
    };
    matches!(
        name.as_str(),
        "identifier"
            | "indexed"
            | "unique"
            | "reference"
            | "key_part"
            | "text"
            | "decimal"
            | "money"
            | "time"
            | "sequence"
            | "map"
            | "element"
            | "map_key"
            | "map_value"
            | "validator"
            | "codec"
            | "redact"
            | "opaque"
            | "keep_serializing"
    )
}

/// Combines one diagnostic into an existing optional error accumulator.
pub(super) fn combine(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(current) => current.combine(error),
        None => *errors = Some(error),
    }
}
