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
use qubit_model_metadata::NamedTypeRef;
use qubit_model_metadata::StructMetadata;
use qubit_model_metadata::TypeCapabilities;
use qubit_model_metadata::TypeIdentity;
use qubit_model_metadata::TypeKind;
use qubit_model_metadata::TypeMetadata;
use qubit_model_metadata::TypeShape;

struct ReferencedModel;

static REFERENCED_METADATA: TypeMetadata = TypeMetadata::new(
    ModelId::from_static("test.referenced.ReferencedModel"),
    TypeIdentity::of::<ReferencedModel>(),
    TypeKind::Struct(StructMetadata::new(&[])),
    &[],
);

impl HasTypeShape for ReferencedModel {
    const TYPE_SHAPE: TypeShape = TypeShape::Opaque;
    const CAPABILITIES: TypeCapabilities = TypeCapabilities::NONE;
}

impl HasTypeMetadata for ReferencedModel {
    fn type_metadata() -> &'static TypeMetadata {
        &REFERENCED_METADATA
    }
}

#[test]
fn test_named_type_ref_resolves_registered_metadata() {
    let reference = NamedTypeRef::of::<ReferencedModel>();

    assert_eq!(
        reference.metadata().map(|metadata| metadata.id().as_str()),
        Some("test.referenced.ReferencedModel")
    );
}
