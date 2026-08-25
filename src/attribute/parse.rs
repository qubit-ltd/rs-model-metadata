// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parsing for `#[model(...)]` attributes.

use std::fmt::Display;
use std::str::FromStr;

use proc_macro2::Span;
use proc_macro2::TokenTree;
use syn::Attribute;
use syn::Error;
use syn::Ident;
use syn::LitBool;
use syn::LitInt;
use syn::LitStr;
use syn::Path;
use syn::Result;
use syn::Token;
use syn::bracketed;
use syn::ext::IdentExt;
use syn::meta::ParseNestedMeta;
use syn::parse::ParseStream;
use syn::parse_str;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Paren;

use super::decimal_attribute::DecimalAttribute;
use super::element_attribute::ElementAttribute;
use super::element_constraint_attribute::ElementConstraintAttribute;
use super::field_attribute::FieldAttribute;
use super::field_name::FieldName;
use super::field_unique_attribute::FieldUniqueAttribute;
use super::identifier_attribute::IdentifierAttribute;
use super::lookup_relation_attribute::LookupRelationAttribute;
use super::map_attribute::MapAttribute;
use super::model_attribute::ModelAttribute;
use super::model_attributes::ModelAttributes;
use super::named_fields_attribute::NamedFieldsAttribute;
use super::ownership_attribute::OwnershipAttribute;
use super::primary_key_attribute::PrimaryKeyAttribute;
use super::reference_attribute::ReferenceAttribute;
use super::reference_attribute::ReferencePathSegment;
use super::rounding_mode::RoundingMode;
use super::sequence_attribute::SequenceAttribute;
use super::spanned_value::SpannedValue;
use super::strategy_attribute::StrategyAttribute;
use super::temporal_attribute::TemporalAttribute;
use super::temporal_precision::TemporalPrecision;
use super::text_attribute::TextAttribute;
use super::text_format::TextFormat;
use super::text_repertoire::TextRepertoire;

/// Parses every model-level `#[model(...)]` item in source order.
///
/// Returns an error when an item is unknown or its value cannot be parsed.
pub(crate) fn parse_model_attributes(attributes: &[Attribute]) -> Result<ModelAttributes> {
    let mut parsed = Vec::new();
    let mut id = Vec::new();
    let mut errors = None;
    for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("model")) {
        let result = attribute.parse_nested_meta(|meta| {
            let input = meta.input;
            if let Err(error) = parse_model_attribute(meta, &mut id, &mut parsed) {
                combine_error(&mut errors, error);
                discard_nested_meta_input(input)?;
            }
            Ok(())
        });
        if let Err(error) = result {
            combine_error(&mut errors, error);
        }
    }
    if let Some(error) = errors {
        Err(error)
    } else {
        Ok(ModelAttributes { id, attributes: parsed })
    }
}

/// Parses every field-level `#[model(...)]` item in source order.
///
/// Returns an error when an item is unknown or its value cannot be parsed.
pub(crate) fn parse_field_attributes(attributes: &[Attribute]) -> Result<Vec<FieldAttribute>> {
    let mut parsed = Vec::new();
    let mut errors = None;
    for attribute in attributes.iter().filter(|attribute| attribute.path().is_ident("model")) {
        let result = attribute.parse_nested_meta(|meta| {
            let input = meta.input;
            if let Err(error) = parse_field_attribute(meta, &mut parsed) {
                combine_error(&mut errors, error);
                discard_nested_meta_input(input)?;
            }
            Ok(())
        });
        if let Err(error) = result {
            combine_error(&mut errors, error);
        }
    }
    if let Some(error) = errors {
        Err(error)
    } else {
        Ok(parsed)
    }
}

