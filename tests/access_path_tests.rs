// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public contracts for compiled dynamic property access paths.

use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::OnceLock;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_metadata::PropertyAccessPath;
use qubit_model_metadata::PropertyAccessPathError;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_reflect::ReflectedMut;
use qubit_reflect::ReflectedOwned;
use qubit_reflect::ReflectedRef;
use qubit_reflect::identity::FragmentIdentity;

thread_local! {
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
}

/// Counts allocations made by only the active test thread.
struct ThreadCountingAllocator;

#[global_allocator]
static ALLOCATOR: ThreadCountingAllocator = ThreadCountingAllocator;

// SAFETY: Allocation calls delegate unchanged pointers and layouts to System.
unsafe impl GlobalAlloc for ThreadCountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = COUNT_ALLOCATIONS.try_with(|enabled| {
            if enabled.get() {
                ALLOCATION_COUNT.with(|count| count.set(count.get() + 1));
            }
        });
        // SAFETY: The caller guarantees a valid allocation layout.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let _ = COUNT_ALLOCATIONS.try_with(|enabled| {
            if enabled.get() {
                ALLOCATION_COUNT.with(|count| count.set(count.get() + 1));
            }
        });
        // SAFETY: The caller guarantees a valid allocation layout.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let _ = COUNT_ALLOCATIONS.try_with(|enabled| {
            if enabled.get() {
                ALLOCATION_COUNT.with(|count| count.set(count.get() + 1));
            }
        });
        // SAFETY: The caller guarantees the pointer, layout, and new size.
        unsafe { System.realloc(pointer, layout, new_size) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: The caller guarantees the pointer and layout.
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[Model(id = "access.Leaf")]
struct AccessLeaf {
    value: String,
}

#[Model(id = "access.Middle")]
struct AccessMiddle {
    leaf: AccessLeaf,
}

#[Model(id = "access.Root")]
struct AccessRoot {
    middle: AccessMiddle,
}

#[Model(id = "access.OptionalRoot")]
struct OptionalRoot {
    middle: Option<AccessMiddle>,
}

#[Model(id = "access.ReadOnlyRoot")]
struct ReadOnlyRoot {
    #[reflect(read_only)]
    middle: AccessMiddle,
}

#[Model(id = "access.SmartPointerRoot")]
struct SmartPointerRoot {
    middle: Box<AccessMiddle>,
}

#[Model(id = "access.OwnedParent")]
struct OwnedParent {
    child_value: String,
}

#[Model(id = "access.SetterOnly")]
struct SetterOnly {}

static SETTER_ONLY_CALLS: AtomicUsize = AtomicUsize::new(0);

#[ModelImpl]
impl SetterOnly {
    pub fn set_value(&mut self, _value: String) {
        SETTER_ONLY_CALLS.fetch_add(1, Ordering::Relaxed);
    }
}

static OWNED_GETTER_CALLS: AtomicUsize = AtomicUsize::new(0);

#[ModelImpl]
impl OwnedParent {
    /// Returns an owned child to verify access-path compile-time rejection.
    pub fn owned_child(&self) -> AccessLeaf {
        OWNED_GETTER_CALLS.fetch_add(1, Ordering::Relaxed);
        AccessLeaf {
            value: self.child_value.clone(),
        }
    }
}

/// A static isolated registry keeps property compilation independent of
/// globals.
fn test_registry() -> ModelRegistry<'static> {
    static LEAF_SOURCE: OnceLock<FragmentIdentity> = OnceLock::new();
    static MIDDLE_SOURCE: OnceLock<FragmentIdentity> = OnceLock::new();
    static ROOT_SOURCE: OnceLock<FragmentIdentity> = OnceLock::new();
    static OPTIONAL_ROOT_SOURCE: OnceLock<FragmentIdentity> = OnceLock::new();
    static READ_ONLY_ROOT_SOURCE: OnceLock<FragmentIdentity> = OnceLock::new();
    static SMART_POINTER_ROOT_SOURCE: OnceLock<FragmentIdentity> = OnceLock::new();
    static SETTER_ONLY_SOURCE: OnceLock<FragmentIdentity> = OnceLock::new();
    let leaf_source = LEAF_SOURCE.get_or_init(|| FragmentIdentity::new("fixture", "access_path", 1, 1, "leaf", 1));
    let middle_source =
        MIDDLE_SOURCE.get_or_init(|| FragmentIdentity::new("fixture", "access_path", 2, 1, "middle", 2));
    let root_source = ROOT_SOURCE.get_or_init(|| FragmentIdentity::new("fixture", "access_path", 3, 1, "root", 3));
    let optional_root_source =
        OPTIONAL_ROOT_SOURCE.get_or_init(|| FragmentIdentity::new("fixture", "access_path", 4, 1, "optional-root", 4));
    let read_only_root_source = READ_ONLY_ROOT_SOURCE
        .get_or_init(|| FragmentIdentity::new("fixture", "access_path", 5, 1, "read-only-root", 5));
    let smart_pointer_root_source = SMART_POINTER_ROOT_SOURCE
        .get_or_init(|| FragmentIdentity::new("fixture", "access_path", 6, 1, "smart-pointer-root", 6));
    let setter_only_source =
        SETTER_ONLY_SOURCE.get_or_init(|| FragmentIdentity::new("fixture", "access_path", 6, 1, "setter-only", 6));
    ModelRegistry::from_static_metadata(&[
        (TypeMetadata::of::<AccessLeaf>(), leaf_source),
        (TypeMetadata::of::<AccessMiddle>(), middle_source),
        (TypeMetadata::of::<AccessRoot>(), root_source),
        (TypeMetadata::of::<OptionalRoot>(), optional_root_source),
        (TypeMetadata::of::<ReadOnlyRoot>(), read_only_root_source),
        (TypeMetadata::of::<SmartPointerRoot>(), smart_pointer_root_source),
        (TypeMetadata::of::<SetterOnly>(), setter_only_source),
    ])
    .expect("isolated model registry")
}

