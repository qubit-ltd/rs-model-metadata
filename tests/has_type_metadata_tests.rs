// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_model_metadata::HasTypeMetadata;
use qubit_model_metadata::HasTypeShape;
use qubit_model_metadata::ModelId;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::TypeShape;

struct TestModel;

static TEST_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::new("test.model.TestModel"),
    TypeIdentity::of::<TestModel>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);

impl HasTypeShape for TestModel {
    const TYPE_SHAPE: TypeShape = TypeShape::Opaque;
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for TestModel {
    fn type_metadata() -> &'static TypeMetadata {
        &TEST_METADATA
    }
}

#[test]
fn test_has_type_metadata_returns_static_metadata() {
    assert_eq!(TestModel::type_metadata().id().as_str(), "test.model.TestModel");
}
