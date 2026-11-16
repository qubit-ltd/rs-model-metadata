// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests field-attribute parsing through its crate-internal contract.

use syn::Field;
use syn::parse_quote;

use crate::ir::declaration::CodecIr;
use crate::ir::declaration::ConstraintIr;
use crate::ir::declaration::FieldOccurrence;
use crate::ir::declaration::IdentifierAssignmentIr;
use crate::ir::declaration::RedactModeIr;
use crate::ir::declaration::ReferenceTargetIr;
use crate::ir::declaration::SelectorPositionIr;
use crate::ir::declaration::StrategyArgumentIr;
use crate::parse::fields::parse_field;

/// Exercises the complete supported field-attribute vocabulary in source
/// order.
#[test]
fn test_parse_field_attribute_vocabulary() {
    let field: Field = parse_quote! {
        #[identifier(assigned_by = database)]
        #[indexed]
        #[unique(respect_to(tenant::id), ignore_case = false)]
        #[reference(entity_id = "example.Owner", property = id, path = "owner/id", existing = false)]
        #[key_part(order = 0)]
        #[text(min_chars = 1, max_chars = 8, non_blank, allowed_chars = ascii, format = email_ascii)]
        #[sequence(min_items = 1, max_items = 3, unique_items)]
        #[element(
            text(max_chars = 4),
            validator(id = "example.element"),
            codec(id = "example.codec"),
            redact(level = "public")
        )]
        #[validator(id = "example.field", params(limit = 3))]
        #[codec(type = Codec)]
        #[redact(keyed_by = owner::id)]
        #[serde(
            rename(serialize = "out", deserialize = "in"),
            skip_serializing,
            flatten,
            with = "helper",
            default = "make",
            skip_serializing_if = "skip",
            unknown = true
        )]
        #[opaque]
        #[keep_serializing]
        value: Vec<String>
    };
    let parsed = parse_field(3, &field.ty, &field.attrs, true).expect("supported field attributes");

    assert_eq!(*parsed.index.value(), 3);
    assert!(parsed.named);
    assert!(parsed.keep_serializing);
    let [
        FieldOccurrence::Identifier(identifier),
        FieldOccurrence::Indexed,
        FieldOccurrence::Unique(unique),
        FieldOccurrence::Reference(reference),
        FieldOccurrence::KeyPart(0),
        FieldOccurrence::Constraint(ConstraintIr::Text(text)),
        FieldOccurrence::Constraint(ConstraintIr::Sequence {
            min: Some(1),
            max: Some(3),
            unique: true,
        }),
        FieldOccurrence::Selector(selector),
        FieldOccurrence::Validator(field_validator),
        FieldOccurrence::Codec(CodecIr::RustType(codec_type)),
        FieldOccurrence::Redact(redact),
        FieldOccurrence::Serde(serde),
        FieldOccurrence::Opaque,
    ] = parsed.occurrences.as_slice()
    else {
        panic!("field attributes should produce their exact IR variants and order");
    };

    assert!(matches!(identifier, IdentifierAssignmentIr::Database));
    assert_eq!(unique.respect_to, vec![vec!["tenant".to_owned(), "id".to_owned()]]);
    assert_eq!(unique.ignore_case, Some(false));

    let ReferenceTargetIr::ModelId(entity_id) = &reference.target else {
        panic!("reference entity_id should produce a model ID target");
    };
    assert_eq!(entity_id.value(), "example.Owner");
    assert_eq!(
        reference.property.as_ref().expect("property path"),
        &vec!["id".to_owned()]
    );
    assert!(!reference.existing);
    assert_eq!(
        reference.same_as.as_ref().expect("object path"),
        &vec!["owner".to_owned(), "id".to_owned()]
    );

    assert_eq!(text.min_chars, Some(1));
    assert_eq!(text.max_chars, Some(8));
    assert!(text.non_blank);
    assert_eq!(text.allowed_chars.as_deref(), Some("ascii"));
    assert_eq!(text.format.as_deref(), Some("email_ascii"));

    assert!(matches!(&selector.position, SelectorPositionIr::Element));
    assert!(matches!(
        selector.constraints.as_slice(),
        [ConstraintIr::Text(text)] if text.max_chars == Some(4)
    ));
    assert_eq!(selector.validators.len(), 1);
    assert_eq!(selector.validators[0].id.value(), "example.element");
    assert!(matches!(
        &selector.codec,
        Some(CodecIr::DeclaredId(id)) if id.value() == "example.codec"
    ));
    assert!(matches!(
        &selector.redact,
        Some(redact) if matches!(&redact.mode, RedactModeIr::Level(level) if level == "public")
    ));

    assert_eq!(field_validator.id.value(), "example.field");
    assert_eq!(field_validator.params.len(), 1);
    assert_eq!(field_validator.params[0].0, "limit");
    assert!(matches!(&field_validator.params[0].1, StrategyArgumentIr::Unsigned(3)));

    let syn::Type::Path(codec_type) = codec_type.as_ref() else {
        panic!("field codec type should be a Rust path");
    };
    assert_eq!(codec_type.path.segments.last().expect("codec path").ident, "Codec");
    assert!(matches!(&redact.mode, RedactModeIr::KeyedBy(path) if path == "owner.id"));

    assert_eq!(serde.serialize_name.as_ref().expect("serialize name").value(), "out");
    assert_eq!(serde.deserialize_name.as_ref().expect("deserialize name").value(), "in");
    assert!(serde.skip_serializing);
    assert!(!serde.skip_deserializing);
    assert!(serde.flatten);
    assert_eq!(serde.with.as_ref().expect("Serde adapter").value(), "helper");
    assert!(serde.default);
    assert!(serde.explicit_skip_serializing_if);

    assert!(matches!(
        parsed.occurrences.as_slice(),
        [
            FieldOccurrence::Identifier(_),
            FieldOccurrence::Indexed,
            FieldOccurrence::Unique(_),
            FieldOccurrence::Reference(_),
            FieldOccurrence::KeyPart(0),
            FieldOccurrence::Constraint(ConstraintIr::Text(_)),
            FieldOccurrence::Constraint(ConstraintIr::Sequence { .. }),
            FieldOccurrence::Selector(_),
            FieldOccurrence::Validator(_),
            FieldOccurrence::Codec(_),
            FieldOccurrence::Redact(_),
            FieldOccurrence::Serde(_),
            FieldOccurrence::Opaque,
        ]
    ));
}

