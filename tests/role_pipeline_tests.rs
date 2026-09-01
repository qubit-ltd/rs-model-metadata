// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Runtime smoke coverage for all six shared macro entry points.

use model_runtime::__private::qubit_id::Id;
use model_runtime::TypeDescriptor;
use model_runtime::TypeMetadata;
use qubit_model_derive::Entity;
use qubit_model_derive::Enum;
use qubit_model_derive::Model;
use qubit_model_derive::ModelImpl;
use qubit_model_derive::Projection;
use qubit_model_derive::Value;

#[Entity(id = "example.EntityFixture")]
struct EntityFixture {
    #[identifier]
    id: Id,
}

#[Entity(id = "example.OpaqueIdentifierFixture")]
struct OpaqueIdentifierFixture {
    #[identifier]
    #[opaque]
    id: Id,
}

#[Projection(source_id = "example.EntityFixture")]
struct ProjectionFixture {
    #[identifier]
    id: Id,
}

#[Model]
struct ModelFixture {
    value: String,
}

#[Enum]
enum EnumFixture {
    Ready,
    Data(u64),
}

#[Value(transparent)]
struct ValueFixture(String);

#[Model(id = "example.GenericModel")]
struct GenericModel<T> {
    value: T,
}

#[ModelImpl]
impl ModelFixture {
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[test]
fn test_six_entry_points_share_reflection_expansion() {
    assert_eq!(TypeDescriptor::of::<EntityFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<OpaqueIdentifierFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<ProjectionFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<ModelFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<EnumFixture>().variants().len(), 2);
    assert_eq!(TypeDescriptor::of::<ValueFixture>().fields().len(), 1);
    assert_eq!(TypeDescriptor::of::<GenericModel<u64>>().fields().len(), 1);
    let generic_u64 = TypeMetadata::of::<GenericModel<u64>>();
    let generic_string = TypeMetadata::of::<GenericModel<String>>();
    assert!(!std::ptr::eq(generic_u64, generic_string));
    assert_eq!(generic_u64.model_id(), None);
    assert!(std::ptr::eq(
        generic_u64.generic_definition().expect("generic definition"),
        generic_string.generic_definition().expect("shared generic definition"),
    ));
    assert_eq!(ModelFixture { value: "ok".into() }.value(), "ok");
    let _ = EntityFixture { id: Id::new(1) }.id;
    let _ = OpaqueIdentifierFixture { id: Id::new(2) }.id;
    let _ = ProjectionFixture { id: Id::new(1) }.id;
    let _ = EnumFixture::Ready;
    let EnumFixture::Data(value) = EnumFixture::Data(2) else {
        unreachable!()
    };
    assert_eq!(value, 2);
    let _ = ValueFixture("value".into()).0;
    let _ = GenericModel { value: 3_u64 }.value;
}

#[test]
fn test_generic_metadata_initialization_is_unique_across_threads() {
    let addresses = (0..8)
        .map(|_| std::thread::spawn(|| TypeMetadata::of::<GenericModel<u32>>() as *const TypeMetadata as usize))
        .map(|thread| thread.join().expect("metadata thread must complete"))
        .collect::<Vec<_>>();

    assert!(addresses.iter().all(|address| *address == addresses[0]));
}
