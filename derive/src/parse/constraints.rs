// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parses field and selector constraint attributes.

use core::cmp::Ordering;
use std::collections::HashSet;
use std::iter::repeat_n;

use syn::Attribute;
use syn::Error;
use syn::Expr;
use syn::LitBool;
use syn::LitInt;
use syn::LitStr;
use syn::Result;

use super::fields::parse_ident_value;
use super::vocabulary::validate_closed_value;
use crate::ir::declaration::ConstraintIr;
use crate::ir::declaration::DecimalConstraintIr;
use crate::ir::declaration::TextConstraintIr;

/// Reports whether `attribute` names one of the supported constraints.
pub(crate) fn is_constraint_attribute(attribute: &Attribute) -> bool {
    attribute.path().is_ident("text")
        || attribute.path().is_ident("decimal")
        || attribute.path().is_ident("money")
        || attribute.path().is_ident("time")
        || attribute.path().is_ident("sequence")
        || attribute.path().is_ident("map")
}

/// Parses one textual, decimal, temporal, sequence, or map constraint.
pub(crate) fn parse_constraint(attribute: &Attribute) -> Result<ConstraintIr> {
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
                let expression: Expr = meta.value()?.parse()?;
                let value = parse_ident_value(expression.clone())?;
                validate_closed_value(
                    &expression,
                    &value,
                    &["second", "millisecond", "microsecond", "nanosecond"],
                    "invalid time precision",
                )?;
                if precision.replace(value).is_some() {
                    return Err(meta.error("duplicate time `precision` option"));
                }
                Ok(())
            } else {
                Err(meta.error("unsupported time option"))
            }
        })?;
        return Ok(ConstraintIr::Time(
            precision.ok_or_else(|| Error::new_spanned(attribute, "time requires precision"))?,
        ));
    }
    if attribute.path().is_ident("sequence") {
        let (mut min, mut max, mut unique) = (None, None, false);
        let mut any = false;
        let mut seen = HashSet::new();
        attribute.parse_nested_meta(|meta| {
            any = true;
            let option = meta.path.get_ident().map(ToString::to_string);
            if let Some(option) = option.as_deref()
                && !seen.insert(option.to_owned())
            {
                return Err(meta.error(format!("duplicate sequence `{option}` option")));
            }
            if meta.path.is_ident("min_items") {
                min = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
                Ok(())
            } else if meta.path.is_ident("max_items") {
                max = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
                Ok(())
            } else if meta.path.is_ident("unique_items") {
                unique = true;
                Ok(())
            } else {
                Err(meta.error("unsupported sequence option"))
            }
        })?;
        if !any {
            return Err(Error::new_spanned(attribute, "sequence requires at least one option"));
        }
        return Ok(ConstraintIr::Sequence { min, max, unique });
    }
    let (mut min, mut max) = (None, None);
    let mut any = false;
    let mut seen = HashSet::new();
    attribute.parse_nested_meta(|meta| {
        any = true;
        let option = meta.path.get_ident().map(ToString::to_string);
        if let Some(option) = option.as_deref()
            && !seen.insert(option.to_owned())
        {
            return Err(meta.error(format!("duplicate map `{option}` option")));
        }
        if meta.path.is_ident("min_entries") {
            min = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_entries") {
            max = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported map option"))
        }
    })?;
    if !any {
        return Err(Error::new_spanned(attribute, "map requires min_entries or max_entries"));
    }
    Ok(ConstraintIr::Map { min, max })
}

