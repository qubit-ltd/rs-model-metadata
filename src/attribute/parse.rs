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
use syn::Expr;
use syn::Ident;
use syn::Lit;
use syn::LitBool;
use syn::LitInt;
use syn::LitStr;
use syn::Meta;
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

use super::allowed_chars::AllowedChars;
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

/// Returns whether an attribute path names a supported field-level helper
/// attribute consumed by `#[Model]`.
pub(crate) fn is_field_level_helper_attribute(path: &Path) -> bool {
    is_field_attribute_path(path) || path.is_ident("unique") || path.is_ident("keep_serializing")
}

/// Returns the record-level helper name when it cannot describe an enum
/// payload field independently.
///
/// Enum variants do not share one record-wide field set, so these helpers
/// cannot retain their usual model-level semantics on a payload field.
pub(crate) fn enum_payload_unsupported_field_attribute(path: &Path) -> Option<&'static str> {
    if path.is_ident("identifier") {
        Some("identifier")
    } else if path.is_ident("unique") {
        Some("unique")
    } else if path.is_ident("indexed") {
        Some("indexed")
    } else if path.is_ident("reference") {
        Some("reference")
    } else if path.is_ident("lookup_relation") {
        Some("lookup_relation")
    } else {
        None
    }
}

/// Returns whether an attribute on a model field should be ignored by scope
/// validation because another tool owns it.
fn is_allowed_foreign_field_attribute(path: &Path) -> bool {
    path.is_ident("serde")
        || path.is_ident("doc")
        || path.is_ident("default")
        || path.is_ident("allow")
        || path.is_ident("deny")
        || path.is_ident("cfg")
        || path.is_ident("must_use")
        || path.is_ident("redact")
}

/// Rejects removed or misplaced attributes on field declarations.
pub(crate) fn validate_field_attribute_scope(attributes: &[Attribute]) -> Result<()> {
    let mut errors = None;
    for attribute in attributes {
        let path = attribute.path();
        if is_allowed_foreign_field_attribute(path) || is_field_level_helper_attribute(path) {
            continue;
        }
        if path.is_ident("field") {
            combine_error(
                &mut errors,
                Error::new_spanned(
                    attribute,
                    "`#[field(...)]` was removed; use standalone field attributes such as \
                     `#[identifier]` and `#[indexed]`",
                ),
            );
            continue;
        }
        if path.is_ident("model") {
            combine_error(
                &mut errors,
                Error::new_spanned(
                    attribute,
                    "field-level `#[model(...)]` attributes are not supported; use standalone \
                     field attributes such as `#[identifier]` and `#[text(...)]`",
                ),
            );
            continue;
        }
        if is_model_attribute_path(path) {
            combine_error(
                &mut errors,
                Error::new_spanned(attribute, "this model-level attribute cannot be used on a field"),
            );
            continue;
        }
        if path.is_ident("nullable") {
            combine_error(
                &mut errors,
                Error::new_spanned(
                    attribute,
                    "`nullable` is not supported; use `Option<T>` for nullability",
                ),
            );
            continue;
        }
        if path.is_ident("computed") {
            combine_error(
                &mut errors,
                Error::new_spanned(
                    attribute,
                    "`computed` is not supported; declare a real Rust field instead",
                ),
            );
            continue;
        }
        combine_error(
            &mut errors,
            Error::new_spanned(attribute, "unknown field-level attribute"),
        );
    }
    if let Some(error) = errors { Err(error) } else { Ok(()) }
}

/// Rejects field helper attributes declared on the model instead of its fields.
pub(crate) fn validate_model_attribute_scope(attributes: &[Attribute]) -> Result<()> {
    let mut errors = None;
    for attribute in attributes {
        let path = attribute.path();
        if path.is_ident("model") || is_allowed_foreign_field_attribute(path) {
            continue;
        }
        if path.is_ident("nullable") {
            combine_error(
                &mut errors,
                Error::new_spanned(
                    attribute,
                    "`nullable` is not supported; use `Option<T>` for nullability",
                ),
            );
            continue;
        }
        if path.is_ident("computed") {
            combine_error(
                &mut errors,
                Error::new_spanned(
                    attribute,
                    "`computed` is not supported; declare a real Rust field instead",
                ),
            );
            continue;
        }
        if is_field_level_helper_attribute(path) || path.is_ident("field") {
            combine_error(
                &mut errors,
                Error::new_spanned(
                    attribute,
                    "field helper attributes must be declared on fields, not on the model",
                ),
            );
        }
    }
    if let Some(error) = errors { Err(error) } else { Ok(()) }
}