/// Repeated read compilation reuses one path within a registry snapshot.
#[test]
fn test_cached_read_path_reuses_successful_compilation() {
    let registry = test_registry();
    let root = TypeMetadata::of::<AccessRoot>();
    let first = registry
        .compile_read_path_cached(root, &["middle", "leaf", "value"])
        .expect("first read path compiles");
    let second = registry
        .compile_read_path_cached(root, &["middle", "leaf", "value"])
        .expect("second read path uses cache");

    assert!(Arc::ptr_eq(&first, &second));
}

/// A warmed public cache hit clones its Arc without heap allocation.
#[test]
fn test_cached_read_path_hit_does_not_allocate() {
    let registry = test_registry();
    let root = TypeMetadata::of::<AccessRoot>();
    let segments = &["middle", "leaf", "value"];
    let first = registry
        .compile_read_path_cached(root, segments)
        .expect("path warms cache");

    ALLOCATION_COUNT.with(|count| count.set(0));
    COUNT_ALLOCATIONS.with(|enabled| enabled.set(true));
    let second = registry
        .compile_read_path_cached(root, segments)
        .expect("warmed path hits cache");
    COUNT_ALLOCATIONS.with(|enabled| enabled.set(false));
    let allocations = ALLOCATION_COUNT.with(Cell::get);

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(allocations, 0, "cache hit allocated {allocations} times");
}

/// Read and write modes hold distinct compiled paths for the same segments.
#[test]
fn test_cached_path_separates_read_and_write_modes() {
    let registry = test_registry();
    let root = TypeMetadata::of::<AccessRoot>();
    let read = registry
        .compile_read_path_cached(root, &["middle", "leaf", "value"])
        .expect("read path compiles");
    let write = registry
        .compile_write_path_cached(root, &["middle", "leaf", "value"])
        .expect("write path compiles");
    let write_again = registry
        .compile_write_path_cached(root, &["middle", "leaf", "value"])
        .expect("write path uses cache");

    assert!(!Arc::ptr_eq(&read, &write));
    assert!(Arc::ptr_eq(&write, &write_again));
}

/// Separate registries never share cached paths for the same metadata.
#[test]
fn test_cached_path_isolated_between_registry_snapshots() {
    let first_registry = test_registry();
    let second_registry = test_registry();
    let root = TypeMetadata::of::<AccessRoot>();
    let first = first_registry
        .compile_read_path_cached(root, &["middle", "leaf", "value"])
        .expect("first registry compiles");
    let second = second_registry
        .compile_read_path_cached(root, &["middle", "leaf", "value"])
        .expect("second registry compiles");

    assert!(!Arc::ptr_eq(&first, &second));
}

/// Distinct static metadata allocations for one Rust type have distinct keys.
#[test]
fn test_cached_path_separates_metadata_addresses_for_one_type() {
    let registry = test_registry();
    let original = TypeMetadata::of::<AccessRoot>();
    let overlay = Box::leak(Box::new(*original));
    assert_eq!(original.type_id(), overlay.type_id());

    let first = registry
        .compile_read_path_cached(original, &["middle", "leaf", "value"])
        .expect("original metadata path compiles");
    let second = registry
        .compile_read_path_cached(overlay, &["middle", "leaf", "value"])
        .expect("alternate metadata path compiles");

    assert!(!Arc::ptr_eq(&first, &second));
}

