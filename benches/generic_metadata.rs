// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Measures one cold initialization per concrete type and warm lookup costs.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::hint::black_box;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Instant;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;
use qubit_model_derive::Model;
use qubit_model_metadata::__private::TypeMetadataProvider;
use qubit_model_metadata::metadata::HasTypeMetadata;
use qubit_model_metadata::metadata::TypeMetadata;

#[Model(id = "bench.GenericMetadata")]
struct GenericMetadata<T> {
    value: T,
}

struct CountingAllocator;

static ALLOCATION_CALLS: AtomicU64 = AtomicU64::new(0);
static DEALLOCATION_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            ALLOCATION_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            ALLOCATION_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        DEALLOCATION_CALLS.fetch_add(1, Ordering::Relaxed);
        DEALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, new_size) };
        if !replacement.is_null() {
            ALLOCATION_CALLS.fetch_add(1, Ordering::Relaxed);
            DEALLOCATION_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
            DEALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        }
        replacement
    }
}

#[derive(Clone, Copy)]
struct AllocationSnapshot {
    allocations: u64,
    deallocations: u64,
    allocated_bytes: u64,
    deallocated_bytes: u64,
}

impl AllocationSnapshot {
    fn capture() -> Self {
        Self {
            allocations: ALLOCATION_CALLS.load(Ordering::Relaxed),
            deallocations: DEALLOCATION_CALLS.load(Ordering::Relaxed),
            allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
            deallocated_bytes: DEALLOCATED_BYTES.load(Ordering::Relaxed),
        }
    }

    fn live_bytes(self) -> i128 {
        i128::from(self.allocated_bytes) - i128::from(self.deallocated_bytes)
    }
}

/// Invokes the checked provider once for this concrete type and reports its
/// allocator activity. The first type also initializes shared definition data.
fn cold_once<T: HasTypeMetadata>(name: &str) {
    let before = AllocationSnapshot::capture();
    let started = Instant::now();
    let metadata = TypeMetadata::try_of::<T>().expect("valid generated metadata");
    let elapsed = started.elapsed();
    black_box(metadata);
    let after = AllocationSnapshot::capture();
    println!(
        "cold/{name}: elapsed_ns={} allocation_calls={} deallocation_calls={} allocated_bytes={} retained_bytes_delta={}",
        elapsed.as_nanos(),
        after.allocations - before.allocations,
        after.deallocations - before.deallocations,
        after.allocated_bytes - before.allocated_bytes,
        after.live_bytes() - before.live_bytes(),
    );
}

fn warm<T: HasTypeMetadata>(criterion: &mut Criterion, name: &str) {
    let metadata = <T as TypeMetadataProvider>::__type_metadata();
    criterion.bench_function(&format!("generic_metadata/static_ref/{name}"), |b| {
        b.iter(|| black_box(metadata));
    });
    criterion.bench_function(&format!("generic_metadata/try_of/{name}"), |b| {
        b.iter(|| black_box(TypeMetadata::try_of::<T>().expect("valid generated metadata")));
    });
    criterion.bench_function(&format!("generic_metadata/provider/{name}"), |b| {
        b.iter(|| black_box(<T as TypeMetadataProvider>::__type_metadata()));
    });
}

fn generic_metadata(criterion: &mut Criterion) {
    cold_once::<GenericMetadata<u32>>("u32");
    cold_once::<GenericMetadata<u64>>("u64");
    cold_once::<GenericMetadata<String>>("String");

    warm::<GenericMetadata<u32>>(criterion, "u32");
    warm::<GenericMetadata<u64>>(criterion, "u64");
    warm::<GenericMetadata<String>>(criterion, "String");
}

criterion_group!(benches, generic_metadata);
criterion_main!(benches);
