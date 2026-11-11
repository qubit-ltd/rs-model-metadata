// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for explicit codec binding after structure resolution.

#![cfg(feature = "codec")]

use core::convert::Infallible;
use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::error::Error;
use std::num::ParseIntError;

use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;
use qubit_codec::ValueStringCodecRegistry;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_model_metadata::codec::CodecBindErrorKind;
use qubit_model_metadata::codec::CodecBindInputs;
use qubit_model_metadata::codec::CodecOccurrenceId;
use qubit_model_metadata::codec::bind_codecs;
use qubit_model_metadata::metadata::CodecSource;
use qubit_model_metadata::metadata::SelectorPosition;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_model_metadata::resolve::ModelGraph;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::ReflectionContext;

#[derive(Default)]
struct StringCodec;

impl ValueEncoder<String> for StringCodec {
    type Output = String;
    type Error = Infallible;

    fn encode(&mut self, input: &String) -> Result<Self::Output, Self::Error> {
        Ok(input.clone())
    }
}

impl ValueDecoder<str> for StringCodec {
    type Output = String;
    type Error = Infallible;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

#[derive(Default)]
struct U64Codec;

impl ValueEncoder<u64> for U64Codec {
    type Output = String;
    type Error = Infallible;

    fn encode(&mut self, input: &u64) -> Result<Self::Output, Self::Error> {
        Ok(input.to_string())
    }
}

impl ValueDecoder<str> for U64Codec {
    type Output = u64;
    type Error = ParseIntError;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input.parse()
    }
}

static STRING_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<StringCodec, String>();
static STRING_REGISTRATION: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("test.string"),
    &STRING_DESCRIPTOR,
    ValueCodecRegistrationSource::new("codec-tests", "fixture", file!(), line!()),
);
static STRING_ALIAS_REGISTRATION: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("test.string.alias"),
    &STRING_DESCRIPTOR,
    ValueCodecRegistrationSource::new("codec-tests", "fixture", file!(), line!()),
);
static U64_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<U64Codec, u64>();
static U64_REGISTRATION: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("test.wrong"),
    &U64_DESCRIPTOR,
    ValueCodecRegistrationSource::new("codec-tests", "fixture", file!(), line!()),
);

#[Model(id = "codec.Success")]
struct Success {
    #[codec(id = "test.string")]
    declared: String,
}

#[Model(id = "codec.RustType")]
struct RustType {
    #[codec(StringCodec)]
    value: String,
}

#[Model(id = "codec.Missing")]
struct Missing {
    #[codec(id = "test.missing")]
    value: String,
}

#[Model(id = "codec.Mismatch")]
struct Mismatch {
    #[codec(id = "test.wrong")]
    value: String,
}

#[Model]
struct AnonymousCodec {
    #[codec(id = "test.string")]
    value: String,
}

/// Codec occurrence identity remains exact for anonymous graph roots.
#[test]
fn test_binds_anonymous_root_codecs() {
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[]).expect("empty registry");
    let metadata = TypeMetadata::of::<AnonymousCodec>();
    let roots = [metadata];
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("anonymous graph");
    let codecs = ValueStringCodecRegistry::from_registrations([&STRING_REGISTRATION]).expect("codec registry");
    let bindings = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect("anonymous codec");
    assert_eq!(bindings.bindings().len(), 1);
}

/// Builds the shared provenance fixture used when registering test metadata.
///
/// The location fields point to this helper's call site, giving registry tests
/// deterministic crate and module context without duplicating origin setup.
fn source() -> ModelRegistryOrigin {
    ModelRegistryOrigin {
        crate_name: "codec-tests",
        crate_version: None,
        module_path: module_path!(),
        file: Some(file!()),
        line: Some(line!()),
        column: Some(1),
    }
}

/// Resolves one metadata root into a graph for codec-binding assertions.
///
/// The registry borrows the supplied provenance, so it is boxed and leaked to
/// keep the registry alive for the returned graph's lifetime. This fixture is
/// only used by tests and avoids changing the production graph ownership API.
fn graph<'a>(metadata: &'static TypeMetadata, source: &'a ModelRegistryOrigin) -> ModelGraph<'a> {
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[(metadata, source)])
        .expect("model registry");
    let models = Box::leak(Box::new(models));
    StructureResolver::new(ResolveInputs { roots: &[], models })
        .resolve()
        .expect("model graph")
}