/// Confirms singleton field markers and redact levels are rejected early.
#[test]
fn test_rejects_duplicate_markers_and_unknown_redact_level() {
    let fields: [Field; 7] = [
        parse_quote!(#[identifier] #[identifier] value: String),
        parse_quote!(#[identifier(assigned_by = application, assigned_by = database)] value: String),
        parse_quote!(#[indexed] #[indexed] value: String),
        parse_quote!(#[opaque] #[opaque] value: String),
        parse_quote!(#[indexed(unexpected)] value: String),
        parse_quote!(#[opaque(unexpected)] value: String),
        parse_quote!(#[redact(level = "unsupported")] value: String),
    ];
    for field in fields {
        assert!(
            parse_field(0, &field.ty, &field.attrs, true).is_err(),
            "expected duplicate or unsupported attributes to be rejected: {field:?}"
        );
    }
}

/// Covers invalid field options and marker shapes without panicking.
#[test]
fn test_parse_field_attribute_errors() {
    let fields: [Field; 8] = [
        parse_quote!(#[identifier(other)] value: String),
        parse_quote!(#[identifier(assigned_by = other)] value: String),
        parse_quote!(#[reference(property = id)] value: String),
        parse_quote!(#[key_part] value: String),
        parse_quote!(#[codec] value: String),
        parse_quote!(#[redact] value: String),
        parse_quote!(#[element(sequence(min_items = 1))] value: Vec<String>),
        parse_quote!(#[keep_serializing = true] value: String),
    ];

    for field in fields {
        assert!(
            parse_field(0, &field.ty, &field.attrs, true).is_err(),
            "expected invalid field attributes to be rejected: {field:?}"
        );
    }
}
