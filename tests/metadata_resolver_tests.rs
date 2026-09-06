// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow explicit-imports
//! Integration tests for explicit model graph resolution.

use qubit_codec::ValueCodecDescriptor;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueCodecRegistry;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_model_metadata::__private::v4;
use qubit_model_metadata::CodecMetadata;
use qubit_model_metadata::CodecReference;
use qubit_model_metadata::CodecSource;
use qubit_model_metadata::DeclaredEntityTarget;
use qubit_model_metadata::FieldAttributeMetadata;
use qubit_model_metadata::FieldDescriptor;
use qubit_model_metadata::FieldMetadata;
use qubit_model_metadata::FieldReferenceMetadata;
use qubit_model_metadata::FragmentIdentity;
use qubit_model_metadata::IdentifierAssignment;
use qubit_model_metadata::IdentifierMetadata;
use qubit_model_metadata::IndexingReasons;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ModelRegistry;
use qubit_model_metadata::ModelResolveErrorKind;
use qubit_model_metadata::ModelResolver;
use qubit_model_metadata::PropertyMetadata;
use qubit_model_metadata::PropertyPath;
use qubit_model_metadata::ReferenceSelection;
use qubit_model_metadata::Reflect;
use qubit_model_metadata::ResolveInputs;
use qubit_model_metadata::SerdeFieldMetadata;
use qubit_model_metadata::TypeDescriptor;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::ValidatorMetadata;

fn model_registry(entries: &[(&'static TypeMetadata, &'static FragmentIdentity)]) -> ModelRegistry<'static> {
    ModelRegistry::from_metadata(entries, &[]).expect("valid isolated model registry")
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct TargetFixture {
    id: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct SourceFixture {
    target_id: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct NestedQueryFixture {
    b: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct RootQueryFixture {
    id: u64,
    nested: NestedQueryFixture,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct ConflictingQueryFixture {
    id: u64,
    a_b: u64,
    a: NestedQueryFixture,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct PlainModelFixture {
    value: u64,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct InvalidValueFixture {
    model: PlainModelFixture,
}

#[derive(Reflect)]
#[reflect(crate = qubit_model_metadata)]
struct StrategyFixture {
    value: String,
    other: String,
}

#[derive(Default)]
struct StringCodec;

impl ValueEncoder<String> for StringCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&mut self, input: &String) -> Result<Self::Output, Self::Error> {
        Ok(input.clone())
    }
}

impl ValueDecoder<str> for StringCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
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

static STRING_CODEC_DESCRIPTOR: ValueCodecDescriptor = ValueCodecDescriptor::of::<StringCodec, String>();
static STRING_CODEC_REGISTRATION: ValueCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("test.strategy.codec"),
    &STRING_CODEC_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "tests", file!(), line!()),
);
static U64_CODEC_DESCRIPTOR: ValueCodecDescriptor = ValueCodecDescriptor::of::<U64Codec, u64>();
static U64_CODEC_REGISTRATION: ValueCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("test.strategy.codec"),
    &U64_CODEC_DESCRIPTOR,
    ValueCodecRegistrationSource::new("fixture", "tests", file!(), line!()),
);

fn source_identity(line: u32) -> &'static FragmentIdentity {
    Box::leak(Box::new(FragmentIdentity::new(
        "fixture",
        "tests",
        line,
        1,
        "model",
        line as u64,
    )))
}

