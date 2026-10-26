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
use qubit_model_metadata::__private::v8::model_impl_registration;
use qubit_model_metadata::__private::v8::model_metadata_registration;
use qubit_model_metadata::PropertyAccessPath;
use qubit_model_metadata::PropertyAccessPathError;
use qubit_model_metadata::metadata::PropertyBorrowed;
use qubit_model_metadata::metadata::PropertyMutable;
use qubit_model_metadata::metadata::PropertyOwned;
use qubit_model_metadata::metadata::PropertyValue;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_model_metadata::registry::ModelRegistry;
use qubit_model_metadata::registry::ModelRegistryOrigin;
use qubit_reflect::ReflectionContext;
use qubit_reflect::value::DynamicMut;
use qubit_reflect::value::DynamicOwned;
use qubit_reflect::value::DynamicRef;
use qubit_reflect::value::Local;

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
pub struct AccessLeaf {
    pub value: String,
}

#[Model(id = "access.Middle")]
pub struct AccessMiddle {
    pub leaf: AccessLeaf,
    pub value: String,
}

#[Model(id = "access.Root")]
pub struct AccessRoot {
    pub middle: AccessMiddle,
}

#[Model(id = "access.OptionalRoot")]
pub struct OptionalRoot {
    pub middle: Option<AccessMiddle>,
}

#[Model(id = "access.PrivateRoot")]
struct PrivateRoot {
    middle: AccessMiddle,
}

#[Model(id = "access.SmartPointerRoot")]
pub struct SmartPointerRoot {
    pub middle: Box<AccessMiddle>,
}

#[Model(id = "access.OwnedParent")]
struct OwnedParent {
    child_value: String,
}

#[Model(id = "access.SliceParent")]
struct SliceParent {
    items: Vec<AccessLeaf>,
}

#[Model(id = "access.BorrowedParent")]
struct BorrowedParent {
    child: AccessLeaf,
}

#[Model(id = "access.BorrowedOptionalParent")]
struct BorrowedOptionalParent {
    child: Option<AccessMiddle>,
}

#[Model(id = "access.SetterOnly")]
struct SetterOnly {}

#[Model(id = "access.PublicRoot")]
struct PublicRoot {
    pub value: String,
}

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

#[ModelImpl]
impl SliceParent {
    pub fn children(&self) -> &[AccessLeaf] {
        &self.items
    }
}

#[ModelImpl]
impl BorrowedParent {
    pub fn child(&self) -> &AccessLeaf {
        &self.child
    }
}

#[ModelImpl]
impl BorrowedOptionalParent {
    pub fn child(&self) -> Option<&AccessMiddle> {
        self.child.as_ref()
    }
}