/// Parses text length, character-set, blankness, and format options.
fn parse_text_constraint(attribute: &Attribute) -> Result<TextConstraintIr> {
    let mut value = TextConstraintIr::default();
    let mut any = false;
    let mut seen = HashSet::new();
    attribute.parse_nested_meta(|meta| {
        any = true;
        let option = meta.path.get_ident().map(ToString::to_string);
        if let Some(option) = option.as_deref()
            && !seen.insert(option.to_owned())
        {
            return Err(meta.error(format!("duplicate text `{option}` option")));
        }
        if meta.path.is_ident("min_chars") {
            value.min_chars = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_chars") {
            value.max_chars = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("min_bytes") {
            value.min_bytes = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("max_bytes") {
            value.max_bytes = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("non_blank") {
            value.non_blank = true;
            Ok(())
        } else if meta.path.is_ident("allowed_chars") {
            let expression: Expr = meta.value()?.parse()?;
            let allowed_chars = parse_ident_value(expression.clone())?;
            validate_closed_value(
                &expression,
                &allowed_chars,
                &["unicode", "printable_unicode", "ascii", "printable_ascii", "code"],
                "invalid allowed_chars value",
            )?;
            value.allowed_chars = Some(allowed_chars);
            Ok(())
        } else if meta.path.is_ident("format") {
            let expression: Expr = meta.value()?.parse()?;
            let format = parse_ident_value(expression.clone())?;
            validate_closed_value(
                &expression,
                &format,
                &["email", "cn_mobile", "uri", "uuid"],
                "invalid text format",
            )?;
            value.format = Some(format);
            Ok(())
        } else {
            Err(meta.error("unsupported text option"))
        }
    })?;
    if !any {
        return Err(Error::new_spanned(attribute, "text requires at least one option"));
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
    let mut seen = HashSet::new();
    attribute.parse_nested_meta(|meta| {
        any = true;
        let option = meta.path.get_ident().map(ToString::to_string);
        if let Some(option) = option.as_deref()
            && !seen.insert(option.to_owned())
        {
            return Err(meta.error(format!("duplicate decimal `{option}` option")));
        }
        if meta.path.is_ident("precision") {
            precision = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("scale") {
            scale = Some(meta.value()?.parse::<LitInt>()?.base10_parse()?);
            Ok(())
        } else if meta.path.is_ident("rounding") {
            let expression: Expr = meta.value()?.parse()?;
            let value = parse_ident_value(expression.clone())?;
            validate_closed_value(
                &expression,
                &value,
                &[
                    "down",
                    "up",
                    "ceiling",
                    "floor",
                    "half_up",
                    "half_down",
                    "half_even",
                    "unnecessary",
                ],
                "invalid rounding mode",
            )?;
            rounding = Some(value);
            Ok(())
        } else if meta.path.is_ident("min") {
            min = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("max") {
            max = Some(meta.value()?.parse()?);
            Ok(())
        } else if meta.path.is_ident("min_inclusive") {
            min_inclusive = meta.value()?.parse::<LitBool>()?.value;
            Ok(())
        } else if meta.path.is_ident("max_inclusive") {
            max_inclusive = meta.value()?.parse::<LitBool>()?.value;
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
        return Err(Error::new_spanned(attribute, "decimal scale cannot exceed precision"));
    }
    if let (Some(minimum), Some(maximum)) = (&min, &max) {
        match compare_decimal_literals(&minimum.value(), &maximum.value()) {
            Some(Ordering::Greater) => {
                return Err(Error::new_spanned(attribute, "decimal min cannot exceed max"));
            }
            Some(Ordering::Equal) if !min_inclusive && !max_inclusive => {
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
    let (negative, unsigned) = value.strip_prefix('-').map_or((false, value), |value| (true, value));
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
    let normalized = if digits.is_empty() { "0".to_owned() } else { digits };
    let scale = fraction.len();
    Some((negative && normalized != "0", normalized, scale))
}

/// Compares two normalized decimal literal strings without floating point.
fn compare_decimal_literals(left: &str, right: &str) -> Option<Ordering> {
    let (left_negative, mut left_digits, left_scale) = parse_decimal_literal(left)?;
    let (right_negative, mut right_digits, right_scale) = parse_decimal_literal(right)?;
    let scale = left_scale.max(right_scale);
    left_digits.extend(repeat_n('0', scale - left_scale));
    right_digits.extend(repeat_n('0', scale - right_scale));
    let magnitude = left_digits
        .len()
        .cmp(&right_digits.len())
        .then_with(|| left_digits.cmp(&right_digits));
    Some(match (left_negative, right_negative) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (true, true) => magnitude.reverse(),
        (false, false) => magnitude,
    })
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;

    use syn::parse_quote;

    use super::compare_decimal_literals;
    use super::parse_constraint;
    use super::parse_decimal_literal;

    /// Verifies the documented decimal spelling boundaries and normalization
    /// policy.
    #[test]
    fn test_decimal_literal_boundaries() {
        assert_eq!(parse_decimal_literal("1."), Some((false, "1".into(), 0)));
        assert_eq!(parse_decimal_literal("001.2300"), Some((false, "123".into(), 2)));
        assert_eq!(parse_decimal_literal("-0.0"), Some((false, "0".into(), 0)));
        assert_eq!(parse_decimal_literal("+1"), None);
        assert_eq!(parse_decimal_literal("1.2.3"), None);
        assert_eq!(compare_decimal_literals("001.20", "1.2"), Some(Ordering::Equal));
    }

    /// Confirms closed-vocabulary constraint options are rejected by parsing.
    #[test]
    fn test_rejects_unknown_closed_vocabulary_values() {
        for attribute in [
            parse_quote!(#[text(allowed_chars = unsupported)]),
            parse_quote!(#[text(format = unsupported)]),
            parse_quote!(#[decimal(rounding = unsupported)]),
            parse_quote!(#[time(precision = unsupported)]),
        ] {
            assert!(parse_constraint(&attribute).is_err());
        }
    }
}