/// Parses one model-level nested item and appends its syntax node.
fn parse_model_attribute(
    meta: ParseNestedMeta<'_>,
    id: &mut Vec<LitStr>,
    parsed: &mut Vec<ModelAttribute>,
) -> Result<()> {
    if meta.path.is_ident("id") {
        id.push(meta.value()?.parse()?);
    } else if meta.path.is_ident("textual") {
        if parsed
            .iter()
            .any(|attribute| matches!(attribute, ModelAttribute::Textual))
        {
            return Err(meta.error("duplicate `textual` model capability"));
        }
        if meta.input.peek(Paren) || meta.input.peek(Token![=]) {
            return Err(meta.error("`textual` does not accept arguments"));
        }
        parsed.push(ModelAttribute::Textual);
    } else if meta.path.is_ident("primary_key") {
        parsed.push(ModelAttribute::PrimaryKey(parse_primary_key(meta)?));
    } else if meta.path.is_ident("index") {
        parsed.push(ModelAttribute::Index(parse_named_fields(meta)?));
    } else if meta.path.is_ident("key") {
        parsed.push(ModelAttribute::Key(parse_named_fields(meta)?));
    } else if meta.path.is_ident("ownership") {
        parsed.push(ModelAttribute::Ownership(parse_ownership(meta)?));
    } else if meta.path.is_ident("nullable") {
        return Err(meta.error("`nullable` is not supported; use `Option<T>` for nullability"));
    } else if meta.path.is_ident("computed") {
        return Err(meta.error("`computed` is not supported; declare a real Rust field instead"));
    } else if is_field_attribute_path(&meta.path) {
        return Err(meta.error("this field-level `model` attribute cannot be used on a model"));
    } else {
        return Err(meta.error("unknown model-level `model` attribute"));
    }
    Ok(())
}

/// Parses one field-level nested item and appends its syntax node.
fn parse_field_attribute(meta: ParseNestedMeta<'_>, parsed: &mut Vec<FieldAttribute>) -> Result<()> {
    let span = meta.path.span();
    if meta.path.is_ident("identifier") {
        parsed.push(FieldAttribute::Identifier(parse_identifier(meta)?));
    } else if meta.path.is_ident("unique") {
        parsed.push(FieldAttribute::Unique(parse_field_unique(meta)?));
    } else if meta.path.is_ident("index") {
        parsed.push(FieldAttribute::Index(span));
    } else if meta.path.is_ident("text") {
        parsed.push(FieldAttribute::Text(parse_text(meta)?));
    } else if meta.path.is_ident("sequence") {
        parsed.push(FieldAttribute::Sequence(parse_sequence(meta)?));
    } else if meta.path.is_ident("map") {
        parsed.push(FieldAttribute::Map(parse_map(meta)?));
    } else if meta.path.is_ident("time") {
        parsed.push(FieldAttribute::Temporal(parse_temporal(meta)?));
    } else if meta.path.is_ident("decimal") {
        parsed.push(FieldAttribute::Decimal(parse_decimal(meta)?));
    } else if meta.path.is_ident("money") {
        parsed.push(FieldAttribute::Money(parse_decimal(meta)?));
    } else if meta.path.is_ident("element") {
        parsed.push(FieldAttribute::Element(parse_element(meta)?));
    } else if meta.path.is_ident("reference") {
        parsed.push(FieldAttribute::Reference(parse_reference(meta)?));
    } else if meta.path.is_ident("lookup_relation") {
        parsed.push(FieldAttribute::LookupRelation(parse_lookup_relation(meta)?));
    } else if meta.path.is_ident("codec") {
        parsed.push(FieldAttribute::Codec(parse_strategy(meta)?));
    } else if meta.path.is_ident("generator") {
        parsed.push(FieldAttribute::Generator(parse_strategy(meta)?));
    } else if meta.path.is_ident("opaque") {
        parsed.push(FieldAttribute::Opaque(span));
    } else if meta.path.is_ident("keep_serializing") {
        parsed.push(FieldAttribute::KeepSerializing);
    } else if meta.path.is_ident("nullable") {
        return Err(meta.error("`nullable` is not supported; use `Option<T>` for nullability"));
    } else if meta.path.is_ident("computed") {
        return Err(meta.error("`computed` is not supported; declare a real Rust field instead"));
    } else if is_model_attribute_path(&meta.path) {
        return Err(meta.error("this model-level `model` attribute cannot be used on a field"));
    } else {
        return Err(meta.error("unknown field-level `model` attribute"));
    }
    Ok(())
}