/// A static isolated registry keeps property compilation independent of
/// globals.
fn test_registry() -> ModelRegistry<'static> {
    static SOURCE: ModelRegistryOrigin = ModelRegistryOrigin {
        crate_name: "fixture",
        crate_version: None,
        module_path: "access_path",
        file: Some("tests/access_path_tests.rs"),
        line: Some(1),
        column: None,
    };
    static CONTEXT: OnceLock<ReflectionContext> = OnceLock::new();
    let context = CONTEXT.get_or_init(|| {
        ReflectionContext::builder()
            .include_type::<AccessLeaf>()
            .include_type::<AccessMiddle>()
            .include_type::<AccessRoot>()
            .include_type::<OptionalRoot>()
            .include_type::<Option<AccessMiddle>>()
            .include_type::<PrivateRoot>()
            .include_type::<SmartPointerRoot>()
            .include_type::<SetterOnly>()
            .include_type::<PublicRoot>()
            .include_type::<OwnedParent>()
            .include_type::<SliceParent>()
            .include_type::<BorrowedParent>()
            .include_type::<BorrowedOptionalParent>()
            .register(model_metadata_registration::<OwnedParent>())
            .register(model_impl_registration::<SetterOnly>())
            .register(model_impl_registration::<OwnedParent>())
            .register(model_impl_registration::<SliceParent>())
            .register(model_impl_registration::<BorrowedParent>())
            .register(model_impl_registration::<BorrowedOptionalParent>())
            .build()
            .expect("reflection context builds")
    });
    ModelRegistry::from_static_metadata(
        context,
        &[
            (TypeMetadata::of::<AccessLeaf>(), &SOURCE),
            (TypeMetadata::of::<AccessMiddle>(), &SOURCE),
            (TypeMetadata::of::<AccessRoot>(), &SOURCE),
            (TypeMetadata::of::<OptionalRoot>(), &SOURCE),
            (TypeMetadata::of::<PrivateRoot>(), &SOURCE),
            (TypeMetadata::of::<SmartPointerRoot>(), &SOURCE),
            (TypeMetadata::of::<SetterOnly>(), &SOURCE),
            (TypeMetadata::of::<PublicRoot>(), &SOURCE),
            (TypeMetadata::of::<OwnedParent>(), &SOURCE),
            (TypeMetadata::of::<SliceParent>(), &SOURCE),
            (TypeMetadata::of::<BorrowedParent>(), &SOURCE),
            (TypeMetadata::of::<BorrowedOptionalParent>(), &SOURCE),
        ],
    )
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
    let registry = test_registry();
    let path = PropertyAccessPath::compile_for_write(&registry, TypeMetadata::of::<SetterOnly>(), &["value"])
        .expect("setter-only leaf compiles for writing");
    assert!(path.check_writable().is_ok());
    let mut root = SetterOnly {};
    SETTER_ONLY_CALLS.store(0, Ordering::Relaxed);

    path.write_dynamic(
        DynamicMut::<Local>::new(&mut root),
        DynamicOwned::<Local>::new("after".to_owned()),
    )
    .expect("setter-only leaf accepts replacement");
    assert_eq!(SETTER_ONLY_CALLS.load(Ordering::Relaxed), 1);
    assert!(matches!(
        PropertyAccessPath::compile(&registry, TypeMetadata::of::<SetterOnly>(), &["value"],),
        Err(PropertyAccessPathError::UnreadableIntermediate { index: 0, .. })
    ));
}

/// Reports an absent optional intermediate separately from an unknown property.
#[test]
fn test_read_optional_intermediate_reports_missing_value() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(&registry, TypeMetadata::of::<OptionalRoot>(), &["middle", "value"])
        .expect("optional model path compiles");
    let root = OptionalRoot { middle: None };

    let read = path.read_dynamic(DynamicRef::<Local>::new(&root));
    assert!(matches!(
        read,
        Err(PropertyAccessPathError::MissingIntermediate { index: 0, .. })
    ));
    assert!(path.check_writable().is_ok());

    let mut root = OptionalRoot {
        middle: Some(AccessMiddle {
            leaf: AccessLeaf {
                value: "nested".to_owned(),
            },
            value: "before".to_owned(),
        }),
    };
    let value = path.read(&root).expect("optional path reads present value");
    assert!(match &value {
        PropertyValue::Borrowed(value) => value.downcast_ref::<String>().is_some_and(|value| value == "before"),
        _ => false,
    });
    drop(value);
    path.write(&mut root, "after".to_owned())
        .expect("optional path writes through present value");
    assert_eq!(root.middle.expect("middle remains present").value, "after");
}

/// Preserves a replacement when writing through an absent optional parent.
#[test]
fn test_write_missing_optional_intermediate_preserves_replacement() {
    let registry = test_registry();
    let path =
        PropertyAccessPath::compile_for_write(&registry, TypeMetadata::of::<OptionalRoot>(), &["middle", "value"])
            .expect("optional model path compiles for writing");
    let mut root = OptionalRoot { middle: None };

    let failure = path
        .write(&mut root, "replacement".to_owned())
        .expect_err("an absent optional parent cannot be traversed");

    assert!(matches!(
        failure.path_error(),
        Some(PropertyAccessPathError::MissingIntermediate { index: 0, name }) if name == "middle"
    ));
    assert_eq!(
        failure
            .replacement()
            .and_then(|value| value.downcast_ref::<String>())
            .map(String::as_str),
        Some("replacement")
    );
    assert!(root.middle.is_none());
}

/// Rejects a getter that returns ownership before any getter invocation.
#[test]
fn test_compile_rejects_owned_intermediate_getter() {
    let registry = test_registry();
    OWNED_GETTER_CALLS.store(0, Ordering::Relaxed);
    let result = PropertyAccessPath::compile(&registry, TypeMetadata::of::<OwnedParent>(), &["owned_child", "value"]);

    assert!(
        matches!(
            &result,
            Err(PropertyAccessPathError::UnsupportedIntermediate { index: 0, .. })
        ),
        "unexpected compile result: {}",
        result
            .err()
            .map_or_else(|| "success".to_owned(), |error| format!("{error:?}"))
    );
    assert_eq!(OWNED_GETTER_CALLS.load(Ordering::Relaxed), 0);
}