/// An invalid path keeps its original error on every attempt.
#[test]
fn test_cached_path_does_not_store_unknown_property_errors() {
    let registry = test_registry();
    let root = TypeMetadata::of::<AccessRoot>();
    let first = registry
        .compile_read_path_cached(root, &["middle", "leaf", "value"])
        .expect("first valid path");
    for _ in 0..2 {
        assert!(matches!(
            registry.compile_read_path_cached(root, &["middle", "missing"]),
            Err(PropertyAccessPathError::UnknownProperty { index: 1, name }) if name == "missing"
        ));
    }
    for _ in 1..256 {
        let overlay = Box::leak(Box::new(*root));
        registry
            .compile_read_path_cached(overlay, &["middle", "leaf", "value"])
            .expect("distinct metadata path compiles");
    }
    let first_again = registry
        .compile_read_path_cached(root, &["middle", "leaf", "value"])
        .expect("first path remains within 256 successful entries");
    assert!(
        Arc::ptr_eq(&first, &first_again),
        "invalid paths must not consume cache capacity"
    );
}

/// Concurrent initial misses still return usable paths for every caller.
#[test]
fn test_cached_path_concurrent_initial_misses_return_valid_paths() {
    let registry = Arc::new(test_registry());
    let barrier = Arc::new(Barrier::new(8));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let registry = Arc::clone(&registry);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                registry
                    .compile_read_path_cached(TypeMetadata::of::<AccessRoot>(), &["middle", "leaf", "value"])
                    .expect("concurrent path compiles")
            })
        })
        .collect();
    let mut first = None;
    for handle in handles {
        let path = handle.join().expect("path compilation thread succeeds");
        assert_eq!(path.leaf_property().name(), "value");
        if let Some(first) = &first {
            assert!(Arc::ptr_eq(first, &path));
        } else {
            first = Some(path);
        }
    }
}

/// Compiles and writes a setter-only leaf without requiring a getter.
#[test]
fn test_compile_for_write_accepts_setter_only_leaf() {
    let registry = ModelRegistry::try_global().expect("linked setter-only model registration");
    let path = PropertyAccessPath::compile_for_write(registry, TypeMetadata::of::<SetterOnly>(), &["value"])
        .expect("setter-only leaf compiles for writing");
    assert!(path.check_writable().is_ok());
    let mut root = SetterOnly {};
    SETTER_ONLY_CALLS.store(0, Ordering::Relaxed);

    path.write(ReflectedMut::new(&mut root), ReflectedOwned::new("after".to_owned()))
        .expect("setter-only leaf accepts replacement");
    assert_eq!(SETTER_ONLY_CALLS.load(Ordering::Relaxed), 1);
    assert!(matches!(
        PropertyAccessPath::compile(registry, TypeMetadata::of::<SetterOnly>(), &["value"],),
        Err(PropertyAccessPathError::UnreadableIntermediate { index: 0, .. })
    ));
}

/// Reports an absent optional intermediate separately from an unknown property.
#[test]
fn test_read_optional_intermediate_reports_missing_value() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<OptionalRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("optional model path compiles");
    let root = OptionalRoot { middle: None };

    assert!(matches!(
        path.read(ReflectedRef::new(&root)),
        Err(PropertyAccessPathError::MissingIntermediate { index: 0, .. })
    ));
    assert!(matches!(
        path.check_writable(),
        Err(PropertyAccessPathError::UnwritableIntermediate { index: 0, .. })
    ));
}

/// Rejects a getter that returns ownership before any getter invocation.
#[test]
fn test_compile_rejects_owned_intermediate_getter() {
    let registry = ModelRegistry::try_global().expect("linked model registrations");
    OWNED_GETTER_CALLS.store(0, Ordering::Relaxed);
    let result = PropertyAccessPath::compile(registry, TypeMetadata::of::<OwnedParent>(), &["owned_child", "value"]);

    assert!(matches!(
        result,
        Err(PropertyAccessPathError::UnsupportedIntermediate { index: 0, .. })
    ));
    assert_eq!(OWNED_GETTER_CALLS.load(Ordering::Relaxed), 0);
}

/// Rejects smart-pointer intermediates rather than treating the pointer as its
/// target.
#[test]
fn test_compile_rejects_smart_pointer_intermediate() {
    let registry = test_registry();
    let result = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<SmartPointerRoot>(),
        &["middle", "leaf", "value"],
    );

    assert!(matches!(
        result,
        Err(PropertyAccessPathError::UnsupportedIntermediate { index: 0, .. })
    ));
}