/// Parses `primary_key(fields(...), generated(...))`.
fn parse_primary_key(meta: ParseNestedMeta<'_>) -> Result<PrimaryKeyAttribute> {
    let span = meta.path.span();
    let mut fields = Vec::new();
    let mut generated = Vec::new();
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("fields") {
            fields.extend(parse_field_names(nested)?);
        } else if nested.path.is_ident("generated") {
            generated.extend(parse_field_names(nested)?);
        } else {
            return Err(nested.error("expected `fields(...)` or `generated(...)`"));
        }
        Ok(())
    })?;
    Ok(PrimaryKeyAttribute {
        fields,
        generated,
        span,
    })
}

/// Parses `index(...)` or `key(...)` values.
fn parse_named_fields(meta: ParseNestedMeta<'_>) -> Result<NamedFieldsAttribute> {
    let span = meta.path.span();
    let mut name = Vec::new();
    let mut fields = Vec::new();
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("name") {
            name.push(parse_string(&nested)?);
        } else if nested.path.is_ident("fields") {
            fields.extend(parse_field_names(nested)?);
        } else {
            return Err(nested.error("expected `name` or `fields(...)`"));
        }
        Ok(())
    })?;
    Ok(NamedFieldsAttribute { name, fields, span })
}

/// Parses `ownership(owner = Type)`; `target = Type` is accepted as an alias.
fn parse_ownership(meta: ParseNestedMeta<'_>) -> Result<OwnershipAttribute> {
    let span = meta.path.span();
    let mut owner = Vec::new();
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("owner") || nested.path.is_ident("target") {
            owner.push(nested.value()?.parse()?);
            Ok(())
        } else {
            Err(nested.error("expected `owner = Type`"))
        }
    })?;
    if owner.is_empty() {
        return Err(Error::new(span, "ownership requires `owner = Type`"));
    }
    Ok(OwnershipAttribute { owner, span })
}

/// Parses `identifier` and its optional `generated` marker.
fn parse_identifier(meta: ParseNestedMeta<'_>) -> Result<IdentifierAttribute> {
    let span = meta.path.span();
    let mut generated = Vec::new();
    if meta.input.peek(Paren) {
        meta.parse_nested_meta(|nested| {
            if nested.path.is_ident("generated") {
                generated.push(nested.path.span());
                Ok(())
            } else {
                Err(nested.error("expected `generated`"))
            }
        })?;
    }
    Ok(IdentifierAttribute { generated, span })
}

/// Parses field `unique` and its optional `ignore_case` marker.
fn parse_field_unique(meta: ParseNestedMeta<'_>) -> Result<FieldUniqueAttribute> {
    let span = meta.path.span();
    let mut name = Vec::new();
    let mut respect_to = Vec::new();
    let mut respect_to_count = 0;
    let mut ignore_case = Vec::new();
    let mut ignore_case_values = Vec::new();
    if meta.input.peek(Paren) {
        meta.parse_nested_meta(|nested| {
            if nested.path.is_ident("name") {
                name.push(parse_string(&nested)?);
                Ok(())
            } else if nested.path.is_ident("respectTo") {
                respect_to_count += 1;
                respect_to.extend(parse_field_name_list(&nested)?);
                Ok(())
            } else if nested.path.is_ident("ignoreCase") {
                ignore_case_values.push(parse_bool(&nested)?);
                Ok(())
            } else if nested.path.is_ident("ignore_case") {
                ignore_case.push(nested.path.span());
                Ok(())
            } else {
                Err(nested.error("expected `name`, `respectTo`, `ignoreCase`, or `ignore_case`"))
            }
        })?;
    }
    if !ignore_case.is_empty() && !ignore_case_values.is_empty() {
        return Err(Error::new(
            span,
            "`ignoreCase` and `ignore_case` cannot be used together",
        ));
    }
    if name.len() > 1 {
        return Err(Error::new(name[1].span(), "duplicate `name` argument"));
    }
    if ignore_case_values.len() > 1 {
        return Err(Error::new(
            ignore_case_values[1].span,
            "duplicate `ignoreCase` argument",
        ));
    }
    if ignore_case.len() > 1 {
        return Err(Error::new(ignore_case[1], "duplicate `ignore_case` argument"));
    }
    if respect_to_count > 1 {
        return Err(Error::new(span, "duplicate `respectTo` argument"));
    }
    Ok(FieldUniqueAttribute {
        name,
        respect_to,
        ignore_case_values,
        legacy_ignore_case: !ignore_case.is_empty(),
        span,
    })
}

