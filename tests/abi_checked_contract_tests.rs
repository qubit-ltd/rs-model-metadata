// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for the checked generated-code v7 ABI.

#[cfg(feature = "generic")]
use qubit_model_derive::Enum;
use qubit_model_metadata::__private::ModelTypeSeal;
use qubit_model_metadata::__private::TypeMetadataProvider;
use qubit_model_metadata::__private::v7;
use qubit_model_metadata::metadata::ConstraintMetadata;
use qubit_model_metadata::metadata::FieldAttributeMetadata;
use qubit_model_metadata::metadata::FieldMetadata;
use qubit_model_metadata::metadata::GetterMetadata;
use qubit_model_metadata::metadata::GetterOutputKind;
use qubit_model_metadata::metadata::IndexingReasons;
use qubit_model_metadata::metadata::PropertyAccessError;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::SelectorMetadata;
use qubit_model_metadata::metadata::SelectorPosition;
use qubit_model_metadata::metadata::SequenceConstraint;
use qubit_model_metadata::metadata::SerdeFieldMetadata;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::Reflect;
use qubit_reflect::ReflectedRef;
use qubit_reflect::TypeDescriptor;

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct OneField {
    value: String,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct TwoFields {
    first: String,
    second: String,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct WrongTarget;

impl ModelTypeSeal for WrongTarget {}

impl TypeMetadataProvider for WrongTarget {
    fn __type_metadata() -> &'static TypeMetadata {
        static METADATA: std::sync::OnceLock<TypeMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| {
            let role = v7::leak(v7::model_role());
            v7::GeneratedTypeMetadataBuilder::new(TypeDescriptor::of::<OneField>(), None, &[], role).finish_unchecked()
        })
    }
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
enum OneVariant {
    Ready,
}

fn panic_message(action: impl FnOnce() + std::panic::UnwindSafe) -> String {
    let payload = std::panic::catch_unwind(action).expect_err("invalid ABI input must panic");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|message| (*message).to_owned()))
        .expect("ABI panic payload must be text")
}

fn unavailable_getter(_: ReflectedRef<'_>) -> Result<PropertyValue<'_>, PropertyAccessError> {
    Err(PropertyAccessError::AdapterUnavailable)
}

#[test]
fn finish_rejects_descriptor_and_field_overlay_mismatches() {
    let role = v7::leak(v7::model_role());
    let descriptor = TypeDescriptor::of::<OneField>();

    let mismatch = panic_message(|| {
        let _ = v7::GeneratedTypeMetadataBuilder::new(descriptor, None, &[], role).finish::<WrongTarget>();
    });
    assert!(mismatch.starts_with("QMM-ABI-001:"));

    let missing_field = panic_message(|| {
        let _ = v7::GeneratedTypeMetadataBuilder::new(descriptor, None, &[], role).finish::<OneField>();
    });
    assert!(missing_field.starts_with("QMM-ABI-003:"));
}

#[test]
fn try_of_returns_structured_abi_violation_without_panicking() {
    let error = TypeMetadata::try_of::<WrongTarget>().expect_err("wrong descriptor must fail");
    assert_eq!(error.code(), "QMM-ABI-001");
}

#[test]
fn finish_rejects_duplicate_properties_and_wrong_getter_targets() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let fields = v7::leak_slice(vec![FieldMetadata::from_reflect(
        descriptor.fields()[0].declaring_type().type_id(),
        &descriptor.fields()[0],
    )]);
    let role = v7::leak(v7::model_role());
    let property = v7::property_metadata("value", fields[0].type_ref(), Some(&fields[0]), None, None);
    let properties = v7::leak_slice(vec![property, property]);
    let duplicate = panic_message(|| {
        let _ = v7::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role)
            .properties(properties)
            .finish::<OneField>();
    });
    assert!(duplicate.starts_with("QMM-ABI-004:"));

    let getter = v7::leak(GetterMetadata::new::<WrongTarget>(
        "value",
        fields[0].type_ref(),
        GetterOutputKind::Borrowed,
        unavailable_getter,
    ));
    let properties = v7::leak_slice(vec![v7::property_metadata(
        "value",
        fields[0].type_ref(),
        Some(&fields[0]),
        Some(getter),
        None,
    )]);
    let wrong_target = panic_message(|| {
        let _ = v7::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role)
            .properties(properties)
            .finish::<OneField>();
    });
    assert!(wrong_target.starts_with("QMM-ABI-004:"));
}