/// Reports a reflection read-only policy at runtime and retains the input.
#[test]
fn test_write_read_only_intermediate_reports_access_failure() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<ReadOnlyRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("read-only field remains a valid path shape");
    assert!(path.check_writable().is_ok());
    let mut root = ReadOnlyRoot {
        middle: AccessMiddle {
            leaf: AccessLeaf {
                value: "unchanged".to_owned(),
            },
        },
    };

    let failure = path
        .write(
            ReflectedMut::new(&mut root),
            ReflectedOwned::new("secret-value".to_owned()),
        )
        .expect_err("reflection policy blocks mutable projection");
    assert!(matches!(
        failure.path_error(),
        Some(PropertyAccessPathError::AccessFailure { .. })
    ));
    assert!(failure.property_failure().is_none());
    assert!(std::error::Error::source(&failure).is_some());
    assert_eq!(
        failure.to_string(),
        failure.path_error().expect("path failure").to_string()
    );
    assert!(!format!("{failure:?}").contains("secret-value"));
    assert_eq!(
        failure
            .replacement()
            .and_then(|value| value.downcast_ref::<String>())
            .map(String::as_str),
        Some("secret-value")
    );
    assert_eq!(root.middle.leaf.value, "unchanged");
}

/// Reads a leaf property by traversing each real intermediate object.
#[test]
fn test_read_nested_property_from_root_instance() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<AccessRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("valid three-level property path");
    let root = AccessRoot {
        middle: AccessMiddle {
            leaf: AccessLeaf {
                value: "nested".to_owned(),
            },
        },
    };

    let value = path
        .read(ReflectedRef::new(&root))
        .expect("nested property is readable");
    let PropertyValue::Borrowed(value) = value else {
        panic!("string field should be returned as a borrow");
    };
    assert_eq!(value.downcast_ref::<String>().map(String::as_str), Some("nested"));
    assert_eq!(path.leaf_property().name(), "value");
}

/// Writes a leaf value through mutable field projections from the actual root.
#[test]
fn test_write_nested_property_through_root_instance() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<AccessRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("valid three-level property path");
    assert!(path.check_writable().is_ok());
    let mut root = AccessRoot {
        middle: AccessMiddle {
            leaf: AccessLeaf {
                value: "before".to_owned(),
            },
        },
    };

    path.write(ReflectedMut::new(&mut root), ReflectedOwned::new("after".to_owned()))
        .expect("nested property is writable");
    assert_eq!(root.middle.leaf.value, "after");
}

/// Rejects empty, unknown, and root-mismatched path inputs structurally.
#[test]
fn test_compile_and_read_report_structured_path_errors() {
    let registry = test_registry();
    assert!(matches!(
        PropertyAccessPath::compile(&registry, TypeMetadata::of::<AccessRoot>(), &[],),
        Err(PropertyAccessPathError::EmptyPath)
    ));
    assert!(matches!(
        PropertyAccessPath::compile(&registry, TypeMetadata::of::<AccessRoot>(), &[""],),
        Err(PropertyAccessPathError::EmptySegment { index: 0 })
    ));
    assert!(matches!(
        PropertyAccessPath::compile(&registry, TypeMetadata::of::<AccessRoot>(), &["middle", "missing"],),
        Err(PropertyAccessPathError::UnknownProperty { index: 1, .. })
    ));
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<AccessRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("valid property path");
    assert!(matches!(
        path.read(ReflectedRef::new(&AccessLeaf {
            value: "wrong root".to_owned(),
        })),
        Err(PropertyAccessPathError::RootTypeMismatch { .. })
    ));
}

/// Keeps a replacement recoverable when root validation fails before traversal.
#[test]
fn test_write_root_mismatch_preserves_replacement() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<AccessRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("valid property path");
    let mut wrong_root = AccessLeaf {
        value: "unchanged".to_owned(),
    };
    let failure = path
        .write(
            ReflectedMut::new(&mut wrong_root),
            ReflectedOwned::new("replacement".to_owned()),
        )
        .expect_err("root type must match");

    assert!(matches!(
        failure.path_error(),
        Some(PropertyAccessPathError::RootTypeMismatch { .. })
    ));
    assert_eq!(
        failure
            .replacement()
            .and_then(|value| value.downcast_ref::<String>())
            .map(String::as_str),
        Some("replacement")
    );
}

/// Preserves the leaf setter's replacement recovery on a value type mismatch.
#[test]
fn test_write_leaf_failure_preserves_replacement() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<AccessRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("valid property path");
    let mut root = AccessRoot {
        middle: AccessMiddle {
            leaf: AccessLeaf {
                value: "unchanged".to_owned(),
            },
        },
    };
    let failure = path
        .write(ReflectedMut::new(&mut root), ReflectedOwned::new(17_u32))
        .expect_err("leaf value type must match");

    assert!(failure.property_failure().is_some());
    assert!(failure.path_error().is_none());
    assert!(std::error::Error::source(&failure).is_some());
    assert_eq!(
        failure.to_string(),
        failure.property_failure().expect("leaf failure").to_string()
    );
    assert!(format!("{failure:?}").contains("Property"));
    assert_eq!(
        failure.replacement().and_then(|value| value.downcast_ref::<u32>()),
        Some(&17)
    );
    assert_eq!(root.middle.leaf.value, "unchanged");
}
