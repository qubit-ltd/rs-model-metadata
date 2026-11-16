// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Parses validator declarations and typed strategy arguments.

use std::collections::HashSet;

use proc_macro2::Span;
use syn::Attribute;
use syn::Error;
use syn::Expr;
use syn::ExprLit;
use syn::Lit;
use syn::LitInt;
use syn::LitStr;
use syn::Result;
use syn::Token;
use syn::UnOp;
use syn::token::Paren;

use super::fields::validate_ascii_id;
use crate::ir::declaration::StrategyArgumentIr;
use crate::ir::declaration::ValidatorIr;

/// Parses a validator ID, dependencies, and strategy parameters from metadata.
///
/// # Parameters
/// - `attribute`: The validator metadata attribute to parse.
///
/// # Returns
/// The validator's ID, normalized strategy parameters, and dependency bindings.
///
/// # Errors
/// Returns an error for missing or duplicate IDs, malformed or duplicate
/// dependencies and parameters, unsupported options, or invalid values.
pub(crate) fn parse_validator(attribute: &Attribute) -> Result<ValidatorIr> {
    let mut id = None;
    let mut params = Vec::new();
    let depends_on = Vec::new();
    let mut dependency_bindings = Vec::new();
    let mut parameter_names = HashSet::new();
    let mut dependency_paths = HashSet::new();
    let mut dependency_names = HashSet::new();
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            let value: LitStr = meta.value()?.parse()?;
            if id.replace(value).is_some() {
                return Err(meta.error("duplicate validator ID"));
            }
            Ok(())
        } else if meta.path.is_ident("depends_on") {
            meta.parse_nested_meta(|slot| {
                let name = slot
                    .path
                    .get_ident()
                    .ok_or_else(|| slot.error("dependency slot must be an identifier"))?
                    .to_string();
                if !dependency_names.insert(name.clone()) {
                    return Err(slot.error("duplicate validator dependency slot"));
                }
                let mut navigation = Vec::new();
                let property = if slot.input.peek(Paren) {
                    let mut property = None;
                    let mut saw_path = false;
                    slot.parse_nested_meta(|part| {
                        if part.path.is_ident("path") {
                            if saw_path {
                                return Err(part.error("duplicate dependency path"));
                            }
                            saw_path = true;
                            navigation = super::fields::parse_object_path(part.value()?.parse()?)?;
                        } else if part.path.is_ident("property") {
                            if property.is_some() {
                                return Err(part.error("duplicate dependency property"));
                            }
                            property = Some(super::fields::parse_path_value(part.value()?.parse()?)?);
                        } else {
                            return Err(part.error("expected path or property"));
                        }
                        Ok(())
                    })?;
                    property.ok_or_else(|| slot.error("structured dependency requires property"))?
                } else if slot.input.peek(Token![=]) {
                    return Err(slot.error("use slot(property = path) instead of slot = path"));
                } else {
                    vec![name.clone()]
                };
                if property.is_empty() || property.iter().any(String::is_empty) {
                    return Err(slot.error("dependency property cannot be empty"));
                }
                if !dependency_paths.insert((navigation.clone(), property.clone())) {
                    return Err(slot.error("duplicate validator dependency path"));
                }
                dependency_bindings.push((name, navigation, property));
                Ok(())
            })
        } else if meta.path.is_ident("params") {
            meta.parse_nested_meta(|parameter| {
                let name = parameter
                    .path
                    .get_ident()
                    .ok_or_else(|| parameter.error("validator parameter name must be an identifier"))?
                    .to_string();
                if !parameter_names.insert(name.clone()) {
                    return Err(parameter.error(format!("duplicate validator `{name}` parameter")));
                }
                let expression: Expr = parameter.value()?.parse()?;
                params.push((name, parse_strategy_argument(expression)?));
                Ok(())
            })
        } else if meta.path.is_ident("target") || meta.path.is_ident("on_none") {
            Err(meta.error(
                "validator execution policy belongs to the consumer; remove target/on_none from metadata declarations",
            ))
        } else {
            Err(meta.error("unsupported validator option"))
        }
    })?;
    let id = id.ok_or_else(|| Error::new_spanned(attribute, "validator requires `id = \"...\"`"))?;
    validate_ascii_id(&id, "validator ID")?;
    Ok(ValidatorIr {
        id,
        params,
        depends_on,
        dependency_bindings,
    })
}

