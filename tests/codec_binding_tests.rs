// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for explicit codec binding after structure resolution.

#![cfg(feature = "codec")]

use qubit_codec::ValueCodecDescriptor;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueCodecRegistry;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_model_derive::Model;
use qubit_model_metadata::codec::CodecBindErrorKind;
use qubit_model_metadata::codec::CodecBindInputs;
use qubit_model_metadata::codec::bind_codecs;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ModelGraph;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::identity::FragmentIdentity;

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

static STRING_DESCRIPTOR: ValueCodecDescriptor =
    ValueCodecDescriptor::of::<StringCodec, String>();
static STRING_REGISTRATION: ValueCodecRegistration =
    ValueCodecRegistration::new(
        ValueCodecId::new("test.string"),
        &STRING_DESCRIPTOR,
        ValueCodecRegistrationSource::new(
            "codec-tests",
            "fixture",
            file!(),
            line!(),
        ),
    );
static STRING_ALIAS_REGISTRATION: ValueCodecRegistration =
    ValueCodecRegistration::new(
        ValueCodecId::new("test.string.alias"),
        &STRING_DESCRIPTOR,
        ValueCodecRegistrationSource::new(
            "codec-tests",
            "fixture",
            file!(),
            line!(),
        ),
    );
static U64_DESCRIPTOR: ValueCodecDescriptor =
    ValueCodecDescriptor::of::<U64Codec, u64>();
static U64_REGISTRATION: ValueCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("test.wrong"),
    &U64_DESCRIPTOR,
    ValueCodecRegistrationSource::new(
        "codec-tests",
        "fixture",
        file!(),
        line!(),
    ),
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

fn source() -> FragmentIdentity {
    FragmentIdentity::new("codec-tests", "fixture", line!(), 1, "model", 1)
}

fn graph<'a>(
    metadata: &'static TypeMetadata,
    source: &'a FragmentIdentity,
) -> ModelGraph<'a> {
    let models = ModelRegistry::from_metadata(&[(metadata, source)])
        .expect("model registry");
    let models = Box::leak(Box::new(models));
    StructureResolver::new(ResolveInputs { models })
        .resolve()
        .expect("model graph")
}

#[test]
fn binds_declared_and_rust_type_references() {
    let source = source();
    let declared_graph = graph(TypeMetadata::of::<Success>(), &source);
    let rust_graph = graph(TypeMetadata::of::<RustType>(), &source);
    let codecs = ValueCodecRegistry::from_registrations([&STRING_REGISTRATION])
        .expect("codec registry");

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
fn reports_sorted_missing_ambiguous_and_type_mismatch_errors() {
    let source = source();
    let missing_graph = graph(TypeMetadata::of::<Missing>(), &source);
    let mismatch_graph = graph(TypeMetadata::of::<Mismatch>(), &source);
    let rust_graph = graph(TypeMetadata::of::<RustType>(), &source);
    let codecs = ValueCodecRegistry::from_registrations([
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
    assert_eq!(
        mismatch.errors()[0].kind(),
        CodecBindErrorKind::ValueTypeMismatch
    );
    assert_eq!(
        mismatch.errors()[0].candidate_sources(),
        &[U64_REGISTRATION.source()]
    );
    let ambiguous = bind_codecs(CodecBindInputs {
        graph: &rust_graph,
        codecs: &codecs,
    })
    .unwrap_err();
    assert_eq!(ambiguous.errors()[0].kind(), CodecBindErrorKind::Ambiguous);
    assert_eq!(ambiguous.errors()[0].candidate_sources().len(), 2);
}