#[test]
fn test_binds_declared_and_rust_type_references() {
    let source = source();
    let declared_graph = graph(TypeMetadata::of::<Success>(), &source);
    let rust_graph = graph(TypeMetadata::of::<RustType>(), &source);
    let codecs = ValueStringCodecRegistry::from_registrations([&STRING_REGISTRATION]).expect("codec registry");

    assert_eq!(
        bind_codecs(CodecBindInputs {
            graph: &declared_graph,
            codecs: &codecs
        })
        .unwrap()
        .bindings()
        .len(),
        1
    );
    assert_eq!(
        bind_codecs(CodecBindInputs {
            graph: &rust_graph,
            codecs: &codecs
        })
        .unwrap()
        .bindings()
        .len(),
        1
    );
}

#[test]
fn test_reports_sorted_missing_ambiguous_and_type_mismatch_errors() {
    let source = source();
    let missing_graph = graph(TypeMetadata::of::<Missing>(), &source);
    let mismatch_graph = graph(TypeMetadata::of::<Mismatch>(), &source);
    let rust_graph = graph(TypeMetadata::of::<RustType>(), &source);
    let codecs = ValueStringCodecRegistry::from_registrations([
        &STRING_REGISTRATION,
        &STRING_ALIAS_REGISTRATION,
        &U64_REGISTRATION,
    ])
    .expect("codec registry");

    let missing = bind_codecs(CodecBindInputs {
        graph: &missing_graph,
        codecs: &codecs,
    })
    .unwrap_err();
    assert_eq!(missing.errors()[0].kind(), CodecBindErrorKind::Missing);
    assert!(missing.errors()[0].candidate_sources().is_empty());
    let mismatch = bind_codecs(CodecBindInputs {
        graph: &mismatch_graph,
        codecs: &codecs,
    })
    .unwrap_err();
    assert_eq!(mismatch.errors()[0].kind(), CodecBindErrorKind::ValueTypeMismatch);
    assert_eq!(mismatch.errors()[0].candidate_sources(), &[U64_REGISTRATION.source()]);
    assert_eq!(mismatch.errors()[0].actual_type(), Some(TypeId::of::<u64>()));
    assert_eq!(mismatch.errors()[0].declaration().declared_id(), Some("test.wrong"));
    assert!(mismatch.errors()[0].to_string().contains("codec binding failed"));
    assert_eq!(mismatch.to_string(), "1 codec binding error(s)");
    let ambiguous = bind_codecs(CodecBindInputs {
        graph: &rust_graph,
        codecs: &codecs,
    })
    .unwrap_err();
    assert_eq!(ambiguous.errors()[0].kind(), CodecBindErrorKind::Ambiguous);
    assert_eq!(ambiguous.errors()[0].candidate_sources().len(), 2);
    assert_eq!(ambiguous.into_vec().len(), 1);
}

#[derive(Default)]
struct NameCodec;

#[Value(id = "codec.Name", transparent, codec = NameCodec)]
struct Name(String);

impl ValueEncoder<Name> for NameCodec {
    type Output = String;
    type Error = Infallible;
    fn encode(&mut self, input: &Name) -> Result<String, Self::Error> {
        Ok(input.0.clone())
    }
}

impl ValueDecoder<str> for NameCodec {
    type Output = Name;
    type Error = Infallible;
    fn decode(&mut self, input: &str) -> Result<Name, Self::Error> {
        Ok(Name(input.to_owned()))
    }
}

#[derive(Default)]
struct BoxNameCodec;

impl ValueEncoder<Box<Name>> for BoxNameCodec {
    type Output = String;
    type Error = Infallible;

    fn encode(&mut self, input: &Box<Name>) -> Result<String, Self::Error> {
        Ok(input.0.clone())
    }
}

impl ValueDecoder<str> for BoxNameCodec {
    type Output = Box<Name>;
    type Error = Infallible;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(Box::new(Name(input.to_owned())))
    }
}

static NAME_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<NameCodec, Name>();
static NAME_REGISTRATION: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("test.name.direct"),
    &NAME_DESCRIPTOR,
    ValueCodecRegistrationSource::new("codec-tests", "direct-field", file!(), line!()),
);

