// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Measures codec binding across graphs with fixed field counts and varying
//! numbers of unrelated empty model roots.

use std::convert::Infallible;
use std::hint::black_box;

use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;
use qubit_codec::ValueStringCodecRegistry;
use qubit_model_derive::Model;
use qubit_model_derive::Value;
use qubit_model_metadata::codec::CodecBindInputs;
use qubit_model_metadata::codec::bind_codecs;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::resolve::ResolveInputs;
use qubit_model_metadata::resolve::StructureResolver;
use qubit_reflect::ReflectionContext;

#[derive(Default)]
struct BenchNameCodec;

#[Value(id = "benchmark.CodecBindingName", transparent, codec = BenchNameCodec)]
struct BenchName(String);

impl ValueEncoder<BenchName> for BenchNameCodec {
    type Output = String;
    type Error = Infallible;

    fn encode(&mut self, input: &BenchName) -> Result<Self::Output, Self::Error> {
        Ok(input.0.clone())
    }
}

impl ValueDecoder<str> for BenchNameCodec {
    type Output = BenchName;
    type Error = Infallible;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(BenchName(input.to_owned()))
    }
}

static NAME_CODEC_DESCRIPTOR: ValueStringCodecDescriptor =
    ValueStringCodecDescriptor::of::<BenchNameCodec, BenchName>();
static NAME_CODEC_REGISTRATION: ValueStringCodecRegistration = ValueStringCodecRegistration::new(
    ValueCodecId::new("benchmark.name"),
    &NAME_CODEC_DESCRIPTOR,
    ValueCodecRegistrationSource::new("qubit-model-metadata", "codec-binding-bench", file!(), line!()),
);

#[Model(id = "benchmark.CodecBindingHitOwner")]
#[allow(dead_code)]
struct HitOwner {
    field_00: BenchName,
    field_01: BenchName,
    field_02: BenchName,
    field_03: BenchName,
    field_04: BenchName,
    field_05: BenchName,
    field_06: BenchName,
    field_07: BenchName,
    field_08: BenchName,
    field_09: BenchName,
    field_10: BenchName,
    field_11: BenchName,
    field_12: BenchName,
    field_13: BenchName,
    field_14: BenchName,
    field_15: BenchName,
    field_16: BenchName,
    field_17: BenchName,
    field_18: BenchName,
    field_19: BenchName,
    field_20: BenchName,
    field_21: BenchName,
    field_22: BenchName,
    field_23: BenchName,
    field_24: BenchName,
    field_25: BenchName,
    field_26: BenchName,
    field_27: BenchName,
    field_28: BenchName,
    field_29: BenchName,
    field_30: BenchName,
    field_31: BenchName,
}

#[Model(id = "benchmark.CodecBindingMissOwner")]
#[allow(dead_code)]
struct MissOwner {
    field_00: String,
    field_01: String,
    field_02: String,
    field_03: String,
    field_04: String,
    field_05: String,
    field_06: String,
    field_07: String,
    field_08: String,
    field_09: String,
    field_10: String,
    field_11: String,
    field_12: String,
    field_13: String,
    field_14: String,
    field_15: String,
    field_16: String,
    field_17: String,
    field_18: String,
    field_19: String,
    field_20: String,
    field_21: String,
    field_22: String,
    field_23: String,
    field_24: String,
    field_25: String,
    field_26: String,
    field_27: String,
    field_28: String,
    field_29: String,
    field_30: String,
    field_31: String,
}

macro_rules! empty_models {
    ($(($model:ident, $id:literal)),+ $(,)?) => {
        $(
            #[Model(id = $id)]
            #[allow(dead_code)]
            struct $model;
        )+
        fn empty_model_metadata() -> Vec<&'static TypeMetadata> {
            vec![$(TypeMetadata::of::<$model>()),+]
        }
    };
}

