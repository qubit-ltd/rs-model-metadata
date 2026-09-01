// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for the checked generated-code ABI.

use qubit_codec::ValueCodecDescriptor;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_model_metadata::__private::v2;
use qubit_model_metadata::CodecMetadata;
use qubit_model_metadata::CodecReference;
use qubit_model_metadata::CodecSource;
use qubit_model_metadata::ConstraintMetadata;
use qubit_model_metadata::FieldAttributeMetadata;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::GetterMetadata;
use qubit_model_metadata::GetterOutputKind;
use qubit_model_metadata::PropertyAccessError;
use qubit_model_metadata::PropertyValue;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::ReflectedRef;
use qubit_model_metadata::SelectorMetadata;
use qubit_model_metadata::SelectorPosition;
use qubit_model_metadata::SequenceConstraint;
use qubit_model_metadata::SerdeFieldMetadata;
use qubit_model_metadata::TypeDescriptor;

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

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
enum OneVariant {
    Ready,
}

#[derive(Default)]
struct U64Codec;

impl ValueEncoder<u64> for U64Codec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&mut self, input: &u64) -> Result<Self::Output, Self::Error> {
        Ok(input.to_string())
    }
}

impl ValueDecoder<str> for U64Codec {
    type Output = u64;
    type Error = std::num::ParseIntError;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input.parse()
    }
}

static U64_CODEC_DESCRIPTOR: ValueCodecDescriptor = ValueCodecDescriptor::of::<U64Codec, u64>();

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
    let role = v2::leak(v2::model_role());
    let descriptor = TypeDescriptor::of::<OneField>();

    let mismatch = panic_message(|| {
        let _ = v2::GeneratedTypeMetadataBuilder::new(descriptor, None, &[], role).finish::<WrongTarget>();
    });
    assert!(mismatch.starts_with("QMM-ABI-001:"));

    let missing_field = panic_message(|| {
        let _ = v2::GeneratedTypeMetadataBuilder::new(descriptor, None, &[], role).finish::<OneField>();
    });
    assert!(missing_field.starts_with("QMM-ABI-003:"));
}

#[test]
fn finish_rejects_duplicate_properties_and_wrong_getter_targets() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let fields = v2::leak_slice(vec![FieldMetadata::from_reflect(&descriptor.fields()[0])]);
    let role = v2::leak(v2::model_role());
    let property = v2::property_metadata("value", fields[0].type_ref(), Some(&fields[0]), None, None);
    let properties = v2::leak_slice(vec![property, property]);
    let duplicate = panic_message(|| {
        let _ = v2::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role)
            .properties(properties)
            .finish::<OneField>();
    });
    assert!(duplicate.starts_with("QMM-ABI-004:"));

    let getter = v2::leak(GetterMetadata::new::<WrongTarget>(
        "value",
        fields[0].type_ref(),
        GetterOutputKind::Borrowed,
        unavailable_getter,
    ));
    let properties = v2::leak_slice(vec![v2::property_metadata(
        "value",
        fields[0].type_ref(),
        Some(&fields[0]),
        Some(getter),
        None,
    )]);
    let wrong_target = panic_message(|| {
        let _ = v2::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role)
            .properties(properties)
            .finish::<OneField>();
    });
    assert!(wrong_target.starts_with("QMM-ABI-008:"));
}

#[test]
fn finish_rejects_invalid_role_payloads() {
    let one_descriptor = TypeDescriptor::of::<OneField>();
    let one_fields = v2::leak_slice(vec![FieldMetadata::from_reflect(&one_descriptor.fields()[0])]);
    let entity_role = v2::leak(v2::entity_role(&one_fields[0]));
    let invalid_identifier = panic_message(|| {
        let _ =
            v2::GeneratedTypeMetadataBuilder::new(one_descriptor, None, one_fields, entity_role).finish::<OneField>();
    });
    assert!(invalid_identifier.starts_with("QMM-ABI-010:"));

    let two_descriptor = TypeDescriptor::of::<TwoFields>();
    let two_fields = v2::leak_slice(
        two_descriptor
            .fields()
            .iter()
            .map(FieldMetadata::from_reflect)
            .collect(),
    );
    let value_role = v2::leak(v2::value_role(Some(&two_fields[0]), None));
    let invalid_transparent = panic_message(|| {
        let _ =
            v2::GeneratedTypeMetadataBuilder::new(two_descriptor, None, two_fields, value_role).finish::<TwoFields>();
    });
    assert!(invalid_transparent.starts_with("QMM-ABI-011:"));

    let enum_descriptor = TypeDescriptor::of::<OneVariant>();
    let enum_role = v2::leak(v2::enum_role(&[]));
    let invalid_enum = panic_message(|| {
        let _ = v2::GeneratedTypeMetadataBuilder::new(enum_descriptor, None, &[], enum_role).finish::<OneVariant>();
    });
    assert!(invalid_enum.starts_with("QMM-ABI-012:"));
}

#[test]
fn finish_rejects_selector_on_incompatible_field_shape() {
    let descriptor = TypeDescriptor::of::<OneField>();
    let selector = v2::leak(SelectorMetadata::new(SelectorPosition::Element, &[], &[], None, None));
    let constraints = v2::leak_slice(vec![ConstraintMetadata::Sequence(
        SequenceConstraint::new(None, None, false).with_element(selector),
    )]);
    let attributes = v2::leak_slice(vec![FieldAttributeMetadata::Constraint(&constraints[0])]);
    let fields = v2::leak_slice(vec![v2::field_metadata(
        &descriptor.fields()[0],
        attributes,
        constraints,
        &[],
        &SerdeFieldMetadata::DEFAULT,
    )]);
    let role = v2::leak(v2::model_role());

    let mismatch = panic_message(|| {
        let _ = v2::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role).finish::<OneField>();
    });
    assert!(mismatch.starts_with("QMM-ABI-020:"));
}

#[test]
fn finish_rejects_field_codec_for_different_value_type() {
    let descriptor = TypeDescriptor::of::<OneField>();
    assert_eq!(
        descriptor.fields()[0].field_type().as_resolved().unwrap().type_id(),
        std::any::TypeId::of::<String>()
    );
    assert_eq!(U64_CODEC_DESCRIPTOR.value_type_id(), std::any::TypeId::of::<u64>());
    assert_ne!(U64_CODEC_DESCRIPTOR.value_type_id(), std::any::TypeId::of::<String>());
    let reference = v2::leak(CodecReference::RustType(&U64_CODEC_DESCRIPTOR));
    let codec = v2::leak(CodecMetadata::new(reference, CodecSource::Field));
    let attributes = v2::leak_slice(vec![FieldAttributeMetadata::Codec(codec)]);
    let fields = v2::leak_slice(vec![v2::field_metadata(
        &descriptor.fields()[0],
        attributes,
        &[],
        &[],
        &SerdeFieldMetadata::DEFAULT,
    )]);
    assert!(fields[0].codec().is_some());
    let role = v2::leak(v2::model_role());

    let mismatch = panic_message(|| {
        let _ = v2::GeneratedTypeMetadataBuilder::new(descriptor, None, fields, role).finish::<OneField>();
    });
    assert!(mismatch.starts_with("QMM-ABI-025:"));
}
