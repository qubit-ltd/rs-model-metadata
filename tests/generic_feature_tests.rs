// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "generic")]

//! Generic metadata specialization cache contracts.

use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_metadata::metadata::TypeMetadata;
use qubit_reflect::access::FieldVisibility;
use qubit_reflect::identity::Visibility;

#[Model(id = "generic.CacheFixture")]
struct CacheFixture<T> {
    value: T,
}

#[Enum(id = "generic.VariantFixture")]
enum VariantFixture<T> {
    Named { value: T },
    Pair(T, T),
}

/// Symbolic payloads preserve lookup coordinates without claiming an instance
/// identity.
#[test]
fn generic_enum_payloads_separate_definition_and_concrete_field_identity() {
    let concrete = TypeMetadata::of::<VariantFixture<String>>();
    let other = TypeMetadata::of::<VariantFixture<u64>>();
    let definition = concrete.generic_definition().expect("generic definition");
    assert!(core::ptr::eq(
        definition,
        other.generic_definition().expect("same definition")
    ));
    let variants = concrete.as_enum().expect("concrete enum").variants();
    assert_eq!(variants.len(), 2);
    assert_eq!(definition.variants().len(), variants.len());
    for (index, (symbolic, concrete_variant)) in definition.variants().iter().zip(variants).enumerate() {
        assert_eq!(symbolic.index(), index);
        assert_eq!(concrete_variant.index(), index);
        assert!(symbolic.reflect().is_none());
        assert!(symbolic.definition().is_some());
        assert!(concrete_variant.reflect().is_some());
        assert!(concrete_variant.definition().is_none());
        assert_eq!(symbolic.rust_name(), concrete_variant.rust_name());
        assert_eq!(symbolic.canonical_name(), concrete_variant.canonical_name());
        assert_eq!(symbolic.serialized_name(), concrete_variant.serialized_name());
        assert_eq!(symbolic.deserialized_name(), concrete_variant.deserialized_name());
        assert_eq!(symbolic.is_default(), concrete_variant.is_default());
        assert_eq!(symbolic.fields().len(), concrete_variant.fields().len());
        assert!(symbolic.field_at(symbolic.fields().len()).is_none());
        assert!(concrete_variant.field_at(usize::MAX).is_none());
        assert!(symbolic.field("absent").is_none());
        assert!(concrete_variant.field("absent").is_none());
        for (field_index, symbolic_field) in symbolic.fields().iter().enumerate() {
            assert!(symbolic_field.location().is_none());
            assert!(symbolic_field.reflect().is_none());
            assert!(symbolic_field.definition().is_some());
            assert!(matches!(symbolic_field.visibility(), FieldVisibility::VariantInherited));
            let field = concrete_variant.field_at(field_index).expect("concrete payload");
            assert!(field.definition().is_none());
            assert!(matches!(field.visibility(), FieldVisibility::VariantInherited));
            let location = field.location().expect("concrete field identity");
            assert_eq!(location.owner(), concrete.type_id());
            assert_eq!(location.variant(), Some(index));
            assert_eq!(location.index(), field_index);
            let other_location = other.as_enum().expect("other enum").variants()[index]
                .field_at(field_index)
                .expect("other payload")
                .location()
                .expect("other identity");
            assert_ne!(location, other_location);
        }
    }
    assert_eq!(definition.variants()[0].rust_name(), "Named");
    let named = definition.variants()[0].field("value").expect("named symbolic payload");
    assert!(core::ptr::eq(
        named,
        definition.variants()[0].field_at(0).expect("first payload")
    ));
    assert!(core::ptr::eq(
        variants[0].field("value").expect("named concrete payload"),
        variants[0].field_at(0).expect("first concrete payload")
    ));
    assert!(
        definition.variants()[1].field("0").is_none(),
        "tuple indices are not field names"
    );
    assert!(variants[1].field("0").is_none());
}

#[test]
fn specialization_cache_is_stable_per_concrete_type() {
    let first = TypeMetadata::of::<CacheFixture<u32>>();
    let second = TypeMetadata::of::<CacheFixture<u32>>();
    let different = TypeMetadata::of::<CacheFixture<u64>>();

    assert!(core::ptr::eq(first, second));
    assert!(!core::ptr::eq(first, different));
    assert!(first.generic_definition().is_some());
    let definition_field = &first.generic_definition().expect("definition").fields()[0];
    assert!(definition_field.definition().is_some());
    assert!(matches!(
        definition_field.visibility(),
        FieldVisibility::Declared(Visibility::Private)
    ));
    assert!(matches!(
        first.fields()[0].visibility(),
        FieldVisibility::Declared(Visibility::Private)
    ));
}

/// Concurrent queries share one metadata allocation for each concrete type.
#[test]
fn concurrent_specializations_share_metadata() {
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|_| scope.spawn(TypeMetadata::of::<CacheFixture<String>>))
            .collect();
        let expected = TypeMetadata::of::<CacheFixture<String>>();
        for handle in handles {
            assert!(core::ptr::eq(handle.join().unwrap(), expected));
        }
    });
}