/// Parses a comma-separated bracketed list of Rust field identifiers.
fn parse_field_name_list(meta: &ParseNestedMeta<'_>) -> Result<Vec<FieldName>> {
    let input = meta.value()?;
    let content;
    bracketed!(content in input);
    Punctuated::<Ident, Token![,]>::parse_terminated(&content)
        .map(|idents| idents.iter().map(field_name_from_ident).collect())
}

/// Parses text constraint arguments.
fn parse_text(meta: ParseNestedMeta<'_>) -> Result<TextAttribute> {
    let span = meta.path.span();
    let mut value = TextAttribute {
        min_chars: Vec::new(),
        max_chars: Vec::new(),
        min_bytes: Vec::new(),
        max_bytes: Vec::new(),
        repertoire: Vec::new(),
        non_blank: Vec::new(),
        format: Vec::new(),
        span,
    };
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("min_chars") {
            value.min_chars.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("max_chars") {
            value.max_chars.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("min_bytes") {
            value.min_bytes.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("max_bytes") {
            value.max_bytes.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("repertoire") {
            let ident = parse_ident(&nested)?;
            let repertoire = match ident.to_string().as_str() {
                "unicode" => TextRepertoire::Unicode,
                "ascii" => TextRepertoire::Ascii,
                _ => return Err(nested.error("expected `unicode` or `ascii`")),
            };
            value.repertoire.push(SpannedValue {
                value: repertoire,
                span: ident.span(),
            });
        } else if nested.path.is_ident("non_blank") {
            value.non_blank.push(nested.path.span());
        } else if nested.path.is_ident("format") {
            let ident = parse_ident(&nested)?;
            let format = match ident.to_string().as_str() {
                "email" => TextFormat::Email,
                "mobile" => TextFormat::Mobile,
                "uri" => TextFormat::Uri,
                "uuid" => TextFormat::Uuid,
                _ => {
                    return Err(nested.error("expected `email`, `mobile`, `uri`, or `uuid`"));
                }
            };
            value.format.push(SpannedValue {
                value: format,
                span: ident.span(),
            });
        } else {
            return Err(nested.error("unknown `text` argument"));
        }
        Ok(())
    })?;
    Ok(value)
}

/// Parses ordered-sequence constraint arguments.
fn parse_sequence(meta: ParseNestedMeta<'_>) -> Result<SequenceAttribute> {
    let span = meta.path.span();
    let mut value = SequenceAttribute {
        min_items: Vec::new(),
        max_items: Vec::new(),
        unique_items: Vec::new(),
        span,
    };
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("min_items") {
            value.min_items.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("max_items") {
            value.max_items.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("unique_items") {
            value.unique_items.push(nested.path.span());
        } else {
            return Err(nested.error("unknown `sequence` argument"));
        }
        Ok(())
    })?;
    Ok(value)
}

/// Parses map constraint arguments.
fn parse_map(meta: ParseNestedMeta<'_>) -> Result<MapAttribute> {
    let span = meta.path.span();
    let mut value = MapAttribute {
        min_entries: Vec::new(),
        max_entries: Vec::new(),
        span,
    };
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("min_entries") {
            value.min_entries.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("max_entries") {
            value.max_entries.push(parse_integer(&nested)?);
        } else {
            return Err(nested.error("unknown `map` argument"));
        }
        Ok(())
    })?;
    Ok(value)
}

/// Parses temporal constraint arguments.
fn parse_temporal(meta: ParseNestedMeta<'_>) -> Result<TemporalAttribute> {
    let span = meta.path.span();
    let mut value = TemporalAttribute {
        precision: Vec::new(),
        span,
    };
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("precision") {
            let ident = parse_ident(&nested)?;
            let precision = match ident.to_string().as_str() {
                "second" => TemporalPrecision::Second,
                "millisecond" => TemporalPrecision::Millisecond,
                "microsecond" => TemporalPrecision::Microsecond,
                "nanosecond" => TemporalPrecision::Nanosecond,
                _ => return Err(nested.error("unknown temporal precision")),
            };
            value.precision.push(SpannedValue {
                value: precision,
                span: ident.span(),
            });
        } else {
            return Err(nested.error("unknown `time` argument"));
        }
        Ok(())
    })?;
    Ok(value)
}

