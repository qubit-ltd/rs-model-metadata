// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests validator metadata parsing and diagnostics.

use syn::Attribute;
use syn::parse_quote;

use crate::ir::declaration::StrategyArgumentIr;
use crate::parse::validator::parse_validator;

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
            depends_on(owner(property = owner.id), tenant)
        )]
    );
    let validator = parse_validator(&attribute).expect("supported validator");

    assert_eq!(validator.id.value(), "example.rule");
    assert_eq!(validator.params.len(), 8);
    assert_eq!(
        validator.dependency_bindings,
        vec![
            ("owner".to_owned(), vec![], vec!["owner".to_owned(), "id".to_owned()]),
            ("tenant".to_owned(), vec![], vec!["tenant".to_owned()])
        ]
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
            depends_on(kind(property = owner.kind))
        )]
    );
    let validator = parse_validator(&attribute).expect("named dependency");

    assert_eq!(
        validator.dependency_bindings,
        vec![(
            "kind".to_owned(),
            Vec::new(),
            vec!["owner".to_owned(), "kind".to_owned()]
        )]
    );
}

/// Execution policy is not part of metadata declaration syntax.
#[test]
fn test_reject_execution_policy_options() {
    for attribute in [
        parse_quote!(#[validator(id = "example.rule", target = "container")]),
        parse_quote!(#[validator(id = "example.rule", on_none = "skip")]),
    ] {
        assert!(parse_validator(&attribute).is_err());
    }
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
        assert!(
            parse_validator(&attribute).is_err(),
            "unexpectedly accepted validator attribute: {attribute:?}"
        );
    }
}
