// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Orchestrates declaration parsing across role, field, and variant syntax.

use heck::ToShoutySnakeCase;
use syn::Attribute;
use syn::Data;
use syn::DataEnum;
use syn::DeriveInput;
use syn::Error;
use syn::Fields;
use syn::LitStr;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::punctuated::Punctuated;

use super::fields::parse_serde;
use super::fields::validate_ascii_id;
use crate::ir::MacroKind;
use crate::ir::declaration::DeclarationIr;
use crate::ir::declaration::DeclarationOptions;
use crate::ir::declaration::FieldIr;
use crate::ir::declaration::VariantIr;
use crate::validate::declaration::combine;

/// Parses a declaration for one model role without normalizing or validating
/// it.
pub(crate) fn parse_declaration(
    kind: MacroKind,
    options: Punctuated<Meta, Token![,]>,
    item: &DeriveInput,
) -> Result<DeclarationIr> {
    let mut errors = None;
    let options = match DeclarationOptions::parse(options) {
        Ok(options) => Some(options),
        Err(error) => {
            combine(&mut errors, error);
            None
        }
    };
    let (fields, variants) = match &item.data {
        Data::Struct(data) => match parse_fields(&data.fields) {
            Ok(fields) => (Some(fields), Some(Vec::new())),
            Err(error) => {
                combine(&mut errors, error);
                (None, None)
            }
        },
        Data::Enum(data) => match parse_variants(data) {
            Ok(variants) => (Some(Vec::new()), Some(variants)),
            Err(error) => {
                combine(&mut errors, error);
                (None, None)
            }
        },
        Data::Union(_) => {
            combine(
                &mut errors,
                Error::new_spanned(item, "model role macros do not support unions"),
            );
            (None, None)
        }
    };
    if let Some(error) = errors {
        return Err(error);
    }
    let options = options.expect("errors returned when declaration options are unavailable");
    let fields = fields.expect("errors returned when fields are unavailable");
    let variants = variants.expect("errors returned when variants are unavailable");
    if let Some(id) = &options.id {
        validate_ascii_id(id, "model ID")?;
    }
    if let Some(source_id) = &options.source_id {
        validate_ascii_id(source_id, "Projection source ID")?;
    }
    Ok(DeclarationIr {
        kind,
        options,
        fields,
        variants,
    })
}
/// Parses every field and combines independent field diagnostics.
pub(crate) fn parse_fields(fields: &Fields) -> Result<Vec<FieldIr>> {
    let mut parsed = Vec::new();
    let mut errors = None;
    for (index, field) in fields.iter().enumerate() {
        match FieldIr::parse(index, &field.ty, &field.attrs, field.ident.is_some()) {
            Ok(field) => parsed.push(field),
            Err(error) => combine(&mut errors, error),
        }
    }
    match errors {
        Some(error) => Err(error),
        None => Ok(parsed),
    }
}

/// Parses enum variants, including Serde names and nested field metadata.
pub(crate) fn parse_variants(data: &DataEnum) -> Result<Vec<VariantIr>> {
    let mut parsed = Vec::new();
    let mut errors = None;
    for variant in &data.variants {
        let default_name = variant.ident.to_string().to_shouty_snake_case();
        let canonical_name = parse_variant_name(&variant.attrs, &default_name);
        let names = parse_variant_serde_names(&variant.attrs, canonical_name.as_deref().unwrap_or(&default_name));
        let fields = parse_fields(&variant.fields);
        match (canonical_name, names, fields) {
            (Ok(canonical_name), Ok((serialized_name, deserialized_name)), Ok(fields)) => parsed.push(VariantIr {
                rust_name: variant.ident.to_string(),
                canonical_name,
                serialized_name,
                deserialized_name,
                default: variant
                    .attrs
                    .iter()
                    .any(|attribute| attribute.path().is_ident("default")),
                fields,
            }),
            (canonical_name, names, fields) => {
                if let Err(error) = canonical_name {
                    combine(&mut errors, error);
                }
                if let Err(error) = names {
                    combine(&mut errors, error);
                }
                if let Err(error) = fields {
                    combine(&mut errors, error);
                }
            }
        }
    }
    match errors {
        Some(error) => Err(error),
        None => Ok(parsed),
    }
}

/// Parses an optional stable variant name, defaulting to the Rust name.
fn parse_variant_name(attributes: &[Attribute], default: &str) -> Result<String> {
    let mut name = None;
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("variant"))
    {
        attribute.parse_nested_meta(|meta| {
            if !meta.path.is_ident("name") {
                return Err(meta.error("unsupported variant option"));
            }
            let value: LitStr = meta.value()?.parse()?;
            validate_ascii_id(&value, "variant name")?;
            if value.value().is_empty() {
                return Err(Error::new_spanned(value, "variant name cannot be empty"));
            }
            if name.replace(value.value()).is_some() {
                return Err(meta.error("duplicate variant `name` option"));
            }
            Ok(())
        })?;
    }
    Ok(name.unwrap_or_else(|| default.to_owned()))
}

/// Parses variant rename attributes and returns serialized/deserialized names.
fn parse_variant_serde_names(attributes: &[Attribute], canonical: &str) -> Result<(String, String)> {
    let mut serialize = canonical.to_owned();
    let mut deserialize = canonical.to_owned();
    for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("serde")) {
        let value = parse_serde(attribute)?;
        if let Some(name) = value.serialize_name {
            serialize = name.value();
        }
        if let Some(name) = value.deserialize_name {
            deserialize = name.value();
        }
    }
    Ok((serialize, deserialize))
}