macro_rules! empty_noise_models {
    ($($name:ident),+ $(,)?) => {
        $(
            #[Model]
            struct $name {}
        )+
    };
}

empty_noise_models!(
    EmptyNoise01,
    EmptyNoise02,
    EmptyNoise03,
    EmptyNoise04,
    EmptyNoise05,
    EmptyNoise06,
    EmptyNoise07,
    EmptyNoise08,
    EmptyNoise09,
    EmptyNoise10,
    EmptyNoise11,
    EmptyNoise12,
    EmptyNoise13,
    EmptyNoise14,
    EmptyNoise15,
    EmptyNoise16,
);
static BOX_NAME_DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<BoxNameCodec, Box<Name>>();
static BOX_NAME_REGISTRATION: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("test.box_name.direct"),
    &BOX_NAME_DESCRIPTOR,
    ValueCodecRegistrationSource::new("codec-tests", "direct-box-field", file!(), line!()),
);

#[Model(id = "codec.DirectValue")]
struct DirectValue {
    value: Name,
}

#[Model(id = "codec.DirectOptional")]
struct DirectOptional {
    value: Option<Name>,
}

#[Model(id = "codec.DirectBox")]
struct DirectBox {
    value: Box<Name>,
}

#[Model(id = "codec.DirectOptionalBox")]
struct DirectOptionalBox {
    value: Option<Box<Name>>,
}

#[Model(id = "codec.DirectNestedOptional")]
struct DirectNestedOptional {
    value: Option<Option<Name>>,
}

#[Model(id = "codec.ExplicitValue")]
struct ExplicitValue {
    #[codec(NameCodec)]
    value: Name,
}

#[Model(id = "codec.ExplicitOptional")]
struct ExplicitOptional {
    #[codec(NameCodec)]
    value: Option<Name>,
}

#[Model(id = "codec.ExplicitBox")]
struct ExplicitBox {
    #[codec(NameCodec)]
    value: Box<Name>,
}

#[Model(id = "codec.ExplicitOptionalBox")]
struct ExplicitOptionalBox {
    #[codec(NameCodec)]
    value: Option<Box<Name>>,
}

#[Model(id = "codec.ExplicitNestedOptional")]
struct ExplicitNestedOptional {
    #[codec(NameCodec)]
    value: Option<Option<Name>>,
}

#[Model(id = "codec.ExactBox")]
struct ExactBox {
    #[codec(BoxNameCodec)]
    value: Box<Name>,
}

#[Model(id = "codec.ExactOptionalBox")]
struct ExactOptionalBox {
    #[codec(BoxNameCodec)]
    value: Option<Box<Name>>,
}

fn graph_with_name<'a>(owner: &'static TypeMetadata, source: &'a ModelRegistryOrigin) -> ModelGraph<'a> {
    graph_with_name_and_roots(owner, source, &[])
}

fn graph_with_name_and_roots<'a>(
    owner: &'static TypeMetadata,
    source: &'a ModelRegistryOrigin,
    roots: &'a [&'static TypeMetadata],
) -> ModelGraph<'a> {
    let models = ModelRegistry::from_static_metadata(
        ReflectionContext::global(),
        &[(owner, source), (TypeMetadata::of::<Name>(), source)],
    )
    .expect("owner and canonical value models");
    let models = Box::leak(Box::new(models));
    StructureResolver::new(ResolveInputs { roots, models })
        .resolve()
        .expect("direct field graph")
}

fn assert_canonical_field(owner: &'static TypeMetadata, property: &str, expected: bool) {
    let source = source();
    let graph = graph_with_name(owner, &source);
    assert!(
        graph
            .models()
            .iter()
            .any(|metadata| metadata.type_id() == TypeId::of::<Name>())
    );
    let codecs = ValueStringCodecRegistry::from_registrations([&NAME_REGISTRATION]).expect("codec registry");
    let bindings = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect("canonical codec binding");
    assert!(
        bindings
            .get(&CodecOccurrenceId::new(
                TypeMetadata::of::<Name>(),
                "",
                CodecSource::CanonicalValue,
            ))
            .is_some(),
        "Name retains its own canonical occurrence"
    );
    let field = bindings.get(&CodecOccurrenceId::new(owner, property, CodecSource::CanonicalValue));
    assert_eq!(
        field.is_some(),
        expected,
        "owner field occurrence for {}",
        owner.type_name()
    );
    if let Some(binding) = field {
        assert_eq!(binding.occurrence().type_id(), owner.type_id());
        assert_eq!(binding.occurrence().property(), property);
        assert_eq!(binding.declaration().source(), CodecSource::CanonicalValue);
        assert_eq!(binding.descriptor().value_type_id(), TypeId::of::<Name>());
    }
}

