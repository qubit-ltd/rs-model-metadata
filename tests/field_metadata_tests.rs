// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Field overlays preserve concrete, opaque, and symbolic type facts.

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::Utc;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::TemporalPrecision;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::ReflectionContext;
use qubit_reflect::descriptor::TypeExpressionKind;

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct Fields {
    value: String,
    #[opaque]
    secret: Vec<u8>,
}

#[cfg(feature = "generic")]
#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct SymbolicOwner<T> {
    value: T,
}

#[test]
fn test_field_overlay_binds_concrete_and_opaque_type_facts() {
    let metadata = TypeMetadata::of::<Fields>();
    let reflection = ReflectionContext::global();
    let descriptor = metadata.descriptor_in(reflection);
    let resolved = metadata.field("value").expect("resolved field");
    let opaque = metadata.field("secret").expect("opaque field");

    assert_eq!(resolved.name(), Some("value"));
    assert_eq!(resolved.index(), 0);
    assert_eq!(resolved.reflect_in(reflection).unwrap().name(), "value");
    assert!(matches!(
        resolved.type_expression_in(reflection).unwrap().kind(),
        TypeExpressionKind::Named(named) if named.path() == "String"
    ));
    assert!(descriptor.fields().get_at(0).is_some());

    assert!(opaque.is_opaque(), "M static facts preserve the opaque boundary");
    assert!(opaque.reflect_in(reflection).is_none(), "R omits opaque typed fields");
    assert_eq!(opaque.index(), 1);
    assert!(matches!(
        opaque.type_expression_in(reflection).unwrap().kind(),
        TypeExpressionKind::Named(named) if named.path() == "Vec"
    ));
    assert!(
        opaque
            .attributes()
            .iter()
            .any(|attribute| matches!(attribute, FieldAttributeMetadata::Opaque))
    );
    assert!(opaque.attributes().iter().all(|attribute| matches!(
        attribute,
        FieldAttributeMetadata::Opaque | FieldAttributeMetadata::Serde(_)
    )));
}

#[cfg(feature = "generic")]
#[test]
fn test_generic_field_overlay_preserves_symbolic_expression() {
    let definition = TypeMetadata::of::<SymbolicOwner<String>>()
        .generic_definition()
        .expect("generic declaration");
    let field = &definition.fields()[0];
    let reflection = ReflectionContext::global();

    assert_eq!(field.name(), Some("value"));
    assert!(field.reflect_in(reflection).is_none());
    assert!(matches!(
        field.type_expression_in(reflection).map(|expression| expression.kind()),
        Some(TypeExpressionKind::ParameterName("T")),
    ));
}

#[Model(no_redact, no_display, no_debug, no_serialize, no_deserialize)]
struct PrecisionFields {
    #[decimal(precision = 8, scale = 2, min = "1.25", max = "9.75")]
    amount: BigDecimal,
    #[time(precision = second)]
    instant: DateTime<Utc>,
    #[text(non_blank)]
    name: String,
}

/// Typed constraint queries retain declarations even without an execution
/// adapter.
#[test]
fn test_precision_queries_preserve_values_and_absence() {
    let metadata = TypeMetadata::of::<PrecisionFields>();
    let amount = metadata.field("amount").expect("decimal field");
    let instant = metadata.field("instant").expect("time field");
    let name = metadata.field("name").expect("text field");
    let decimal = amount.decimal_constraint().expect("declared precision");
    assert_eq!(decimal.precision(), Some(8));
    assert_eq!(decimal.scale(), 2);
    assert_eq!(decimal.min(), Some("1.25"));
    assert_eq!(decimal.max(), Some("9.75"));
    assert!(decimal.min_inclusive());
    assert!(decimal.max_inclusive());
    assert_eq!(
        instant.time_constraint().expect("declared time precision").precision(),
        TemporalPrecision::Second
    );
    assert!(amount.time_constraint().is_none());
    assert!(instant.decimal_constraint().is_none());
    assert!(name.time_constraint().is_none());
    assert!(name.decimal_constraint().is_none());
    assert!(name.text_constraint().is_some());
}

#[test]
fn test_field_metadata_debug_redacts_decimal_bounds() {
    let metadata = TypeMetadata::of::<PrecisionFields>();
    let amount = metadata.field("amount").expect("decimal field");

    let debug = format!("{amount:?}");
    assert!(
        !debug.contains("1.25"),
        "lower bound leaked through field Debug: {debug}"
    );
    assert!(
        !debug.contains("9.75"),
        "upper bound leaked through field Debug: {debug}"
    );
}