#[test]
fn finish_rejects_invalid_role_payloads() {
    let one_descriptor = TypeDescriptor::of::<OneField>();
    let one_fields = v7::leak_slice(vec![FieldMetadata::from_reflect(
        one_descriptor.fields()[0].declaring_type().type_id(),
        &one_descriptor.fields()[0],
    )]);
    let entity_role = v7::leak(v7::entity_role(&one_fields[0]));
    let invalid_identifier = panic_message(|| {
        let _ =
            v7::GeneratedTypeMetadataBuilder::new(one_descriptor, None, one_fields, entity_role).finish::<OneField>();
    });
    assert!(invalid_identifier.starts_with("QMM-ABI-010:"));

    let two_descriptor = TypeDescriptor::of::<TwoFields>();
    let two_fields = v7::leak_slice(
        two_descriptor
            .fields()
            .iter()
            .map(|field| FieldMetadata::from_reflect(field.declaring_type().type_id(), field))
            .collect(),
    );
    let value_role = v7::leak(v7::value_role(Some(&two_fields[0]), None));
    let invalid_transparent = panic_message(|| {
        let _ =
            v7::GeneratedTypeMetadataBuilder::new(two_descriptor, None, two_fields, value_role).finish::<TwoFields>();
    });
    assert!(invalid_transparent.starts_with("QMM-ABI-011:"));

    let enum_descriptor = TypeDescriptor::of::<OneVariant>();
    let enum_role = v7::leak(v7::enum_role(&[]));
    let invalid_enum = panic_message(|| {
        let _ = v7::GeneratedTypeMetadataBuilder::new(enum_descriptor, None, &[], enum_role).finish::<OneVariant>();
    });
    assert!(invalid_enum.starts_with("QMM-ABI-012:"));
}

#[test]
fn finish_rejects_selector_on_incompatible_field_shape() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let selector = v7::leak(SelectorMetadata::new(SelectorPosition::Element, &[], &[], None, None));
    let constraints = v7::leak_slice(vec![ConstraintMetadata::Sequence(
        SequenceConstraint::new(None, None, false).with_element(selector),
    )]);
    let attributes = v7::leak_slice(vec![FieldAttributeMetadata::Constraint(&constraints[0])]);
    let fields = v7::leak_slice(vec![v7::field_metadata(
        descriptor.fields()[0].declaring_type().type_id(),
        &descriptor.fields()[0],
        attributes,
        constraints,
        &[],
        &SerdeFieldMetadata::DEFAULT,
    )]);
    let role = v7::leak(v7::model_role());

    let mismatch = panic_message(|| {
        let _ = v7::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role).finish::<OneField>();
    });
    assert!(mismatch.starts_with("QMM-ABI-020:"));
}

/// Checked construction rejects a field whose declared owner is another type.
#[test]
fn test_finish_rejects_wrong_field_owner_identity() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let fields = v7::leak_slice(vec![FieldMetadata::from_reflect(
        std::any::TypeId::of::<TwoFields>(),
        &descriptor.fields()[0],
    )]);
    let role = v7::leak(v7::model_role());
    let message = panic_message(|| {
        let _ = v7::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role).finish::<OneField>();
    });
    assert!(message.starts_with("QMM-ABI-003:"));
    assert!(message.contains("identity"));
}

/// Checked ABI validation must reject large duplicate lists without a numeric
/// overflow panic or a wrapped counter accepting the malformed declaration.
#[test]
fn test_singleton_semantic_counts_cannot_overflow() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let role = v7::leak(v7::model_role());
    for count in [2, 255, 256, 257] {
        let attributes = v7::leak_slice(vec![FieldAttributeMetadata::Indexed(IndexingReasons::EXPLICIT); count]);
        let fields = v7::leak_slice(vec![v7::field_metadata(
            descriptor.type_id(),
            &descriptor.fields()[0],
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]);
        let metadata = v7::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role).finish_unchecked();
        let error = metadata
            .validate_for::<OneField>()
            .expect_err("duplicate singleton semantics");
        assert_eq!(error.code(), "QMM-ABI-015", "duplicate count {count}");
    }
}

#[cfg(feature = "generic")]
#[Enum]
enum GenericVariant<T> {
    Value(T),
}

/// A source-level variant is not a concrete overlay, even when counts and
/// indices happen to match the requested concrete enum.
#[cfg(feature = "generic")]
#[test]
fn test_concrete_enum_rejects_definition_only_variants_without_panicking() {
    let definition = TypeMetadata::of::<GenericVariant<String>>()
        .generic_definition()
        .expect("generic definition retained");
    let role = v7::leak(v7::enum_role(definition.variants()));
    let metadata =
        v7::GeneratedTypeMetadataBuilder::new(TypeDescriptor::of::<OneVariant>(), None, &[], role).finish_unchecked();
    let error = metadata
        .validate_for::<OneVariant>()
        .expect_err("definition cannot stand in for concrete variants");
    assert_eq!(error.code(), "QMM-ABI-012");
}