/// Direct field canonical fallback unwraps only one outer `Option` and uses
/// exact runtime type identity for smart pointers and nested options.
#[test]
fn test_direct_field_canonical_wrapper_matrix() {
    assert_canonical_field(TypeMetadata::of::<DirectValue>(), "value", true);
    assert_canonical_field(TypeMetadata::of::<DirectOptional>(), "value", true);
    assert_canonical_field(TypeMetadata::of::<DirectBox>(), "value", false);
    assert_canonical_field(TypeMetadata::of::<DirectOptionalBox>(), "value", false);
    assert_canonical_field(TypeMetadata::of::<DirectNestedOptional>(), "value", false);
}

#[test]
fn test_canonical_binding_is_stable_when_graph_gains_unrelated_models() {
    let source = source();
    let owner = TypeMetadata::of::<DirectValue>();
    let original = graph_with_name(owner, &source);
    let noise = [
        TypeMetadata::of::<EmptyNoise01>(),
        TypeMetadata::of::<EmptyNoise02>(),
        TypeMetadata::of::<EmptyNoise03>(),
        TypeMetadata::of::<EmptyNoise04>(),
        TypeMetadata::of::<EmptyNoise05>(),
        TypeMetadata::of::<EmptyNoise06>(),
        TypeMetadata::of::<EmptyNoise07>(),
        TypeMetadata::of::<EmptyNoise08>(),
        TypeMetadata::of::<EmptyNoise09>(),
        TypeMetadata::of::<EmptyNoise10>(),
        TypeMetadata::of::<EmptyNoise11>(),
        TypeMetadata::of::<EmptyNoise12>(),
        TypeMetadata::of::<EmptyNoise13>(),
        TypeMetadata::of::<EmptyNoise14>(),
        TypeMetadata::of::<EmptyNoise15>(),
        TypeMetadata::of::<EmptyNoise16>(),
    ];
    let expanded = graph_with_name_and_roots(owner, &source, &noise);
    assert_eq!(expanded.models().len(), original.models().len() + noise.len());

    let codecs = ValueStringCodecRegistry::from_registrations([&NAME_REGISTRATION]).expect("codec registry");
    let original_bindings = bind_codecs(CodecBindInputs {
        graph: &original,
        codecs: &codecs,
    })
    .expect("original graph bindings");
    let expanded_bindings = bind_codecs(CodecBindInputs {
        graph: &expanded,
        codecs: &codecs,
    })
    .expect("expanded graph bindings");
    let occurrence = CodecOccurrenceId::new(owner, "value", CodecSource::CanonicalValue);
    let original_value = original_bindings.get(&occurrence).expect("original canonical field");
    let expanded_value = expanded_bindings.get(&occurrence).expect("expanded canonical field");
    assert_eq!(expanded_bindings.bindings().len(), original_bindings.bindings().len());
    assert_eq!(expanded_value.occurrence(), original_value.occurrence());
    assert_eq!(
        expanded_value.descriptor().value_type_id(),
        original_value.descriptor().value_type_id()
    );
}

fn assert_explicit_mismatch(owner: &'static TypeMetadata, expected: TypeId) {
    let source = source();
    let graph = graph_with_name(owner, &source);
    assert!(
        graph
            .models()
            .iter()
            .any(|metadata| metadata.type_id() == TypeId::of::<Name>())
    );
    let codecs = ValueStringCodecRegistry::from_registrations([&NAME_REGISTRATION]).expect("codec registry");
    let errors = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect_err("exact type mismatch");
    assert_eq!(errors.errors().len(), 1);
    let error = &errors.errors()[0];
    assert_eq!(error.kind(), CodecBindErrorKind::ValueTypeMismatch);
    assert_eq!(error.occurrence().type_id(), owner.type_id());
    assert_eq!(error.occurrence().property(), "value");
    assert_eq!(error.occurrence().source(), CodecSource::Field);
    assert_eq!(error.expected_type(), expected);
    assert_eq!(error.actual_type(), Some(TypeId::of::<Name>()));
}