empty_models! {
    (Empty00, "benchmark.CodecBindingEmpty00"), (Empty01, "benchmark.CodecBindingEmpty01"),
    (Empty02, "benchmark.CodecBindingEmpty02"), (Empty03, "benchmark.CodecBindingEmpty03"),
    (Empty04, "benchmark.CodecBindingEmpty04"), (Empty05, "benchmark.CodecBindingEmpty05"),
    (Empty06, "benchmark.CodecBindingEmpty06"), (Empty07, "benchmark.CodecBindingEmpty07"),
    (Empty08, "benchmark.CodecBindingEmpty08"), (Empty09, "benchmark.CodecBindingEmpty09"),
    (Empty10, "benchmark.CodecBindingEmpty10"), (Empty11, "benchmark.CodecBindingEmpty11"),
    (Empty12, "benchmark.CodecBindingEmpty12"), (Empty13, "benchmark.CodecBindingEmpty13"),
    (Empty14, "benchmark.CodecBindingEmpty14"), (Empty15, "benchmark.CodecBindingEmpty15"),
    (Empty16, "benchmark.CodecBindingEmpty16"), (Empty17, "benchmark.CodecBindingEmpty17"),
    (Empty18, "benchmark.CodecBindingEmpty18"), (Empty19, "benchmark.CodecBindingEmpty19"),
    (Empty20, "benchmark.CodecBindingEmpty20"), (Empty21, "benchmark.CodecBindingEmpty21"),
    (Empty22, "benchmark.CodecBindingEmpty22"), (Empty23, "benchmark.CodecBindingEmpty23"),
    (Empty24, "benchmark.CodecBindingEmpty24"), (Empty25, "benchmark.CodecBindingEmpty25"),
    (Empty26, "benchmark.CodecBindingEmpty26"), (Empty27, "benchmark.CodecBindingEmpty27"),
    (Empty28, "benchmark.CodecBindingEmpty28"), (Empty29, "benchmark.CodecBindingEmpty29"),
    (Empty30, "benchmark.CodecBindingEmpty30"), (Empty31, "benchmark.CodecBindingEmpty31"),
    (Empty32, "benchmark.CodecBindingEmpty32"), (Empty33, "benchmark.CodecBindingEmpty33"),
    (Empty34, "benchmark.CodecBindingEmpty34"), (Empty35, "benchmark.CodecBindingEmpty35"),
    (Empty36, "benchmark.CodecBindingEmpty36"), (Empty37, "benchmark.CodecBindingEmpty37"),
    (Empty38, "benchmark.CodecBindingEmpty38"), (Empty39, "benchmark.CodecBindingEmpty39"),
    (Empty40, "benchmark.CodecBindingEmpty40"), (Empty41, "benchmark.CodecBindingEmpty41"),
    (Empty42, "benchmark.CodecBindingEmpty42"), (Empty43, "benchmark.CodecBindingEmpty43"),
    (Empty44, "benchmark.CodecBindingEmpty44"), (Empty45, "benchmark.CodecBindingEmpty45"),
    (Empty46, "benchmark.CodecBindingEmpty46"), (Empty47, "benchmark.CodecBindingEmpty47"),
    (Empty48, "benchmark.CodecBindingEmpty48"), (Empty49, "benchmark.CodecBindingEmpty49"),
    (Empty50, "benchmark.CodecBindingEmpty50"), (Empty51, "benchmark.CodecBindingEmpty51"),
    (Empty52, "benchmark.CodecBindingEmpty52"), (Empty53, "benchmark.CodecBindingEmpty53"),
    (Empty54, "benchmark.CodecBindingEmpty54"), (Empty55, "benchmark.CodecBindingEmpty55"),
    (Empty56, "benchmark.CodecBindingEmpty56"), (Empty57, "benchmark.CodecBindingEmpty57"),
    (Empty58, "benchmark.CodecBindingEmpty58"), (Empty59, "benchmark.CodecBindingEmpty59"),
    (Empty60, "benchmark.CodecBindingEmpty60"), (Empty61, "benchmark.CodecBindingEmpty61"),
    (Empty62, "benchmark.CodecBindingEmpty62"), (Empty63, "benchmark.CodecBindingEmpty63"),
}

fn run_benchmarks(criterion: &mut Criterion) {
    let reflection = ReflectionContext::builder()
        .include_linked()
        .build()
        .expect("benchmark reflection context");
    let models = ModelRegistry::from_context(&reflection).expect("benchmark model registry");
    let codecs =
        ValueStringCodecRegistry::from_registrations([&NAME_CODEC_REGISTRATION]).expect("benchmark codec registry");
    let empty_models = empty_model_metadata();

    for (group_name, owner, expected_bindings) in [
        ("canonical_hit", Some(TypeMetadata::of::<HitOwner>()), 33),
        ("canonical_miss", Some(TypeMetadata::of::<MissOwner>()), 0),
    ] {
        let mut group = criterion.benchmark_group(group_name);
        for noise_count in [0, 16, 64] {
            let mut roots = empty_models[..noise_count].to_vec();
            roots.extend(owner);
            let roots = Box::leak(roots.into_boxed_slice());
            let graph = StructureResolver::for_roots(ResolveInputs { models: &models, roots })
                .resolve()
                .expect("benchmark graph");
            let expected_nodes = noise_count + if group_name == "canonical_hit" { 2 } else { 1 };
            assert_eq!(graph.models().len(), expected_nodes);
            let initial = bind_codecs(CodecBindInputs {
                graph: &graph,
                codecs: &codecs,
            })
            .expect("benchmark binding succeeds");
            assert_eq!(initial.bindings().len(), expected_bindings);
            group.bench_with_input(
                BenchmarkId::new("fields_32_nodes", expected_nodes),
                &graph,
                |bencher, graph| {
                    bencher.iter(|| {
                        let result = bind_codecs(CodecBindInputs {
                            graph: black_box(graph),
                            codecs: black_box(&codecs),
                        })
                        .expect("benchmark binding succeeds");
                        black_box(result)
                    });
                },
            );
        }
        group.finish();
    }

    let mut group = criterion.benchmark_group("empty_graph_walk");
    for noise_count in [0, 16, 64] {
        let roots = Box::leak(empty_models[..noise_count].to_vec().into_boxed_slice());
        let graph = StructureResolver::for_roots(ResolveInputs { models: &models, roots })
            .resolve()
            .expect("empty benchmark graph");
        assert_eq!(graph.models().len(), noise_count);
        let initial = bind_codecs(CodecBindInputs {
            graph: &graph,
            codecs: &codecs,
        })
        .expect("empty graph has no bindings");
        assert_eq!(initial.bindings().len(), 0);
        group.bench_with_input(
            BenchmarkId::new("fields_0_nodes", noise_count),
            &graph,
            |bencher, graph| {
                bencher.iter(|| {
                    let result = bind_codecs(CodecBindInputs {
                        graph: black_box(graph),
                        codecs: black_box(&codecs),
                    })
                    .expect("empty graph has no bindings");
                    black_box(result)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, run_benchmarks);
criterion_main!(benches);