/// Parses one validator strategy expression into its supported literal variant.
///
/// # Parameters
/// - `expression`: The expression supplied as a validator strategy argument.
///
/// # Returns
/// The typed scalar or homogeneous-list strategy argument.
///
/// # Errors
/// Returns a diagnostic when the expression is not a supported literal shape.
fn parse_strategy_argument(expression: Expr) -> Result<StrategyArgumentIr> {
    match expression {
        Expr::Lit(ExprLit {
            lit: Lit::Bool(value), ..
        }) => Ok(StrategyArgumentIr::Bool(value.value)),
        Expr::Lit(ExprLit {
            lit: Lit::Int(value), ..
        }) => {
            let text = value.base10_digits();
            if text.starts_with('-') {
                Ok(StrategyArgumentIr::Integer(value.base10_parse()?))
            } else {
                Ok(StrategyArgumentIr::Unsigned(value.base10_parse()?))
            }
        }
        Expr::Lit(ExprLit {
            lit: Lit::Str(value), ..
        }) => Ok(StrategyArgumentIr::String(value)),
        Expr::Unary(unary) if matches!(unary.op, UnOp::Neg(_)) => {
            let Expr::Lit(ExprLit {
                lit: Lit::Int(value), ..
            }) = *unary.expr
            else {
                return Err(Error::new_spanned(
                    unary,
                    "negative validator parameters require an integer literal",
                ));
            };
            parse_negative_strategy_integer(&value).map(StrategyArgumentIr::Integer)
        }
        Expr::Array(array) => parse_strategy_array(array.elems.into_iter().collect()),
        other => Err(Error::new_spanned(
            other,
            "validator params support bool, integer, string, and homogeneous arrays",
        )),
    }
}

/// Parses a nonempty homogeneous array of validator strategy literals.
///
/// # Parameters
/// - `values`: The array elements in their original order.
///
/// # Returns
/// The boolean, signed integer, unsigned integer, or string list variant.
///
/// # Errors
/// Returns a diagnostic for an empty, mixed, or unsupported array. A boolean
/// first element selects boolean parsing; otherwise, any unary expression
/// selects signed integer parsing, and the first element selects the remaining
/// literal categories.
fn parse_strategy_array(values: Vec<Expr>) -> Result<StrategyArgumentIr> {
    if values.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "validator parameter arrays cannot be empty because their element type cannot be inferred",
        ));
    }
    if matches!(values.first(), Some(Expr::Lit(ExprLit { lit: Lit::Bool(_), .. }))) {
        return values
            .into_iter()
            .map(|value| match value {
                Expr::Lit(ExprLit {
                    lit: Lit::Bool(value), ..
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
    if matches!(values.first(), Some(Expr::Lit(ExprLit { lit: Lit::Int(_), .. }))) {
        return values
            .into_iter()
            .map(|value| match value {
                Expr::Lit(ExprLit {
                    lit: Lit::Int(value), ..
                }) => value
                    .base10_parse::<u128>()
                    .map_err(|error| Error::new_spanned(value, format!("invalid unsigned validator integer: {error}"))),
                other => Err(Error::new_spanned(
                    other,
                    "validator parameter arrays must be homogeneous",
                )),
            })
            .collect::<Result<Vec<_>>>()
            .map(StrategyArgumentIr::UnsignedList);
    }
    if matches!(values.first(), Some(Expr::Lit(ExprLit { lit: Lit::Str(_), .. }))) {
        return values
            .into_iter()
            .map(|value| match value {
                Expr::Lit(ExprLit {
                    lit: Lit::Str(value), ..
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
///
/// # Parameters
/// - `value`: The expression to parse as a signed integer.
///
/// # Returns
/// The signed value represented by the literal.
///
/// # Errors
/// Returns a diagnostic when the expression is not an integer literal with an
/// optional unary minus, or when its value is out of range.
fn parse_strategy_signed_integer(value: &Expr) -> Result<i128> {
    match value {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value), ..
        }) => value
            .base10_parse::<i128>()
            .map_err(|error| Error::new_spanned(value, format!("invalid signed validator integer: {error}"))),
        Expr::Unary(unary) if matches!(unary.op, UnOp::Neg(_)) => match unary.expr.as_ref() {
            Expr::Lit(ExprLit {
                lit: Lit::Int(value), ..
            }) => parse_negative_strategy_integer(value),
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

/// Parses a negated literal without requiring its positive magnitude to fit in
/// `i128`. Returns a diagnostic at the literal when the signed value
/// underflows.
///
/// # Parameters
/// - `value`: The positive-magnitude integer literal to negate.
///
/// # Returns
/// The representable signed value after negation.
///
/// # Errors
/// Returns a diagnostic when the negated value is outside the `i128` range.
fn parse_negative_strategy_integer(value: &LitInt) -> Result<i128> {
    format!("-{}", value.base10_digits())
        .parse::<i128>()
        .map_err(|error| Error::new_spanned(value, format!("invalid signed validator integer: {error}")))
}
