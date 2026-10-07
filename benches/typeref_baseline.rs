// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Measures first-use allocation and elapsed time for repeated generated
//! getter and setter type adapters.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::any::TypeId;
use std::hint::black_box;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Instant;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::registry::ReflectRegistry;

struct CountingAllocator;

static MEASURE: AtomicBool = AtomicBool::new(false);
static ALLOCATED: AtomicU64 = AtomicU64::new(0);
static DEALLOCATED: AtomicU64 = AtomicU64::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() && MEASURE.load(Ordering::Relaxed) {
            ALLOCATED.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() && MEASURE.load(Ordering::Relaxed) {
            ALLOCATED.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        if MEASURE.load(Ordering::Relaxed) {
            DEALLOCATED.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
        if !replacement.is_null() && MEASURE.load(Ordering::Relaxed) {
            ALLOCATED.fetch_add(new_size as u64, Ordering::Relaxed);
            DEALLOCATED.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        replacement
    }
}

#[Model(id = "benchmark.TypeRefRepeatedAccessors")]
struct RepeatedAccessors {
    field_0: String,
    field_1: String,
    field_2: String,
    field_3: String,
    field_4: String,
    field_5: String,
    field_6: String,
    field_7: String,
}

#[ModelImpl]
impl RepeatedAccessors {
    pub fn field_0(&self) -> &str {
        &self.field_0
    }
    pub fn field_1(&self) -> &str {
        &self.field_1
    }
    pub fn field_2(&self) -> &str {
        &self.field_2
    }
    pub fn field_3(&self) -> &str {
        &self.field_3
    }
    pub fn field_4(&self) -> &str {
        &self.field_4
    }
    pub fn field_5(&self) -> &str {
        &self.field_5
    }
    pub fn field_6(&self) -> &str {
        &self.field_6
    }
    pub fn field_7(&self) -> &str {
        &self.field_7
    }
    pub fn set_field_0(&mut self, value: String) {
        self.field_0 = value;
    }
    pub fn set_field_1(&mut self, value: String) {
        self.field_1 = value;
    }
    pub fn set_field_2(&mut self, value: String) {
        self.field_2 = value;
    }
    pub fn set_field_3(&mut self, value: String) {
        self.field_3 = value;
    }
    pub fn set_field_4(&mut self, value: String) {
        self.field_4 = value;
    }
    pub fn set_field_5(&mut self, value: String) {
        self.field_5 = value;
    }
    pub fn set_field_6(&mut self, value: String) {
        self.field_6 = value;
    }
    pub fn set_field_7(&mut self, value: String) {
        self.field_7 = value;
    }
}

fn repeated_accessors(criterion: &mut Criterion) {
    let reflection = ReflectRegistry::initialize().expect("reflection registry");
    let metadata = TypeMetadata::of::<RepeatedAccessors>();
    assert_eq!(metadata.fields().len(), 8);
    let getter_count = 8;
    let setter_count = 8;
    let reflected_type_ref_calls = getter_count + setter_count;
    let distinct_type_ids = [TypeId::of::<String>(), TypeId::of::<str>()];
    println!(
        "fixture/repeated_accessors: fields=8 getters=8 setters=8 reflected_type_ref_call_upper_bound={} distinct_type_ids={} upper_bound_bytes={}",
        reflected_type_ref_calls,
        distinct_type_ids.len(),
        reflected_type_ref_calls * std::mem::size_of::<qubit_reflect::descriptor::TypeRef>(),
    );

    let before_alloc = ALLOCATED.load(Ordering::Relaxed);
    let before_dealloc = DEALLOCATED.load(Ordering::Relaxed);
    let started = Instant::now();
    MEASURE.store(true, Ordering::SeqCst);
    let properties = black_box(
        metadata
            .try_properties_in(black_box(reflection))
            .expect("generated getter and setter properties"),
    );
    MEASURE.store(false, Ordering::SeqCst);
    let elapsed = started.elapsed();
    assert_eq!(properties.properties().len(), 8);
    let allocated = ALLOCATED.load(Ordering::Relaxed) - before_alloc;
    let deallocated = DEALLOCATED.load(Ordering::Relaxed) - before_dealloc;
    println!(
        "cold/repeated_accessors: elapsed_ns={} allocation_bytes={} net_retained_bytes={}",
        elapsed.as_nanos(),
        allocated,
        allocated as i128 - deallocated as i128,
    );

    let mut group = criterion.benchmark_group("typeref_baseline/repeated_accessors");
    group.sample_size(30);
    group.bench_function("warm_properties", |bencher| {
        bencher.iter(|| black_box(metadata.try_properties_in(black_box(reflection))));
    });
    group.finish();
}

criterion_group!(benches, repeated_accessors);
criterion_main!(benches);
