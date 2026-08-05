// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for pre-validation semantic IR retention.

#![allow(dead_code)]

#[path = "../src/attribute.rs"]
mod attribute;
#[path = "../src/input.rs"]
mod input;
#[path = "../src/normalize.rs"]
mod normalize;

use proc_macro2::Span;
use syn::parse_quote;

use crate::normalize::{
    FieldAttributeIr,
    ModelAttributeIr,
    ModelShapeIr,
};

/// Accepts a span so tests prove every retained occurrence still carries one.
fn assert_span(_: Span) {}

#[test]
fn test_primary_key_ir_retains_generated_field_references_before_validation() {
    let input = parse_quote! {
        #[model(primary_key(fields(id), generated(missing, missing)))]
        struct User {
            id: i64,
        }
    };
    let model = normalize::normalize(
        input::ModelInput::parse(input).expect("parsed model"),
    );
    let primary_key = model
        .attributes
        .iter()
        .find_map(|attribute| match attribute {
            ModelAttributeIr::PrimaryKey(primary_key) => Some(primary_key),
            _ => None,
        })
        .expect("primary-key IR");

    assert_eq!(
        primary_key
            .generated
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        ["missing", "missing"]
    );
    for field in &primary_key.generated {
        assert_span(field.span);
    }
}

#[test]
fn test_unique_ir_retains_ignore_case_field_references_before_validation() {
    let input = parse_quote! {
        #[model(unique(fields(username), ignore_case(missing, missing)))]
        struct User {
            username: String,
        }
    };
    let model = normalize::normalize(
        input::ModelInput::parse(input).expect("parsed model"),
    );
    let unique = model
        .attributes
        .iter()
        .find_map(|attribute| match attribute {
            ModelAttributeIr::Unique(unique) => Some(unique),
            _ => None,
        })
        .expect("unique IR");

    assert_eq!(
        unique
            .ignore_case
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        ["missing", "missing"]
    );
    for field in &unique.ignore_case {
        assert_span(field.span);
    }
}

#[test]
fn test_attribute_ir_retains_repeated_single_value_occurrences() {
    let input = parse_quote! {
        struct User {
            #[model(text(max_chars = 16, max_chars = 32))]
            username: String,
        }
    };
    let model = normalize::normalize(
        input::ModelInput::parse(input).expect("parsed model"),
    );
    let ModelShapeIr::NamedStruct(fields) = &model.shape else {
        panic!("expected named struct IR");
    };
    let text = fields[0]
        .attributes
        .iter()
        .find_map(|attribute| match attribute {
            FieldAttributeIr::Text(text) => Some(text),
            _ => None,
        })
        .expect("text IR");

    assert_eq!(
        text.max_chars
            .iter()
            .map(|occurrence| occurrence.value)
            .collect::<Vec<_>>(),
        [16, 32]
    );
    for occurrence in &text.max_chars {
        assert_span(occurrence.span);
    }
}
