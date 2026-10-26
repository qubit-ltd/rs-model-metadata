// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests declaration option parsing through its crate-visible contract.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Meta;
use syn::Token;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use crate::parse::options::parse_declaration_options;

/// Covers every supported declaration option and its stored representation.
#[test]
fn test_parse_all_declaration_options() {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let options = parser
        .parse2(quote!(
            id = "example.Model",
            source_id = "example.Source",
            source = Source,
            codec = Codec,
            open,
            transparent
        ))
        .expect("option syntax");
    let parsed = parse_declaration_options(options).expect("supported options");

    assert_eq!(parsed.id.expect("id").value(), "example.Model");
    assert_eq!(parsed.source_id.expect("source id").value(), "example.Source");
    assert!(parsed.source.is_some());
    assert!(parsed.codec.is_some());
    assert!(parsed.open && parsed.transparent);
}

/// Preserves every supported behavior switch for capability expansion.
#[test]
fn test_parse_behavior_options() {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    for name in ["eq", "hash", "no_hash", "copy", "default", "partial_ord", "ord"] {
        let option: TokenStream = name.parse().expect("option tokens");
        let options = parser.parse2(option).expect("option syntax");
        let options = parse_declaration_options(options).expect("supported behavior option");
        assert!(options.behavior.contains(name));
    }
}

/// Confirms unsupported and duplicate options are accumulated as diagnostics.
#[test]
fn test_parse_declaration_option_errors() {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let options = parser
        .parse2(quote!(
            id = "one",
            id = 2,
            source = One,
            source = Two,
            codec = A,
            codec = B,
            open,
            open,
            unknown
        ))
        .expect("option syntax");
    let error = match parse_declaration_options(options) {
        Ok(_) => panic!("invalid options were accepted"),
        Err(error) => error,
    };
    let text = error.into_compile_error().to_string();

    assert!(text.contains("duplicate `id` option"));
    assert!(text.contains("duplicate `source` option"));
    assert!(text.contains("duplicate `codec` option"));
    assert!(text.contains("duplicate `open` option"));
    assert!(text.contains("unsupported model option"));
}