/// Explicit Name codecs accept `Name` and `Option<Name>`; other wrappers use
/// their one-layer effective type and report the exact expected and actual
/// types.
#[test]
fn test_direct_field_explicit_wrapper_matrix() {
    for owner in [
        TypeMetadata::of::<ExplicitValue>(),
        TypeMetadata::of::<ExplicitOptional>(),
    ] {
        let source = source();
        let graph = graph_with_name(owner, &source);
        let codecs = ValueStringCodecRegistry::from_registrations([&NAME_REGISTRATION]).expect("codec registry");
        let bindings = bind_codecs(CodecBindInputs {
            graph: &graph,
            codecs: &codecs,
        })
        .expect("Name codec binding");
        let binding = bindings
            .get(&CodecOccurrenceId::new(owner, "value", CodecSource::Field))
            .unwrap();
        assert_eq!(binding.descriptor().value_type_id(), TypeId::of::<Name>());
    }
    assert_explicit_mismatch(TypeMetadata::of::<ExplicitBox>(), TypeId::of::<Box<Name>>());
    assert_explicit_mismatch(TypeMetadata::of::<ExplicitOptionalBox>(), TypeId::of::<Box<Name>>());
    assert_explicit_mismatch(
        TypeMetadata::of::<ExplicitNestedOptional>(),
        TypeId::of::<Option<Name>>(),
    );
}

/// A codec registered for `Box<Name>` binds exactly and executes a round trip
/// for both direct and optional smart-pointer fields.
#[test]
fn test_direct_box_field_codec_matches_and_executes() {
    for owner in [TypeMetadata::of::<ExactBox>(), TypeMetadata::of::<ExactOptionalBox>()] {
        let source = source();
        let graph = graph_with_name(owner, &source);
        let codecs = ValueStringCodecRegistry::from_registrations([&NAME_REGISTRATION, &BOX_NAME_REGISTRATION])
            .expect("codec registry");
        let bindings = bind_codecs(CodecBindInputs {
            graph: &graph,
            codecs: &codecs,
        })
        .expect("Box<Name> codec binding");
        let binding = bindings
            .get(&CodecOccurrenceId::new(owner, "value", CodecSource::Field))
            .unwrap();
        let descriptor = binding.descriptor();
        assert_eq!(descriptor.value_type_id(), TypeId::of::<Box<Name>>());
        let input = Box::new(Name("wrapped".to_owned()));
        let encoded = descriptor.encode(&input).expect("encode Box<Name>");
        let decoded = descriptor.decode(&encoded).expect("decode Box<Name>");
        assert_eq!(decoded.downcast::<Box<Name>>().unwrap().as_ref().as_ref().0, "wrapped");
    }
}

#[Model(id = "codec.CanonicalOwner")]
struct CanonicalOwner {
    inherited: Name,
    #[codec(NameCodec)]
    explicit_same: Name,
}

/// Explicit equality with canonical is legal and unannotated uses inherit it.
#[test]
fn test_canonical_fallback_preserves_occurrence_sources() {
    static DESCRIPTOR: ValueStringCodecDescriptor = ValueStringCodecDescriptor::of::<NameCodec, Name>();
    static REGISTRATION: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
        ValueCodecId::new("test.name"),
        &DESCRIPTOR,
        ValueCodecRegistrationSource::new("codec-tests", "canonical", file!(), line!()),
    );
    let owner = TypeMetadata::of::<CanonicalOwner>();
    let name = TypeMetadata::of::<Name>();
    let source = source();
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[(owner, &source), (name, &source)])
        .expect("models");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &[],
    })
    .resolve()
    .expect("structure");
    let codecs = ValueStringCodecRegistry::from_registrations([&REGISTRATION]).expect("codecs");
    let bound = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect("canonical binding");
    let fields: Vec<_> = bound
        .bindings()
        .filter(|binding| binding.occurrence().type_id() == owner.type_id())
        .collect();
    assert_eq!(fields.len(), 2);
    assert_eq!(
        fields
            .iter()
            .find(|binding| binding.occurrence().property() == "inherited")
            .unwrap()
            .declaration()
            .source(),
        CodecSource::CanonicalValue
    );
    assert_eq!(
        fields
            .iter()
            .find(|binding| binding.occurrence().property() == "explicit_same")
            .unwrap()
            .declaration()
            .source(),
        CodecSource::Field
    );
}

