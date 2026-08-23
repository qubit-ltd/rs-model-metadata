// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Integration tests for field metadata validation and typed queries.

use qubit_model_metadata::AttributeMetadata;
use qubit_model_metadata::DecimalConstraint;
use qubit_model_metadata::DecimalSemantic;
use qubit_model_metadata::ElementMetadata;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::FieldPath;
use qubit_model_metadata::HasTypeShape;
use qubit_model_metadata::LookupRelationMetadata;
use qubit_model_metadata::MapConstraint;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::PrimaryKeyFieldMetadata;
use qubit_model_metadata::PrimaryKeyMetadata;
use qubit_model_metadata::ReferenceMetadata;
use qubit_model_metadata::RoundingMode;
use qubit_model_metadata::SequenceConstraint;
use qubit_model_metadata::StrategyRef;
use qubit_model_metadata::TemporalConstraint;
use qubit_model_metadata::TemporalNormalization;
use qubit_model_metadata::TemporalPrecision;
use qubit_model_metadata::TextConstraint;
use qubit_model_metadata::TextRepertoire;
use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeRef;
use qubit_model_metadata::TypeShape;

struct TemporalValue;
struct DecimalValue;

impl HasTypeShape for TemporalValue {
    const TYPE_SHAPE: TypeShape = TypeShape::Opaque;
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::TEMPORAL;
}

impl HasTypeShape for DecimalValue {
    const TYPE_SHAPE: TypeShape = TypeShape::Opaque;
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::DECIMAL;
}

static INVALID_TEXT_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Text(TextConstraint::new(
    None,
    Some(32),
    None,
    None,
    TextRepertoire::Unicode,
    false,
    None,
))];
static PRIMARY_KEY_FIELDS: [PrimaryKeyFieldMetadata; 1] = [PrimaryKeyFieldMetadata::new("id", true)];
static INVALID_PRIMARY_KEY_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::PrimaryKey(
    PrimaryKeyMetadata::new(&PRIMARY_KEY_FIELDS),
)];
static SEQUENCE_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Sequence(SequenceConstraint::new(
    Some(1),
    Some(3),
    true,
))];
static ELEMENT_TEXT_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Text(TextConstraint::new(
    None,
    None,
    None,
    None,
    TextRepertoire::Ascii,
    false,
    None,
))];
static ELEMENT_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Element(ElementMetadata::new(
    &ELEMENT_TEXT_ATTRIBUTES,
))];
static VALID_TEXT_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Text(TextConstraint::new(
    Some(1),
    Some(32),
    None,
    None,
    TextRepertoire::Unicode,
    true,
    None,
))];
static MAP_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Map(MapConstraint::new(Some(1), Some(4)))];
static TEMPORAL_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Temporal(TemporalConstraint::new(
    TemporalPrecision::Second,
    TemporalNormalization::Utc,
))];
static DECIMAL_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Decimal(DecimalConstraint::new(
    Some(8),
    2,
    RoundingMode::HalfEven,
    DecimalSemantic::Money,
))];
static DECIMAL_ELEMENT_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Decimal(DecimalConstraint::new(
    None,
    0,
    RoundingMode::Down,
    DecimalSemantic::Number,
))];
static ELEMENT_DECIMAL_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Element(ElementMetadata::new(
    &DECIMAL_ELEMENT_ATTRIBUTES,
))];
static RELATION_ATTRIBUTES: [AttributeMetadata; 4] = [
    AttributeMetadata::Reference(ReferenceMetadata::new(
        ModelId::new("test.Target"),
        FieldPath::new(&["id"]),
        true,
        None,
    )),
    AttributeMetadata::LookupRelation(LookupRelationMetadata::new(
        NamedTypeRef::unresolved(TypeIdentity::of::<DecimalValue>()),
        FieldPath::new(&["id"]),
    )),
    AttributeMetadata::Codec(StrategyRef::new("decode")),
    AttributeMetadata::Generator(StrategyRef::new("generate")),
];
static FIXED_SEQUENCE_ATTRIBUTES: [AttributeMetadata; 1] =
    [AttributeMetadata::Sequence(SequenceConstraint::new(None, None, false))];
static FIXED_SEQUENCE_WITH_LENGTH_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Sequence(
    SequenceConstraint::new(Some(1), None, false),
)];
static ELEMENT_DECIMAL_ON_INTEGER_ATTRIBUTES: [AttributeMetadata; 1] = [AttributeMetadata::Element(
    ElementMetadata::new(&DECIMAL_ELEMENT_ATTRIBUTES),
)];

#[test]
fn test_field_metadata_exposes_declaration_details() {
    let field = FieldMetadata::new(2, "amount", "i64", TypeRef::of::<i64>(), &[]);

    assert_eq!(field.ordinal(), 2);
    assert_eq!(field.name(), "amount");
    assert_eq!(field.rust_type_name(), "i64");
    assert_eq!(field.field_type().type_name(), core::any::type_name::<i64>());
    assert!(!field.is_nullable());
    assert!(field.attributes().is_empty());
}

