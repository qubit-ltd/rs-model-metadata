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
use syn::LitStr;
use syn::Result;
use syn::UnOp;

use super::fields::path_from_syn;
use super::fields::validate_ascii_id;
use crate::ir::declaration::StrategyArgumentIr;
use crate::ir::declaration::ValidatorIr;

/// Parses a validator ID, dependencies, and strategy parameters.
pub(crate) fn parse_validator(attribute: &Attribute) -> Result<ValidatorIr> {
    let mut id = None;
    let mut params = Vec::new();
    let mut depends_on = Vec::new();
    let mut parameter_names = HashSet::new();
    let mut dependency_paths = HashSet::new();
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            let value: LitStr = meta.value()?.parse()?;
            if id.replace(value).is_some() {
                return Err(meta.error("duplicate validator ID"));
            }
            Ok(())
        } else if meta.path.is_ident("depends_on") {
            meta.parse_nested_meta(|path| {
                let dependency = path_from_syn(&path.path);
                if !dependency_paths.insert(dependency.clone()) {
                    return Err(path.error("duplicate validator dependency path"));
                }
                depends_on.push(dependency);
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
        } else {
            Err(meta.error("unsupported validator option"))
        }
    })?;
    let id = id.ok_or_else(|| Error::new_spanned(attribute, "validator requires `id = \"...\"`"))?;
    validate_ascii_id(&id, "validator ID")?;
    Ok(ValidatorIr { id, params, depends_on })
}

/// Parses one validator strategy argument into a supported literal variant.
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
            }) => value
                .base10_parse::<i128>()
                .map(|value| -value)
                .map_err(|error| Error::new_spanned(value, format!("invalid signed validator integer: {error}"))),
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

#[cfg(test)]
mod tests {
    use syn::Attribute;
    use syn::parse_quote;

    use super::parse_validator;
    use crate::ir::declaration::StrategyArgumentIr;

    /// Parses every supported scalar and homogeneous validator argument shape.
    #[test]
    fn test_parse_validator_argument_shapes() {
        let attribute: Attribute = parse_quote!(
            #[validator(
                id = "example.rule",
                params(
                    enabled = true,
                    count = 7,
                    offset = -2,
                    label = "ok",
                    flags = [true, false],
                    counts = [1, 2],
                    offsets = [-1, 2],
                    labels = ["a", "b"]
                ),
                depends_on(owner::id, tenant)
            )]
        );
        let validator = parse_validator(&attribute).expect("supported validator");

        assert_eq!(validator.id.value(), "example.rule");
        assert_eq!(validator.params.len(), 8);
        assert_eq!(
            validator.depends_on,
            vec![vec!["owner".to_owned(), "id".to_owned()], vec!["tenant".to_owned()]]
        );
        assert!(matches!(validator.params[0].1, StrategyArgumentIr::Bool(true)));
        assert!(matches!(validator.params[1].1, StrategyArgumentIr::Unsigned(7)));
        assert!(matches!(validator.params[2].1, StrategyArgumentIr::Integer(-2)));
        assert!(matches!(validator.params[3].1, StrategyArgumentIr::String(_)));
        assert!(matches!(validator.params[4].1, StrategyArgumentIr::BoolList(_)));
        assert!(matches!(validator.params[5].1, StrategyArgumentIr::UnsignedList(_)));
        assert!(matches!(validator.params[6].1, StrategyArgumentIr::IntegerList(_)));
        assert!(matches!(validator.params[7].1, StrategyArgumentIr::StringList(_)));
    }

    /// Covers missing IDs, unsupported options, and heterogeneous arrays.
    #[test]
    fn test_parse_validator_errors() {
        let cases: [Attribute; 7] = [
            parse_quote!(#[validator(params(value = 1))]),
            parse_quote!(#[validator(id = "bad id")]),
            parse_quote!(#[validator(id = "a", id = "b")]),
            parse_quote!(#[validator(id = "a", unsupported)]),
            parse_quote!(#[validator(id = "a", params(value = [true, 1]))]),
            parse_quote!(#[validator(id = "a", params(value = [-true]))]),
            parse_quote!(#[validator(id = "a", params(value = [path]))]),
        ];

        for attribute in cases {
            assert!(parse_validator(&attribute).is_err());
        }
    }
}
