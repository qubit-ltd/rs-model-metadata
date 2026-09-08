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
use syn::Path;
use syn::Result;
use syn::Token;
use syn::UnOp;

use super::fields::path_from_syn;
use super::fields::validate_ascii_id;
use crate::ir::declaration::OnNoneIr;
use crate::ir::declaration::StrategyArgumentIr;
use crate::ir::declaration::TargetModeIr;
use crate::ir::declaration::ValidatorIr;

/// Parses a validator ID, dependencies, and strategy parameters.
pub(crate) fn parse_validator(attribute: &Attribute) -> Result<ValidatorIr> {
    let mut id = None;
    let mut params = Vec::new();
    let mut depends_on = Vec::new();
    let mut dependency_bindings = Vec::new();
    let mut parameter_names = HashSet::new();
    let mut dependency_paths = HashSet::new();
    let mut dependency_names = HashSet::new();
    let mut saw_named_dependency = false;
    let mut saw_bare_dependency = false;
    let mut target = TargetModeIr::Value;
    let mut on_none = OnNoneIr::Skip;
    let mut saw_target = false;
    let mut saw_on_none = false;
    attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("id") {
            let value: LitStr = meta.value()?.parse()?;
            if id.replace(value).is_some() {
                return Err(meta.error("duplicate validator ID"));
            }
            Ok(())
        } else if meta.path.is_ident("depends_on") {
            meta.parse_nested_meta(|path| {
                if path.input.peek(Token![=]) {
                    if saw_bare_dependency {
                        return Err(path.error("validator dependencies cannot mix named and bare forms"));
                    }
                    saw_named_dependency = true;
                    let name = path
                        .path
                        .get_ident()
                        .ok_or_else(|| path.error("named validator dependency must be an identifier"))?
                        .to_string();
                    if !dependency_names.insert(name.clone()) {
                        return Err(path.error("duplicate validator dependency slot"));
                    }
                    let value: Path = path.value()?.parse()?;
                    let dependency = path_from_syn(&value);
                    if dependency.is_empty() {
                        return Err(path.error("validator dependency path cannot be empty"));
                    }
                    depends_on.push(dependency.clone());
                    dependency_bindings.push((name, dependency));
                } else {
                    if saw_named_dependency {
                        return Err(path.error("validator dependencies cannot mix named and bare forms"));
                    }
                    saw_bare_dependency = true;
                    let dependency = path_from_syn(&path.path);
                    let name = dependency.join(".");
                    if !dependency_paths.insert(dependency.clone()) {
                        return Err(path.error("duplicate validator dependency path"));
                    }
                    if !dependency_names.insert(name) {
                        return Err(path.error("duplicate validator dependency slot"));
                    }
                    depends_on.push(dependency);
                }
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
        } else if meta.path.is_ident("target") {
            if saw_target {
                return Err(meta.error("duplicate validator `target` option"));
            }
            saw_target = true;
            let value: LitStr = meta.value()?.parse()?;
            target = match value.value().as_str() {
                "value" => TargetModeIr::Value,
                "container" => TargetModeIr::Container,
                _ => return Err(meta.error("validator target must be value or container")),
            };
            Ok(())
        } else if meta.path.is_ident("on_none") {
            if saw_on_none {
                return Err(meta.error("duplicate validator `on_none` option"));
            }
            saw_on_none = true;
            let value: LitStr = meta.value()?.parse()?;
            on_none = match value.value().as_str() {
                "skip" => OnNoneIr::Skip,
                "reject" => OnNoneIr::Reject,
                _ => return Err(meta.error("validator on_none must be skip or reject")),
            };
            Ok(())
        } else {
            Err(meta.error("unsupported validator option"))
        }
    })?;
    let id = id.ok_or_else(|| Error::new_spanned(attribute, "validator requires `id = \"...\"`"))?;
    validate_ascii_id(&id, "validator ID")?;
    if matches!(target, TargetModeIr::Container) && matches!(on_none, OnNoneIr::Reject) {
        return Err(Error::new_spanned(
            attribute,
            "validator `on_none = \"reject\"` requires target = \"value\"",
        ));
    }
    Ok(ValidatorIr {
        id,
        params,
        depends_on,
        dependency_bindings,
        target,
        on_none,
    })
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

    /// Preserves a named dependency slot separately from its source path.
    #[test]
    fn test_parse_named_validator_dependency() {
        let attribute: Attribute = parse_quote!(
            #[validator(
                id = "example.rule",
                depends_on(kind = owner::kind),
                target = "container",
                on_none = "skip"
            )]
        );
        let validator = parse_validator(&attribute).expect("named dependency");

        assert_eq!(
            validator.dependency_bindings,
            vec![("kind".to_owned(), vec!["owner".to_owned(), "kind".to_owned()])]
        );
        assert_eq!(validator.target, crate::ir::declaration::TargetModeIr::Container);
        assert_eq!(validator.on_none, crate::ir::declaration::OnNoneIr::Skip);
    }

    /// Covers missing IDs, unsupported options, and heterogeneous arrays.
    #[test]
    fn test_parse_validator_errors() {
        let cases: [Attribute; 10] = [
            parse_quote!(#[validator(params(value = 1))]),
            parse_quote!(#[validator(id = "bad id")]),
            parse_quote!(#[validator(id = "a", id = "b")]),
            parse_quote!(#[validator(id = "a", unsupported)]),
            parse_quote!(#[validator(id = "a", params(value = [true, 1]))]),
            parse_quote!(#[validator(id = "a", params(value = [-true]))]),
            parse_quote!(#[validator(id = "a", params(value = [path]))]),
            parse_quote!(#[validator(id = "a", depends_on(value = first, value = second))]),
            parse_quote!(#[validator(id = "a", target = "unknown")]),
            parse_quote!(#[validator(id = "a", target = "container", on_none = "reject")]),
        ];

        for attribute in cases {
            assert!(parse_validator(&attribute).is_err());
        }
    }
}