#[Enum]
enum TupleCodecs {
    Pair(#[codec(StringCodec)] String, #[codec(StringCodec)] String),
}

/// Tuple payload occurrences cannot overwrite each other under an unnamed key.
#[test]
fn test_tuple_payload_codecs_have_distinct_occurrences() {
    let roots = [TypeMetadata::of::<TupleCodecs>()];
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[]).expect("empty registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("tuple graph");
    let codecs = ValueStringCodecRegistry::from_registrations([&STRING_REGISTRATION]).expect("codecs");
    let bound = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect("payload codecs");
    assert_eq!(bound.bindings().len(), 2);
    let paths: Vec<_> = bound
        .bindings()
        .map(|binding| binding.occurrence().property())
        .collect();
    assert_eq!(paths, ["PAIR.0", "PAIR.1"]);
}

#[Model]
struct CodecContainers {
    #[element(codec(StringCodec))]
    sequence: Vec<String>,
    #[element(codec(StringCodec))]
    optional: Option<Vec<String>>,
    #[element(codec(StringCodec))]
    // Exercise a smart-pointer layer outside the collection itself.
    #[allow(clippy::box_collection)]
    pointer: Box<Vec<String>>,
    #[element(codec(StringCodec))]
    #[allow(clippy::box_collection)]
    chain: Option<Box<Vec<String>>>,
    #[element(codec(StringCodec))]
    set: BTreeSet<String>,
    #[element(codec(StringCodec))]
    array: [String; 2],
    #[map_key(codec(StringCodec))]
    #[map_value(codec(StringCodec))]
    map: BTreeMap<String, String>,
}

/// Bind selector positions to executable codecs rather than just counting them.
#[test]
fn test_container_selector_bindings_preserve_identity_and_execute() {
    let root = TypeMetadata::of::<CodecContainers>();
    let roots = [root];
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("container graph");
    let codecs = ValueStringCodecRegistry::from_registrations([&STRING_REGISTRATION]).expect("codec registry");
    let bindings = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect("selector codecs");
    assert_eq!(bindings.bindings().len(), 8);
    for (field, selector) in [
        ("sequence", SelectorPosition::Element),
        ("optional", SelectorPosition::Element),
        ("pointer", SelectorPosition::Element),
        ("chain", SelectorPosition::Element),
        ("set", SelectorPosition::Element),
        ("array", SelectorPosition::Element),
        ("map", SelectorPosition::MapKey),
        ("map", SelectorPosition::MapValue),
    ] {
        let id = CodecOccurrenceId::new(root, field, CodecSource::Selector(selector));
        let binding = bindings.get(&id).expect("each distinct selector is indexed");
        assert_eq!(binding.occurrence(), &id);
        assert_eq!(id.model(), None, "anonymous identity does not fabricate a model ID");
        assert_eq!(id.source(), CodecSource::Selector(selector));
        assert_eq!(binding.registration().source(), STRING_REGISTRATION.source());
        let descriptor = binding.descriptor();
        let input = format!("{field} value");
        let encoded = descriptor.encode(&input).expect("bound String encoder");
        let decoded = descriptor.decode(&encoded).expect("bound String decoder");
        assert_eq!(*decoded.downcast::<String>().expect("exact bound value type"), input);
        assert!(id.to_string().ends_with(&format!("::{field}")));
    }
    assert!(
        bindings
            .get(&CodecOccurrenceId::new(root, "absent", CodecSource::Field))
            .is_none()
    );
}

#[Model]
struct OpaqueCodecContainer {
    #[opaque]
    #[codec(id = "test.string")]
    hidden: Option<Vec<String>>,
}

#[test]
fn test_opaque_collection_field_is_not_resolved_for_codec_binding() {
    let root = TypeMetadata::of::<OpaqueCodecContainer>();
    let roots = [root];
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("opaque graph");
    let codecs = ValueStringCodecRegistry::from_registrations([&STRING_REGISTRATION]).expect("codec registry");
    let bindings = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect("opaque field skipped");
    assert_eq!(bindings.bindings().len(), 0);
}

#[Model]
struct OpaqueMissingCodecContainer {
    #[opaque]
    #[codec(id = "test.unregistered.opaque")]
    hidden: Option<Vec<String>>,
}

#[test]
fn test_opaque_unresolved_field_skips_missing_codec_registration() {
    let root = TypeMetadata::of::<OpaqueMissingCodecContainer>();
    let roots = [root];
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("opaque graph");
    let codecs = ValueStringCodecRegistry::empty();
    let bindings = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect("unresolved opaque field is skipped before registry lookup");
    assert_eq!(bindings.bindings().len(), 0);
}

#[Model]
struct WholeCollectionCodec {
    #[codec(id = "test.string")]
    values: Vec<String>,
}

#[test]
fn test_collection_is_not_unwrapped_for_direct_codec_binding() {
    let root = TypeMetadata::of::<WholeCollectionCodec>();
    let roots = [root];
    let models = ModelRegistry::from_static_metadata(ReflectionContext::global(), &[]).expect("isolated registry");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &roots,
    })
    .resolve()
    .expect("collection graph");
    let codecs = ValueStringCodecRegistry::from_registrations([&STRING_REGISTRATION]).expect("codec registry");
    let errors = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect_err("collection type differs from element codec");
    assert_eq!(errors.errors().len(), 1);
    let error = &errors.errors()[0];
    assert_eq!(error.kind(), CodecBindErrorKind::ValueTypeMismatch);
    assert_eq!(error.expected_type(), TypeId::of::<Vec<String>>());
    assert_eq!(error.actual_type(), Some(TypeId::of::<String>()));
}

