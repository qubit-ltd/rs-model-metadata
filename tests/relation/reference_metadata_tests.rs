// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for [`ReferenceMetadata`].

use qubit_model_metadata::FieldPath;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::ReferenceMetadata;
use qubit_model_metadata::ReferencePath;
use qubit_model_metadata::ReferencePathSegment;
use qubit_model_metadata::ReferenceTarget;

#[test]
fn test_reference_metadata_preserves_property_reference_details() {
    let entity = ModelId::new("test.metadata.Target");
    let property = FieldPath::new(&["id"]);
    let path = ReferencePath::new(&[
        ReferencePathSegment::Field("account"),
        ReferencePathSegment::Field("target"),
    ]);
    let reference = ReferenceMetadata::new(entity, ReferenceTarget::Property(property), false, Some(path));

    assert_eq!(reference.entity(), entity);
    assert_eq!(reference.target(), ReferenceTarget::Property(property));
    assert!(!reference.existing());
    assert_eq!(reference.path(), Some(path));
}

#[test]
fn test_reference_metadata_supports_whole_model_reference() {
    let entity = ModelId::new("test.metadata.Target");
    let reference = ReferenceMetadata::new(entity, ReferenceTarget::WholeModel, true, None);

    assert_eq!(reference.entity(), entity);
    assert_eq!(reference.target(), ReferenceTarget::WholeModel);
    assert!(reference.existing());
    assert_eq!(reference.path(), None);
}

#[test]
#[should_panic(expected = "reference property path cannot be empty")]
fn test_reference_rejects_empty_property_path() {
    let entity = ModelId::new("test.metadata.Target");
    let _ = ReferenceMetadata::new(entity, ReferenceTarget::Property(FieldPath::new(&[])), true, None);
}

#[test]
#[should_panic(expected = "reference property path cannot contain empty segments")]
fn test_reference_rejects_empty_property_path_segments() {
    let entity = ModelId::new("test.metadata.Target");
    let _ = ReferenceMetadata::new(entity, ReferenceTarget::Property(FieldPath::new(&[""])), true, None);
}

#[test]
#[should_panic(expected = "reference path cannot be empty")]
fn test_reference_rejects_empty_path() {
    let entity = ModelId::new("test.metadata.Target");
    let _ = ReferenceMetadata::new(entity, ReferenceTarget::WholeModel, true, Some(ReferencePath::new(&[])));
}
