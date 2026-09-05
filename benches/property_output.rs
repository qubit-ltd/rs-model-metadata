// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Measures explicit output conversion separately from lazy slice access.

use std::hint::black_box;

use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_model_metadata::BorrowedPropertySlice;
use qubit_model_metadata::PropertyValue;
use qubit_model_metadata::ReflectedOwned;
use qubit_model_metadata::ReflectedRef;

/// Includes wrapper construction and output destruction, but excludes fixtures.
fn property_output(criterion: &mut Criterion) {
    let value = 7_u64;
    let mut scalar = criterion.benchmark_group("property_scalar");
    scalar.bench_function("borrowed", |bencher| {
        bencher.iter(|| {
            black_box(PropertyValue::Borrowed(ReflectedRef::new(black_box(&value))).into_invocation_output());
        })
    });
    scalar.bench_function("optional_some", |bencher| {
        bencher.iter(|| {
            black_box(
                PropertyValue::OptionalBorrowed(Some(ReflectedRef::new(black_box(&value)))).into_invocation_output(),
            );
        })
    });
    scalar.bench_function("optional_none", |bencher| {
        bencher.iter(|| {
            black_box(PropertyValue::OptionalBorrowed(None).into_invocation_output());
        })
    });
    scalar.bench_function("owned", |bencher| {
        bencher.iter_batched(
            || PropertyValue::Owned(ReflectedOwned::new(black_box(value))),
            |input| {
                black_box(input.into_invocation_output());
            },
            BatchSize::SmallInput,
        )
    });
    scalar.finish();
    let mut slices = criterion.benchmark_group("property_slice");
    for length in [0, 1, 32, 1024, 65536] {
        let values = vec![value; length];
        slices.bench_with_input(BenchmarkId::new("native", length), &values, |bencher, values| {
            bencher.iter(|| {
                black_box(black_box(values.as_slice()).get(values.len() / 2));
            });
        });
        slices.bench_with_input(BenchmarkId::new("direct", length), &values, |bencher, values| {
            bencher.iter(|| {
                let slice = BorrowedPropertySlice::new(black_box(values.as_slice()));
                black_box(slice.get(slice.len() / 2));
            });
        });
        slices.bench_with_input(BenchmarkId::new("convert", length), &values, |bencher, values| {
            bencher.iter(|| {
                let input = PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(black_box(values.as_slice())));
                black_box(input.into_invocation_output());
            });
        });
        slices.bench_with_input(
            BenchmarkId::new("conversion_only", length),
            &values,
            |bencher, values| {
                bencher.iter_batched(
                    || PropertyValue::BorrowedSlice(BorrowedPropertySlice::new(black_box(values.as_slice()))),
                    |input| {
                        black_box(input.into_invocation_output());
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    slices.finish();
}

criterion_group!(benches, property_output);
criterion_main!(benches);