/// Diagnostics retain the expected and registered types after collection moves.
#[test]
fn test_codec_errors_expose_recoverable_declaration_context() {
    let source = source();
    let models = ModelRegistry::from_static_metadata(
        ReflectionContext::global(),
        &[
            (TypeMetadata::of::<Missing>(), &source),
            (TypeMetadata::of::<Mismatch>(), &source),
            (TypeMetadata::of::<RustType>(), &source),
        ],
    )
    .expect("diagnostic roots");
    let graph = StructureResolver::new(ResolveInputs {
        models: &models,
        roots: &[],
    })
    .resolve()
    .expect("valid declarations");
    let codecs = ValueStringCodecRegistry::from_registrations([
        &STRING_ALIAS_REGISTRATION,
        &U64_REGISTRATION,
        &STRING_REGISTRATION,
    ])
    .expect("independent codec IDs");
    let errors = bind_codecs(CodecBindInputs {
        graph: &graph,
        codecs: &codecs,
    })
    .expect_err("three distinct failures");
    assert_eq!(errors.to_string(), "3 codec binding error(s)");
    assert!(errors.source().is_none());
    let owned = errors.into_vec();
    assert_eq!(owned.len(), 3);
    for error in &owned {
        assert_eq!(error.expected_type(), TypeId::of::<String>());
        assert_eq!(error.occurrence().property(), "value");
        assert_eq!(error.occurrence().source(), CodecSource::Field);
        assert!(error.occurrence().model().is_some());
        assert!(error.to_string().contains(&error.occurrence().to_string()));
        assert!(error.source().is_none());
        match error.kind() {
            CodecBindErrorKind::Missing => {
                assert_eq!(error.actual_type(), None);
                assert_eq!(error.declaration().declared_id(), Some("test.missing"));
            }
            CodecBindErrorKind::ValueTypeMismatch => {
                assert_eq!(error.actual_type(), Some(TypeId::of::<u64>()));
                assert_eq!(error.declaration().declared_id(), Some("test.wrong"));
            }
            CodecBindErrorKind::Ambiguous => {
                assert_eq!(error.actual_type(), None);
                assert_eq!(
                    error
                        .declaration()
                        .rust_type()
                        .expect("Rust codec declaration")
                        .type_id(),
                    TypeId::of::<StringCodec>()
                );
                let candidates = error.candidate_sources();
                assert_eq!(candidates.len(), 2);
                assert!(candidates.windows(2).all(|pair| pair[0] <= pair[1]));
            }
        }
    }
}
