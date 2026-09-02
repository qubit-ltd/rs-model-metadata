// =============================================================================

use super::declaration_ir::CodecIr;
use super::declaration_ir::ConstraintIr;
use super::declaration_ir::DecimalConstraintIr;
use super::declaration_ir::DeclarationOptions;
use super::declaration_ir::FieldIr;
use super::declaration_ir::FieldOccurrence;
use super::declaration_ir::IdentifierAssignmentIr;
use super::declaration_ir::RedactIr;
use super::declaration_ir::RedactModeIr;
use super::declaration_ir::ReferenceIr;
use super::declaration_ir::ReferenceTargetIr;
use super::declaration_ir::SelectorIr;
use super::declaration_ir::SelectorPositionIr;
use super::declaration_ir::SerdeIr;
use super::declaration_ir::StrategyArgumentIr;
use super::declaration_ir::TextConstraintIr;
use super::declaration_ir::UniqueIr;
use super::declaration_ir::ValidatorIr;
use super::declaration_ir::VariantIr;
use super::declaration_validate::combine;
use heck::ToShoutySnakeCase;
use quote::quote;
use syn::Attribute;
use syn::Expr;
use syn::ExprLit;
use syn::Error;
use syn::Fields;
use syn::Lit;
use syn::LitStr;
use syn::Meta;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::parse::Parser;
use syn::parse_quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Parses every field and combines independent field diagnostics.
pub(super) fn parse_fields(fields: &Fields) -> Result<Vec<FieldIr>> {
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
pub(super) fn parse_variants(data: &syn::DataEnum) -> Result<Vec<VariantIr>> {
    let mut parsed = Vec::new();
    let mut errors = None;
    for variant in &data.variants {
        let canonical_name = variant.ident.to_string().to_shouty_snake_case();
        let names = parse_variant_serde_names(&variant.attrs, &canonical_name);
        let fields = parse_fields(&variant.fields);
        match (names, fields) {
            (Ok((serialized_name, deserialized_name)), Ok(fields)) => parsed.push(VariantIr {
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
            (names, fields) => {
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

/// Parses variant rename attributes and returns serialized/deserialized names.
fn parse_variant_serde_names(
    attributes: &[Attribute],
    canonical: &str,
) -> Result<(String, String)> {
    let mut serialize = canonical.to_owned();
    let mut deserialize = canonical.to_owned();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("serde"))
    {
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

impl DeclarationOptions {
    /// Parses declaration-level options and rejects duplicates or bad values.
    pub(super) fn parse(options: Punctuated<Meta, Token![,]>) -> Result<Self> {
        let mut result = Self {
            id: None,
            source: None,
            source_id: None,
            open: false,
            transparent: false,
            no_clone: false,
            no_debug: false,
            no_display: false,
            no_partial_eq: false,
            no_eq: false,
            no_hash: false,
            no_serialize: false,
            no_deserialize: false,
            no_redact: false,
            no_copy: false,
            copy: false,
            default: false,
            partial_ord: false,
            ord: false,
            codec: None,
        };
        for option in options {
            match option {
                Meta::NameValue(value) if value.path.is_ident("id") => {
                    set_lit_str(&mut result.id, value.value, "id")?;
                }
                Meta::NameValue(value) if value.path.is_ident("source_id") => {
                    set_lit_str(&mut result.source_id, value.value, "source_id")?;
                }
                Meta::NameValue(value) if value.path.is_ident("source") => {
                    if result.source.is_some() {
                        return Err(Error::new_spanned(value, "duplicate `source` option"));
                    }
                    let expression = value.value;
                    result.source = Some(syn::parse2(quote!(#expression))?);
                }
                Meta::NameValue(value) if value.path.is_ident("codec") => {
                    if result.codec.is_some() {
                        return Err(Error::new_spanned(value, "duplicate `codec` option"));
                    }
                    let expression = value.value;
                    result.codec = Some(syn::parse2(quote!(#expression))?);
                }
                Meta::Path(path) if path.is_ident("open") => result.open = true,
                Meta::Path(path) if path.is_ident("transparent") => result.transparent = true,
                Meta::Path(path) if path.is_ident("no_clone") => result.no_clone = true,
                Meta::Path(path) if path.is_ident("no_debug") => result.no_debug = true,
                Meta::Path(path) if path.is_ident("no_display") => result.no_display = true,
                Meta::Path(path) if path.is_ident("no_partial_eq") => result.no_partial_eq = true,
                Meta::Path(path) if path.is_ident("no_eq") => result.no_eq = true,
                Meta::Path(path) if path.is_ident("no_hash") => result.no_hash = true,
                Meta::Path(path) if path.is_ident("no_serialize") => result.no_serialize = true,
                Meta::Path(path) if path.is_ident("no_deserialize") => result.no_deserialize = true,
                Meta::Path(path) if path.is_ident("no_redact") => result.no_redact = true,
                Meta::Path(path) if path.is_ident("no_copy") => result.no_copy = true,
                Meta::Path(path) if path.is_ident("copy") => result.copy = true,
                Meta::Path(path) if path.is_ident("default") => result.default = true,
                Meta::Path(path) if path.is_ident("partial_ord") => result.partial_ord = true,
                Meta::Path(path) if path.is_ident("ord") => result.ord = true,
                other => {
                    return Err(Error::new_spanned(other, "unsupported model option"));
                }
            }
        }
        Ok(result)
    }
}

impl FieldIr {
    /// Parses one field's attributes into normalized intermediate metadata.
    fn parse(index: usize, ty: &Type, attributes: &[Attribute], named: bool) -> Result<Self> {
        let mut occurrences = Vec::new();
        let mut keep_serializing = false;
        for attribute in attributes {
            if attribute.path().is_ident("identifier") {
                occurrences.push(FieldOccurrence::Identifier(parse_identifier(attribute)?));
            } else if attribute.path().is_ident("indexed") {
                occurrences.push(FieldOccurrence::Indexed);
            } else if attribute.path().is_ident("unique") {
                occurrences.push(FieldOccurrence::Unique(parse_unique(attribute)?));
            } else if attribute.path().is_ident("reference") {
                occurrences.push(FieldOccurrence::Reference(parse_reference(attribute)?));
            } else if attribute.path().is_ident("key_part") {
                occurrences.push(FieldOccurrence::KeyPart(parse_key_part(attribute)?));
            } else if is_constraint_attribute(attribute) {
                occurrences.push(FieldOccurrence::Constraint(parse_constraint(attribute)?));
            } else if attribute.path().is_ident("element") {
                occurrences.push(FieldOccurrence::Selector(parse_selector(
                    attribute,
                    SelectorPositionIr::Element,
                )?));
            } else if attribute.path().is_ident("map_key") {
                occurrences.push(FieldOccurrence::Selector(parse_selector(
                    attribute,
                    SelectorPositionIr::MapKey,
                )?));
            } else if attribute.path().is_ident("map_value") {
                occurrences.push(FieldOccurrence::Selector(parse_selector(
                    attribute,
                    SelectorPositionIr::MapValue,
                )?));
            } else if attribute.path().is_ident("validator") {
                occurrences.push(FieldOccurrence::Validator(parse_validator(attribute)?));
            } else if attribute.path().is_ident("codec") {
                occurrences.push(FieldOccurrence::Codec(parse_codec(attribute)?));
            } else if attribute.path().is_ident("redact") {
                occurrences.push(FieldOccurrence::Redact(parse_redact(attribute)?));
            } else if attribute.path().is_ident("serde") {
                occurrences.push(FieldOccurrence::Serde(parse_serde(attribute)?));
            } else if attribute.path().is_ident("opaque") {
                occurrences.push(FieldOccurrence::Opaque);
            } else if attribute.path().is_ident("keep_serializing") {
                if !matches!(attribute.meta, Meta::Path(_)) {
                    return Err(Error::new_spanned(
                        attribute,
                        "keep_serializing is a marker without arguments",
                    ));
                }
                if keep_serializing {
                    return Err(Error::new_spanned(
                        attribute,
                        "duplicate keep_serializing marker",
                    ));
                }
                keep_serializing = true;
            }
        }
        Ok(Self {
            index,
            ty: ty.clone(),
            span: ty.span(),
            occurrences,
            keep_serializing,
            named,
        })
    }
}

/// Parses an identifier assignment attribute.
fn parse_identifier(attribute: &Attribute) -> Result<IdentifierAssignmentIr> {
    if matches!(attribute.meta, Meta::Path(_)) {
        return Ok(IdentifierAssignmentIr::Application);
    }
    let mut assignment = None;
    attribute.parse_nested_meta(|meta| {
        if !meta.path.is_ident("assigned_by") {
            return Err(meta.error("unsupported identifier option"));
        }
        let value = parse_ident_value(meta.value()?.parse()?)?;
        assignment = Some(match value.as_str() {
            "application" => IdentifierAssignmentIr::Application,
            "database" => IdentifierAssignmentIr::Database,
            _ => {
                return Err(meta.error("assigned_by must be application or database"));
            }
        });
        Ok(())
    })?;
    assignment.ok_or_else(|| Error::new_spanned(attribute, "identifier requires assigned_by"))
}

/// Sets a string option while rejecting duplicate declarations.
fn set_lit_str(slot: &mut Option<LitStr>, value: Expr, name: &str) -> Result<()> {
    if slot.is_some() {
        return Err(Error::new_spanned(
            value,
            format!("duplicate `{name}` option"),
        ));
    }
    let Expr::Lit(ExprLit {
        lit: Lit::Str(value),
        ..
    }) = value
    else {
        return Err(Error::new_spanned(
            value,
            format!("`{name}` requires a string literal"),
        ));
    };
    *slot = Some(value);
    Ok(())
}

/// Parses a uniqueness declaration and its comparison paths.
fn parse_unique(attribute: &Attribute) -> Result<UniqueIr> {
    let mut value = UniqueIr {
        respect_to: Vec::new(),
        ignore_case: true,
    };
    if matches!(attribute.meta, Meta::Path(_)) {
        return Ok(value);
    }
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("respect_to") {
            meta.parse_nested_meta(|path| {
                value.respect_to.push(path_from_syn(&path.path));
                Ok(())
            })
        } else if meta.path.is_ident("ignore_case") {
            value.ignore_case = meta.value()?.parse::<syn::LitBool>()?.value;
            Ok(())
        } else {
            Err(meta.error("unsupported unique option"))
        }
    })?;
    Ok(value)
}

/// Parses a relationship declaration and target selector.
fn parse_reference(attribute: &Attribute) -> Result<ReferenceIr> {
    let mut target = None;
    let mut property = None;
    let mut existing = true;
    let mut same_as = None;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("entity") {
            let value = meta.value()?;
            let entity_target = if value.peek(LitStr) {
                let id: LitStr = value.parse()?;
                validate_ascii_id(&id, "reference entity ID")?;
                ReferenceTargetIr::ModelId(id)
            } else {
                ReferenceTargetIr::RustType(Box::new(value.parse()?))
            };
            if target.replace(entity_target).is_some() {
                return Err(meta.error("reference requires exactly one entity target"));
            }
            Ok(())
        } else if meta.path.is_ident("entity_id") {
            let id: LitStr = meta.value()?.parse()?;
            validate_ascii_id(&id, "reference entity ID")?;
            if target.replace(ReferenceTargetIr::ModelId(id)).is_some() {
                return Err(meta.error("reference requires exactly one entity target"));
            }
            Ok(())
        } else if meta.path.is_ident("property") {
            property = Some(parse_path_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("path") {
            same_as = Some(parse_path_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("existing") {
            existing = meta.value()?.parse::<syn::LitBool>()?.value;
            Ok(())
        } else {
            Err(meta.error("unsupported reference option"))
        }
    })?;
    let target = target.ok_or_else(|| {
        Error::new_spanned(attribute, "reference requires `entity` or `entity_id`")
    })?;
    Ok(ReferenceIr {
        target,
        property,
        existing,
        same_as,
    })
}

/// Parses a zero-based composite-key position.
fn parse_key_part(attribute: &Attribute) -> Result<usize> {
    let mut order = None;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("order") {
            let value = meta.value()?.parse::<syn::LitInt>()?.base10_parse()?;
            if order.replace(value).is_some() {
                return Err(meta.error("duplicate key_part order"));
            }
            Ok(())
        } else {
            Err(meta.error("unsupported key_part option"))
        }
    })?;
    order.ok_or_else(|| Error::new_spanned(attribute, "key_part requires `order = n`"))
}

/// Reports whether `attribute` names one of the supported constraints.
fn is_constraint_attribute(attribute: &Attribute) -> bool {
    attribute.path().is_ident("text")
        || attribute.path().is_ident("decimal")
        || attribute.path().is_ident("money")
        || attribute.path().is_ident("time")
        || attribute.path().is_ident("sequence")
        || attribute.path().is_ident("map")
}

/// Parses one textual, decimal, temporal, sequence, or map constraint.
fn parse_constraint(attribute: &Attribute) -> Result<ConstraintIr> {
    if attribute.path().is_ident("text") {
        return parse_text_constraint(attribute).map(ConstraintIr::Text);
    }
    if attribute.path().is_ident("decimal") {
        return parse_decimal_constraint(attribute, false).map(ConstraintIr::Decimal);
    }
    if attribute.path().is_ident("money") {
        return parse_decimal_constraint(attribute, true).map(ConstraintIr::Decimal);
    }
    if attribute.path().is_ident("time") {
        let mut precision = None;
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("precision") {
                precision = Some(parse_ident_value(meta.value()?.parse()?)?);
                Ok(())
            } else {
                Err(meta.error("unsupported time option"))
            }
        })?;
        return Ok(ConstraintIr::Time(precision.ok_or_else(|| {
            Error::new_spanned(attribute, "time requires precision")
        })?));
    }
    if attribute.path().is_ident("sequence") {
        let (mut min, mut max, mut unique) = (None, None, false);
        let mut any = false;
        attribute.parse_nested_meta(|meta| {
            any = true;
            if meta.path.is_ident("min_items") {
                min = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
                Ok(())
            } else if meta.path.is_ident("max_items") {
                max = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
                Ok(())
            } else if meta.path.is_ident("unique_items") {
                unique = true;
                Ok(())
            } else {
                Err(meta.error("unsupported sequence option"))
            }
        })?;
        if !any {
            return Err(Error::new_spanned(
                attribute,
                "sequence requires at least one option",
            ));
        }
        return Ok(ConstraintIr::Sequence { min, max, unique });
    }
    let (mut min, mut max) = (None, None);
    let mut any = false;
    attribute.parse_nested_meta(|meta| {
        any = true;
        if meta.path.is_ident("min_entries") {
            min = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_entries") {
            max = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported map option"))
        }
    })?;
    if !any {
        return Err(Error::new_spanned(
            attribute,
            "map requires min_entries or max_entries",
        ));
    }
    Ok(ConstraintIr::Map { min, max })
}

/// Parses text length, character-set, blankness, and format options.
fn parse_text_constraint(attribute: &Attribute) -> Result<TextConstraintIr> {
    let mut value = TextConstraintIr::default();
    let mut any = false;
    attribute.parse_nested_meta(|meta| {
        any = true;
        if meta.path.is_ident("min_chars") {
            value.min_chars = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_chars") {
            value.max_chars = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("min_bytes") {
            value.min_bytes = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_bytes") {
            value.max_bytes = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("non_blank") {
            value.non_blank = true;
            Ok(())
        } else if meta.path.is_ident("allowed_chars") {
            value.allowed_chars = Some(parse_ident_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("format") {
            value.format = Some(parse_ident_value(meta.value()?.parse()?)?);
            Ok(())
        } else {
            Err(meta.error("unsupported text option"))
        }
    })?;
    if !any {
        return Err(Error::new_spanned(
            attribute,
            "text requires at least one option",
        ));
    }
    Ok(value)
}

/// Parses decimal precision, scale, bounds, and rounding options.
fn parse_decimal_constraint(attribute: &Attribute, money: bool) -> Result<DecimalConstraintIr> {
    let mut precision = None;
    let mut scale = None;
    let mut rounding = None;
    let mut min: Option<LitStr> = None;
    let mut max: Option<LitStr> = None;
    let mut min_inclusive = true;
    let mut max_inclusive = true;
    let mut any = false;
    attribute.parse_nested_meta(|meta| {
        any = true;
        if meta.path.is_ident("precision") {
            precision = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("scale") {
            scale = Some(meta.value()?.parse::<syn::LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("rounding") {
            rounding = Some(parse_ident_value(meta.value()?.parse()?)?);
            Ok(())
        } else if meta.path.is_ident("min") {
            min = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("max") {
            max = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("min_inclusive") {
            min_inclusive = meta.value()?.parse::<syn::LitBool>()?.value;
            Ok(())
        } else if meta.path.is_ident("max_inclusive") {
            max_inclusive = meta.value()?.parse::<syn::LitBool>()?.value;
            Ok(())
        } else {
            Err(meta.error("unsupported decimal option"))
        }
    })?;
    if !any {
        return Err(Error::new_spanned(
            attribute,
            "decimal and money require at least one option",
        ));
    }
    if money && scale.is_none() {
        return Err(Error::new_spanned(attribute, "money requires scale"));
    }
    if precision.is_some_and(|precision| scale.is_some_and(|scale| scale > precision)) {
        return Err(Error::new_spanned(
            attribute,
            "decimal scale cannot exceed precision",
        ));
    }
    if let (Some(minimum), Some(maximum)) = (&min, &max) {
        match compare_decimal_literals(&minimum.value(), &maximum.value()) {
            Some(core::cmp::Ordering::Greater) => {
                return Err(Error::new_spanned(
                    attribute,
                    "decimal min cannot exceed max",
                ));
            }
            Some(core::cmp::Ordering::Equal) if !min_inclusive && !max_inclusive => {
                return Err(Error::new_spanned(
                    attribute,
                    "equal decimal bounds cannot both be exclusive",
                ));
            }
            Some(_) => {}
            None => {
                return Err(Error::new_spanned(
                    attribute,
                    "decimal bounds require canonical decimal strings",
                ));
            }
        }
    } else {
        for bound in min.iter().chain(max.iter()) {
            if parse_decimal_literal(&bound.value()).is_none() {
                return Err(Error::new_spanned(
                    bound,
                    "decimal bounds require canonical decimal strings",
                ));
            }
        }
    }
    Ok(DecimalConstraintIr {
        precision,
        scale: scale.unwrap_or(0),
        rounding: rounding.unwrap_or_else(|| {
            if money {
                "unnecessary".into()
            } else {
                "half_even".into()
            }
        }),
        money,
        min,
        max,
        min_inclusive,
        max_inclusive,
    })
}

/// Splits a decimal literal into sign, digits, and fractional scale.
fn parse_decimal_literal(value: &str) -> Option<(bool, String, usize)> {
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |value| (true, value));
    if unsigned.is_empty() || unsigned.starts_with('+') {
        return None;
    }
    let mut parts = unsigned.split('.');
    let integer = parts.next()?;
    let fraction = parts.next().unwrap_or("");
    if parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };
    let fraction = fraction.trim_end_matches('0');
    let digits = format!("{integer}{fraction}");
    let digits = digits.trim_start_matches('0').to_owned();
    let normalized = if digits.is_empty() {
        "0".to_owned()
    } else {
        digits
    };
    let scale = fraction.len();
    Some((negative && normalized != "0", normalized, scale))
}

/// Compares two normalized decimal literal strings without floating point.
fn compare_decimal_literals(left: &str, right: &str) -> Option<core::cmp::Ordering> {
    let (left_negative, mut left_digits, left_scale) = parse_decimal_literal(left)?;
    let (right_negative, mut right_digits, right_scale) = parse_decimal_literal(right)?;
    let scale = left_scale.max(right_scale);
    left_digits.extend(std::iter::repeat_n('0', scale - left_scale));
    right_digits.extend(std::iter::repeat_n('0', scale - right_scale));
    let magnitude = left_digits
        .len()
        .cmp(&right_digits.len())
        .then_with(|| left_digits.cmp(&right_digits));
    Some(match (left_negative, right_negative) {
        (true, false) => core::cmp::Ordering::Less,
        (false, true) => core::cmp::Ordering::Greater,
        (true, true) => magnitude.reverse(),
        (false, false) => magnitude,
    })
}

/// Parses a selector and its nested constraints, validators, and redaction.
fn parse_selector(attribute: &Attribute, position: SelectorPositionIr) -> Result<SelectorIr> {
    let Meta::List(list) = &attribute.meta else {
        return Err(Error::new_spanned(
            attribute,
            "selector requires nested declarations",
        ));
    };
    let values = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(list.tokens.clone())?;
    let mut selector = SelectorIr {
        position,
        constraints: Vec::new(),
        validators: Vec::new(),
        codec: None,
        redact: None,
    };
    for value in values {
        let nested: Attribute = parse_quote!(#[#value]);
        if is_constraint_attribute(&nested) {
            if matches!(
                nested
                    .path()
                    .get_ident()
                    .map(ToString::to_string)
                    .as_deref(),
                Some("sequence" | "map")
            ) {
                return Err(Error::new_spanned(
                    nested,
                    "selectors cannot recursively contain collection selectors",
                ));
            }
            selector.constraints.push(parse_constraint(&nested)?);
        } else if nested.path().is_ident("validator") {
            selector.validators.push(parse_validator(&nested)?);
        } else if nested.path().is_ident("codec") {
            if selector.codec.replace(parse_codec(&nested)?).is_some() {
                return Err(Error::new_spanned(nested, "selector accepts one codec"));
            }
        } else if nested.path().is_ident("redact") {
            if selector.redact.replace(parse_redact(&nested)?).is_some() {
                return Err(Error::new_spanned(
                    nested,
                    "selector accepts one redact declaration",
                ));
            }
        } else {
            return Err(Error::new_spanned(
                nested,
                "unsupported selector declaration",
            ));
        }
    }
    Ok(selector)
}

/// Converts an identifier expression into its canonical path text.
fn parse_ident_value(expression: Expr) -> Result<String> {
    match expression {
        Expr::Path(path) if path.path.segments.len() == 1 => {
            Ok(path.path.segments[0].ident.to_string())
        }
        other => Err(Error::new_spanned(other, "expected an identifier value")),
    }
}

/// Parses a validator ID, dependencies, and strategy parameters.
fn parse_validator(attribute: &Attribute) -> Result<ValidatorIr> {
    let mut id = None;
    let mut params = Vec::new();
    let mut depends_on = Vec::new();
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            let value: LitStr = meta.value()?.parse()?;
            if id.replace(value).is_some() {
                return Err(meta.error("duplicate validator ID"));
            }
            Ok(())
        } else if meta.path.is_ident("depends_on") {
            meta.parse_nested_meta(|path| {
                depends_on.push(path_from_syn(&path.path));
                Ok(())
            })
        } else if meta.path.is_ident("params") {
            meta.parse_nested_meta(|parameter| {
                let name = parameter
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        parameter.error("validator parameter name must be an identifier")
                    })?
                    .to_string();
                let expression: Expr = parameter.value()?.parse()?;
                params.push((name, parse_strategy_argument(expression)?));
                Ok(())
            })
        } else {
            Err(meta.error("unsupported validator option"))
        }
    })?;
    let id =
        id.ok_or_else(|| Error::new_spanned(attribute, "validator requires `id = \"...\"`"))?;
    validate_ascii_id(&id, "validator ID")?;
    Ok(ValidatorIr {
        id,
        params,
        depends_on,
    })
}

/// Parses a declared codec ID or Rust codec type.
fn parse_codec(attribute: &Attribute) -> Result<CodecIr> {
    if let Ok(ty) = attribute.parse_args::<Type>()
        && !matches!(&ty, Type::Path(path) if path.path.is_ident("id"))
    {
        return Ok(CodecIr::RustType(Box::new(ty)));
    }
    let mut result = None;
    attribute.parse_nested_meta(|meta| {
        let value = if meta.path.is_ident("id") {
            let id: LitStr = meta.value()?.parse()?;
            validate_ascii_id(&id, "codec ID")?;
            CodecIr::DeclaredId(id)
        } else if meta.path.is_ident("type") {
            CodecIr::RustType(Box::new(meta.value()?.parse()?))
        } else {
            return Err(meta.error("unsupported codec option"));
        };
        if result.replace(value).is_some() {
            return Err(meta.error("codec accepts one reference"));
        }
        Ok(())
    })?;
    result.ok_or_else(|| {
        Error::new_spanned(attribute, "codec requires a Rust type or `id = \"...\"`")
    })
}

/// Parses a field or selector redaction mode.
fn parse_redact(attribute: &Attribute) -> Result<RedactIr> {
    let mut mode = None;
    attribute.parse_nested_meta(|meta| {
        let current = if meta.path.is_ident("level") {
            RedactModeIr::Level(meta.value()?.parse::<LitStr>()?.value())
        } else if meta.path.is_ident("skip") {
            RedactModeIr::Skip
        } else if meta.path.is_ident("nested") {
            RedactModeIr::Nested
        } else if meta.path.is_ident("map") {
            RedactModeIr::Map
        } else if meta.path.is_ident("keyed_by") {
            let expression: Expr = meta.value()?.parse()?;
            RedactModeIr::KeyedBy(path_text(expression)?)
        } else if meta.path.is_ident("json") {
            RedactModeIr::Json
        } else {
            return Err(meta.error("unsupported redact mode"));
        };
        if mode.replace(current).is_some() {
            return Err(meta.error("redact requires exactly one mode"));
        }
        Ok(())
    })?;
    Ok(RedactIr {
        mode: mode.ok_or_else(|| Error::new_spanned(attribute, "redact requires one mode"))?,
    })
}

/// Parses Serde rename, skip, flatten, default, and custom-handler options.
fn parse_serde(attribute: &Attribute) -> Result<SerdeIr> {
    let mut serde = SerdeIr::default();
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") {
            if meta.input.peek(Token![=]) {
                let value: LitStr = meta.value()?.parse()?;
                serde.serialize_name = Some(value.clone());
                serde.deserialize_name = Some(value);
                Ok(())
            } else {
                meta.parse_nested_meta(|direction| {
                    let value: LitStr = direction.value()?.parse()?;
                    if direction.path.is_ident("serialize") {
                        serde.serialize_name = Some(value);
                    } else if direction.path.is_ident("deserialize") {
                        serde.deserialize_name = Some(value);
                    } else {
                        return Err(direction.error("unsupported serde rename direction"));
                    }
                    Ok(())
                })
            }
        } else if meta.path.is_ident("skip") {
            serde.skip_serializing = true;
            serde.skip_deserializing = true;
            Ok(())
        } else if meta.path.is_ident("skip_serializing") {
            serde.skip_serializing = true;
            Ok(())
        } else if meta.path.is_ident("skip_deserializing") {
            serde.skip_deserializing = true;
            Ok(())
        } else if meta.path.is_ident("flatten") {
            serde.flatten = true;
            Ok(())
        } else if meta.path.is_ident("with") {
            serde.with = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("default") {
            serde.default = true;
            if meta.input.peek(Token![=]) {
                let _: Expr = meta.value()?.parse()?;
            }
            Ok(())
        } else if meta.path.is_ident("skip_serializing_if") {
            serde.explicit_skip_serializing_if = true;
            let _: LitStr = meta.value()?.parse()?;
            Ok(())
        } else {
            // Serde owns its wider syntax; metadata records only the stable
            // subset.
            if meta.input.peek(Token![=]) {
                let _: Expr = meta.value()?.parse()?;
            }
            Ok(())
        }
    })?;
    Ok(serde)
}

/// Parses one validator strategy argument into a supported literal variant.
fn parse_strategy_argument(expression: Expr) -> Result<StrategyArgumentIr> {
    match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value),
            ..
        }) => Ok(StrategyArgumentIr::Bool(value.value)),
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => {
            let text = value.base10_digits();
            if text.starts_with('-') {
                Ok(StrategyArgumentIr::Integer(value.base10_parse()?))
            } else {
                Ok(StrategyArgumentIr::Unsigned(value.base10_parse()?))
            }
        }
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Ok(StrategyArgumentIr::String(value)),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => {
            let Expr::Lit(ExprLit {
                lit: Lit::Int(value),
                ..
            }) = *unary.expr
            else {
                return Err(Error::new_spanned(
                    unary,
                    "negative validator parameters require an integer literal",
                ));
            };
            Ok(StrategyArgumentIr::Integer(-value.base10_parse::<i128>()?))
        }
        Expr::Array(array) => parse_strategy_array(array.elems.into_iter().collect()),
        other => Err(Error::new_spanned(
            other,
            "validator params support bool, integer, string, and homogeneous arrays",
        )),
    }
}

/// Parses a homogeneous array of validator strategy literals.
fn parse_strategy_array(values: Vec<Expr>) -> Result<StrategyArgumentIr> {
    if values.is_empty() {
        return Ok(StrategyArgumentIr::StringList(Vec::new()));
    }
    if matches!(
        values.first(),
        Some(Expr::Lit(ExprLit {
            lit: Lit::Bool(_),
            ..
        }))
    ) {
        return values
            .into_iter()
            .map(|value| match value {
                Expr::Lit(ExprLit {
                    lit: Lit::Bool(value),
                    ..
                }) => Ok(value.value),
                other => Err(Error::new_spanned(
                    other,
                    "validator parameter arrays must be homogeneous",
                )),
            })
            .collect::<Result<Vec<_>>>()
            .map(StrategyArgumentIr::BoolList);
    }
    if values.iter().any(|value| matches!(value, Expr::Unary(_))) {
        return values
            .iter()
            .map(parse_strategy_signed_integer)
            .collect::<Result<Vec<_>>>()
            .map(StrategyArgumentIr::IntegerList);
    }
    if matches!(
        values.first(),
        Some(Expr::Lit(ExprLit {
            lit: Lit::Int(_),
            ..
        }))
    ) {
        return values
            .into_iter()
            .map(|value| match value {
                Expr::Lit(ExprLit {
                    lit: Lit::Int(value),
                    ..
                }) => value.base10_parse::<u128>().map_err(|error| {
                    Error::new_spanned(
                        value,
                        format!("invalid unsigned validator integer: {error}"),
                    )
                }),
                other => Err(Error::new_spanned(
                    other,
                    "validator parameter arrays must be homogeneous",
                )),
            })
            .collect::<Result<Vec<_>>>()
            .map(StrategyArgumentIr::UnsignedList);
    }
    if matches!(
        values.first(),
        Some(Expr::Lit(ExprLit {
            lit: Lit::Str(_),
            ..
        }))
    ) {
        return values
            .into_iter()
            .map(|value| match value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) => Ok(value),
                other => Err(Error::new_spanned(
                    other,
                    "validator parameter arrays must be homogeneous",
                )),
            })
            .collect::<Result<Vec<_>>>()
            .map(StrategyArgumentIr::StringList);
    }
    Err(Error::new_spanned(
        &values[0],
        "validator params support homogeneous bool, integer, and string arrays",
    ))
}

/// Parses one signed integer from an integer literal or unary-minus expression.
fn parse_strategy_signed_integer(value: &Expr) -> Result<i128> {
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value.base10_parse::<i128>().map_err(|error| {
            Error::new_spanned(value, format!("invalid signed validator integer: {error}"))
        }),
        Expr::Unary(unary) if matches!(unary.op, syn::UnOp::Neg(_)) => match unary.expr.as_ref() {
            Expr::Lit(ExprLit {
                lit: Lit::Int(value),
                ..
            }) => value
                .base10_parse::<i128>()
                .map(|value| -value)
                .map_err(|error| {
                    Error::new_spanned(value, format!("invalid signed validator integer: {error}"))
                }),
            other => Err(Error::new_spanned(
                other,
                "negative validator parameters require an integer literal",
            )),
        },
        other => Err(Error::new_spanned(
            other,
            "validator parameter arrays must be homogeneous integer arrays",
        )),
    }
}

/// Parses a field path represented as identifiers or string segments.
fn parse_path_value(expression: Expr) -> Result<Vec<String>> {
    match expression {
        Expr::Path(path) => Ok(path_from_syn(&path.path)),
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Ok(value.value().split('.').map(str::to_owned).collect()),
        other => Err(Error::new_spanned(
            other,
            "expected an identifier path or string path",
        )),
    }
}

/// Converts one path expression into its textual segment.
fn path_text(expression: Expr) -> Result<String> {
    parse_path_value(expression).map(|segments| segments.join("."))
}

/// Converts a Syn path into owned identifier segments.
fn path_from_syn(path: &syn::Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

/// Validates that a model or validator ID is non-empty ASCII text.
pub(super) fn validate_ascii_id(value: &LitStr, kind: &str) -> Result<()> {
    let text = value.value();
    let valid = !text.is_empty()
        && text.split('.').all(|segment| {
            let mut bytes = segment.bytes();
            bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
                && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        });
    if valid {
        Ok(())
    } else {
        Err(Error::new(value.span(), format!("invalid {kind}")))
    }
}