/// Borrowed slice getters cannot be traversed as a single intermediate value.
#[test]
fn test_compile_rejects_borrowed_slice_intermediate() {
    let registry = test_registry();
    let error = match PropertyAccessPath::compile(&registry, TypeMetadata::of::<SliceParent>(), &["children", "value"])
    {
        Err(error) => error,
        Ok(_) => panic!("a slice cannot be treated as one nested model"),
    };

    assert!(matches!(
        error,
        PropertyAccessPathError::UnsupportedIntermediate { index: 0, name } if name == "children"
    ));
}

#[test]
fn test_borrowed_getter_intermediate_is_readable_but_not_writable() {
    let registry = test_registry();
    let path =
        PropertyAccessPath::compile_for_write(&registry, TypeMetadata::of::<BorrowedParent>(), &["child", "value"])
            .expect("borrowed getter intermediate compiles for a write path");
    let mut root = BorrowedParent {
        child: AccessLeaf {
            value: "before".to_owned(),
        },
    };

    let failure = path
        .write(&mut root, "replacement".to_owned())
        .expect_err("a borrowed getter cannot be traversed mutably");
    assert!(matches!(
        failure.path_error(),
        Some(PropertyAccessPathError::UnwritableIntermediate { index: 0, name }) if name == "child"
    ));
    assert_eq!(
        failure.replacement().and_then(|value| value.downcast_ref::<String>()),
        Some(&"replacement".to_owned())
    );
    assert_eq!(root.child.value, "before");
}

#[test]
fn test_optional_borrowed_getter_intermediate_reads_present_and_missing_values() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<BorrowedOptionalParent>(),
        &["child", "value"],
    )
    .expect("optional borrowed getter intermediate compiles");

    let missing = BorrowedOptionalParent { child: None };
    assert!(matches!(
        path.read(&missing),
        Err(PropertyAccessPathError::MissingIntermediate { index: 0, name }) if name == "child"
    ));

    let present = BorrowedOptionalParent {
        child: Some(AccessMiddle {
            leaf: AccessLeaf {
                value: "leaf".to_owned(),
            },
            value: "middle".to_owned(),
        }),
    };
    assert!(matches!(
        path.read(&present).expect("present optional getter is traversed"),
        PropertyValue::Borrowed(value)
            if value
                .downcast_ref::<String>()
                .is_some_and(|value| value == "middle")
    ));
}

/// Reports a readable getter-only leaf as unwritable and retains replacements.
#[test]
fn test_getter_only_leaf_is_not_writable() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(&registry, TypeMetadata::of::<OwnedParent>(), &["owned_child"])
        .expect("readable getter compiles as a leaf");

    assert!(matches!(
        path.check_writable(),
        Err(PropertyAccessPathError::NotWritable { index: 0, name }) if name == "owned_child"
    ));

    let mut root = OwnedParent {
        child_value: "unchanged".to_owned(),
    };
    let failure = path
        .write(&mut root, "replacement".to_owned())
        .expect_err("getter-only property has no write destination");

    assert!(matches!(
        failure.path_error(),
        Some(PropertyAccessPathError::NotWritable { index: 0, name }) if name == "owned_child"
    ));
    assert_eq!(
        failure
            .replacement()
            .and_then(|value| value.downcast_ref::<String>())
            .map(String::as_str),
        Some("replacement")
    );
    assert_eq!(root.child_value, "unchanged");
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

/// Reports a private field visibility policy at runtime and retains the input.
#[test]
fn test_write_private_intermediate_reports_access_failure() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<PrivateRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("private field remains a valid path shape");
    assert!(path.check_writable().is_ok());
    let mut root = PrivateRoot {
        middle: AccessMiddle {
            leaf: AccessLeaf {
                value: "unchanged".to_owned(),
            },
            value: "middle".to_owned(),
        },
    };

    let failure = path
        .write_dynamic(
            DynamicMut::<Local>::new(&mut root),
            DynamicOwned::<Local>::new("secret-value".to_owned()),
        )
        .expect_err("visibility policy blocks mutable projection");
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