/// Parses decimal or monetary constraint arguments.
fn parse_decimal(meta: ParseNestedMeta<'_>) -> Result<DecimalAttribute> {
    let span = meta.path.span();
    let mut value = DecimalAttribute {
        precision: Vec::new(),
        scale: Vec::new(),
        rounding: Vec::new(),
        span,
    };
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("precision") {
            value.precision.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("scale") {
            value.scale.push(parse_integer(&nested)?);
        } else if nested.path.is_ident("rounding") {
            let ident = parse_ident(&nested)?;
            let rounding = match ident.to_string().as_str() {
                "down" => RoundingMode::Down,
                "up" => RoundingMode::Up,
                "half_up" => RoundingMode::HalfUp,
                "half_even" => RoundingMode::HalfEven,
                _ => return Err(nested.error("unknown decimal rounding mode")),
            };
            value.rounding.push(SpannedValue {
                value: rounding,
                span: ident.span(),
            });
        } else {
            return Err(nested.error("unknown decimal argument"));
        }
        Ok(())
    })?;
    Ok(value)
}

/// Parses constraints that apply to every element of a sequence.
fn parse_element(meta: ParseNestedMeta<'_>) -> Result<ElementAttribute> {
    let span = meta.path.span();
    let mut attributes = Vec::new();
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("text") {
            attributes.push(ElementConstraintAttribute::Text(parse_text(nested)?));
        } else if nested.path.is_ident("decimal") {
            attributes.push(ElementConstraintAttribute::Decimal(parse_decimal(nested)?));
        } else {
            discard_nested_meta_input(nested.input)?;
            return Err(nested.error("element only supports `text(...)` or `decimal(...)`"));
        }
        Ok(())
    })?;
    if attributes.is_empty() {
        return Err(Error::new(span, "element requires `text(...)` or `decimal(...)`"));
    }
    Ok(ElementAttribute { attributes, span })
}

/// Parses a direct-reference declaration.
fn parse_reference(meta: ParseNestedMeta<'_>) -> Result<ReferenceAttribute> {
    let span = meta.path.span();
    if meta.input.is_empty() {
        return Err(Error::new(
            span,
            "bare `reference` is not supported; specify `entity = \"module.Type\"`",
        ));
    }
    let mut entity = Vec::new();
    let mut property = Vec::new();
    let mut existing = Vec::new();
    let mut path = Vec::new();
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("entity") {
            entity.push(nested.value()?.parse()?);
        } else if nested.path.is_ident("property") {
            property.push(parse_field_path(&nested)?);
        } else if nested.path.is_ident("existing") {
            existing.push(parse_bool(&nested)?);
        } else if nested.path.is_ident("path") {
            path.push(parse_reference_path(&nested)?);
        } else {
            return Err(nested.error("unknown `reference` argument"));
        }
        Ok(())
    })?;
    if entity.is_empty() {
        return Err(Error::new(span, "reference requires `entity = \"module.Type\"`"));
    }
    Ok(ReferenceAttribute {
        entity,
        property,
        existing,
        path,
        span,
    })
}