#[test]
fn test_field_metadata_exposes_all_typed_attribute_queries() {
    let text = FieldMetadata::new(0, "name", "String", TypeRef::of::<String>(), &VALID_TEXT_ATTRIBUTES);
    let map = FieldMetadata::new(
        1,
        "labels",
        "HashMap<String, String>",
        TypeRef::of::<std::collections::HashMap<String, String>>(),
        &MAP_ATTRIBUTES,
    );
    let temporal = FieldMetadata::new(
        2,
        "created_at",
        "TemporalValue",
        TypeRef::of::<TemporalValue>(),
        &TEMPORAL_ATTRIBUTES,
    );
    let decimal = FieldMetadata::new(
        3,
        "amount",
        "DecimalValue",
        TypeRef::of::<DecimalValue>(),
        &DECIMAL_ATTRIBUTES,
    );
    let relations = FieldMetadata::new(
        4,
        "target",
        "DecimalValue",
        TypeRef::of::<DecimalValue>(),
        &RELATION_ATTRIBUTES,
    );

    assert_eq!(text.text_constraint().and_then(|value| value.max_chars()), Some(32));
    assert_eq!(map.map_constraint().and_then(|value| value.max_entries()), Some(4));
    assert_eq!(
        temporal.temporal_constraint().map(|value| value.precision()),
        Some(TemporalPrecision::Second)
    );
    assert_eq!(decimal.decimal_constraint().map(|value| value.scale()), Some(2));
    assert!(relations.reference().is_some());
    assert!(relations.lookup_relation().is_some());
    assert_eq!(relations.codec().map(|value| value.name()), Some("decode"));
    assert_eq!(relations.generator().map(|value| value.name()), Some("generate"));
}

#[test]
fn test_field_metadata_accepts_decimal_element_constraints() {
    let field = FieldMetadata::new(
        0,
        "amounts",
        "Vec<DecimalValue>",
        TypeRef::of::<Vec<DecimalValue>>(),
        &ELEMENT_DECIMAL_ATTRIBUTES,
    );

    assert!(field.element_metadata().is_some());
}

#[test]
fn test_field_metadata_accepts_fixed_array_without_length_constraints() {
    let field = FieldMetadata::new(
        0,
        "names",
        "[String; 2]",
        TypeRef::of::<[String; 2]>(),
        &FIXED_SEQUENCE_ATTRIBUTES,
    );

    assert_eq!(field.sequence_constraint().map(|value| value.max_items()), Some(None));
}

#[test]
fn test_field_metadata_reports_nullable_outer_option() {
    let field = FieldMetadata::new(0, "nickname", "Option<String>", TypeRef::of::<Option<String>>(), &[]);

    assert!(field.is_nullable());
}

#[test]
#[should_panic(expected = "text attributes require a text-capable field")]
fn test_field_metadata_rejects_text_constraints_on_non_text_types() {
    let _ = FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &INVALID_TEXT_ATTRIBUTES);
}

#[test]
#[should_panic(expected = "primary-key attributes are only valid at model scope")]
fn test_field_metadata_rejects_model_level_attributes() {
    let _ = FieldMetadata::new(0, "id", "i64", TypeRef::of::<i64>(), &INVALID_PRIMARY_KEY_ATTRIBUTES);
}

#[test]
fn test_field_metadata_exposes_sequence_constraint() {
    let field = FieldMetadata::new(
        0,
        "tags",
        "Vec<String>",
        TypeRef::of::<Vec<String>>(),
        &SEQUENCE_ATTRIBUTES,
    );

    assert_eq!(
        field.sequence_constraint().map(|value| value.max_items()),
        Some(Some(3))
    );
}

#[test]
fn test_field_metadata_exposes_sequence_element_constraints() {
    let field = FieldMetadata::new(
        0,
        "aliases",
        "Vec<String>",
        TypeRef::of::<Vec<String>>(),
        &ELEMENT_ATTRIBUTES,
    );

    let element = field.element_metadata().expect("element metadata");
    assert!(matches!(
        element.attributes(),
        [AttributeMetadata::Text(constraint)]
            if constraint.repertoire() == TextRepertoire::Ascii
    ));
}

#[test]
#[should_panic(expected = "element attributes require a sequence field")]
fn test_field_metadata_rejects_element_constraints_on_scalar_fields() {
    let _ = FieldMetadata::new(0, "alias", "String", TypeRef::of::<String>(), &ELEMENT_ATTRIBUTES);
}

#[test]
#[should_panic(expected = "text attributes require a text-capable element")]
fn test_field_metadata_rejects_constraints_unsupported_by_element_type() {
    let _ = FieldMetadata::new(0, "ids", "Vec<i64>", TypeRef::of::<Vec<i64>>(), &ELEMENT_ATTRIBUTES);
}

#[test]
#[should_panic(expected = "array length is fixed by its type")]
fn test_field_metadata_rejects_length_constraints_on_fixed_arrays() {
    let _ = FieldMetadata::new(
        0,
        "names",
        "[String; 2]",
        TypeRef::of::<[String; 2]>(),
        &FIXED_SEQUENCE_WITH_LENGTH_ATTRIBUTES,
    );
}

#[test]
#[should_panic(expected = "decimal attributes require a decimal-capable element")]
fn test_field_metadata_rejects_decimal_constraints_on_integer_elements() {
    let _ = FieldMetadata::new(
        0,
        "ids",
        "Vec<i64>",
        TypeRef::of::<Vec<i64>>(),
        &ELEMENT_DECIMAL_ON_INTEGER_ATTRIBUTES,
    );
}
