// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow explicit-imports
//! Field overlays preserve resolved, opaque, and symbolic reflection facts.

use std::sync::LazyLock;

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::Utc;
use qubit_model_derive::Model;
use qubit_model_metadata::__private::codegen_v3::descriptor::field as reflect_field;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::TemporalPrecision;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::FieldDescriptor;
use qubit_reflect::Reflect;
use qubit_reflect::TypeDescriptor;
use qubit_reflect::descriptor::TypeRef;
use qubit_reflect::expression::TypeExpression;
use qubit_reflect::identity::Visibility;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct Fields {
    value: String,
    #[reflect(opaque)]
    secret: Vec<u8>,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct SymbolicOwner;

static SYMBOLIC_FIELD: LazyLock<FieldDescriptor> = LazyLock::new(|| {
    let relationship = Box::leak(Box::new(TypeRef::Symbolic(TypeExpression::Parameter("T".into()))));
    reflect_field(
        <SymbolicOwner as Reflect>::type_descriptor,
        0,
        Some("value"),
        Some("value"),
        relationship,
        Visibility::Private,
    )
});

#[test]
fn test_field_overlay_delegates_structure_and_preserves_type_ref_kind() {
    let descriptor = TypeDescriptor::of::<Fields>();
    let resolved = FieldMetadata::from_reflect(
        descriptor.fields()[0].declaring_type().type_id(),
        &descriptor.fields()[0],
    );
    let opaque = FieldMetadata::from_reflect(
        descriptor.fields()[1].declaring_type().type_id(),
        &descriptor.fields()[1],
    );
    let symbolic = FieldMetadata::from_reflect(SYMBOLIC_FIELD.declaring_type().type_id(), &SYMBOLIC_FIELD);

    assert_eq!(resolved.name(), Some("value"));
    assert_eq!(resolved.index(), 0);
    assert!(resolved.descriptor().is_some());
    assert!(matches!(resolved.type_ref(), TypeRef::Resolved(_)));
    assert!(opaque.descriptor().is_none());
    assert!(matches!(opaque.type_ref(), TypeRef::Opaque(_)));
    assert!(
        opaque.attributes().is_empty(),
        "structural overlays do not synthesize domain declarations"
    );
    assert!(
        !opaque.is_opaque(),
        "the domain marker is distinct from an opaque structural type"
    );
    assert!(symbolic.descriptor().is_none());
    assert!(matches!(symbolic.type_ref(), TypeRef::Symbolic(_)));
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