#[test]
fn test_resolver_resolves_reference_targets_and_properties() {
    let target_descriptor = TypeDescriptor::of::<TargetFixture>();
    let identifier = Box::leak(Box::new(IdentifierMetadata::new(IdentifierAssignment::Application)));
    let target_attributes = Box::leak(
        vec![
            FieldAttributeMetadata::Identifier(identifier),
            FieldAttributeMetadata::Indexed(IndexingReasons::IDENTIFIER),
        ]
        .into_boxed_slice(),
    );
    let target_fields = Box::leak(
        vec![v4::field_metadata(
            target_descriptor.field_at(0).unwrap(),
            target_attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let target_role = v4::leak(v4::entity_role(&target_fields[0]));
    let target_properties = Box::leak(
        vec![v4::property_metadata(
            "id",
            target_fields[0].type_ref(),
            Some(&target_fields[0]),
            None,
            None,
        )]
        .into_boxed_slice(),
    );
    let target_metadata = Box::leak(Box::new(
        v4::GeneratedTypeMetadataBuilder::new(
            target_descriptor,
            Some(ModelId::new("example.Target")),
            target_fields,
            target_role,
        )
        .properties(target_properties)
        .finish::<TargetFixture>(),
    ));

    let declared_target = Box::leak(Box::new(DeclaredEntityTarget::ModelId(ModelId::new("example.Target"))));
    let selection = Box::leak(Box::new(ReferenceSelection::Property(PropertyPath::new(&["id"]))));
    let reference = Box::leak(Box::new(FieldReferenceMetadata::new(
        declared_target,
        selection,
        true,
        None,
    )));
    let attributes = Box::leak(vec![FieldAttributeMetadata::Reference(reference)].into_boxed_slice());
    let source_descriptor = TypeDescriptor::of::<SourceFixture>();
    let source_fields = Box::leak(
        vec![v4::field_metadata(
            source_descriptor.field_at(0).unwrap(),
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let source_role = v4::leak(v4::model_role());
    let source_metadata = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(
            source_descriptor,
            Some(ModelId::new("example.Source")),
            source_fields,
            source_role,
        )
        .finish::<SourceFixture>(),
    );

    let registry = model_registry(&[
        (target_metadata, source_identity(1)),
        (source_metadata, source_identity(2)),
    ]);
    let graph = ModelResolver::new(ResolveInputs {
        models: &registry,
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .unwrap();
    let resolved = graph.reference(&source_fields[0]).expect("resolved reference");

    assert!(std::ptr::eq(resolved.target(), target_metadata));
    assert_eq!(resolved.property().map(PropertyMetadata::name), Some("id"));
    let query = graph.query(target_metadata.as_entity().unwrap()).expect("entity query");
    assert!(query.filters().is_empty());
    assert_eq!(query.unique_keys().len(), 1);
}

#[test]
fn test_resolver_aggregates_missing_targets_deterministically() {
    let descriptor = TypeDescriptor::of::<SourceFixture>();
    let target = Box::leak(Box::new(DeclaredEntityTarget::ModelId(ModelId::new("missing.Target"))));
    let selection = Box::leak(Box::new(ReferenceSelection::Entity));
    let reference = Box::leak(Box::new(FieldReferenceMetadata::new(target, selection, true, None)));
    let attributes = Box::leak(vec![FieldAttributeMetadata::Reference(reference)].into_boxed_slice());
    let fields = Box::leak(
        vec![v4::field_metadata(
            descriptor.field_at(0).unwrap(),
            attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        )]
        .into_boxed_slice(),
    );
    let role = v4::leak(v4::model_role());
    let metadata = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(descriptor, Some(ModelId::new("example.SourceMissing")), fields, role)
            .finish::<SourceFixture>(),
    );
    let registry = model_registry(&[(metadata, source_identity(3))]);
    let errors = ModelResolver::new(ResolveInputs {
        models: &registry,
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .expect_err("missing targets must prevent graph publication");

    assert_eq!(errors.errors().len(), 1);
    assert_eq!(errors.errors()[0].kind(), ModelResolveErrorKind::MissingModelId);
    assert_eq!(errors.errors()[0].model_id(), Some("missing.Target"));
}

fn indexed_field(reflect: &'static FieldDescriptor) -> FieldMetadata {
    let attributes = Box::leak(vec![FieldAttributeMetadata::Indexed(IndexingReasons::EXPLICIT)].into_boxed_slice());
    v4::field_metadata(reflect, attributes, &[], &[], &SerdeFieldMetadata::DEFAULT)
}

fn entity_metadata<T: 'static>(
    descriptor: &'static TypeDescriptor,
    id: &'static str,
    fields: &'static [FieldMetadata],
) -> &'static TypeMetadata {
    let role = v4::leak(v4::entity_role(&fields[0]));
    v4::leak(v4::GeneratedTypeMetadataBuilder::new(descriptor, Some(ModelId::new(id)), fields, role).finish::<T>())
}

#[test]
fn test_query_recurses_indexed_value_fields_and_reports_flat_name_conflicts() {
    let nested_descriptor = TypeDescriptor::of::<NestedQueryFixture>();
    let nested_fields = Box::leak(vec![indexed_field(nested_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let nested_role = v4::leak(v4::value_role(None, None));
    let nested = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(
            nested_descriptor,
            Some(ModelId::new("query.Nested")),
            nested_fields,
            nested_role,
        )
        .finish::<NestedQueryFixture>(),
    );

    let root_descriptor = TypeDescriptor::of::<RootQueryFixture>();
    let identifier = Box::leak(Box::new(IdentifierMetadata::new(IdentifierAssignment::Application)));
    let identifier_attributes = Box::leak(
        vec![
            FieldAttributeMetadata::Identifier(identifier),
            FieldAttributeMetadata::Indexed(IndexingReasons::IDENTIFIER),
        ]
        .into_boxed_slice(),
    );
    let root_fields = Box::leak(
        vec![
            v4::field_metadata(
                root_descriptor.field_at(0).unwrap(),
                identifier_attributes,
                &[],
                &[],
                &SerdeFieldMetadata::DEFAULT,
            ),
            indexed_field(root_descriptor.field_at(1).unwrap()),
        ]
        .into_boxed_slice(),
    );
    let root = entity_metadata::<RootQueryFixture>(root_descriptor, "query.Root", root_fields);
    let registry = model_registry(&[(nested, source_identity(10)), (root, source_identity(11))]);
    let graph = ModelResolver::new(ResolveInputs {
        models: &registry,
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .unwrap();
    let query = graph.query(root.as_entity().unwrap()).unwrap();
    assert_eq!(
        query.filter_by_flat_name("nested_b").unwrap().path().segments(),
        &["nested", "b"]
    );

    let conflict_descriptor = TypeDescriptor::of::<ConflictingQueryFixture>();
    let conflict_fields = Box::leak(
        vec![
            v4::field_metadata(
                conflict_descriptor.field_at(0).unwrap(),
                identifier_attributes,
                &[],
                &[],
                &SerdeFieldMetadata::DEFAULT,
            ),
            indexed_field(conflict_descriptor.field_at(1).unwrap()),
            indexed_field(conflict_descriptor.field_at(2).unwrap()),
        ]
        .into_boxed_slice(),
    );
    let conflict = entity_metadata::<ConflictingQueryFixture>(conflict_descriptor, "query.Conflict", conflict_fields);
    let registry = model_registry(&[(nested, source_identity(10)), (conflict, source_identity(12))]);
    let errors = ModelResolver::new(ResolveInputs {
        models: &registry,
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .unwrap_err();
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::QueryNameConflict)
    );
}

#[test]
fn test_resolver_rejects_value_closure_over_model_role() {
    let model_descriptor = TypeDescriptor::of::<PlainModelFixture>();
    let model_fields =
        Box::leak(vec![FieldMetadata::from_reflect(model_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let model_role = v4::leak(v4::model_role());
    let model = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(
            model_descriptor,
            Some(ModelId::new("closure.Model")),
            model_fields,
            model_role,
        )
        .finish::<PlainModelFixture>(),
    );
    let value_descriptor = TypeDescriptor::of::<InvalidValueFixture>();
    let value_fields =
        Box::leak(vec![FieldMetadata::from_reflect(value_descriptor.field_at(0).unwrap())].into_boxed_slice());
    let value_role = v4::leak(v4::value_role(None, None));
    let value = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(
            value_descriptor,
            Some(ModelId::new("closure.Value")),
            value_fields,
            value_role,
        )
        .finish::<InvalidValueFixture>(),
    );
    let registry = model_registry(&[(model, source_identity(20)), (value, source_identity(21))]);
    let errors = ModelResolver::new(ResolveInputs {
        models: &registry,
        codecs: ValueCodecRegistry::global(),
    })
    .resolve_all()
    .unwrap_err();
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::InvalidValueClosure)
    );
}

fn strategy_metadata() -> (&'static TypeMetadata, &'static CodecMetadata, &'static CodecMetadata) {
    let descriptor = TypeDescriptor::of::<StrategyFixture>();
    let dependency = PropertyPath::new(&["other"]);
    let validators = v4::leak_slice(vec![ValidatorMetadata::new(
        "test.strategy.validator",
        &[],
        v4::leak_slice(vec![dependency]),
    )]);
    let validator = &validators[0];
    let direct_reference = v4::leak(CodecReference::RustType(&STRING_CODEC_DESCRIPTOR));
    let direct_codec = v4::leak(CodecMetadata::new(direct_reference, CodecSource::Field));
    let id_reference = v4::leak(CodecReference::DeclaredId("test.strategy.codec"));
    let id_codec = v4::leak(CodecMetadata::new(id_reference, CodecSource::Field));
    let validator_attributes = v4::leak_slice(vec![
        FieldAttributeMetadata::Validator(validator),
        FieldAttributeMetadata::Codec(direct_codec),
    ]);
    let codec_attributes = v4::leak_slice(vec![FieldAttributeMetadata::Codec(id_codec)]);
    let fields = v4::leak_slice(vec![
        v4::field_metadata(
            &descriptor.fields()[0],
            validator_attributes,
            &[],
            validators,
            &SerdeFieldMetadata::DEFAULT,
        ),
        v4::field_metadata(
            &descriptor.fields()[1],
            codec_attributes,
            &[],
            &[],
            &SerdeFieldMetadata::DEFAULT,
        ),
    ]);
    let properties = v4::leak_slice(
        fields
            .iter()
            .map(|field| {
                v4::property_metadata(
                    field.name().expect("fixture field name"),
                    field.type_ref(),
                    Some(field),
                    None,
                    None,
                )
            })
            .collect(),
    );
    let role = v4::leak(v4::model_role());
    let metadata = v4::leak(
        v4::GeneratedTypeMetadataBuilder::new(descriptor, Some(ModelId::new("strategy.Fixture")), fields, role)
            .properties(properties)
            .finish::<StrategyFixture>(),
    );
    (metadata, direct_codec, id_codec)
}

#[test]
fn test_resolver_binds_executable_codec_descriptors() {
    let (metadata, direct_codec, id_codec) = strategy_metadata();
    let models = model_registry(&[(metadata, source_identity(30))]);
    let codecs = ValueCodecRegistry::from_registrations([&STRING_CODEC_REGISTRATION]).unwrap();
    let graph = ModelResolver::new(ResolveInputs {
        models: &models,
        codecs: &codecs,
    })
    .resolve_all()
    .expect("all executable codecs must resolve");

    assert!(graph.codec(direct_codec).unwrap().registration().is_none());
    assert_eq!(
        graph.codec(id_codec).unwrap().registration().unwrap().id().as_str(),
        "test.strategy.codec",
    );
}

#[test]
fn test_resolver_aggregates_missing_codec_ids() {
    let (metadata, _, _) = strategy_metadata();
    let models = model_registry(&[(metadata, source_identity(31))]);
    let codecs = ValueCodecRegistry::empty();
    let errors = ModelResolver::new(ResolveInputs {
        models: &models,
        codecs: &codecs,
    })
    .resolve_all()
    .expect_err("missing executable codec IDs must reject the graph");

    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::MissingCodec)
    );
}

#[test]
fn test_resolver_aggregates_codec_type_mismatches() {
    let (metadata, _, _) = strategy_metadata();
    let models = model_registry(&[(metadata, source_identity(32))]);
    let codecs = ValueCodecRegistry::from_registrations([&U64_CODEC_REGISTRATION]).unwrap();
    let errors = ModelResolver::new(ResolveInputs {
        models: &models,
        codecs: &codecs,
    })
    .resolve_all()
    .expect_err("codec value-type mismatches must reject the graph");
    assert!(
        errors
            .errors()
            .iter()
            .any(|error| error.kind() == ModelResolveErrorKind::CodecTypeMismatch)
    );
}
