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
use qubit_model_metadata::TypeMetadata;

/// Synthetic model used to measure generated getter adapters.
#[qubit_model_derive::Model]
struct GetterFixture {
    number: u64,
    label: String,
    alias: Option<String>,
    values: Vec<u64>,
}

#[qubit_model_derive::ModelImpl]
impl GetterFixture {
    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn alias(&self) -> Option<&String> {
        self.alias.as_ref()
    }

    pub fn values(&self) -> &[u64] {
        &self.values
    }
}

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

/// Measures native getters and generated metadata adapters independently.
fn generated_getters(criterion: &mut Criterion) {
    let metadata = TypeMetadata::of::<GetterFixture>();
    let properties = metadata.try_properties().expect("valid getter fixture");
    let number = properties.property("number").expect("generated scalar getter");
    let label = properties.property("label").expect("generated string getter");
    let alias = properties.property("alias").expect("generated optional getter");
    let values = properties.property("values").expect("generated slice getter");
    assert!(number.is_getter());
    assert!(label.is_getter());
    assert!(alias.is_getter());
    assert!(values.is_getter());

    let scalar_fixture = GetterFixture {
        number: 7,
        label: "fixture".to_owned(),
        alias: Some("alias".to_owned()),
        values: Vec::new(),
    };
    assert!(matches!(
        number.get(ReflectedRef::new(&scalar_fixture)),
        Ok(PropertyValue::Owned(_))
    ));
    assert!(matches!(
        label.get(ReflectedRef::new(&scalar_fixture)),
        Ok(PropertyValue::Borrowed(_))
    ));
    assert!(matches!(
        alias.get(ReflectedRef::new(&scalar_fixture)),
        Ok(PropertyValue::OptionalBorrowed(Some(_)))
    ));

    let mut scalar = criterion.benchmark_group("generated_getter/scalar");
    scalar.bench_function("native_getter", |bencher| {
        bencher.iter(|| black_box(black_box(&scalar_fixture).number()))
    });
    scalar.bench_function("property_get", |bencher| {
        bencher.iter(|| black_box(number.get(ReflectedRef::new(black_box(&scalar_fixture)))))
    });
    scalar.bench_function("property_get_and_convert", |bencher| {
        bencher.iter(|| {
            black_box(
                number
                    .get(ReflectedRef::new(black_box(&scalar_fixture)))
                    .map(PropertyValue::into_invocation_output),
            )
        })
    });
    scalar.finish();

    let mut string = criterion.benchmark_group("generated_getter/string");
    string.bench_function("native_getter", |bencher| {
        bencher.iter(|| black_box(black_box(&scalar_fixture).label()))
    });
    string.bench_function("property_get", |bencher| {
        bencher.iter(|| black_box(label.get(ReflectedRef::new(black_box(&scalar_fixture)))))
    });
    string.bench_function("property_get_and_convert", |bencher| {
        bencher.iter(|| {
            black_box(
                label
                    .get(ReflectedRef::new(black_box(&scalar_fixture)))
                    .map(PropertyValue::into_invocation_output),
            )
        })
    });
    string.finish();

    let mut optional = criterion.benchmark_group("generated_getter/optional");
    for (state, alias_value) in [("some", Some("alias".to_owned())), ("none", None)] {
        let fixture = GetterFixture {
            number: 7,
            label: "fixture".to_owned(),
            alias: alias_value,
            values: Vec::new(),
        };
        let expected_some = state == "some";
        let output = alias
            .get(ReflectedRef::new(&fixture))
            .expect("valid optional getter output");
        assert!(matches!(output, PropertyValue::OptionalBorrowed(Some(_))) == expected_some);
        optional.bench_with_input(
            BenchmarkId::new("native_getter", state),
            &fixture,
            |bencher, fixture| bencher.iter(|| black_box(black_box(fixture).alias())),
        );
        optional.bench_with_input(BenchmarkId::new("property_get", state), &fixture, |bencher, fixture| {
            bencher.iter(|| black_box(alias.get(ReflectedRef::new(black_box(fixture)))))
        });
        optional.bench_with_input(
            BenchmarkId::new("property_get_and_convert", state),
            &fixture,
            |bencher, fixture| {
                bencher.iter(|| {
                    black_box(
                        alias
                            .get(ReflectedRef::new(black_box(fixture)))
                            .map(PropertyValue::into_invocation_output),
                    )
                })
            },
        );
    }
    optional.finish();

    let mut slices = criterion.benchmark_group("generated_getter/slice");
    for length in [0, 1, 32, 1024, 65536] {
        let fixture = GetterFixture {
            number: 7,
            label: "fixture".to_owned(),
            alias: None,
            values: vec![7; length],
        };
        let output = values
            .get(ReflectedRef::new(&fixture))
            .expect("valid slice getter output");
        let PropertyValue::BorrowedSlice(output) = output else {
            panic!("slice getter must preserve borrowing");
        };
        assert_eq!(output.len(), length);
        slices.bench_with_input(
            BenchmarkId::new("native_getter", length),
            &fixture,
            |bencher, fixture| bencher.iter(|| black_box(black_box(fixture).values())),
        );
        slices.bench_with_input(
            BenchmarkId::new("property_get", length),
            &fixture,
            |bencher, fixture| bencher.iter(|| black_box(values.get(ReflectedRef::new(black_box(fixture))))),
        );
        slices.bench_with_input(
            BenchmarkId::new("property_get_and_convert", length),
            &fixture,
            |bencher, fixture| {
                bencher.iter(|| {
                    black_box(
                        values
                            .get(ReflectedRef::new(black_box(fixture)))
                            .map(PropertyValue::into_invocation_output),
                    )
                })
            },
        );
    }
    slices.finish();

    let mut lookup = criterion.benchmark_group("generated_getter/lookup_and_get");
    lookup.bench_function("scalar", |bencher| {
        bencher.iter(|| {
            black_box(
                properties
                    .property(black_box("number"))
                    .map(|property| property.get(ReflectedRef::new(black_box(&scalar_fixture)))),
            )
        })
    });
    lookup.bench_function("string", |bencher| {
        bencher.iter(|| {
            black_box(
                properties
                    .property(black_box("label"))
                    .map(|property| property.get(ReflectedRef::new(black_box(&scalar_fixture)))),
            )
        })
    });
    for (state, alias_value) in [("some", Some("alias".to_owned())), ("none", None)] {
        let fixture = GetterFixture {
            number: 7,
            label: "fixture".to_owned(),
            alias: alias_value,
            values: Vec::new(),
        };
        lookup.bench_with_input(BenchmarkId::new("optional", state), &fixture, |bencher, fixture| {
            bencher.iter(|| {
                black_box(
                    properties
                        .property(black_box("alias"))
                        .map(|property| property.get(ReflectedRef::new(black_box(fixture)))),
                )
            })
        });
    }
    for length in [0, 1, 32, 1024, 65536] {
        let fixture = GetterFixture {
            number: 7,
            label: "fixture".to_owned(),
            alias: None,
            values: vec![7; length],
        };
        lookup.bench_with_input(BenchmarkId::new("slice", length), &fixture, |bencher, fixture| {
            bencher.iter(|| {
                black_box(
                    properties
                        .property(black_box("values"))
                        .map(|property| property.get(ReflectedRef::new(black_box(fixture)))),
                )
            })
        });
    }
    lookup.finish();
}

criterion_group!(benches, property_output, generated_getters);
criterion_main!(benches);