/// Parses every supported field-level helper attribute in source order.
///
/// Returns an error when an item is unknown or its value cannot be parsed.
pub(crate) fn parse_field_attributes(attributes: &[Attribute]) -> Result<Vec<FieldAttribute>> {
    validate_field_attribute_scope(attributes)?;
    let mut parsed = Vec::new();
    let mut errors = None;
    for attribute in attributes
        .iter()
        .filter(|attribute| is_field_level_helper_attribute(attribute.path()))
    {
        if let Err(error) = parse_standalone_field_attribute(attribute, &mut parsed) {
            combine_error(&mut errors, error);
        }
    }
    if let Some(error) = errors {
        Err(error)
    } else {
        Ok(parsed)
    }
}

/// Parses one standalone field-level helper attribute.
fn parse_standalone_field_attribute(attribute: &Attribute, parsed: &mut Vec<FieldAttribute>) -> Result<()> {
    let path = attribute.path();
    let span = path.span();
    if path.is_ident("identifier") {
        parsed.push(FieldAttribute::Identifier(parse_identifier_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("unique") {
        parsed.push(FieldAttribute::Unique(parse_field_unique_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("indexed") {
        reject_field_attribute_arguments(attribute, "indexed")?;
        parsed.push(FieldAttribute::Index(span));
        return Ok(());
    }
    if path.is_ident("text") {
        parsed.push(FieldAttribute::Text(parse_text_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("sequence") {
        parsed.push(FieldAttribute::Sequence(parse_sequence_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("map") {
        parsed.push(FieldAttribute::Map(parse_map_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("time") {
        parsed.push(FieldAttribute::Temporal(parse_temporal_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("decimal") {
        parsed.push(FieldAttribute::Decimal(parse_decimal_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("money") {
        parsed.push(FieldAttribute::Money(parse_decimal_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("element") {
        parsed.push(FieldAttribute::Element(parse_element_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("reference") {
        parsed.push(FieldAttribute::Reference(parse_reference_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("lookup_relation") {
        parsed.push(FieldAttribute::LookupRelation(parse_lookup_relation_attribute(
            attribute,
        )?));
        return Ok(());
    }
    if path.is_ident("codec") {
        parsed.push(FieldAttribute::Codec(parse_codec_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("generator") {
        parsed.push(FieldAttribute::Generator(parse_generator_attribute(attribute)?));
        return Ok(());
    }
    if path.is_ident("opaque") {
        reject_field_attribute_arguments(attribute, "opaque")?;
        parsed.push(FieldAttribute::Opaque(span));
        return Ok(());
    }
    if path.is_ident("keep_serializing") {
        reject_field_attribute_arguments(attribute, "keep_serializing")?;
        parsed.push(FieldAttribute::KeepSerializing);
        return Ok(());
    }
    Err(Error::new(span, "unknown field-level attribute"))
}

fn parse_identifier_attribute(attribute: &Attribute) -> Result<IdentifierAttribute> {
    let span = attribute.path().span();
    let mut generated = Vec::new();
    match &attribute.meta {
        Meta::Path(_) => {}
        Meta::List(_) => {
            attribute.parse_nested_meta(|nested| {
                if nested.path.is_ident("generated") {
                    generated.push(nested.path.span());
                    Ok(())
                } else {
                    Err(nested.error("expected `generated`"))
                }
            })?;
        }
        Meta::NameValue(name_value) => {
            return Err(Error::new(
                name_value.span(),
                "`identifier` does not accept name-value arguments",
            ));
        }
    }
    Ok(IdentifierAttribute { generated, span })
}

fn parse_field_unique_attribute(attribute: &Attribute) -> Result<FieldUniqueAttribute> {
    let span = attribute.path().span();
    let mut name = Vec::new();
    let mut respect_to = Vec::new();
    let mut respect_to_count = 0;
    let mut ignore_case = Vec::new();
    let mut ignore_case_values = Vec::new();
    match &attribute.meta {
        Meta::Path(_) => {}
        Meta::List(_) => {
            attribute.parse_nested_meta(|nested| {
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
        Meta::NameValue(name_value) => {
            return Err(Error::new(
                name_value.span(),
                "`unique` does not accept name-value arguments",
            ));
        }
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

fn parse_text_attribute(attribute: &Attribute) -> Result<TextAttribute> {
    let span = attribute.path().span();
    let mut value = TextAttribute {
        min_chars: Vec::new(),
        max_chars: Vec::new(),
        min_bytes: Vec::new(),
        max_bytes: Vec::new(),
        allowed_chars: Vec::new(),
        non_blank: Vec::new(),
        format: Vec::new(),
        span,
    };
    attribute.parse_nested_meta(|nested| parse_text_argument(&mut value, nested))?;
    Ok(value)
}

fn parse_text_argument(value: &mut TextAttribute, nested: ParseNestedMeta<'_>) -> Result<()> {
    if nested.path.is_ident("min_chars") {
        value.min_chars.push(parse_integer(&nested)?);
    } else if nested.path.is_ident("max_chars") {
        value.max_chars.push(parse_integer(&nested)?);
    } else if nested.path.is_ident("min_bytes") {
        value.min_bytes.push(parse_integer(&nested)?);
    } else if nested.path.is_ident("max_bytes") {
        value.max_bytes.push(parse_integer(&nested)?);
    } else if nested.path.is_ident("allowed_chars") {
        let ident = parse_ident(&nested)?;
        let allowed_chars = match ident.to_string().as_str() {
            "unicode" => AllowedChars::Unicode,
            "ascii" => AllowedChars::Ascii,
            _ => return Err(nested.error("expected `unicode` or `ascii`")),
        };
        value.allowed_chars.push(SpannedValue {
            value: allowed_chars,
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
            _ => return Err(nested.error("expected `email`, `mobile`, `uri`, or `uuid`")),
        };
        value.format.push(SpannedValue {
            value: format,
            span: ident.span(),
        });
    } else {
        return Err(nested.error("unknown `text` argument"));
    }
    Ok(())
}

fn parse_sequence_attribute(attribute: &Attribute) -> Result<SequenceAttribute> {
    let span = attribute.path().span();
    let mut value = SequenceAttribute {
        min_items: Vec::new(),
        max_items: Vec::new(),
        unique_items: Vec::new(),
        span,
    };
    attribute.parse_nested_meta(|nested| parse_sequence_argument(&mut value, nested))?;
    Ok(value)
}

fn parse_sequence_argument(value: &mut SequenceAttribute, nested: ParseNestedMeta<'_>) -> Result<()> {
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
}

fn parse_map_attribute(attribute: &Attribute) -> Result<MapAttribute> {
    let span = attribute.path().span();
    let mut value = MapAttribute {
        min_entries: Vec::new(),
        max_entries: Vec::new(),
        span,
    };
    attribute.parse_nested_meta(|nested| parse_map_argument(&mut value, nested))?;
    Ok(value)
}

fn parse_map_argument(value: &mut MapAttribute, nested: ParseNestedMeta<'_>) -> Result<()> {
    if nested.path.is_ident("min_entries") {
        value.min_entries.push(parse_integer(&nested)?);
    } else if nested.path.is_ident("max_entries") {
        value.max_entries.push(parse_integer(&nested)?);
    } else {
        return Err(nested.error("unknown `map` argument"));
    }
    Ok(())
}

fn parse_temporal_attribute(attribute: &Attribute) -> Result<TemporalAttribute> {
    let span = attribute.path().span();
    let mut value = TemporalAttribute {
        precision: Vec::new(),
        span,
    };
    attribute.parse_nested_meta(|nested| parse_temporal_argument(&mut value, nested))?;
    Ok(value)
}

fn parse_temporal_argument(value: &mut TemporalAttribute, nested: ParseNestedMeta<'_>) -> Result<()> {
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
}

fn parse_decimal_attribute(attribute: &Attribute) -> Result<DecimalAttribute> {
    let span = attribute.path().span();
    let mut value = DecimalAttribute {
        precision: Vec::new(),
        scale: Vec::new(),
        rounding: Vec::new(),
        span,
    };
    attribute.parse_nested_meta(|nested| parse_decimal_argument(&mut value, nested))?;
    Ok(value)
}

fn parse_decimal_argument(value: &mut DecimalAttribute, nested: ParseNestedMeta<'_>) -> Result<()> {
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
}

fn parse_element_attribute(attribute: &Attribute) -> Result<ElementAttribute> {
    let span = attribute.path().span();
    let mut attributes = Vec::new();
    attribute.parse_nested_meta(|nested| {
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

fn parse_reference_attribute(attribute: &Attribute) -> Result<ReferenceAttribute> {
    let span = attribute.path().span();
    if matches!(attribute.meta, Meta::Path(_)) {
        return Err(Error::new(
            span,
            "bare `reference` is not supported; specify `entity = \"module.Type\"`",
        ));
    }
    let mut entity = Vec::new();
    let mut property = Vec::new();
    let mut existing = Vec::new();
    let mut path = Vec::new();
    attribute.parse_nested_meta(|nested| {
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

fn parse_lookup_relation_attribute(attribute: &Attribute) -> Result<LookupRelationAttribute> {
    let span = attribute.path().span();
    if matches!(attribute.meta, Meta::Path(_)) {
        return Err(Error::new(
            span,
            "bare `lookup_relation` is not supported; specify `target = Type` and `target_field = field`",
        ));
    }
    let mut target = Vec::new();
    let mut target_field = Vec::new();
    attribute.parse_nested_meta(|nested| {
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

fn parse_generator_attribute(attribute: &Attribute) -> Result<StrategyAttribute> {
    let span = attribute.path().span();
    if matches!(attribute.meta, Meta::Path(_)) {
        return Err(Error::new(span, "generator attribute requires a name"));
    }
    let mut name = Vec::new();
    attribute.parse_nested_meta(|nested| {
        if nested.path.is_ident("name") || nested.path.is_ident("strategy") {
            name.push(parse_string(&nested)?);
            Ok(())
        } else {
            Err(nested.error("expected `name = \"...\"`"))
        }
    })?;
    if name.is_empty() {
        return Err(Error::new(span, "generator attribute requires a name"));
    }
    Ok(StrategyAttribute { name, span })
}

/// Parses `codec = "name"` or `codec(name = "name")`.
fn parse_codec_attribute(attribute: &Attribute) -> Result<StrategyAttribute> {
    match &attribute.meta {
        Meta::NameValue(name_value) => {
            let Expr::Lit(expr_lit) = &name_value.value else {
                return Err(Error::new(
                    name_value.span(),
                    "codec attribute requires a string literal name",
                ));
            };
            let Lit::Str(literal) = &expr_lit.lit else {
                return Err(Error::new(
                    expr_lit.span(),
                    "codec attribute requires a string literal name",
                ));
            };
            Ok(StrategyAttribute {
                name: vec![literal.clone()],
                span: attribute.path().span(),
            })
        }
        Meta::List(_) => parse_generator_attribute(attribute),
        Meta::Path(path) => Err(Error::new(path.span(), "codec attribute requires a name")),
    }
}

/// Rejects helper attributes that must be written without arguments.
fn reject_field_attribute_arguments(attribute: &Attribute, name: &str) -> Result<()> {
    match &attribute.meta {
        Meta::Path(_) => Ok(()),
        Meta::List(list) if list.tokens.is_empty() => Ok(()),
        Meta::List(list) => Err(Error::new(list.span(), format!("`{name}` does not accept arguments"))),
        Meta::NameValue(name_value) => Err(Error::new(
            name_value.span(),
            format!("`{name}` does not accept arguments"),
        )),
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
            .any(|attribute| matches!(attribute, ModelAttribute::Textual(_)))
        {
            return Err(meta.error("duplicate `textual` model capability"));
        }
        if meta.input.peek(Paren) || meta.input.peek(Token![=]) {
            return Err(meta.error("`textual` does not accept arguments"));
        }
        parsed.push(ModelAttribute::Textual(meta.path.span()));
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

/// Parses a comma-separated bracketed list of Rust field identifiers.
fn parse_field_name_list(meta: &ParseNestedMeta<'_>) -> Result<Vec<FieldName>> {
    let input = meta.value()?;
    let content;
    bracketed!(content in input);
    Punctuated::<Ident, Token![,]>::parse_terminated(&content)
        .map(|idents| idents.iter().map(field_name_from_ident).collect())
}

/// Parses nested `text(...)` arguments inside an `element(...)` attribute.
fn parse_text(meta: ParseNestedMeta<'_>) -> Result<TextAttribute> {
    let span = meta.path.span();
    let mut value = TextAttribute {
        min_chars: Vec::new(),
        max_chars: Vec::new(),
        min_bytes: Vec::new(),
        max_bytes: Vec::new(),
        allowed_chars: Vec::new(),
        non_blank: Vec::new(),
        format: Vec::new(),
        span,
    };
    meta.parse_nested_meta(|nested| parse_text_argument(&mut value, nested))?;
    Ok(value)
}

/// Parses nested `decimal(...)` arguments inside an `element(...)` attribute.
fn parse_decimal(meta: ParseNestedMeta<'_>) -> Result<DecimalAttribute> {
    let span = meta.path.span();
    let mut value = DecimalAttribute {
        precision: Vec::new(),
        scale: Vec::new(),
        rounding: Vec::new(),
        span,
    };
    meta.parse_nested_meta(|nested| parse_decimal_argument(&mut value, nested))?;
    Ok(value)
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
        || path.is_ident("indexed")
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