/// Parses a lookup-relation declaration.
fn parse_lookup_relation(meta: ParseNestedMeta<'_>) -> Result<LookupRelationAttribute> {
    let span = meta.path.span();
    if meta.input.is_empty() {
        return Err(Error::new(
            span,
            "bare `lookup_relation` is not supported; specify `target = Type` and `target_field = field`",
        ));
    }
    let mut target = Vec::new();
    let mut target_field = Vec::new();
    meta.parse_nested_meta(|nested| {
        if nested.path.is_ident("target") {
            target.push(nested.value()?.parse()?);
        } else if nested.path.is_ident("target_field") {
            target_field.push(parse_field_path(&nested)?);
        } else {
            return Err(nested.error("unknown `lookup_relation` argument"));
        }
        Ok(())
    })?;
    if target.is_empty() {
        return Err(Error::new(span, "lookup_relation requires `target = Type`"));
    }
    if target_field.is_empty() {
        return Err(Error::new(span, "lookup_relation requires `target_field = field`"));
    }
    Ok(LookupRelationAttribute {
        target,
        target_field,
        span,
    })
}

/// Parses a strategy as `codec = "name"` or `codec(name = "name")`.
fn parse_strategy(meta: ParseNestedMeta<'_>) -> Result<StrategyAttribute> {
    let span = meta.path.span();
    let mut name = Vec::new();
    if meta.input.peek(Token![=]) {
        name.push(parse_string(&meta)?);
    } else {
        meta.parse_nested_meta(|nested| {
            if nested.path.is_ident("name") || nested.path.is_ident("strategy") {
                name.push(parse_string(&nested)?);
                Ok(())
            } else {
                Err(nested.error("expected `name = \"...\"`"))
            }
        })?;
    }
    if name.is_empty() {
        return Err(Error::new(span, "strategy attribute requires a name"));
    }
    Ok(StrategyAttribute { name, span })
}

/// Parses an ordered list of Rust field identifiers.
fn parse_field_names(meta: ParseNestedMeta<'_>) -> Result<Vec<FieldName>> {
    let mut fields = Vec::new();
    meta.parse_nested_meta(|nested| {
        let ident = nested
            .path
            .get_ident()
            .ok_or_else(|| nested.error("expected a field identifier"))?;
        fields.push(field_name_from_ident(ident));
        Ok(())
    })?;
    Ok(fields)
}

/// Parses a field path from a string literal or Rust path.
fn parse_field_path(meta: &ParseNestedMeta<'_>) -> Result<Vec<FieldName>> {
    let input = meta.value()?;
    if input.peek(LitStr) {
        let literal: LitStr = input.parse()?;
        literal
            .value()
            .split('.')
            .map(|name| parse_field_path_segment(name, literal.span()))
            .collect()
    } else {
        let path: Path = input.parse()?;
        Ok(path
            .segments
            .iter()
            .map(|segment| field_name_from_ident(&segment.ident))
            .collect())
    }
}

