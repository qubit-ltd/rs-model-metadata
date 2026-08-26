// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Crate-internal tests for shared declaration-attribute support.

use syn::DeriveInput;
use syn::parse_quote;

use crate::attribute_support::has_must_use;
use crate::attribute_support::serialized_variant_name;

#[test]
fn test_serialized_variant_name_honors_serde_serialization_rename() {
    let variant = parse_quote! {
        #[serde(rename(serialize = "custom-name"))]
        CustomName
    };

    assert_eq!(
        serialized_variant_name(&variant).expect("the variant should parse"),
        "custom-name"
    );
}

#[test]
fn test_serialized_variant_name_ignores_serde_alias() {
    let variant = parse_quote! {
        #[serde(alias = "OLD_VALUE")]
        Value
    };

    assert_eq!(
        serialized_variant_name(&variant).expect("the variant should parse"),
        "VALUE"
    );
}

#[test]
fn test_serialized_variant_name_reads_combined_serde_rename() {
    let variant = parse_quote! {
        #[serde(rename(serialize = "OUT", deserialize = "IN"))]
        Value
    };

    assert_eq!(
        serialized_variant_name(&variant).expect("the variant should parse"),
        "OUT"
    );
}

#[test]
fn test_serialized_variant_name_defaults_to_screaming_snake_case() {
    let variant = parse_quote!(InReview);

    assert_eq!(
        serialized_variant_name(&variant).expect("the variant should parse"),
        "IN_REVIEW"
    );
}

#[test]
fn test_has_must_use_recognizes_the_declaration_attribute() {
    let input: DeriveInput = parse_quote! {
        #[must_use]
        enum Status { Draft }
    };

    assert!(has_must_use(&input.attrs));
}