/// Authorized unsafe access can read and write through private intermediates.
#[test]
fn test_unsafe_dynamic_access_can_cross_private_intermediates() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<PrivateRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("private field remains a valid path shape");
    let mut root = PrivateRoot {
        middle: AccessMiddle {
            leaf: AccessLeaf {
                value: "before".to_owned(),
            },
            value: "middle".to_owned(),
        },
    };

    // SAFETY: this test explicitly authorizes access to `PrivateRoot::middle`
    // and to the nested model fields used by the path.
    {
        let value =
            unsafe { path.read_dynamic_unsafe(PropertyBorrowed::new(&root)) }.expect("authorized unsafe read succeeds");
        let PropertyValue::Borrowed(value) = value else {
            panic!("string field should be returned as a borrow");
        };
        assert_eq!(value.downcast_ref::<String>().map(String::as_str), Some("before"));
    }

    // SAFETY: this test explicitly authorizes mutation through the same path.
    unsafe { path.write_dynamic_unsafe(PropertyMutable::new(&mut root), PropertyOwned::new("after".to_owned())) }
        .expect("authorized unsafe write succeeds");
    assert_eq!(root.middle.leaf.value, "after");
}

#[test]
fn test_write_read_only_intermediate_reports_access_failure() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(
        &registry,
        TypeMetadata::of::<PrivateRoot>(),
        &["middle", "leaf", "value"],
    )
    .expect("read-only field remains a valid path shape");
    assert!(path.check_writable().is_ok());
    let mut root = PrivateRoot {
        middle: AccessMiddle {
            leaf: AccessLeaf {
                value: "unchanged".to_owned(),
            },
            value: "middle".to_owned(),
        },
    };

    let failure = path
        .write_dynamic(
            DynamicMut::<Local>::new(&mut root),
            DynamicOwned::<Local>::new("secret-value".to_owned()),
        )
        .expect_err("visibility policy blocks mutable projection");
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
            value: "middle".to_owned(),
        },
    };

    let value = path
        .read_dynamic(DynamicRef::<Local>::new(&root))
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
            value: "middle".to_owned(),
        },
    };

    path.write_dynamic(
        DynamicMut::<Local>::new(&mut root),
        DynamicOwned::<Local>::new("after".to_owned()),
    )
    .expect("nested property is writable");
    assert_eq!(root.middle.leaf.value, "after");
}

/// The typed convenience APIs read and write public fields without unsafe code.
#[test]
fn test_typed_access_path_apis_support_public_fields() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(&registry, TypeMetadata::of::<PublicRoot>(), &["value"])
        .expect("public field path compiles");
    let mut root = PublicRoot {
        value: "before".to_owned(),
    };

    {
        let value = path.read(&root).expect("public field is readable");
        let PropertyValue::Borrowed(value) = value else {
            panic!("string field should be returned as a borrow");
        };
        assert_eq!(value.downcast_ref::<String>().map(String::as_str), Some("before"));
    }

    path.write(&mut root, "after".to_owned())
        .expect("public field is writable");
    assert_eq!(root.value, "after");

    let failure = path
        .write(&mut root, 17_u32)
        .expect_err("a replacement with the wrong type is rejected");
    assert!(failure.property_failure().is_some());
    assert_eq!(root.value, "after");
}

/// The typed read API reports a mismatched concrete root type.
#[test]
fn test_typed_read_reports_root_type_mismatch() {
    let registry = test_registry();
    let path = PropertyAccessPath::compile(&registry, TypeMetadata::of::<PublicRoot>(), &["value"])
        .expect("public field path compiles");
    let error = match path.read(&"wrong root".to_owned()) {
        Ok(_) => panic!("a different root type should be rejected"),
        Err(error) => error,
    };
    assert!(matches!(error, PropertyAccessPathError::RootTypeMismatch { .. }));
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
        path.read_dynamic(DynamicRef::<Local>::new(&AccessLeaf {
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
        .write_dynamic(
            DynamicMut::<Local>::new(&mut wrong_root),
            DynamicOwned::<Local>::new("replacement".to_owned()),
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
            value: "middle".to_owned(),
        },
    };
    let failure = path
        .write_dynamic(DynamicMut::<Local>::new(&mut root), DynamicOwned::<Local>::new(17_u32))
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