/// Parses a reference path through the containing object graph.
fn parse_reference_path(meta: &ParseNestedMeta<'_>) -> Result<Vec<ReferencePathSegment>> {
    let literal: LitStr = meta.value()?.parse()?;
    let value = literal.value();
    if value.is_empty() {
        return Err(Error::new(literal.span(), "reference path cannot be empty"));
    }
    let mut segments = Vec::new();
    let mut start = 0;
    while start < value.len() {
        if value[start..].starts_with("..") {
            segments.push(ReferencePathSegment::Parent(literal.span()));
            start += 2;
            if start < value.len() {
                let separator = value.as_bytes()[start];
                if separator != b'.' && separator != b'/' {
                    return Err(Error::new(
                        literal.span(),
                        "reference path parent segment must be followed by `.` or `/`",
                    ));
                }
                start += 1;
            }
            continue;
        }

        let mut end = start;
        while end < value.len() {
            let byte = value.as_bytes()[end];
            if byte == b'.' || byte == b'/' {
                break;
            }
            end += 1;
        }
        if end == start {
            return Err(Error::new(literal.span(), "reference path contains an empty segment"));
        }
        segments.push(ReferencePathSegment::Field(parse_field_path_segment(
            &value[start..end],
            literal.span(),
        )?));
        start = end;
        if start < value.len() {
            start += 1;
        }
    }
    Ok(segments)
}

/// Parses and normalizes one string-literal path segment as a Rust field name.
fn parse_field_path_segment(name: &str, span: Span) -> Result<FieldName> {
    if name.is_empty() {
        return Err(Error::new(span, "field path contains an empty segment"));
    }
    let ident =
        parse_str::<Ident>(name).map_err(|_| Error::new(span, "field path segments must be Rust identifiers"))?;
    Ok(FieldName {
        name: ident.unraw().to_string(),
        span,
    })
}

/// Converts a Rust identifier to its metadata name while retaining its span.
fn field_name_from_ident(ident: &Ident) -> FieldName {
    FieldName {
        name: ident.unraw().to_string(),
        span: ident.span(),
    }
}

/// Parses an integer assigned to a nested argument.
fn parse_integer<T>(meta: &ParseNestedMeta<'_>) -> Result<SpannedValue<T>>
where
    T: FromStr,
    T::Err: Display,
{
    let literal: LitInt = meta.value()?.parse()?;
    Ok(SpannedValue {
        value: literal.base10_parse()?,
        span: literal.span(),
    })
}

/// Parses a boolean assigned to a nested argument.
fn parse_bool(meta: &ParseNestedMeta<'_>) -> Result<SpannedValue<bool>> {
    let literal: LitBool = meta.value()?.parse()?;
    Ok(SpannedValue {
        value: literal.value,
        span: literal.span(),
    })
}

/// Parses an identifier assigned to a nested argument.
fn parse_ident(meta: &ParseNestedMeta<'_>) -> Result<Ident> {
    meta.value()?.parse()
}

/// Parses a string literal assigned to a nested argument.
fn parse_string(meta: &ParseNestedMeta<'_>) -> Result<LitStr> {
    meta.value()?.parse()
}

/// Returns whether a path names an attribute whose valid scope is a model
/// declaration.
fn is_model_attribute_path(path: &Path) -> bool {
    path.is_ident("primary_key") || path.is_ident("key") || path.is_ident("ownership")
}

/// Returns whether a path names an attribute whose valid scope is a field
/// declaration.
fn is_field_attribute_path(path: &Path) -> bool {
    path.is_ident("identifier")
        || path.is_ident("text")
        || path.is_ident("sequence")
        || path.is_ident("map")
        || path.is_ident("time")
        || path.is_ident("decimal")
        || path.is_ident("money")
        || path.is_ident("element")
        || path.is_ident("reference")
        || path.is_ident("lookup_relation")
        || path.is_ident("codec")
        || path.is_ident("generator")
        || path.is_ident("opaque")
}

/// Discards the unparsed value or parenthesized arguments of one invalid nested
/// item.
fn discard_nested_meta_input(input: ParseStream<'_>) -> Result<()> {
    while !input.is_empty() && !input.peek(Token![,]) {
        input.parse::<TokenTree>()?;
    }
    Ok(())
}

/// Combines one parsing diagnostic with any diagnostics already collected.
fn combine_error(errors: &mut Option<Error>, error: Error) {
    match errors {
        Some(errors) => errors.combine(error),
        None => *errors = Some(error),
    }
}
